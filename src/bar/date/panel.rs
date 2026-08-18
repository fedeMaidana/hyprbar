// ─── < Imports > ────────────────────────────────────────────────────

use chrono::{Datelike, Local, NaiveDate};
use vello::Scene;
use vello::kurbo::{Affine, Circle, RoundedRect};
use vello::peniko::Fill;

use crate::components::{DropdownFrame, Interaction, Panel, Point, RenderCtx};
use crate::locale::{WEEKDAY_HEADERS, month_name};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::CalendarAction;
use super::grid::{GRID_COLUMNS, GRID_ROWS, days_in_month, monday_offset, month_grid, rows_in_month, shift_month};

// ─── < Constants > ────────────────────────────────────────────────────

const NAV_PREV_GLYPH: &str = "\u{f0141}";
const NAV_NEXT_GLYPH: &str = "\u{f0142}";

const WEEKDAY_TEXT_SCALE: f32 = 0.7;

const PAST_DAY_ALPHA: f32 = 0.55;
const ADJACENT_DAY_ALPHA: f32 = 0.35;

const TITLE_HOVER_PAD_X: f32 = 10.0;
const TITLE_HOVER_HEIGHT: f32 = 26.0;
const TITLE_HOVER_RADIUS: f32 = 8.0;
const TODAY_HINT_DOT_RADIUS: f32 = 2.0;
const TODAY_HINT_DOT_OFFSET: f32 = 1.0;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct DatePanel {
    pub month_offset: i32,
}

/// The month the panel is currently looking at.
struct MonthView {
    year: i32,
    month: u32,
    today: NaiveDate,
    viewing_current_month: bool,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl DatePanel {
    pub fn width(theme: &Theme) -> f32 {
        theme.tokens.dropdown_panel_width
    }

    pub fn max_height(theme: &Theme) -> f32 {
        Self::height_for_rows(theme, GRID_ROWS)
    }

    pub fn height(theme: &Theme, month_offset: i32) -> f32 {
        let today = Local::now().date_naive();
        let (year, month) = shift_month(today.year(), today.month(), month_offset);

        Self::height_for_rows(theme, rows_in_month(year, month))
    }

    fn height_for_rows(theme: &Theme, rows: usize) -> f32 {
        let tokens = theme.tokens;
        let rows = rows as f32;

        tokens.dropdown_panel_padding_y * 2.0
            + tokens.dropdown_header_height
            + tokens.dropdown_section_gap
            + tokens.date_weekday_row_height
            + tokens.date_cell_size * rows
            + tokens.date_cell_gap * (rows - 1.0)
    }

    fn month_view(&self) -> MonthView {
        let today = Local::now().date_naive();
        let (year, month) = shift_month(today.year(), today.month(), self.month_offset);

        MonthView {
            year,
            month,
            today,
            viewing_current_month: self.month_offset == 0,
        }
    }
}

impl Panel for DatePanel {
    fn frame(&self, theme: &Theme) -> DropdownFrame {
        DropdownFrame::new(Self::width(theme), Self::height(theme, self.month_offset))
    }

    fn draw_content(&self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        let theme = ctx.theme;
        let tokens = theme.tokens;

        let view = self.month_view();

        let inner_x = bounds.x + tokens.dropdown_panel_padding_x;
        let inner_width = bounds.width - tokens.dropdown_panel_padding_x * 2.0;
        let mut y = bounds.y + tokens.dropdown_panel_padding_y;

        draw_nav_buttons(scene, bounds, ctx);
        draw_header(scene, inner_x, inner_width, y, &header_label(view.year, view.month), view.viewing_current_month, ctx);

        y += tokens.dropdown_header_height;

        DropdownFrame::draw_divider(scene, inner_x, y + tokens.dropdown_section_gap / 2.0, inner_width, theme);

        y += tokens.dropdown_section_gap;

        draw_weekday_headers(scene, inner_x, y, ctx);

        y += tokens.date_weekday_row_height;

        draw_day_grid(scene, inner_x, y, &view, ctx);
    }

