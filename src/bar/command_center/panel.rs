// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, Circle, RoundedRect};
use vello::peniko::{Color, Fill};

use crate::components::{DropdownFrame, Interaction, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::{Theme, ThemeMode};

use super::action::CommandAction;
use super::state::CommandData;

// ─── < Constants > ────────────────────────────────────────────────────

const VOLUME_GLYPH: &str = "\u{f057e}";
const VOLUME_MUTED_GLYPH: &str = "\u{f0581}";
const MIC_GLYPH: &str = "\u{f036c}";
const MIC_OFF_GLYPH: &str = "\u{f036d}";
const WIFI_GLYPH: &str = "\u{f05a9}";
const WIFI_OFF_GLYPH: &str = "\u{f05aa}";
const THEME_GLYPH: &str = "\u{f0599}";

const VALUE_PLACEHOLDER: &str = "—";
const WIFI_LABEL: &str = "WiFi";
const MIC_LABEL: &str = "Mic";
const THEME_LABEL: &str = "Claro";
const ELLIPSIS: &str = "…";

const SMALL_TEXT_SCALE: f32 = 0.78;
const DISABLED_ALPHA: f32 = 0.35;
const TOGGLE_LABEL_EDGE: f32 = 6.0;
const HANDLE_HOVER_GROWTH: f32 = 1.5;
const MUTED_FILL_ALPHA: f32 = 0.35;
const MUTE_HOVER_RADIUS: f32 = 6.0;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct CommandPanel;

#[derive(Debug, Clone, Copy, Default)]
pub struct PanelAvailability {
    pub volume: bool,
    pub mic: bool,
    pub wifi: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToggleState {
    Disabled,
    Inactive,
    Active,
    Alert,
}

struct ToggleVisual {
    action: CommandAction,
    glyph: &'static str,
    label: String,
    state: ToggleState,
}

struct SliderVisual {
    glyph: &'static str,
    icon_color: Color,
    fraction: Option<f32>,
    muted: bool,
    enabled: bool,
    engaged: bool,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl PanelAvailability {
    pub fn from_data(data: &CommandData) -> Self {
        Self {
            volume: data.sink.is_some(),
            mic: data.mic_muted.is_some(),
            wifi: data.wifi.is_some(),
        }
    }
}

impl CommandPanel {
    pub fn height(theme: &Theme) -> f32 {
        let tokens = theme.tokens;

        tokens.command_panel_padding_y * 2.0
            + tokens.command_slider_row_height
            + tokens.command_section_gap
            + tokens.command_toggle_height
    }

    pub fn bounds(surface: Rect, anchor: Rect, theme: &Theme) -> Rect {
        Self::frame(theme).bounds(surface, anchor, theme)
    }

    pub fn slider_fraction(action: CommandAction, point: Point, surface: Rect, anchor: Rect, theme: &Theme) -> Option<f32> {
        if !action.is_slider() {
            return None;
        }

        let bounds = Self::bounds(surface, anchor, theme);
        let track = slider_track_rect(volume_row(bounds, theme), theme);

        if track.width <= 0.0 {
            return None;
        }

        Some(((point.x - track.x) / track.width).clamp(0.0, 1.0))
    }

    pub fn hit_test(point: Point, surface: Rect, anchor: Rect, theme: &Theme, availability: PanelAvailability) -> Option<Interaction> {
        let bounds = Self::bounds(surface, anchor, theme);
        let row = volume_row(bounds, theme);

        if availability.volume && volume_icon_rect(row, theme).contains_point(point.x, point.y) {
            return Some(Interaction::Command(CommandAction::ToggleSinkMute));
        }

        if availability.volume && row.contains_point(point.x, point.y) {
            return Some(Interaction::Command(CommandAction::VolumeSlider));
        }

        for (action, rect) in toggle_rects(bounds, theme) {
            let enabled = match action {
                CommandAction::ToggleWifi => availability.wifi,
                CommandAction::ToggleMicMute => availability.mic,
                _ => true,
            };

            if enabled && rect.contains_point(point.x, point.y) {
                return Some(Interaction::Command(action));
            }
        }

        None
    }

    pub fn draw(
        scene: &mut Scene,
        surface: Rect,
        anchor: Rect,
        data: &CommandData,
        drag: Option<(CommandAction, f32)>,
        ctx: &mut RenderCtx<'_>,
    ) {
        let theme = ctx.theme;
        let availability = PanelAvailability::from_data(data);

        let frame = Self::frame(theme);
        let bounds = frame.bounds(surface, anchor, theme);

        frame.draw_background(scene, bounds, theme);

        let row = volume_row(bounds, theme);

        draw_volume_row(scene, row, data, drag, availability.volume, ctx);

        let toggles_y = toggle_rects(bounds, theme)[0].1.y;

        DropdownFrame::draw_divider(scene, row.x, toggles_y - theme.tokens.command_section_gap / 2.0, row.width, theme);

        draw_toggles(scene, bounds, data, ctx);
    }

    fn frame(theme: &Theme) -> DropdownFrame {
        DropdownFrame::new(theme.tokens.command_panel_width, Self::height(theme))
    }
}

// ─── < Private Functions: Geometry > ────────────────────────────────────────────────────

fn inner_rect(bounds: Rect, theme: &Theme) -> Rect {
    let tokens = theme.tokens;

    Rect::new(
        bounds.x + tokens.command_panel_padding_x,
        bounds.y + tokens.command_panel_padding_y,
        bounds.width - tokens.command_panel_padding_x * 2.0,
        bounds.height - tokens.command_panel_padding_y * 2.0,
    )
}

fn volume_row(bounds: Rect, theme: &Theme) -> Rect {
    let inner = inner_rect(bounds, theme);

    Rect::new(inner.x, inner.y, inner.width, theme.tokens.command_slider_row_height)
}

fn slider_track_rect(row: Rect, theme: &Theme) -> Rect {
    let tokens = theme.tokens;

    let x = row.x + tokens.command_icon_slot + tokens.command_inner_gap;
    let width = (row.width - tokens.command_icon_slot - tokens.command_inner_gap * 2.0 - tokens.command_value_width).max(0.0);
    let y = row.y + (row.height - tokens.command_track_height) / 2.0;

    Rect::new(x, y, width, tokens.command_track_height)
}

fn volume_icon_rect(row: Rect, theme: &Theme) -> Rect {
    Rect::new(row.x, row.y, theme.tokens.command_icon_slot, row.height)
}

fn toggle_rects(bounds: Rect, theme: &Theme) -> [(CommandAction, Rect); 3] {
    let tokens = theme.tokens;
    let inner = inner_rect(bounds, theme);

    let y = inner.y + inner.height - tokens.command_toggle_height;
    let gap = tokens.command_toggle_gap;
    let width = (inner.width - gap * 2.0) / 3.0;

    let actions = [CommandAction::ToggleWifi, CommandAction::ToggleMicMute, CommandAction::ToggleTheme];

    let mut rects = [(CommandAction::ToggleWifi, Rect::new(0.0, 0.0, 0.0, 0.0)); 3];

    for (index, action) in actions.into_iter().enumerate() {
        let x = inner.x + index as f32 * (width + gap);
        rects[index] = (action, Rect::new(x, y, width, tokens.command_toggle_height));
    }

    rects
}

// ─── < Private Functions: Drawing > ────────────────────────────────────────────────────

fn dim(color: Color, alpha: f32) -> Color {
    color.with_alpha(alpha)
}

fn drag_fraction(drag: Option<(CommandAction, f32)>, action: CommandAction) -> Option<f32> {
    drag.filter(|(drag_action, _)| *drag_action == action).map(|(_, fraction)| fraction)
}

fn draw_volume_row(
    scene: &mut Scene,
    row: Rect,
    data: &CommandData,
    drag: Option<(CommandAction, f32)>,
    enabled: bool,
    ctx: &mut RenderCtx<'_>,
) {
    let muted = data.sink.map(|sink| sink.muted).unwrap_or(false);
    let glyph = if muted { VOLUME_MUTED_GLYPH } else { VOLUME_GLYPH };

    let fraction = drag_fraction(drag, CommandAction::VolumeSlider).or_else(|| data.sink.map(|sink| sink.volume.clamp(0.0, 1.0)));

    let icon_color = if muted {
        ctx.theme.palette.text_secondary
    } else {
        ctx.theme.palette.text_primary
    };

    let engaged = drag_fraction(drag, CommandAction::VolumeSlider).is_some()
        || ctx.hovered_interaction == Some(Interaction::Command(CommandAction::VolumeSlider));

    if ctx.hovered_interaction == Some(Interaction::Command(CommandAction::ToggleSinkMute)) {
        draw_mute_hover(scene, volume_icon_rect(row, ctx.theme), ctx.theme);
    }

    let visual = SliderVisual {
        glyph,
        icon_color,
        fraction,
        muted,
        enabled,
        engaged,
    };

    draw_slider_row(scene, row, &visual, ctx);
}

fn draw_mute_hover(scene: &mut Scene, rect: Rect, theme: &Theme) {
    let body = RoundedRect::new(
        rect.x as f64 - 2.0,
        rect.y as f64,
        (rect.x + rect.width) as f64,
        (rect.y + rect.height) as f64,
        MUTE_HOVER_RADIUS as f64,
    );

    scene.fill(Fill::NonZero, Affine::IDENTITY, theme.palette.control_hover_bg, None, &body);
}

fn draw_slider_row(scene: &mut Scene, row: Rect, visual: &SliderVisual, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let icon_size = ctx.theme.typography.size_base * tokens.icon_scale;

    let icon_color = if visual.enabled {
        visual.icon_color
    } else {
        dim(ctx.theme.palette.text_secondary, DISABLED_ALPHA)
    };

    ctx.text.draw_centered_v(
        scene,
        visual.glyph,
        row.x,
        row.y,
        row.height,
        TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, icon_color),
    );

    let track = slider_track_rect(row, ctx.theme);
    let radius = (track.height / 2.0) as f64;

    let track_color = if visual.enabled {
        ctx.theme.palette.panel_raised
    } else {
        dim(ctx.theme.palette.panel_raised, DISABLED_ALPHA)
    };

    let track_shape =
        RoundedRect::new(track.x as f64, track.y as f64, (track.x + track.width) as f64, (track.y + track.height) as f64, radius);

    scene.fill(Fill::NonZero, Affine::IDENTITY, track_color, None, &track_shape);

    if visual.enabled && let Some(fraction) = visual.fraction {
        let fill_width = (track.width * fraction).max(track.height);

        // A muted channel keeps its level visible but visually steps back.
        let fill_color = if visual.muted {
            dim(ctx.theme.palette.accent, MUTED_FILL_ALPHA)
        } else {
            ctx.theme.palette.accent
        };

        let fill_shape =
            RoundedRect::new(track.x as f64, track.y as f64, (track.x + fill_width) as f64, (track.y + track.height) as f64, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, fill_color, None, &fill_shape);

        // The handle grows on hover/drag for a tactile feel.
        let handle_radius = if visual.engaged {
            tokens.command_handle_radius + HANDLE_HOVER_GROWTH
        } else {
            tokens.command_handle_radius
        };

        let handle_x = track.x + track.width * fraction;
        let handle_y = track.y + track.height / 2.0;

        let handle = Circle::new((handle_x as f64, handle_y as f64), handle_radius as f64);

        scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.slot_active_bg, None, &handle);
    }

    let value_text = if visual.enabled {
        visual
            .fraction
            .map(|fraction| format!("{}%", (fraction * 100.0).round() as u32))
            .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string())
    } else {
        VALUE_PLACEHOLDER.to_string()
    };

    let value_color = if !visual.enabled {
        dim(ctx.theme.palette.text_secondary, DISABLED_ALPHA)
    } else if visual.muted {
        ctx.theme.palette.text_secondary
    } else {
        ctx.theme.palette.text_primary
    };

    let value_size = ctx.theme.typography.size_base * SMALL_TEXT_SCALE;
    let (value_width, _) = ctx.text.measure(&value_text, value_size, &ctx.theme.typography.font_family);

    ctx.text.draw_centered_v(
        scene,
        &value_text,
        row.x + row.width - value_width,
        row.y,
        row.height,
        TextStyle::new(value_size, &ctx.theme.typography.font_family, value_color),
    );
}

