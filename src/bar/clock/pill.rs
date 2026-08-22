// ─── < Imports > ────────────────────────────────────────────────────

use std::time::Duration;

use calloop::channel::Sender;
use chrono::Local;
use vello::Scene;
use vello::peniko::Color;

use crate::app::WorkerHandle;
use crate::components::{Component, DropdownId, Interaction, InteractionOutcome, Panel, Pill, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::{ClockAction, ClockTab};
use super::alarms;
use super::panel::ClockPanel;
use super::state::{Alarm, AlarmEditor, ClockStore};
use super::worker::spawn_clock_watcher;

// ─── < Constants > ────────────────────────────────────────────────────

pub(crate) const CLOCK_DROPDOWN: DropdownId = DropdownId::new("clock");

/// Paso de los minutos en el picker de alarmas.
const MINUTE_STEP: u8 = 5;

const STOPWATCH_TICK: Duration = Duration::from_millis(50);
const TIMER_TICK: Duration = Duration::from_millis(200);
const CLOCK_TICK: Duration = Duration::from_secs(1);

// ─── < Structs > ────────────────────────────────────────────────────

pub struct ClockPill {
    store: ClockStore,
    _watcher: Option<WorkerHandle>,
    active_tab: ClockTab,
    editor: Option<AlarmEditor>,
    frame_text: Option<String>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl ClockPill {
    pub fn new(redraw_signal: Sender<()>) -> Self {
        let store = ClockStore::new();

        store.update(|data| data.alarms = alarms::load());

        let watcher = spawn_clock_watcher(store.clone(), redraw_signal);

        Self {
            store,
            _watcher: watcher,
            active_tab: ClockTab::default(),
            editor: None,
            frame_text: None,
        }
    }

    fn current_text(&self) -> String {
        Local::now().format("%H:%M").to_string()
    }

    fn take_frame_text(&mut self) -> String {
        self.frame_text.take().unwrap_or_else(|| self.current_text())
    }

    fn is_active(&self, ctx: &RenderCtx<'_>) -> bool {
        ctx.open_dropdown == Some(CLOCK_DROPDOWN)
    }

    fn background_color(&self, ctx: &RenderCtx<'_>) -> Color {
        let hovered = ctx.hovered_interaction == Some(Interaction::Dropdown(CLOCK_DROPDOWN));

        Pill::background_for(self.is_active(ctx), hovered, ctx.theme)
    }

    fn text_color(&self, ctx: &RenderCtx<'_>) -> Color {
        if self.is_active(ctx) {
            ctx.theme.palette.slot_active_text
        } else {
            ctx.theme.palette.text_primary
        }
    }

    /// Guarda una alarma del editor: reemplaza o agrega, y persiste.
    fn save_editor(&mut self, editor: AlarmEditor) {
        self.store.update(|data| {
            match editor.index {
                Some(index) if index < data.alarms.len() => {
                    let alarm = &mut data.alarms[index];

                    alarm.hour = editor.hour;
                    alarm.minute = editor.minute;
                    alarm.repeat = editor.repeat;
                }
                _ => data.alarms.push(Alarm {
                    hour: editor.hour,
                    minute: editor.minute,
                    repeat: editor.repeat,
                    enabled: true,
                    label: None,
                }),
            }

            data.alarms.sort_by_key(|alarm| (alarm.hour, alarm.minute));
            alarms::save(&data.alarms);
        });
    }
}

impl Component for ClockPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let text = self.current_text();

        let (tw, _) = ctx
            .text
            .measure(&text, ctx.theme.typography.size_base, ctx.theme.typography.font_family);

        self.frame_text = Some(text);

        let w = tw + ctx.theme.tokens.pill_padding_x * 2.0;

        (w, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        // Panel cerrado: la próxima apertura arranca limpia.
        if !self.is_active(ctx) {
            self.active_tab = ClockTab::default();
            self.editor = None;
        }

        Pill::draw_with_background(scene, bounds, ctx.theme, self.background_color(ctx));

        let text = self.take_frame_text();
        let pad_x = ctx.theme.tokens.pill_padding_x;
        let size = ctx.theme.typography.size_base;

        ctx.text.draw_centered_v(
            scene,
            &text,
            bounds.x + pad_x,
            bounds.y,
            bounds.height,
            TextStyle::new(size, ctx.theme.typography.font_family, self.text_color(ctx)),
        );
    }

    fn hit_test(&self, _point: Point, _bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        Some(Interaction::Dropdown(CLOCK_DROPDOWN))
    }

    fn dropdown_id(&self) -> Option<DropdownId> {
        Some(CLOCK_DROPDOWN)
    }

    fn dropdown_max_height(&self, theme: &Theme) -> f32 {
        ClockPanel::max_height(theme)
    }

    /// Cadencia según lo que se mira: centésimas del cronómetro, anillo
    /// del temporizador o el segundero del reloj.
    fn dropdown_tick(&self) -> Option<Duration> {
        let data = self.store.snapshot();

        let interval = match self.active_tab {
            ClockTab::Stopwatch if data.stopwatch.running() => STOPWATCH_TICK,
            ClockTab::Timer if data.timer.running() => TIMER_TICK,
            _ => CLOCK_TICK,
        };

        Some(interval)
    }

    fn render_dropdown(&mut self, scene: &mut Scene, surface: Rect, anchor: Rect, ctx: &mut RenderCtx<'_>) {
        let data = self.store.snapshot();

        ClockPanel {
            data: &data,
            active_tab: self.active_tab,
            editor: self.editor.as_ref(),
        }
        .render(scene, surface, anchor, ctx);
    }

    fn dropdown_bounds(&self, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Rect> {
        let data = self.store.snapshot();

        Some(
            ClockPanel {
                data: &data,
                active_tab: self.active_tab,
                editor: self.editor.as_ref(),
            }
            .bounds(surface, anchor, theme),
        )
    }

    fn hit_test_dropdown(&self, point: Point, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Interaction> {
        let data = self.store.snapshot();

        ClockPanel {
            data: &data,
            active_tab: self.active_tab,
            editor: self.editor.as_ref(),
        }
        .hit_test(point, surface, anchor, theme)
    }

    fn handle_interaction(&mut self, interaction: Interaction) -> Option<InteractionOutcome> {
        let action = ClockAction::from_interaction(interaction)?;

        match action {
            ClockAction::SelectTab(tab) => {
                self.active_tab = tab;
                self.editor = None;
            }
            ClockAction::ToggleAlarm(index) => {
                self.store.update(|data| {
                    if let Some(alarm) = data.alarms.get_mut(index) {
                        alarm.enabled = !alarm.enabled;
                        alarms::save(&data.alarms);
                    }
                });
            }
            ClockAction::EditAlarm(index) => {
                let data = self.store.snapshot();

                if let Some(alarm) = data.alarms.get(index) {
                    self.editor = Some(AlarmEditor::for_alarm(index, alarm));
                }
            }
            ClockAction::NewAlarm => self.editor = Some(AlarmEditor::blank()),
            ClockAction::EditorCancel => self.editor = None,
            ClockAction::EditorHourUp => {
                if let Some(editor) = &mut self.editor {
                    editor.hour = (editor.hour + 1) % 24;
                }
            }
            ClockAction::EditorHourDown => {
                if let Some(editor) = &mut self.editor {
                    editor.hour = (editor.hour + 23) % 24;
                }
            }
            ClockAction::EditorMinuteUp => {
                if let Some(editor) = &mut self.editor {
                    editor.minute = (editor.minute + MINUTE_STEP) % 60;
                }
            }
            ClockAction::EditorMinuteDown => {
                if let Some(editor) = &mut self.editor {
                    editor.minute = (editor.minute + 60 - MINUTE_STEP) % 60;
                }
            }
            ClockAction::EditorRepeat(repeat) => {
                if let Some(editor) = &mut self.editor {
                    editor.repeat = repeat;
                }
            }
            ClockAction::EditorSave => {
                if let Some(editor) = self.editor.take() {
                    self.save_editor(editor);
                }
            }
            ClockAction::DeleteAlarm => {
                if let Some(editor) = self.editor.take()
                    && let Some(index) = editor.index
                {
                    self.store.update(|data| {
                        if index < data.alarms.len() {
                            data.alarms.remove(index);
                            alarms::save(&data.alarms);
                        }
                    });
                }
            }
            ClockAction::StopwatchToggle => self.store.update(|data| data.stopwatch.toggle()),
            ClockAction::StopwatchLap => self.store.update(|data| data.stopwatch.lap()),
            ClockAction::StopwatchReset => self.store.update(|data| data.stopwatch.reset()),
            ClockAction::TimerToggle => self.store.update(|data| data.timer.toggle()),
            ClockAction::TimerReset => self.store.update(|data| data.timer.reset()),
            ClockAction::TimerPreset(minutes) => self.store.update(|data| data.timer.set_minutes(minutes)),
        }

        Some(InteractionOutcome::redraw())
    }
}
