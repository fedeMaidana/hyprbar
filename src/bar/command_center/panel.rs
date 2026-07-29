// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, Circle, RoundedRect, Stroke};
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
const WIFI_TITLE: &str = "Wi-Fi";
const WIFI_ON_TEXT: &str = "Activado";
const WIFI_OFF_TEXT: &str = "Desactivado";
const SOUND_TITLE: &str = "Sonido";
const ELLIPSIS: &str = "…";

const SMALL_TEXT_SCALE: f32 = 0.78;
const SUBTITLE_TEXT_SCALE: f32 = 0.7;
const DISABLED_ALPHA: f32 = 0.35;
const MUTED_FILL_ALPHA: f32 = 0.4;

const WIFI_TEXT_BLOCK_HEIGHT: f32 = 30.0;
const WIFI_TEXT_GAP: f32 = 10.0;
const SLIDER_ICON_INSET: f32 = 8.0;
const SLIDER_MIN_FILL: f32 = 30.0;
const MUTE_ZONE_WIDTH: f32 = 32.0;
const KNOB_EXTRA_RADIUS: f32 = 2.0;
const EMBED_ICON_SCALE: f32 = 0.82;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct CommandPanel;

#[derive(Debug, Clone, Copy, Default)]
pub struct PanelAvailability {
    pub volume: bool,
    pub mic: bool,
    pub wifi: bool,
}

struct SliderVisual {
    glyph: &'static str,
    fraction: Option<f32>,
    muted: bool,
    enabled: bool,
    engaged: bool,
    icon_hovered: bool,
}

struct CircleButton {
    glyph: &'static str,
    background: Color,
    foreground: Color,
    hoverable: bool,
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
            + tokens.command_wifi_card_height
            + tokens.command_card_gap
            + tokens.command_sound_card_height
    }

    pub fn bounds(surface: Rect, anchor: Rect, theme: &Theme) -> Rect {
        Self::frame(theme).bounds(surface, anchor, theme)
    }

    pub fn slider_fraction(action: CommandAction, point: Point, surface: Rect, anchor: Rect, theme: &Theme) -> Option<f32> {
        if !action.is_slider() {
            return None;
        }

        let bounds = Self::bounds(surface, anchor, theme);
        let track = sound_slider_row(bounds, theme);

        if track.width <= 0.0 {
            return None;
        }

        Some(((point.x - track.x) / track.width).clamp(0.0, 1.0))
    }

    pub fn hit_test(point: Point, surface: Rect, anchor: Rect, theme: &Theme, availability: PanelAvailability) -> Option<Interaction> {
        let bounds = Self::bounds(surface, anchor, theme);
        let slider_row = sound_slider_row(bounds, theme);

        if availability.volume && mute_zone_rect(slider_row).contains_point(point.x, point.y) {
            return Some(Interaction::Command(CommandAction::ToggleSinkMute));
        }

        if availability.volume && slider_row.contains_point(point.x, point.y) {
            return Some(Interaction::Command(CommandAction::VolumeSlider));
        }

        if availability.wifi && wifi_card_rect(bounds, theme).contains_point(point.x, point.y) {
            return Some(Interaction::Command(CommandAction::ToggleWifi));
        }

        let [(mic_action, mic_rect), (theme_action, theme_rect)] = circle_button_rects(bounds, theme);

        if availability.mic && mic_rect.contains_point(point.x, point.y) {
            return Some(Interaction::Command(mic_action));
        }

        if theme_rect.contains_point(point.x, point.y) {
            return Some(Interaction::Command(theme_action));
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

        draw_wifi_card(scene, wifi_card_rect(bounds, theme), data, availability.wifi, ctx);
        draw_circle_buttons(scene, bounds, data, availability, ctx);
        draw_sound_card(scene, sound_card_rect(bounds, theme), data, drag, availability.volume, ctx);
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

fn wifi_card_rect(bounds: Rect, theme: &Theme) -> Rect {
    let tokens = theme.tokens;
    let inner = inner_rect(bounds, theme);

    // The card takes whatever the two circle buttons beside it leave free.
    let circles_width = tokens.command_circle_button_radius * 4.0 + tokens.command_circle_button_gap + tokens.command_card_gap;

    Rect::new(inner.x, inner.y, (inner.width - circles_width).max(0.0), tokens.command_wifi_card_height)
}

fn circle_button_rects(bounds: Rect, theme: &Theme) -> [(CommandAction, Rect); 2] {
    let tokens = theme.tokens;
    let inner = inner_rect(bounds, theme);
    let card = wifi_card_rect(bounds, theme);

    let radius = tokens.command_circle_button_radius;
    let gap = tokens.command_circle_button_gap;

    let zone_x = card.x + card.width + tokens.command_card_gap;
    let center_y = inner.y + tokens.command_wifi_card_height / 2.0;

    let mic_center_x = zone_x + radius;
    let theme_center_x = zone_x + radius * 3.0 + gap;

    [
        (CommandAction::ToggleMicMute, circle_bounds(mic_center_x, center_y, radius)),
        (CommandAction::ToggleTheme, circle_bounds(theme_center_x, center_y, radius)),
    ]
}

fn circle_bounds(center_x: f32, center_y: f32, radius: f32) -> Rect {
    Rect::new(center_x - radius, center_y - radius, radius * 2.0, radius * 2.0)
}

fn sound_card_rect(bounds: Rect, theme: &Theme) -> Rect {
    let tokens = theme.tokens;
    let inner = inner_rect(bounds, theme);

    let y = inner.y + tokens.command_wifi_card_height + tokens.command_card_gap;

    Rect::new(inner.x, y, inner.width, tokens.command_sound_card_height)
}

fn sound_slider_row(bounds: Rect, theme: &Theme) -> Rect {
    slider_row_in(sound_card_rect(bounds, theme), theme)
}

fn slider_row_in(card: Rect, theme: &Theme) -> Rect {
    let tokens = theme.tokens;

    Rect::new(
        card.x + tokens.command_card_padding_x,
        card.y + tokens.command_card_padding_y + tokens.command_card_title_height,
        card.width - tokens.command_card_padding_x * 2.0,
        tokens.command_slider_row_height,
    )
}

fn mute_zone_rect(row: Rect) -> Rect {
    Rect::new(row.x, row.y, MUTE_ZONE_WIDTH, row.height)
}

// ─── < Private Functions: Drawing > ────────────────────────────────────────────────────

fn dim(color: Color, alpha: f32) -> Color {
    color.with_alpha(alpha)
}

fn drag_fraction(drag: Option<(CommandAction, f32)>, action: CommandAction) -> Option<f32> {
    drag.filter(|(drag_action, _)| *drag_action == action).map(|(_, fraction)| fraction)
}

fn fill_card(scene: &mut Scene, rect: Rect, radius: f32, color: Color) {
    let body = RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, radius as f64);

    scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &body);
}

