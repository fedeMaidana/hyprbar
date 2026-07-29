// ─── < Imports > ────────────────────────────────────────────────────

use std::time::{Duration, Instant};

use calloop::channel::Sender;
use vello::Scene;
use vello::peniko::Color;

use crate::app::WorkerHandle;
use crate::components::{Component, DropdownId, Interaction, Pill, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::CommandAction;
use super::control;
use super::panel::{CommandPanel, PanelAvailability};
use super::state::CommandStore;
use super::worker::spawn_poller;

// ─── < Constants > ────────────────────────────────────────────────────

const TOGGLE_GLYPH: &str = "\u{f1542}";
const DRAG_WRITE_INTERVAL: Duration = Duration::from_millis(80);

// ─── < Structs > ────────────────────────────────────────────────────

pub struct CommandCenterPill {
    store: CommandStore,
    _poller: Option<WorkerHandle>,
    drag: Option<(CommandAction, f32)>,
    last_drag_write: Option<Instant>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl CommandCenterPill {
    pub fn new(redraw_signal: Sender<()>) -> Self {
        let store = CommandStore::new();
        let poller = spawn_poller(store.clone(), redraw_signal);

        Self {
            store,
            _poller: poller,
            drag: None,
            last_drag_write: None,
        }
    }

    fn is_active(&self, ctx: &RenderCtx<'_>) -> bool {
        ctx.open_dropdown == Some(DropdownId::COMMAND)
    }

    fn background_color(&self, ctx: &RenderCtx<'_>) -> Color {
        if self.is_active(ctx) {
            ctx.theme.palette.slot_active_bg
        } else {
            ctx.theme.palette.pill_bg
        }
    }

    fn icon_color(&self, ctx: &RenderCtx<'_>) -> Color {
        if self.is_active(ctx) {
            ctx.theme.palette.slot_active_text
        } else {
            ctx.theme.palette.text_primary
        }
    }

    fn slider_available(&self, action: CommandAction) -> bool {
        match action {
            CommandAction::VolumeSlider => self.store.data().sink.is_some(),
            _ => false,
        }
    }

    fn write_drag_value(&mut self, action: CommandAction, fraction: f32, force: bool) {
        let due = force
            || self
                .last_drag_write
                .is_none_or(|written_at| written_at.elapsed() >= DRAG_WRITE_INTERVAL);

        if !due {
            return;
        }

        self.last_drag_write = Some(Instant::now());

        let result = match action {
            CommandAction::VolumeSlider => control::set_sink_volume(fraction),
            _ => return,
        };

        if let Err(error) = result {
            log::warn!("slider write failed for {action:?}: {error}");
        }
    }

    fn apply_drag_to_store(&self, action: CommandAction, fraction: f32) {
        if action == CommandAction::VolumeSlider {
            self.store.update(|data| {
                if let Some(sink) = &mut data.sink {
                    sink.volume = fraction;
                }
            });
        }
    }

    fn run_click_action(&self, action: CommandAction) -> bool {
        let result = match action {
            CommandAction::ToggleSinkMute => control::toggle_sink_mute(),
            CommandAction::ToggleMicMute => control::toggle_mic_mute(),
            CommandAction::ToggleWifi => {
                let enabled = self.store.data().wifi.map(|wifi| wifi.enabled).unwrap_or(false);
                control::set_wifi_enabled(!enabled)
            }
            // Theme toggling is handled at the app level, where the theme lives.
            CommandAction::VolumeSlider | CommandAction::ToggleTheme => return false,
        };

        if let Err(error) = result {
            log::warn!("command action {action:?} failed: {error}");
            return false;
        }

        match action {
            CommandAction::ToggleSinkMute => self.store.update(|data| {
                if let Some(sink) = &mut data.sink {
                    sink.muted = !sink.muted;
                }
            }),
            CommandAction::ToggleMicMute => self.store.update(|data| {
                if let Some(muted) = &mut data.mic_muted {
                    *muted = !*muted;
                }
            }),
            CommandAction::ToggleWifi => self.store.update(|data| {
                if let Some(wifi) = &mut data.wifi {
                    wifi.enabled = !wifi.enabled;

                    if !wifi.enabled {
                        wifi.ssid = None;
                    }
                }
            }),
            _ => {}
        }

        true
    }
}

impl Component for CommandCenterPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
        let (iw, _) = ctx.text.measure(TOGGLE_GLYPH, icon_size, &ctx.theme.typography.icon_font_family);
        let w = iw + ctx.theme.tokens.pill_padding_x * 2.0;

        (w, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        let open = self.is_active(ctx);

        self.store.set_panel_open(open);

        if !open {
            self.drag = None;
            self.last_drag_write = None;
        }

        Pill::draw_with_background(scene, bounds, ctx.theme, self.background_color(ctx));

        let pad_x = ctx.theme.tokens.pill_padding_x;
        let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;

        ctx.text.draw_centered_v(
            scene,
            TOGGLE_GLYPH,
            bounds.x + pad_x,
            bounds.y,
            bounds.height,
            TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, self.icon_color(ctx)),
        );
    }

    fn hit_test(&self, _point: Point, _bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        Some(Interaction::Dropdown(DropdownId::COMMAND))
    }

    fn dropdown_id(&self) -> Option<DropdownId> {
        Some(DropdownId::COMMAND)
    }

    fn render_dropdown(&mut self, scene: &mut Scene, surface: Rect, anchor: Rect, ctx: &mut RenderCtx<'_>) {
        let data = self.store.data();

        CommandPanel::draw(scene, surface, anchor, &data, self.drag, ctx);
    }

    fn dropdown_bounds(&self, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Rect> {
        Some(CommandPanel::bounds(surface, anchor, theme))
    }

    fn hit_test_dropdown(&self, point: Point, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Interaction> {
        let availability = PanelAvailability::from_data(&self.store.data());

        CommandPanel::hit_test(point, surface, anchor, theme, availability)
    }

    fn handle_interaction(&mut self, interaction: Interaction) -> bool {
        let Interaction::Command(action) = interaction else {
            return false;
        };

        if action.is_slider() {
            return false;
        }

        self.run_click_action(action)
    }

    fn handle_drag(&mut self, interaction: Interaction, point: Point, surface: Rect, anchor: Rect, theme: &Theme) -> bool {
        let Interaction::Command(action) = interaction else {
            return false;
        };

        if !self.slider_available(action) {
            return false;
        }

        let Some(fraction) = CommandPanel::slider_fraction(action, point, surface, anchor, theme) else {
            return false;
        };

        self.drag = Some((action, fraction));
        self.write_drag_value(action, fraction, false);

        true
    }

    fn end_drag(&mut self, interaction: Interaction) {
        let Interaction::Command(action) = interaction else {
            return;
        };

        let Some((drag_action, fraction)) = self.drag.take() else {
            return;
        };

        self.last_drag_write = None;

        if drag_action != action {
            return;
        }

        self.write_drag_value(action, fraction, true);
        self.apply_drag_to_store(action, fraction);
    }
}