fn draw_toggles(scene: &mut Scene, bounds: Rect, data: &CommandData, ctx: &mut RenderCtx<'_>) {
    let wifi_on = data.wifi.as_ref().map(|wifi| wifi.enabled).unwrap_or(false);
    let wifi_label = data
        .wifi
        .as_ref()
        .filter(|wifi| wifi.enabled)
        .and_then(|wifi| wifi.ssid.clone())
        .unwrap_or_else(|| WIFI_LABEL.to_string());

    let mic_muted = data.mic_muted == Some(true);
    let light_mode = ctx.theme.mode == ThemeMode::Light;

    let toggles = [
        ToggleVisual {
            action: CommandAction::ToggleWifi,
            glyph: if wifi_on { WIFI_GLYPH } else { WIFI_OFF_GLYPH },
            label: wifi_label,
            state: toggle_state(data.wifi.is_some(), wifi_on, false),
        },
        ToggleVisual {
            action: CommandAction::ToggleMicMute,
            glyph: if mic_muted { MIC_OFF_GLYPH } else { MIC_GLYPH },
            label: MIC_LABEL.to_string(),
            state: toggle_state(data.mic_muted.is_some(), !mic_muted, mic_muted),
        },
        ToggleVisual {
            action: CommandAction::ToggleTheme,
            glyph: THEME_GLYPH,
            label: THEME_LABEL.to_string(),
            state: toggle_state(true, light_mode, false),
        },
    ];

    for ((_, rect), toggle) in toggle_rects(bounds, ctx.theme).into_iter().zip(toggles) {
        draw_toggle(scene, rect, &toggle, ctx);
    }
}

