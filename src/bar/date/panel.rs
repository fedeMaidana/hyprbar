// ─── < Imports > ────────────────────────────────────────────────────

use chrono::{Datelike, Local, NaiveDate};
use vello::Scene;
use vello::kurbo::{Affine, Circle, RoundedRect};
use vello::peniko::Fill;

use crate::components::{DropdownFrame, Interaction, Point, RenderCtx};
use crate::locale::{WEEKDAY_HEADERS, month_name, weekday_name};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::CalendarAction;
use super::grid::{GRID_CELLS, GRID_COLUMNS, GRID_ROWS, month_grid, shift_month};

// ─── < Constants > ────────────────────────────────────────────────────

const NAV_PREV_GLYPH: &str = "\u{f0141}";
const NAV_NEXT_GLYPH: &str = "\u{f0142}";

const TODAY_SUBTITLE: &str = "volver a hoy";

const DAY_TEXT_SCALE: f32 = 0.85;
const WEEKDAY_TEXT_SCALE: f32 = 0.7;
const SUBTITLE_TEXT_SCALE: f32 = 0.78;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct DatePanel;

struct HeaderContent {
    title: String,
    subtitle: String,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl DatePanel {
    pub fn width(theme: &Theme) -> f32 {
        let tokens = theme.tokens;

        tokens.date_panel_padding_x * 2.0 + tokens.date_cell_size * GRID_COLUMNS as f32 + tokens.date_cell_gap * (GRID_COLUMNS - 1) as f32
    }

    pub fn height(theme: &Theme) -> f32 {
        let tokens = theme.tokens;

        tokens.date_panel_padding_y * 2.0
            + tokens.dropdown_item_height
            + tokens.date_header_gap
            + tokens.date_weekday_row_height
            + tokens.date_cell_size * GRID_ROWS as f32
            + tokens.date_cell_gap * (GRID_ROWS - 1) as f32
    }

    pub fn bounds(surface: Rect, anchor: Rect, theme: &Theme) -> Rect {
        Self::frame(theme).bounds(surface, anchor, theme)
    }

    pub fn hit_test(point: Point, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Interaction> {
        let bounds = Self::bounds(surface, anchor, theme);

        for (action, rect) in nav_button_rects(bounds, theme) {
            if rect.contains_point(point.x, point.y) {
                return Some(Interaction::Calendar(action));
            }
        }

        if today_hit_rect(bounds, theme).contains_point(point.x, point.y) {
            return Some(Interaction::Calendar(CalendarAction::Today));
        }

        None
    }

    pub fn draw(scene: &mut Scene, surface: Rect, anchor: Rect, month_offset: i32, ctx: &mut RenderCtx<'_>) {
        let theme = ctx.theme;
        let tokens = theme.tokens;

        let frame = Self::frame(theme);
        let bounds = frame.bounds(surface, anchor, theme);

        frame.draw_background(scene, bounds, theme);

        let today = Local::now().date_naive();
        let (year, month) = shift_month(today.year(), today.month(), month_offset);
        let viewing_current_month = month_offset == 0;

        let inner_x = bounds.x + tokens.date_panel_padding_x;
        let mut y = bounds.y + tokens.date_panel_padding_y;

        let header = header_content(today, year, month, viewing_current_month);

        draw_header(scene, inner_x, y, &header, ctx);
        draw_nav_buttons(scene, bounds, ctx);

        y += tokens.dropdown_item_height + tokens.date_header_gap;

        draw_weekday_headers(scene, inner_x, y, ctx);

        y += tokens.date_weekday_row_height;

        draw_day_grid(scene, inner_x, y, year, month, today, viewing_current_month, ctx);
    }

    fn frame(theme: &Theme) -> DropdownFrame {
        DropdownFrame::new(Self::width(theme), Self::height(theme))
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn header_content(today: NaiveDate, year: i32, month: u32, viewing_current_month: bool) -> HeaderContent {
    if viewing_current_month {
        let weekday = weekday_name(today.weekday().num_days_from_monday() as usize);

        return HeaderContent {
            title: format!("{weekday} {} de {}", today.day(), month_name(today.month())),
            subtitle: today.year().to_string(),
        };
    }

    HeaderContent {
        title: format!("{} {year}", month_name(month)),
        subtitle: TODAY_SUBTITLE.to_string(),
    }
}

fn draw_header(scene: &mut Scene, x: f32, y: f32, header: &HeaderContent, ctx: &mut RenderCtx<'_>) {
    let item_height = ctx.theme.tokens.dropdown_item_height;
    let title_size = ctx.theme.typography.size_base;
    let subtitle_size = title_size * SUBTITLE_TEXT_SCALE;

    ctx.text.draw_centered_v(
        scene,
        &header.title,
        x,
        y,
        item_height * 0.55,
        TextStyle::new(title_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    ctx.text.draw_centered_v(
        scene,
        &header.subtitle,
        x,
        y + item_height * 0.45,
        item_height * 0.45,
        TextStyle::new(subtitle_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
}

fn draw_nav_buttons(scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
    let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
    let radius = ctx.theme.tokens.date_nav_button_radius as f64;

    for (action, rect) in nav_button_rects(bounds, ctx.theme) {
        let is_hovered = ctx.hovered_interaction == Some(Interaction::Calendar(action));

        let background = if is_hovered {
            ctx.theme.palette.slot_hover_bg
        } else {
            ctx.theme.palette.slot_inactive_bg
        };

        let body = RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        let glyph = match action {
            CalendarAction::PrevMonth => NAV_PREV_GLYPH,
            _ => NAV_NEXT_GLYPH,
        };

        let (glyph_width, _) = ctx.text.measure(glyph, icon_size, &ctx.theme.typography.icon_font_family);

        ctx.text.draw_centered_v(
            scene,
            glyph,
            rect.x + (rect.width - glyph_width) / 2.0,
            rect.y,
            rect.height,
            TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, ctx.theme.palette.text_primary),
        );
    }
}

fn draw_weekday_headers(scene: &mut Scene, x: f32, y: f32, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let size = ctx.theme.typography.size_base * WEEKDAY_TEXT_SCALE;

    for (index, header) in WEEKDAY_HEADERS.iter().enumerate() {
        let cell_x = x + index as f32 * (tokens.date_cell_size + tokens.date_cell_gap);

        let (text_width, _) = ctx.text.measure(header, size, &ctx.theme.typography.font_family);

        ctx.text.draw_centered_v(
            scene,
            header,
            cell_x + (tokens.date_cell_size - text_width) / 2.0,
            y,
            tokens.date_weekday_row_height,
            TextStyle::new(size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_day_grid(
    scene: &mut Scene,
    x: f32,
    y: f32,
    year: i32,
    month: u32,
    today: NaiveDate,
    viewing_current_month: bool,
    ctx: &mut RenderCtx<'_>,
) {
    let tokens = ctx.theme.tokens;
    let size = ctx.theme.typography.size_base * DAY_TEXT_SCALE;

    let grid = month_grid(year, month);
    let today_day = if viewing_current_month && today.year() == year && today.month() == month {
        Some(today.day() as u8)
    } else {
        None
    };

    if let Some(day) = today_day
        && let Some(row) = grid.iter().position(|cell| *cell == Some(day)).map(|index| index / GRID_COLUMNS)
    {
        draw_week_band(scene, x, y, row, ctx);
    }

    for (index, cell) in grid.iter().enumerate().take(GRID_CELLS) {
        let Some(day) = cell else {
            continue;
        };

        let column = index % GRID_COLUMNS;
        let row = index / GRID_COLUMNS;

        let cell_x = x + column as f32 * (tokens.date_cell_size + tokens.date_cell_gap);
        let cell_y = y + row as f32 * (tokens.date_cell_size + tokens.date_cell_gap);

        let is_today = today_day == Some(*day);
        let is_weekend = column >= 5;

        if is_today {
            let center_x = cell_x + tokens.date_cell_size / 2.0;
            let center_y = cell_y + tokens.date_cell_size / 2.0;
            let radius = tokens.date_cell_size / 2.0 * tokens.date_today_marker_scale;

            let marker = Circle::new((center_x as f64, center_y as f64), radius as f64);
            scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.slot_active_bg, None, &marker);
        }

        let color = if is_today {
            ctx.theme.palette.slot_active_text
        } else if is_weekend {
            ctx.theme.palette.text_secondary
        } else {
            ctx.theme.palette.text_primary
        };

        let label = day.to_string();
        let (text_width, _) = ctx.text.measure(&label, size, &ctx.theme.typography.font_family);

        ctx.text.draw_centered_v(
            scene,
            &label,
            cell_x + (tokens.date_cell_size - text_width) / 2.0,
            cell_y,
            tokens.date_cell_size,
            TextStyle::new(size, &ctx.theme.typography.font_family, color),
        );
    }
}

fn draw_week_band(scene: &mut Scene, x: f32, y: f32, row: usize, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;

    let band_width = tokens.date_cell_size * GRID_COLUMNS as f32 + tokens.date_cell_gap * (GRID_COLUMNS as f32 - 1.0);
    let band_y = y + row as f32 * (tokens.date_cell_size + tokens.date_cell_gap);

    let radius = tokens.date_week_radius as f64;

    let band = RoundedRect::new(x as f64, band_y as f64, (x + band_width) as f64, (band_y + tokens.date_cell_size) as f64, radius);

    let color = ctx.theme.palette.accent.with_alpha(tokens.date_week_highlight_alpha);

    scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &band);
}

fn nav_button_rects(bounds: Rect, theme: &Theme) -> [(CalendarAction, Rect); 2] {
    let tokens = theme.tokens;

    let size = tokens.date_nav_button_size;
    let header_y = bounds.y + tokens.date_panel_padding_y;
    let y = header_y + (tokens.dropdown_item_height - size) / 2.0;

    let next_x = bounds.x + bounds.width - tokens.date_panel_padding_x - size;
    let prev_x = next_x - tokens.date_nav_button_gap - size;

    [
        (CalendarAction::PrevMonth, Rect::new(prev_x, y, size, size)),
        (CalendarAction::NextMonth, Rect::new(next_x, y, size, size)),
    ]
}

fn today_hit_rect(bounds: Rect, theme: &Theme) -> Rect {
    let tokens = theme.tokens;

    let x = bounds.x + tokens.date_panel_padding_x;
    let y = bounds.y + tokens.date_panel_padding_y;

    let [(_, prev_rect), _] = nav_button_rects(bounds, theme);
    let width = (prev_rect.x - tokens.date_nav_button_gap - x).max(0.0);

    Rect::new(x, y, width, tokens.dropdown_item_height)
}