    fn hit_test_content(&self, point: Point, bounds: Rect, theme: &Theme) -> Option<Interaction> {
        for (action, rect) in nav_button_rects(bounds, theme) {
            if rect.contains_point(point.x, point.y) {
                return Some(action.interaction());
            }
        }

        if today_hit_rect(bounds, theme).contains_point(point.x, point.y) {
            return Some(CalendarAction::Today.interaction());
        }

        None
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn header_label(year: i32, month: u32) -> String {
    format!("{} {year}", month_name(month))
}

fn draw_header(
    scene: &mut Scene,
    inner_x: f32,
    inner_width: f32,
    y: f32,
    label: &str,
    viewing_current_month: bool,
    ctx: &mut RenderCtx<'_>,
) {
    let item_height = ctx.theme.tokens.dropdown_header_height;
    let title_size = ctx.theme.typography.size_base * ctx.theme.tokens.dropdown_title_scale;

    let (text_width, _) = ctx.text.measure(label, title_size, ctx.theme.typography.font_family);
    let text_x = inner_x + (inner_width - text_width) / 2.0;

    // The title doubles as a "back to today" button; hover makes that visible.
    if ctx.hovered_interaction == Some(CalendarAction::Today.interaction()) {
        let pill_x = text_x - TITLE_HOVER_PAD_X;
        let pill_y = y + (item_height - TITLE_HOVER_HEIGHT) / 2.0;
        let pill_width = text_width + TITLE_HOVER_PAD_X * 2.0;

        let body = RoundedRect::new(
            pill_x as f64,
            pill_y as f64,
            (pill_x + pill_width) as f64,
            (pill_y + TITLE_HOVER_HEIGHT) as f64,
            TITLE_HOVER_RADIUS as f64,
        );

        scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.control_hover_bg, None, &body);
    }

    ctx.text.draw_centered_v(
        scene,
        label,
        text_x,
        y,
        item_height,
        TextStyle::new(title_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    // Accent dot hints that a shortcut back to the current month exists.
    if !viewing_current_month {
        let center_x = (inner_x + inner_width / 2.0) as f64;
        let center_y = (y + item_height - TODAY_HINT_DOT_OFFSET) as f64;

        let dot = Circle::new((center_x, center_y), TODAY_HINT_DOT_RADIUS as f64);

        scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.accent, None, &dot);
    }
}

fn draw_nav_buttons(scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
    let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
    let radius = ctx.theme.tokens.date_nav_button_radius as f64;

    for (action, rect) in nav_button_rects(bounds, ctx.theme) {
        let is_hovered = ctx.hovered_interaction == Some(action.interaction());

        let background = if is_hovered {
            ctx.theme.palette.control_hover_bg
        } else {
            ctx.theme.palette.control_bg
        };

        let body = RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        let glyph = match action {
            CalendarAction::PrevMonth => NAV_PREV_GLYPH,
            _ => NAV_NEXT_GLYPH,
        };

        let (glyph_width, _) = ctx.text.measure(glyph, icon_size, ctx.theme.typography.icon_font_family);

        ctx.text.draw_centered_v(
            scene,
            glyph,
            rect.x + (rect.width - glyph_width) / 2.0,
            rect.y,
            rect.height,
            TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, ctx.theme.palette.text_primary),
        );
    }
}

fn draw_weekday_headers(scene: &mut Scene, x: f32, y: f32, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let size = ctx.theme.typography.size_base * WEEKDAY_TEXT_SCALE;

    for (index, header) in WEEKDAY_HEADERS.iter().enumerate() {
        let cell_x = x + index as f32 * (tokens.date_cell_size + tokens.date_cell_gap);

        let (text_width, _) = ctx.text.measure(header, size, ctx.theme.typography.font_family);

        ctx.text.draw_centered_v(
            scene,
            header,
            cell_x + (tokens.date_cell_size - text_width) / 2.0,
            y,
            tokens.date_weekday_row_height,
            TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );
    }
}

fn draw_day_grid(scene: &mut Scene, x: f32, y: f32, view: &MonthView, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let size = ctx.theme.typography.size_base * tokens.dropdown_body_scale;

    let MonthView {
        year,
        month,
        today,
        viewing_current_month,
    } = *view;

    let grid = month_grid(year, month);
    let start = monday_offset(year, month) as usize;
    let day_count = days_in_month(year, month) as usize;
    let visible_cells = rows_in_month(year, month) * GRID_COLUMNS;

    let (prev_year, prev_month) = shift_month(year, month, -1);
    let prev_month_days = days_in_month(prev_year, prev_month) as usize;

    let today_day = if viewing_current_month && today.year() == year && today.month() == month {
        Some(today.day() as u8)
    } else {
        None
    };

    let today_row = today_day
        .and_then(|day| grid.iter().position(|cell| *cell == Some(day)))
        .map(|index| index / GRID_COLUMNS);

    if let Some(row) = today_row {
        draw_week_band(scene, x, y, row, ctx);
    }

    for (index, cell) in grid.iter().enumerate().take(visible_cells) {
        let column = index % GRID_COLUMNS;
        let row = index / GRID_COLUMNS;

        let cell_x = x + column as f32 * (tokens.date_cell_size + tokens.date_cell_gap);
        let cell_y = y + row as f32 * (tokens.date_cell_size + tokens.date_cell_gap);

        let (label, color) = match cell {
            Some(day) => {
                let is_today = today_day == Some(*day);
                let is_weekend = column >= 5;
                let is_past = NaiveDate::from_ymd_opt(year, month, *day as u32).is_some_and(|date| date < today);

                if is_today {
                    draw_today_marker(scene, cell_x, cell_y, ctx);
                }

                // Three levels: past < future weekend < future weekday.
                let color = if is_today {
                    ctx.theme.palette.slot_active_text
                } else if is_past {
                    ctx.theme.palette.text_secondary.with_alpha(PAST_DAY_ALPHA)
                } else if is_weekend {
                    ctx.theme.palette.text_secondary
                } else {
                    ctx.theme.palette.text_primary
                };

                (day.to_string(), color)
            }
            None => {
                // Ghost days from the adjacent months keep the grid complete.
                let adjacent_day = if index < start {
                    prev_month_days - start + index + 1
                } else {
                    index - start - day_count + 1
                };

                (adjacent_day.to_string(), ctx.theme.palette.text_secondary.with_alpha(ADJACENT_DAY_ALPHA))
            }
        };

        let (text_width, _) = ctx.text.measure(&label, size, ctx.theme.typography.font_family);
        let text_x = cell_x + (tokens.date_cell_size - text_width) / 2.0;

        ctx.text.draw_centered_v(
            scene,
            &label,
            text_x,
            cell_y,
            tokens.date_cell_size,
            TextStyle::new(size, ctx.theme.typography.font_family, color),
        );
    }
}

fn draw_today_marker(scene: &mut Scene, cell_x: f32, cell_y: f32, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;

    let center_x = cell_x + tokens.date_cell_size / 2.0;
    let center_y = cell_y + tokens.date_cell_size / 2.0;
    let radius = tokens.date_cell_size / 2.0 * tokens.date_today_marker_scale;

    let marker = Circle::new((center_x as f64, center_y as f64), radius as f64);

    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.slot_active_bg, None, &marker);
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
    let header_y = bounds.y + tokens.dropdown_panel_padding_y;
    let y = header_y + (tokens.dropdown_header_height - size) / 2.0;

    let prev_x = bounds.x + tokens.dropdown_panel_padding_x;
    let next_x = bounds.x + bounds.width - tokens.dropdown_panel_padding_x - size;

    [
        (CalendarAction::PrevMonth, Rect::new(prev_x, y, size, size)),
        (CalendarAction::NextMonth, Rect::new(next_x, y, size, size)),
    ]
}

fn today_hit_rect(bounds: Rect, theme: &Theme) -> Rect {
    let tokens = theme.tokens;

    let [(_, prev_rect), (_, next_rect)] = nav_button_rects(bounds, theme);

    let x = prev_rect.x + prev_rect.width + tokens.date_nav_button_gap;
    let y = bounds.y + tokens.dropdown_panel_padding_y;
    let width = (next_rect.x - tokens.date_nav_button_gap - x).max(0.0);

    Rect::new(x, y, width, tokens.dropdown_header_height)
}