fn toggle_state(available: bool, active: bool, alert: bool) -> ToggleState {
    if !available {
        ToggleState::Disabled
    } else if alert {
        ToggleState::Alert
    } else if active {
        ToggleState::Active
    } else {
        ToggleState::Inactive
    }
}

fn draw_toggle(scene: &mut Scene, rect: Rect, toggle: &ToggleVisual, ctx: &mut RenderCtx<'_>) {
    let radius = ctx.theme.tokens.command_toggle_radius as f64;
    let is_hovered = toggle.state == ToggleState::Inactive && ctx.hovered_interaction == Some(Interaction::Command(toggle.action));

    let (background, foreground) = match toggle.state {
        ToggleState::Disabled => {
            (dim(ctx.theme.palette.panel_raised, DISABLED_ALPHA), dim(ctx.theme.palette.text_secondary, DISABLED_ALPHA))
        }
        ToggleState::Active => (ctx.theme.palette.slot_active_bg, ctx.theme.palette.slot_active_text),
        ToggleState::Alert => (ctx.theme.palette.danger_bg, ctx.theme.palette.danger_text),
        ToggleState::Inactive if is_hovered => (ctx.theme.palette.control_hover_bg, ctx.theme.palette.text_primary),
        ToggleState::Inactive => (ctx.theme.palette.control_bg, ctx.theme.palette.text_primary),
    };

    let body = RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, radius);

    scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

    let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
    let label_size = ctx.theme.typography.size_base * SMALL_TEXT_SCALE;

    let (icon_width, _) = ctx.text.measure(toggle.glyph, icon_size, &ctx.theme.typography.icon_font_family);

    let max_label_width = (rect.width - icon_width - ctx.theme.tokens.command_inner_gap - TOGGLE_LABEL_EDGE * 2.0).max(0.0);
    let label = truncate_to_width(ctx, &toggle.label, label_size, max_label_width);

    let (label_width, _) = ctx.text.measure(&label, label_size, &ctx.theme.typography.font_family);

    let group_width = icon_width + ctx.theme.tokens.command_inner_gap + label_width;
    let group_x = rect.x + (rect.width - group_width) / 2.0;

    ctx.text.draw_centered_v(
        scene,
        toggle.glyph,
        group_x,
        rect.y,
        rect.height,
        TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, foreground),
    );

    ctx.text.draw_centered_v(
        scene,
        &label,
        group_x + icon_width + ctx.theme.tokens.command_inner_gap,
        rect.y,
        rect.height,
        TextStyle::new(label_size, &ctx.theme.typography.font_family, foreground),
    );
}

fn truncate_to_width(ctx: &mut RenderCtx<'_>, text: &str, size: f32, max_width: f32) -> String {
    let family = ctx.theme.typography.font_family.clone();

    let (full_width, _) = ctx.text.measure(text, size, &family);

    if full_width <= max_width {
        return text.to_string();
    }

    let mut truncated = String::new();

    for character in text.chars() {
        let candidate = format!("{truncated}{character}{ELLIPSIS}");
        let (candidate_width, _) = ctx.text.measure(&candidate, size, &family);

        if candidate_width > max_width {
            break;
        }

        truncated.push(character);
    }

    format!("{truncated}{ELLIPSIS}")
}
