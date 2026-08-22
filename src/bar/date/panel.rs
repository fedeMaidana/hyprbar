// ─── < Imports > ────────────────────────────────────────────────────

use chrono::{Datelike, Local, NaiveDate};
use vello::Scene;
use vello::kurbo::{Affine, Circle, RoundedRect, Stroke};
use vello::peniko::Fill;

use crate::components::{DropdownFrame, Interaction, Panel, Point, RenderCtx};
use crate::locale::{WEEKDAY_HEADERS, month_abbrev, month_name, weekday_abbrev};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::CalendarAction;
use super::grid::{GRID_COLUMNS, GRID_ROWS, month_grid, rows_in_month, shift_month, week_number};

// ─── < Constants > ────────────────────────────────────────────────────

const NAV_PREV_GLYPH: &str = "\u{f0141}";
const NAV_NEXT_GLYPH: &str = "\u{f0142}";

const WEEKDAY_TEXT_SCALE: f32 = 0.7;
const WEEK_NUMBER_TEXT_SCALE: f32 = 0.68;
const WEEK_NUMBER_ALPHA: f32 = 0.55;
const FOOTER_TEXT_SCALE: f32 = 0.8;

/// Aire entre el nombre del mes y el año del título.
const TITLE_GAP: f32 = 6.0;

const CELL_RADIUS: f64 = 9.0;
/// Borde punteado de las celdas sin día.
const DASH_PATTERN: [f64; 2] = [3.0, 3.0];

const FOOTER_DOT_RADIUS: f32 = 3.0;
const FOOTER_DOT_GAP: f32 = 8.0;

/// Aire entre la banda de la semana actual y sus celdas.
const WEEK_BAND_PADDING: f32 = 2.5;

