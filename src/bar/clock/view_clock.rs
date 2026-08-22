// ─── < Imports > ────────────────────────────────────────────────────

use chrono::{DateTime, Local, Offset, Timelike, Utc};
use vello::Scene;
use vello::kurbo::{Affine, Circle, RoundedRect};
use vello::peniko::Fill;

use crate::components::{DropdownFrame, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::zones::{WORLD_ZONES, is_daytime, local_zone_display_name, offset_label, utc_label, zone_offset_minutes};

// ─── < Constants > ────────────────────────────────────────────────────

const OFFSET_TEXT_SCALE: f32 = 0.78;
const ZONE_ICON_SCALE: f32 = 0.95;
const SECONDS_TEXT_SCALE: f32 = 0.62;

const DAY_GLYPH: &str = "\u{f0599}";
const NIGHT_GLYPH: &str = "\u{f0f61}";

/// Header con altura propia: la hora grande y la ciudad necesitan aire.
const HEADER_H: f32 = 54.0;
const HEADER_TIME_H: f32 = 30.0;
const HEADER_SUBTITLE_GAP: f32 = 6.0;
const HEADER_SUBTITLE_H: f32 = 16.0;

/// Chip circular detrás del sol/luna de cada ciudad.
const ICON_BADGE_RADIUS: f32 = 11.0;
const ICON_BADGE_ALPHA: f32 = 0.16;

// ─── < Structs > ────────────────────────────────────────────────────

struct ZoneRow {
    name: &'static str,
    offset_minutes: i32,
    offset_text: String,
    time_text: String,
    daytime: bool,
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn height(theme: &Theme) -> f32 {
    let tokens = theme.tokens;
    let rows = WORLD_ZONES.len() as f32;

    HEADER_H + tokens.dropdown_section_gap + tokens.clock_row_height * rows + tokens.clock_row_gap * (rows - 1.0)
}

pub(crate) fn draw(scene: &mut Scene, area: Rect, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;

    let now_local = Local::now();
    let now_utc = now_local.with_timezone(&Utc);
    let local_offset_minutes = now_local.offset().fix().local_minus_utc() / 60;

    let mut y = area.y;

    draw_header(scene, area.x, y, now_local, local_offset_minutes, ctx);

    y += HEADER_H;

    DropdownFrame::draw_divider(scene, area.x, y + tokens.dropdown_section_gap / 2.0, area.width, ctx.theme);

    y += tokens.dropdown_section_gap;

    let rows = zone_rows(now_utc, local_offset_minutes);

    for (index, row) in rows.iter().enumerate() {
        let row_y = y + index as f32 * (tokens.clock_row_height + tokens.clock_row_gap);
        draw_zone_row(scene, area.x, row_y, area.width, row, ctx);
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn zone_rows(now_utc: DateTime<Utc>, local_offset_minutes: i32) -> Vec<ZoneRow> {
    let mut rows: Vec<ZoneRow> = WORLD_ZONES
        .iter()
        .map(|&(name, tz)| {
            let remote = now_utc.with_timezone(&tz);
            let offset_minutes = zone_offset_minutes(now_utc, tz) - local_offset_minutes;

            ZoneRow {
                name,
                offset_minutes,
                offset_text: offset_label(offset_minutes),
                time_text: remote.format("%H:%M").to_string(),
                daytime: is_daytime(remote.hour()),
            }
        })
        .collect();

    rows.sort_by_key(|row| row.offset_minutes);

    rows
}

fn draw_header(scene: &mut Scene, x: f32, y: f32, now_local: DateTime<Local>, local_offset_minutes: i32, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;

    let time_size = ctx.theme.typography.size_base * tokens.clock_header_time_scale;
    let subtitle_size = ctx.theme.typography.size_base * tokens.dropdown_subtitle_scale;

    let time_prefix = now_local.format("%H:%M").to_string();
    let time_seconds = now_local.format(":%S").to_string();

    let seconds_size = time_size * SECONDS_TEXT_SCALE;

    let (prefix_width, prefix_height) = ctx.text.measure(&time_prefix, time_size, ctx.theme.typography.font_family);
    let (_, seconds_height) = ctx.text.measure(&time_seconds, seconds_size, ctx.theme.typography.font_family);

    ctx.text.draw_centered_v(
        scene,
        &time_prefix,
        x,
        y,
        HEADER_TIME_H,
        TextStyle::new(time_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    // Smaller seconds, baseline-aligned with the main time.
    let seconds_y = y + (prefix_height - seconds_height) / 2.0;

    ctx.text.draw_centered_v(
        scene,
        &time_seconds,
        x + prefix_width,
        seconds_y,
        HEADER_TIME_H,
        TextStyle::new(seconds_size, ctx.theme.typography.font_family, ctx.theme.palette.accent),
    );

    let subtitle = format!("{} · {}", local_zone_display_name(), utc_label(local_offset_minutes));

    ctx.text.draw_centered_v(
        scene,
        &subtitle,
        x,
        y + HEADER_TIME_H + HEADER_SUBTITLE_GAP,
        HEADER_SUBTITLE_H,
        TextStyle::new(subtitle_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
}

fn draw_zone_row(scene: &mut Scene, x: f32, y: f32, width: f32, row: &ZoneRow, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let row_height = tokens.clock_row_height;

    let name_size = ctx.theme.typography.size_base * tokens.dropdown_body_scale;
    let time_size = ctx.theme.typography.size_base * tokens.dropdown_body_scale;

    let (time_width, _) = ctx.text.measure(&row.time_text, time_size, ctx.theme.typography.font_family);
    let time_x = x + width - time_width;

    ctx.text.draw_centered_v(
        scene,
        &row.time_text,
        time_x,
        y,
        row_height,
        TextStyle::new(time_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    draw_offset_chip(scene, time_x - tokens.clock_row_inner_gap, y, row_height, &row.offset_text, ctx);

    let icon_glyph = if row.daytime { DAY_GLYPH } else { NIGHT_GLYPH };
    let icon_size = ctx.theme.typography.size_base * ZONE_ICON_SCALE;
    let icon_color = if row.daytime {
        ctx.theme.palette.clock_day
    } else {
        ctx.theme.palette.clock_night
    };

    // Chip circular teñido con el color del momento del día.
    let badge_center = ((x + ICON_BADGE_RADIUS) as f64, (y + row_height / 2.0) as f64);
    let badge = Circle::new(badge_center, ICON_BADGE_RADIUS as f64);

    scene.fill(Fill::NonZero, Affine::IDENTITY, icon_color.with_alpha(ICON_BADGE_ALPHA), None, &badge);

    ctx.text.draw_centered(
        scene,
        icon_glyph,
        Rect::new(x, y, ICON_BADGE_RADIUS * 2.0, row_height),
        TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, icon_color),
    );

    let name_color = if row.daytime {
        ctx.theme.palette.text_primary
    } else {
        ctx.theme.palette.text_secondary
    };

    // Fixed slot keeps city names aligned regardless of glyph width.
    ctx.text.draw_centered_v(
        scene,
        row.name,
        x + tokens.clock_icon_slot,
        y,
        row_height,
        TextStyle::new(name_size, ctx.theme.typography.font_family, name_color),
    );
}

fn draw_offset_chip(scene: &mut Scene, right: f32, row_y: f32, row_height: f32, text: &str, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let text_size = ctx.theme.typography.size_base * OFFSET_TEXT_SCALE;

    let (text_width, text_height) = ctx.text.measure(text, text_size, ctx.theme.typography.font_family);

    let chip_width = text_width + tokens.clock_chip_padding_x * 2.0;
    let chip_height = text_height + tokens.clock_chip_padding_y * 2.0;

    let chip_x = right - chip_width;
    let chip_y = row_y + (row_height - chip_height) / 2.0;

    let body = RoundedRect::new(
        chip_x as f64,
        chip_y as f64,
        (chip_x + chip_width) as f64,
        (chip_y + chip_height) as f64,
        tokens.clock_chip_radius as f64,
    );

    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_raised, None, &body);

    ctx.text.draw_centered_v(
        scene,
        text,
        chip_x + tokens.clock_chip_padding_x,
        chip_y,
        chip_height,
        TextStyle::new(text_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
}
