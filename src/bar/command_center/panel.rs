// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, Circle, RoundedRect};
use vello::peniko::{Color, Fill};

use crate::components::{DropdownFrame, Interaction, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::CommandAction;
use super::state::{CommandData, MediaState};

// ─── < Constants > ────────────────────────────────────────────────────

const VOLUME_GLYPH: &str = "\u{f057e}";
const VOLUME_MUTED_GLYPH: &str = "\u{f0581}";
const BRIGHTNESS_GLYPH: &str = "\u{f05a8}";
const MIC_GLYPH: &str = "\u{f036c}";
const MIC_OFF_GLYPH: &str = "\u{f036d}";
const PLAY_GLYPH: &str = "\u{f040a}";
const PAUSE_GLYPH: &str = "\u{f03e4}";
const PREVIOUS_GLYPH: &str = "\u{f04ae}";
const NEXT_GLYPH: &str = "\u{f04ad}";

const VALUE_PLACEHOLDER: &str = "—";
const NO_MEDIA_TEXT: &str = "Sin reproducción";
const UNTITLED_TEXT: &str = "Sin título";
const MIC_LABEL: &str = "Mic";
const ELLIPSIS: &str = "…";

const SMALL_TEXT_SCALE: f32 = 0.78;
const MEDIA_SUBTITLE_SCALE: f32 = 0.7;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct CommandPanel;

// ─── < Implementations > ────────────────────────────────────────────────────

impl CommandPanel {
    pub fn height(theme: &Theme) -> f32 {
        let tokens = theme.tokens;

        tokens.command_panel_padding_y * 2.0
            + tokens.command_slider_row_height * 2.0
            + tokens.command_slider_gap
            + tokens.command_section_gap * 2.0
            + tokens.command_media_height
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
        let row = slider_rows(bounds, theme)
            .into_iter()
            .find(|(row_action, _)| *row_action == action)?
            .1;
        let track = slider_track_rect(row, theme);

        if track.width <= 0.0 {
            return None;
        }

        Some(((point.x - track.x) / track.width).clamp(0.0, 1.0))
    }

    pub fn hit_test(point: Point, surface: Rect, anchor: Rect, theme: &Theme, has_media: bool) -> Option<Interaction> {
        let bounds = Self::bounds(surface, anchor, theme);

        let [(volume_action, volume_row), (brightness_action, brightness_row)] = slider_rows(bounds, theme);

        if volume_icon_rect(volume_row, theme).contains_point(point.x, point.y) {
            return Some(Interaction::Command(CommandAction::ToggleSinkMute));
        }

        if volume_row.contains_point(point.x, point.y) {
            return Some(Interaction::Command(volume_action));
        }

        if brightness_row.contains_point(point.x, point.y) {
            return Some(Interaction::Command(brightness_action));
        }

        if has_media {
            for (action, rect) in media_button_rects(bounds, theme) {
                if rect.contains_point(point.x, point.y) {
                    return Some(Interaction::Command(action));
                }
            }
        }

        if mic_toggle_rect(bounds, theme).contains_point(point.x, point.y) {
            return Some(Interaction::Command(CommandAction::ToggleMicMute));
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

        let frame = Self::frame(theme);
        let bounds = frame.bounds(surface, anchor, theme);

        frame.draw_background(scene, bounds, theme);

        let [(_, volume_row), (_, brightness_row)] = slider_rows(bounds, theme);

        draw_volume_row(scene, volume_row, data, drag, ctx);
        draw_brightness_row(scene, brightness_row, data, drag, ctx);
        draw_media_row(scene, bounds, data.media.as_ref(), ctx);
        draw_mic_toggle(scene, bounds, data.mic_muted, ctx);
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

fn slider_rows(bounds: Rect, theme: &Theme) -> [(CommandAction, Rect); 2] {
    let tokens = theme.tokens;
    let inner = inner_rect(bounds, theme);

    let volume_row = Rect::new(inner.x, inner.y, inner.width, tokens.command_slider_row_height);

    let brightness_row = Rect::new(
        inner.x,
        volume_row.y + tokens.command_slider_row_height + tokens.command_slider_gap,
        inner.width,
        tokens.command_slider_row_height,
    );

    [
        (CommandAction::VolumeSlider, volume_row),
        (CommandAction::BrightnessSlider, brightness_row),
    ]
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

fn media_row_rect(bounds: Rect, theme: &Theme) -> Rect {
    let tokens = theme.tokens;
    let inner = inner_rect(bounds, theme);

    let y = inner.y + tokens.command_slider_row_height * 2.0 + tokens.command_slider_gap + tokens.command_section_gap;

    Rect::new(inner.x, y, inner.width, tokens.command_media_height)
}

fn media_button_rects(bounds: Rect, theme: &Theme) -> [(CommandAction, Rect); 3] {
    let tokens = theme.tokens;
    let row = media_row_rect(bounds, theme);

    let button = tokens.command_media_button_size;
    let play = tokens.command_media_play_size;
    let gap = tokens.command_media_button_gap;

    let next_x = row.x + row.width - button;
    let play_x = next_x - gap - play;
    let prev_x = play_x - gap - button;

    let small_y = row.y + (row.height - button) / 2.0;
    let play_y = row.y + (row.height - play) / 2.0;

    [
        (CommandAction::MediaPrevious, Rect::new(prev_x, small_y, button, button)),
        (CommandAction::MediaPlayPause, Rect::new(play_x, play_y, play, play)),
        (CommandAction::MediaNext, Rect::new(next_x, small_y, button, button)),
    ]
}

fn mic_toggle_rect(bounds: Rect, theme: &Theme) -> Rect {
    let tokens = theme.tokens;
    let inner = inner_rect(bounds, theme);

    let y = inner.y + inner.height - tokens.command_toggle_height;

    Rect::new(inner.x, y, inner.width, tokens.command_toggle_height)
}

// ─── < Private Functions: Drawing > ────────────────────────────────────────────────────

fn draw_volume_row(scene: &mut Scene, row: Rect, data: &CommandData, drag: Option<(CommandAction, f32)>, ctx: &mut RenderCtx<'_>) {
    let muted = data.sink.map(|sink| sink.muted).unwrap_or(false);
    let glyph = if muted { VOLUME_MUTED_GLYPH } else { VOLUME_GLYPH };

    let fraction = drag_fraction(drag, CommandAction::VolumeSlider).or_else(|| data.sink.map(|sink| sink.volume.clamp(0.0, 1.0)));

    let icon_color = if muted {
        ctx.theme.palette.text_secondary
    } else {
        ctx.theme.palette.text_primary
    };

    draw_slider_row(scene, row, glyph, icon_color, fraction, ctx);
}

fn draw_brightness_row(scene: &mut Scene, row: Rect, data: &CommandData, drag: Option<(CommandAction, f32)>, ctx: &mut RenderCtx<'_>) {
    let fraction = drag_fraction(drag, CommandAction::BrightnessSlider).or_else(|| data.brightness.map(|brightness| brightness.fraction));

    draw_slider_row(scene, row, BRIGHTNESS_GLYPH, ctx.theme.palette.text_primary, fraction, ctx);
}

fn drag_fraction(drag: Option<(CommandAction, f32)>, action: CommandAction) -> Option<f32> {
    drag.filter(|(drag_action, _)| *drag_action == action).map(|(_, fraction)| fraction)
}

fn draw_slider_row(scene: &mut Scene, row: Rect, glyph: &str, icon_color: Color, fraction: Option<f32>, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let icon_size = ctx.theme.typography.size_base * tokens.icon_scale;

    ctx.text.draw_centered_v(
        scene,
        glyph,
        row.x,
        row.y,
        row.height,
        TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, icon_color),
    );

    let track = slider_track_rect(row, ctx.theme);
    let radius = (track.height / 2.0) as f64;

    let track_shape =
        RoundedRect::new(track.x as f64, track.y as f64, (track.x + track.width) as f64, (track.y + track.height) as f64, radius);

    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.slot_empty_bg, None, &track_shape);

    if let Some(fraction) = fraction {
        let fill_width = (track.width * fraction).max(track.height);

        let fill_shape =
            RoundedRect::new(track.x as f64, track.y as f64, (track.x + fill_width) as f64, (track.y + track.height) as f64, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.accent, None, &fill_shape);

        let handle_x = track.x + track.width * fraction;
        let handle_y = track.y + track.height / 2.0;

        let handle = Circle::new((handle_x as f64, handle_y as f64), tokens.command_handle_radius as f64);

        scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.slot_active_bg, None, &handle);
    }

    let value_text = fraction
        .map(|fraction| format!("{}%", (fraction * 100.0).round() as u32))
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    let value_size = ctx.theme.typography.size_base * SMALL_TEXT_SCALE;
    let (value_width, _) = ctx.text.measure(&value_text, value_size, &ctx.theme.typography.font_family);

    ctx.text.draw_centered_v(
        scene,
        &value_text,
        row.x + row.width - value_width,
        row.y,
        row.height,
        TextStyle::new(value_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );
}

fn draw_media_row(scene: &mut Scene, bounds: Rect, media: Option<&MediaState>, ctx: &mut RenderCtx<'_>) {
    let row = media_row_rect(bounds, ctx.theme);

    let Some(media) = media else {
        let size = ctx.theme.typography.size_base * SMALL_TEXT_SCALE;

        ctx.text.draw_centered_v(
            scene,
            NO_MEDIA_TEXT,
            row.x,
            row.y,
            row.height,
            TextStyle::new(size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );

        return;
    };

    let tokens = ctx.theme.tokens;

    let buttons_width = tokens.command_media_button_size * 2.0 + tokens.command_media_play_size + tokens.command_media_button_gap * 2.0;

    let text_width = (row.width - buttons_width - tokens.command_inner_gap).max(0.0);

    let title_size = ctx.theme.typography.size_base * SMALL_TEXT_SCALE;
    let artist_size = ctx.theme.typography.size_base * MEDIA_SUBTITLE_SCALE;

    let title = if media.title.is_empty() {
        UNTITLED_TEXT
    } else {
        media.title.as_str()
    };
    let title = truncate_to_width(ctx, title, title_size, text_width);

    ctx.text.draw_centered_v(
        scene,
        &title,
        row.x,
        row.y,
        row.height * 0.55,
        TextStyle::new(title_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    if !media.artist.is_empty() {
        let artist = truncate_to_width(ctx, &media.artist, artist_size, text_width);

        ctx.text.draw_centered_v(
            scene,
            &artist,
            row.x,
            row.y + row.height * 0.55,
            row.height * 0.45,
            TextStyle::new(artist_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );
    }

    draw_media_buttons(scene, bounds, media.playing, ctx);
}

fn draw_media_buttons(scene: &mut Scene, bounds: Rect, playing: bool, ctx: &mut RenderCtx<'_>) {
    let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
    let radius = ctx.theme.tokens.command_media_button_radius as f64;

    for (action, rect) in media_button_rects(bounds, ctx.theme) {
        let is_play = action == CommandAction::MediaPlayPause;
        let is_hovered = ctx.hovered_interaction == Some(Interaction::Command(action));

        let (background, foreground) = if is_play {
            (ctx.theme.palette.slot_active_bg, ctx.theme.palette.slot_active_text)
        } else if is_hovered {
            (ctx.theme.palette.slot_hover_bg, ctx.theme.palette.text_primary)
        } else {
            (ctx.theme.palette.slot_inactive_bg, ctx.theme.palette.text_primary)
        };

        let body = RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        let glyph = match action {
            CommandAction::MediaPrevious => PREVIOUS_GLYPH,
            CommandAction::MediaNext => NEXT_GLYPH,
            _ => {
                if playing {
                    PAUSE_GLYPH
                } else {
                    PLAY_GLYPH
                }
            }
        };

        let (glyph_width, _) = ctx.text.measure(glyph, icon_size, &ctx.theme.typography.icon_font_family);

        ctx.text.draw_centered_v(
            scene,
            glyph,
            rect.x + (rect.width - glyph_width) / 2.0,
            rect.y,
            rect.height,
            TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, foreground),
        );
    }
}

fn draw_mic_toggle(scene: &mut Scene, bounds: Rect, mic_muted: Option<bool>, ctx: &mut RenderCtx<'_>) {
    let rect = mic_toggle_rect(bounds, ctx.theme);
    let radius = ctx.theme.tokens.command_toggle_radius as f64;

    let active = mic_muted == Some(true);
    let is_hovered = ctx.hovered_interaction == Some(Interaction::Command(CommandAction::ToggleMicMute));

    let (background, foreground) = if active {
        (ctx.theme.palette.slot_active_bg, ctx.theme.palette.slot_active_text)
    } else if is_hovered {
        (ctx.theme.palette.slot_hover_bg, ctx.theme.palette.text_primary)
    } else {
        (ctx.theme.palette.slot_inactive_bg, ctx.theme.palette.text_primary)
    };

    let body = RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, radius);

    scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

    let glyph = if active { MIC_OFF_GLYPH } else { MIC_GLYPH };

    let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
    let label_size = ctx.theme.typography.size_base * SMALL_TEXT_SCALE;

    let (icon_width, _) = ctx.text.measure(glyph, icon_size, &ctx.theme.typography.icon_font_family);
    let (label_width, _) = ctx.text.measure(MIC_LABEL, label_size, &ctx.theme.typography.font_family);

    let group_width = icon_width + ctx.theme.tokens.command_inner_gap + label_width;
    let group_x = rect.x + (rect.width - group_width) / 2.0;

    ctx.text.draw_centered_v(
        scene,
        glyph,
        group_x,
        rect.y,
        rect.height,
        TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, foreground),
    );

    ctx.text.draw_centered_v(
        scene,
        MIC_LABEL,
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