fn draw_wifi_card(scene: &mut Scene, card: Rect, data: &CommandData, enabled: bool, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;

    let is_hovered = enabled && ctx.hovered_interaction == Some(Interaction::Command(CommandAction::ToggleWifi));

    let card_bg = if is_hovered {
        ctx.theme.palette.control_hover_bg
    } else {
        ctx.theme.palette.panel_raised
    };

    fill_card(scene, card, tokens.command_card_radius, card_bg);

    let wifi_on = data.wifi.as_ref().map(|wifi| wifi.enabled).unwrap_or(false);

    // Horizontal module: circular icon at the left, labels stacked beside it.
    let circle_x = card.x + tokens.command_card_padding_x + tokens.command_icon_circle_radius;
    let circle_y = card.y + card.height / 2.0;
    let circle = Circle::new((circle_x as f64, circle_y as f64), tokens.command_icon_circle_radius as f64);

    let (circle_bg, icon_color) = if !enabled {
        (dim(ctx.theme.palette.control_bg, DISABLED_ALPHA), dim(ctx.theme.palette.text_secondary, DISABLED_ALPHA))
    } else if wifi_on {
        (ctx.theme.palette.slot_active_bg, ctx.theme.palette.slot_active_text)
    } else {
        (ctx.theme.palette.control_bg, ctx.theme.palette.text_secondary)
    };

    scene.fill(Fill::NonZero, Affine::IDENTITY, circle_bg, None, &circle);

    let glyph = if wifi_on { WIFI_GLYPH } else { WIFI_OFF_GLYPH };
    let icon_size = ctx.theme.typography.size_base * tokens.icon_scale;

    let (glyph_width, _) = ctx.text.measure(glyph, icon_size, &ctx.theme.typography.icon_font_family);

    ctx.text.draw_centered_v(
        scene,
        glyph,
        circle_x - glyph_width / 2.0,
        circle_y - tokens.command_icon_circle_radius,
        tokens.command_icon_circle_radius * 2.0,
        TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, icon_color),
    );

    let text_x = circle_x + tokens.command_icon_circle_radius + WIFI_TEXT_GAP;
    let text_width = (card.x + card.width - tokens.command_card_padding_x - text_x).max(0.0);
    let block_y = card.y + (card.height - WIFI_TEXT_BLOCK_HEIGHT) / 2.0;

    let title_size = ctx.theme.typography.size_base * SMALL_TEXT_SCALE;
    let subtitle_size = ctx.theme.typography.size_base * SUBTITLE_TEXT_SCALE;

    let (title_color, subtitle_color) = if enabled {
        (ctx.theme.palette.text_primary, ctx.theme.palette.text_secondary)
    } else {
        (dim(ctx.theme.palette.text_secondary, DISABLED_ALPHA), dim(ctx.theme.palette.text_secondary, DISABLED_ALPHA))
    };

    ctx.text.draw_centered_v(
        scene,
        WIFI_TITLE,
        text_x,
        block_y,
        WIFI_TEXT_BLOCK_HEIGHT * 0.55,
        TextStyle::new(title_size, &ctx.theme.typography.font_family, title_color),
    );

    let subtitle = wifi_subtitle(data, enabled);
    let subtitle = truncate_to_width(ctx, &subtitle, subtitle_size, text_width);

    ctx.text.draw_centered_v(
        scene,
        &subtitle,
        text_x,
        block_y + WIFI_TEXT_BLOCK_HEIGHT * 0.55,
        WIFI_TEXT_BLOCK_HEIGHT * 0.45,
        TextStyle::new(subtitle_size, &ctx.theme.typography.font_family, subtitle_color),
    );
}

