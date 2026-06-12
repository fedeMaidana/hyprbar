// ─── < Imports > ────────────────────────────────────────────────────

use chrono::{DateTime, Local, Offset, Utc};
use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::Fill;

use crate::components::{DropdownFrame, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::zones::{WORLD_ZONES, day_tag, local_zone_display_name, offset_label, utc_label, zone_offset_minutes};

// ─── < Constants > ────────────────────────────────────────────────────

const SUBTITLE_TEXT_SCALE: f32 = 0.78;
const OFFSET_TEXT_SCALE: f32 = 0.78;
const TAG_TEXT_SCALE: f32 = 0.7;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct ClockPanel;

struct ZoneRow {
    name: &'static str,
    offset_text: String,
    time_text: String,
    tag: Option<&'static str>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl ClockPanel {
    pub fn height(theme: &Theme) -> f32 {
        let tokens = theme.tokens;
        let rows = WORLD_ZONES.len() as f32;

        tokens.clock_panel_padding_y * 2.0
            + tokens.clock_header_height
            + tokens.clock_section_gap
            + tokens.clock_row_height * rows
            + tokens.clock_row_gap * (rows - 1.0)
    }

    pub fn bounds(surface: Rect, anchor: Rect, theme: &Theme) -> Rect {
        Self::frame(theme).bounds(surface, anchor, theme)
    }

    pub fn draw(scene: &mut Scene, surface: Rect, anchor: Rect, ctx: &mut RenderCtx<'_>) {
        let theme = ctx.theme;
        let tokens = theme.tokens;

        let frame = Self::frame(theme);
        let bounds = frame.bounds(surface, anchor, theme);

        frame.draw_background(scene, bounds, theme);

        let now_local = Local::now();
        let now_utc = now_local.with_timezone(&Utc);
        let local_offset_minutes = now_local.offset().fix().local_minus_utc() / 60;

        let inner_x = bounds.x + tokens.clock_panel_padding_x;
        let inner_width = bounds.width - tokens.clock_panel_padding_x * 2.0;
        let mut y = bounds.y + tokens.clock_panel_padding_y;

        draw_header(scene, inner_x, y, now_local, local_offset_minutes, ctx);

        y += tokens.clock_header_height + tokens.clock_section_gap;

        let rows = zone_rows(now_utc, now_local, local_offset_minutes);

        for (index, row) in rows.iter().enumerate() {
            let row_y = y + index as f32 * (tokens.clock_row_height + tokens.clock_row_gap);
            draw_zone_row(scene, inner_x, row_y, inner_width, row, ctx);
        }
    }

    fn frame(theme: &Theme) -> DropdownFrame {
        DropdownFrame::new(theme.tokens.clock_panel_width, Self::height(theme))
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn zone_rows(now_utc: DateTime<Utc>, now_local: DateTime<Local>, local_offset_minutes: i32) -> Vec<ZoneRow> {
    let local_date = now_local.date_naive();

    WORLD_ZONES
        .iter()
        .map(|&(name, tz)| {
            let remote = now_utc.with_timezone(&tz);
            let remote_offset_minutes = zone_offset_minutes(now_utc, tz);

            ZoneRow {
                name,
                offset_text: offset_label(remote_offset_minutes - local_offset_minutes),
                time_text: remote.format("%H:%M").to_string(),
                tag: day_tag(local_date, remote.date_naive()),
            }
        })
        .collect()
}

fn draw_header(scene: &mut Scene, x: f32, y: f32, now_local: DateTime<Local>, local_offset_minutes: i32, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let header_height = tokens.clock_header_height;

    let time_size = ctx.theme.typography.size_base * tokens.clock_header_time_scale;
    let subtitle_size = ctx.theme.typography.size_base * SUBTITLE_TEXT_SCALE;

    let time_prefix = now_local.format("%H:%M").to_string();
    let time_seconds = now_local.format(":%S").to_string();

    let time_box_height = header_height * 0.6;

    let (prefix_width, _) = ctx.text.measure(&time_prefix, time_size, &ctx.theme.typography.font_family);

    ctx.text.draw_centered_v(
        scene,
        &time_prefix,
        x,
        y,
        time_box_height,
        TextStyle::new(time_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    ctx.text.draw_centered_v(
        scene,
        &time_seconds,
        x + prefix_width,
        y,
        time_box_height,
        TextStyle::new(time_size, &ctx.theme.typography.font_family, ctx.theme.palette.accent),
    );

    let subtitle = format!("{} · {}", local_zone_display_name(), utc_label(local_offset_minutes));

    ctx.text.draw_centered_v(
        scene,
        &subtitle,
        x,
        y + time_box_height,
        header_height - time_box_height,
        TextStyle::new(subtitle_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
}

fn draw_zone_row(scene: &mut Scene, x: f32, y: f32, width: f32, row: &ZoneRow, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let row_height = tokens.clock_row_height;

    let name_size = ctx.theme.typography.size_base;
    let offset_size = ctx.theme.typography.size_base * OFFSET_TEXT_SCALE;
    let time_size = ctx.theme.typography.size_base;

    let mut right = x + width;

    if let Some(tag) = row.tag {
        right = draw_day_tag(scene, right, y, row_height, tag, ctx) - tokens.clock_row_inner_gap;
    }

    let (time_width, _) = ctx.text.measure(&row.time_text, time_size, &ctx.theme.typography.font_family);
    let time_x = right - time_width;

    ctx.text.draw_centered_v(
        scene,
        &row.time_text,
        time_x,
        y,
        row_height,
        TextStyle::new(time_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    let (offset_width, _) = ctx.text.measure(&row.offset_text, offset_size, &ctx.theme.typography.font_family);
    let offset_x = time_x - tokens.clock_row_inner_gap - offset_width;

    ctx.text.draw_centered_v(
        scene,
        &row.offset_text,
        offset_x,
        y,
        row_height,
        TextStyle::new(offset_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    ctx.text.draw_centered_v(
        scene,
        row.name,
        x,
        y,
        row_height,
        TextStyle::new(name_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );
}

fn draw_day_tag(scene: &mut Scene, right: f32, row_y: f32, row_height: f32, tag: &str, ctx: &mut RenderCtx<'_>) -> f32 {
    let tokens = ctx.theme.tokens;
    let tag_size = ctx.theme.typography.size_base * TAG_TEXT_SCALE;

    let (text_width, text_height) = ctx.text.measure(tag, tag_size, &ctx.theme.typography.font_family);

    let tag_width = text_width + tokens.clock_tag_padding_x * 2.0;
    let tag_height = text_height + tokens.clock_tag_padding_y * 2.0;

    let tag_x = right - tag_width;
    let tag_y = row_y + (row_height - tag_height) / 2.0;

    let body = RoundedRect::new(
        tag_x as f64,
        tag_y as f64,
        (tag_x + tag_width) as f64,
        (tag_y + tag_height) as f64,
        tokens.clock_tag_radius as f64,
    );

    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.slot_active_bg, None, &body);

    ctx.text.draw_centered_v(
        scene,
        tag,
        tag_x + tokens.clock_tag_padding_x,
        tag_y,
        tag_height,
        TextStyle::new(tag_size, &ctx.theme.typography.font_family, ctx.theme.palette.slot_active_text),
    );

    tag_x
}