const TODAY_BUTTON_W: f32 = 52.0;
const TODAY_BUTTON_H: f32 = 26.0;
const TODAY_BUTTON_RADIUS: f64 = 8.0;

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
        let tokens = theme.tokens;
        let columns = GRID_COLUMNS as f32;

        tokens.dropdown_panel_padding_x * 2.0
            + tokens.date_week_col_width
            + tokens.date_cell_gap
            + tokens.date_cell_size * columns
            + tokens.date_cell_gap * (columns - 1.0)
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
            + tokens.dropdown_section_gap
            + tokens.date_footer_height
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
        draw_header(scene, inner_x, inner_width, y, &view, ctx);

        y += tokens.dropdown_header_height + tokens.dropdown_section_gap;

        draw_weekday_headers(scene, inner_x, y, &view, ctx);

        y += tokens.date_weekday_row_height;

        draw_day_grid(scene, inner_x, y, &view, ctx);

        draw_footer(scene, bounds, inner_x, inner_width, &view, ctx);
    }

    fn hit_test_content(&self, point: Point, bounds: Rect, theme: &Theme) -> Option<Interaction> {
        for (action, rect) in nav_button_rects(bounds, theme) {
            if rect.contains_point(point.x, point.y) {
                return Some(action.interaction());
            }
        }

        if today_button_rect(bounds, theme).contains_point(point.x, point.y) {
            return Some(CalendarAction::Today.interaction());
        }

        None
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

/// "agosto" en primario + "2026" en secundario, centrados como grupo.
fn draw_header(scene: &mut Scene, inner_x: f32, inner_width: f32, y: f32, view: &MonthView, ctx: &mut RenderCtx<'_>) {
    let item_height = ctx.theme.tokens.dropdown_header_height;
    let title_size = ctx.theme.typography.size_base * ctx.theme.tokens.dropdown_title_scale;

    let month_label = month_name(view.month);
    let year_label = view.year.to_string();

    let (month_width, _) = ctx.text.measure(month_label, title_size, ctx.theme.typography.font_family);
    let (year_width, _) = ctx.text.measure(&year_label, title_size, ctx.theme.typography.font_family);

    let text_x = inner_x + (inner_width - month_width - TITLE_GAP - year_width) / 2.0;

    ctx.text.draw_centered_v(
        scene,
        month_label,
        text_x,
        y,
        item_height,
        TextStyle::new(title_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    ctx.text.draw_centered_v(
        scene,
        &year_label,
        text_x + month_width + TITLE_GAP,
        y,
        item_height,
        TextStyle::new(title_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
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
        scene.stroke(&Stroke::new(1.0), Affine::IDENTITY, ctx.theme.palette.pill_border, None, &body);

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

/// D L M X J V S; el día de hoy va en accent.
fn draw_weekday_headers(scene: &mut Scene, inner_x: f32, y: f32, view: &MonthView, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let size = ctx.theme.typography.size_base * WEEKDAY_TEXT_SCALE;

    let today_column = view
        .viewing_current_month
        .then(|| view.today.weekday().num_days_from_sunday() as usize);

    let grid_x = inner_x + tokens.date_week_col_width + tokens.date_cell_gap;

    for (index, header) in WEEKDAY_HEADERS.iter().enumerate() {
        let cell_x = grid_x + index as f32 * (tokens.date_cell_size + tokens.date_cell_gap);

        let color = if today_column == Some(index) {
            ctx.theme.palette.accent
        } else {
            ctx.theme.palette.text_secondary
        };

        let (text_width, _) = ctx.text.measure(header, size, ctx.theme.typography.font_family);

        ctx.text.draw_centered_v(
            scene,
            header,
            cell_x + (tokens.date_cell_size - text_width) / 2.0,
            y,
            tokens.date_weekday_row_height,
            TextStyle::new(size, ctx.theme.typography.font_family, color),
        );
    }
}

fn draw_day_grid(scene: &mut Scene, inner_x: f32, y: f32, view: &MonthView, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let size = ctx.theme.typography.size_base * tokens.dropdown_body_scale;
    let week_size = ctx.theme.typography.size_base * WEEK_NUMBER_TEXT_SCALE;

    let MonthView {
        year,
        month,
        today,
        viewing_current_month,
    } = *view;

    let grid = month_grid(year, month);
    let visible_rows = rows_in_month(year, month);
    let grid_x = inner_x + tokens.date_week_col_width + tokens.date_cell_gap;
    let step = tokens.date_cell_size + tokens.date_cell_gap;

    let today_day = (viewing_current_month && today.year() == year && today.month() == month).then(|| today.day() as u8);

    let today_row = today_day
        .and_then(|day| grid.iter().position(|cell| *cell == Some(day)))
        .map(|index| index / GRID_COLUMNS);

    if let Some(row) = today_row {
        draw_week_band(scene, grid_x, y + row as f32 * step, ctx);
    }

    // Números de semana del año, uno por fila.
    for row in 0..visible_rows {
        let Some(week) = week_number(year, month, row) else {
            continue;
        };

        let label = week.to_string();

        let color = if today_row == Some(row) {
            ctx.theme.palette.accent
        } else {
            ctx.theme.palette.text_secondary.with_alpha(WEEK_NUMBER_ALPHA)
        };

        let (text_width, _) = ctx.text.measure(&label, week_size, ctx.theme.typography.font_family);

        ctx.text.draw_centered_v(
            scene,
            &label,
            inner_x + (tokens.date_week_col_width - text_width) / 2.0,
            y + row as f32 * step,
            tokens.date_cell_size,
            TextStyle::new(week_size, ctx.theme.typography.font_family, color),
        );
    }

    let visible_cells = visible_rows * GRID_COLUMNS;

    for (index, cell) in grid.iter().enumerate().take(visible_cells) {
        let column = index % GRID_COLUMNS;
        let row = index / GRID_COLUMNS;

        let cell_x = grid_x + column as f32 * step;
        let cell_y = y + row as f32 * step;

        let tile = RoundedRect::new(
            cell_x as f64,
            cell_y as f64,
            (cell_x + tokens.date_cell_size) as f64,
            (cell_y + tokens.date_cell_size) as f64,
            CELL_RADIUS,
        );

        let Some(day) = cell else {
            // Sin día: solo el contorno punteado.
            let stroke = Stroke::new(1.0).with_dashes(0.0, DASH_PATTERN);

            scene.stroke(&stroke, Affine::IDENTITY, ctx.theme.palette.pill_border, None, &tile);

            continue;
        };

        let is_today = today_day == Some(*day);
        let is_weekend = column == 0 || column == GRID_COLUMNS - 1;

        let background = if is_today {
            ctx.theme.palette.slot_active_bg
        } else {
            ctx.theme.palette.panel_raised
        };

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &tile);

        let color = if is_today {
            ctx.theme.palette.slot_active_text
        } else if is_weekend {
            ctx.theme.palette.text_secondary
        } else {
            ctx.theme.palette.text_primary
        };

        let label = day.to_string();
        let (text_width, _) = ctx.text.measure(&label, size, ctx.theme.typography.font_family);

        ctx.text.draw_centered_v(
            scene,
            &label,
            cell_x + (tokens.date_cell_size - text_width) / 2.0,
            cell_y,
            tokens.date_cell_size,
            TextStyle::new(size, ctx.theme.typography.font_family, color),
        );
    }
}

/// Banda sutil detrás de la semana actual; sumada a los tiles, la fila
/// entera queda un paso más clara.
fn draw_week_band(scene: &mut Scene, grid_x: f32, band_y: f32, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let columns = GRID_COLUMNS as f32;

    let band_width = tokens.date_cell_size * columns + tokens.date_cell_gap * (columns - 1.0);
    let radius = tokens.date_week_radius as f64;

    let band = RoundedRect::new(
        (grid_x - WEEK_BAND_PADDING) as f64,
        (band_y - WEEK_BAND_PADDING) as f64,
        (grid_x + band_width + WEEK_BAND_PADDING) as f64,
        (band_y + tokens.date_cell_size + WEEK_BAND_PADDING) as f64,
        radius,
    );

    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_raised, None, &band);
}

/// Punto accent + fecha de hoy a la izquierda, botón "Hoy" a la derecha.
fn draw_footer(scene: &mut Scene, bounds: Rect, inner_x: f32, inner_width: f32, view: &MonthView, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let footer_y = bounds.y + bounds.height - tokens.dropdown_panel_padding_y - tokens.date_footer_height;

    DropdownFrame::draw_divider(scene, inner_x, footer_y - tokens.dropdown_section_gap / 2.0, inner_width, ctx.theme);

    let dot_center_y = footer_y + tokens.date_footer_height / 2.0;
    let dot = Circle::new(((inner_x + FOOTER_DOT_RADIUS) as f64, dot_center_y as f64), FOOTER_DOT_RADIUS as f64);

    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.accent, None, &dot);

    let today = view.today;

    let label = format!(
        "{} {} {} {}",
        weekday_abbrev(today.weekday().num_days_from_monday() as usize),
        today.day(),
        month_abbrev(today.month()),
        today.year(),
    );

    let size = ctx.theme.typography.size_base * FOOTER_TEXT_SCALE;

    ctx.text.draw_centered_v(
        scene,
        &label,
        inner_x + FOOTER_DOT_RADIUS * 2.0 + FOOTER_DOT_GAP,
        footer_y,
        tokens.date_footer_height,
        TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    // Botón "Hoy".
    let rect = today_button_rect(bounds, ctx.theme);
    let is_hovered = ctx.hovered_interaction == Some(CalendarAction::Today.interaction());

    let background = if is_hovered {
        ctx.theme.palette.control_hover_bg
    } else {
        ctx.theme.palette.control_bg
    };

    let body =
        RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, TODAY_BUTTON_RADIUS);

    scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);
    scene.stroke(&Stroke::new(1.0), Affine::IDENTITY, ctx.theme.palette.pill_border, None, &body);

    let (text_width, _) = ctx.text.measure("Hoy", size, ctx.theme.typography.font_family);

    ctx.text.draw_centered_v(
        scene,
        "Hoy",
        rect.x + (rect.width - text_width) / 2.0,
        rect.y,
        rect.height,
        TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );
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

fn today_button_rect(bounds: Rect, theme: &Theme) -> Rect {
    let tokens = theme.tokens;

    let x = bounds.x + bounds.width - tokens.dropdown_panel_padding_x - TODAY_BUTTON_W;
    let footer_y = bounds.y + bounds.height - tokens.dropdown_panel_padding_y - tokens.date_footer_height;
    let y = footer_y + (tokens.date_footer_height - TODAY_BUTTON_H) / 2.0;

    Rect::new(x, y, TODAY_BUTTON_W, TODAY_BUTTON_H)
}