fn wifi_subtitle(data: &CommandData, enabled: bool) -> String {
    if !enabled {
        return VALUE_PLACEHOLDER.to_string();
    }

    match data.wifi.as_ref() {
        Some(wifi) if wifi.enabled => wifi.ssid.clone().unwrap_or_else(|| WIFI_ON_TEXT.to_string()),
        Some(_) => WIFI_OFF_TEXT.to_string(),
        None => VALUE_PLACEHOLDER.to_string(),
    }
}

fn draw_circle_buttons(scene: &mut Scene, bounds: Rect, data: &CommandData, availability: PanelAvailability, ctx: &mut RenderCtx<'_>) {
    let mic_muted = data.mic_muted == Some(true);
    let light_mode = ctx.theme.mode == ThemeMode::Light;
    let palette = &ctx.theme.palette;

    // Bare circular buttons, like Bluetooth and AirDrop on macOS.
    let buttons = [
        CircleButton {
            glyph: if mic_muted { MIC_OFF_GLYPH } else { MIC_GLYPH },
            background: if !availability.mic {
                dim(palette.control_bg, DISABLED_ALPHA)
            } else if mic_muted {
                palette.danger_bg
            } else {
                palette.slot_active_bg
            },
            foreground: if !availability.mic {
                dim(palette.text_secondary, DISABLED_ALPHA)
            } else if mic_muted {
                palette.danger_text
            } else {
                palette.slot_active_text
            },
            hoverable: availability.mic,
        },
        CircleButton {
            glyph: THEME_GLYPH,
            background: if light_mode { palette.slot_active_bg } else { palette.control_bg },
            foreground: if light_mode {
                palette.slot_active_text
            } else {
                palette.text_secondary
            },
            hoverable: true,
        },
    ];

    for ((action, rect), button) in circle_button_rects(bounds, ctx.theme).into_iter().zip(buttons) {
        draw_circle_button(scene, action, rect, &button, ctx);
    }
}

fn draw_circle_button(scene: &mut Scene, action: CommandAction, rect: Rect, button: &CircleButton, ctx: &mut RenderCtx<'_>) {
    let radius = rect.width / 2.0;
    let center_x = rect.x + radius;
    let center_y = rect.y + radius;

    let circle = Circle::new((center_x as f64, center_y as f64), radius as f64);

    scene.fill(Fill::NonZero, Affine::IDENTITY, button.background, None, &circle);

    let is_hovered = button.hoverable && ctx.hovered_interaction == Some(Interaction::Command(action));

    if is_hovered {
        scene.stroke(&Stroke::new(1.5), Affine::IDENTITY, ctx.theme.palette.text_primary.with_alpha(0.5), None, &circle);
    }

    let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
    let (glyph_width, _) = ctx.text.measure(button.glyph, icon_size, &ctx.theme.typography.icon_font_family);

    ctx.text.draw_centered_v(
        scene,
        button.glyph,
        center_x - glyph_width / 2.0,
        rect.y,
        rect.height,
        TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, button.foreground),
    );
}

fn draw_sound_card(
    scene: &mut Scene,
    card: Rect,
    data: &CommandData,
    drag: Option<(CommandAction, f32)>,
    enabled: bool,
    ctx: &mut RenderCtx<'_>,
) {
    let tokens = ctx.theme.tokens;

    fill_card(scene, card, tokens.command_card_radius, ctx.theme.palette.panel_raised);

    let title_size = ctx.theme.typography.size_base * SUBTITLE_TEXT_SCALE;

    ctx.text.draw_centered_v(
        scene,
        SOUND_TITLE,
        card.x + tokens.command_card_padding_x,
        card.y + tokens.command_card_padding_y,
        tokens.command_card_title_height,
        TextStyle::new(title_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    let row = slider_row_in(card, ctx.theme);

    let muted = data.sink.map(|sink| sink.muted).unwrap_or(false);

    let visual = SliderVisual {
        glyph: if muted { VOLUME_MUTED_GLYPH } else { VOLUME_GLYPH },
        fraction: drag_fraction(drag, CommandAction::VolumeSlider).or_else(|| data.sink.map(|sink| sink.volume.clamp(0.0, 1.0))),
        muted,
        enabled,
        engaged: drag_fraction(drag, CommandAction::VolumeSlider).is_some()
            || ctx.hovered_interaction == Some(Interaction::Command(CommandAction::VolumeSlider)),
        icon_hovered: ctx.hovered_interaction == Some(Interaction::Command(CommandAction::ToggleSinkMute)),
    };

    draw_thick_slider(scene, row, &visual, ctx);
}

/// macOS-style slider: a tall rounded track with the icon embedded inside.
fn draw_thick_slider(scene: &mut Scene, row: Rect, visual: &SliderVisual, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;

    let track_height = tokens.command_thick_track_height;
    let track = Rect::new(row.x, row.y + (row.height - track_height) / 2.0, row.width, track_height);
    let radius = (track_height / 2.0) as f64;

    let track_color = if visual.enabled {
        ctx.theme.palette.control_bg
    } else {
        dim(ctx.theme.palette.control_bg, DISABLED_ALPHA)
    };

    let track_shape =
        RoundedRect::new(track.x as f64, track.y as f64, (track.x + track.width) as f64, (track.y + track.height) as f64, radius);

    scene.fill(Fill::NonZero, Affine::IDENTITY, track_color, None, &track_shape);

    if visual.enabled && let Some(fraction) = visual.fraction {
        // The fill never uncovers the embedded icon, mirroring macOS.
        let fill_width = (track.width * fraction).max(SLIDER_MIN_FILL);

        let fill_color = if visual.muted {
            dim(ctx.theme.palette.text_primary, MUTED_FILL_ALPHA)
        } else {
            ctx.theme.palette.text_primary
        };

        let fill_shape =
            RoundedRect::new(track.x as f64, track.y as f64, (track.x + fill_width) as f64, (track.y + track.height) as f64, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, fill_color, None, &fill_shape);

        // The knob only shows up while hovering or dragging, like macOS.
        if visual.engaged {
            let knob_radius = track_height / 2.0 + KNOB_EXTRA_RADIUS;
            let knob_x = (track.x + fill_width).min(track.x + track.width - knob_radius).max(track.x + knob_radius);
            let knob_y = track.y + track_height / 2.0;

            let knob = Circle::new((knob_x as f64, knob_y as f64), knob_radius as f64);

            scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.text_primary, None, &knob);
            scene.stroke(&Stroke::new(1.0), Affine::IDENTITY, ctx.theme.palette.panel_border, None, &knob);
        }
    }

    // Embedded icon: dark over the light fill (which always covers it).
    let icon_size = ctx.theme.typography.size_base * EMBED_ICON_SCALE;

    let icon_color = if !visual.enabled {
        dim(ctx.theme.palette.text_secondary, DISABLED_ALPHA)
    } else if visual.icon_hovered {
        ctx.theme.palette.accent
    } else {
        ctx.theme.palette.panel_bg
    };

    ctx.text.draw_centered_v(
        scene,
        visual.glyph,
        track.x + SLIDER_ICON_INSET,
        track.y,
        track_height,
        TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, icon_color),
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
