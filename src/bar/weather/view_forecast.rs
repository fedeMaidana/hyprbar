// ─── < Imports > ────────────────────────────────────────────────────

use chrono::{Datelike, Local, Timelike};
use vello::Scene;
use vello::kurbo::{Affine, BezPath, Circle, Point as KurboPoint, RoundedRect, Stroke};
use vello::peniko::{Fill, Gradient};

use crate::components::RenderCtx;
use crate::locale::WEEKDAY_ABBREVS;
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::icons::{weather_icon, weather_icon_color};
use super::state::WeatherSnapshot;

// ─── < Constants > ────────────────────────────────────────────────────

const CURVE_H: f32 = 64.0;
const CURVE_LABEL_H: f32 = 16.0;
const CURVE_TOP_ALPHA: f32 = 0.3;
const CURVE_BOTTOM_ALPHA: f32 = 0.02;
const CURVE_STROKE: f64 = 2.0;
const NOW_DOT_RADIUS: f64 = 4.0;
const DASH_PATTERN: [f64; 2] = [3.0, 3.0];

const HOURS_ROW_ICON_H: f32 = 22.0;
const HOURS_ROW_LABEL_H: f32 = 16.0;
/// Franjas del día que llevan icono: cada 4 horas.
const HOUR_SLOTS: [u8; 6] = [0, 4, 8, 12, 16, 20];

const SECTION_GAP: f32 = 12.0;

const DAYS_CARD_RADIUS: f64 = 12.0;
const DAYS_CARD_PADDING: f32 = 6.0;
const DAY_LABEL_H: f32 = 20.0;
const DAY_ICON_H: f32 = 24.0;
const DAY_MAX_H: f32 = 22.0;
const DAY_MIN_H: f32 = 16.0;
const DAY_CELL_RADIUS: f64 = 10.0;
const VISIBLE_DAYS: usize = 5;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn height(_theme: &Theme) -> f32 {
    CURVE_LABEL_H + CURVE_H + CURVE_LABEL_H + SECTION_GAP + HOURS_ROW_ICON_H + HOURS_ROW_LABEL_H + SECTION_GAP + days_card_height()
}

pub(crate) fn draw(scene: &mut Scene, area: Rect, snapshot: &WeatherSnapshot, ctx: &mut RenderCtx<'_>) {
    let mut y = area.y;

    y = draw_curve(scene, area, y, snapshot, ctx);
    y += SECTION_GAP;

    y = draw_hour_icons(scene, area, y, snapshot, ctx);
    y += SECTION_GAP;

    draw_days_card(scene, Rect::new(area.x, y, area.width, days_card_height()), snapshot, ctx);
}

// ─── < Private Functions > ────────────────────────────────────────────────────

/// Curva de temperatura del día con el máximo etiquetado y un punto en
/// la hora actual. Devuelve el y final.
fn draw_curve(scene: &mut Scene, area: Rect, y: f32, snapshot: &WeatherSnapshot, ctx: &mut RenderCtx<'_>) -> f32 {
    let accent = ctx.theme.palette.accent;
    let chart = Rect::new(area.x, y + CURVE_LABEL_H, area.width, CURVE_H);

    let points = &snapshot.hourly;

    if points.len() < 2 {
        let size = ctx.theme.typography.size_base * 0.85;

        ctx.text.draw_centered(
            scene,
            "sin datos por hora todavía",
            chart,
            TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );

        return y + CURVE_LABEL_H + CURVE_H + CURVE_LABEL_H;
    }

    let (mut min, mut max) = (f32::MAX, f32::MIN);

    for point in points {
        min = min.min(point.temp_c);
        max = max.max(point.temp_c);
    }

    let span = (max - min).max(1.0);

    let position = |index: usize, temp: f32| {
        let x = chart.x + chart.width * index as f32 / (points.len() - 1) as f32;
        let fraction = (temp - min) / span;
        let point_y = chart.y + chart.height * (1.0 - fraction * 0.85 - 0.075);

        (x, point_y)
    };

    // Área + línea.
    let mut line = BezPath::new();
    let mut fill = BezPath::new();
    let bottom = (chart.y + chart.height) as f64;

    for (index, point) in points.iter().enumerate() {
        let (x, point_y) = position(index, point.temp_c);
        let kurbo_point = KurboPoint::new(x as f64, point_y as f64);

        if index == 0 {
            line.move_to(kurbo_point);
            fill.move_to(KurboPoint::new(x as f64, bottom));
        } else {
            line.line_to(kurbo_point);
        }

        fill.line_to(kurbo_point);
    }

    fill.line_to(KurboPoint::new((chart.x + chart.width) as f64, bottom));
    fill.close_path();

    let gradient = Gradient::new_linear((chart.x as f64, chart.y as f64), (chart.x as f64, bottom))
        .with_stops([accent.with_alpha(CURVE_TOP_ALPHA), accent.with_alpha(CURVE_BOTTOM_ALPHA)]);

    scene.fill(Fill::NonZero, Affine::IDENTITY, &gradient, None, &fill);
    scene.stroke(&Stroke::new(CURVE_STROKE), Affine::IDENTITY, accent, None, &line);

    let label_size = ctx.theme.typography.size_base * 0.78;
    let label_style = TextStyle::new(label_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary);

    // Máximo etiquetado encima de su punto.
    if let Some((max_index, max_point)) = points.iter().enumerate().max_by(|a, b| a.1.temp_c.total_cmp(&b.1.temp_c)) {
        let (max_x, _) = position(max_index, max_point.temp_c);
        let label = format!("{}°", max_point.temp_c.round() as i32);
        let (label_width, _) = ctx.text.measure(&label, label_size, ctx.theme.typography.font_family);

        let label_x = (max_x - label_width / 2.0).clamp(chart.x, chart.x + chart.width - label_width);

        ctx.text.draw_centered_v(scene, &label, label_x, y, CURVE_LABEL_H, label_style);
    }

    // Mínimo abajo a la izquierda.
    let min_label = format!("{}°", min.round() as i32);

    ctx.text.draw_centered_v(
        scene,
        &min_label,
        chart.x + 4.0,
        chart.y + chart.height,
        CURVE_LABEL_H,
        TextStyle::new(label_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    // La hora actual: línea punteada + punto sobre la curva.
    let now_hour = Local::now().hour() as u8;

    if let Some((index, point)) = points.iter().enumerate().find(|(_, point)| point.hour == now_hour) {
        let (x, point_y) = position(index, point.temp_c);

        let guide = vello::kurbo::Line::new((x as f64, chart.y as f64), (x as f64, bottom));
        let stroke = Stroke::new(1.0).with_dashes(0.0, DASH_PATTERN);

        scene.stroke(&stroke, Affine::IDENTITY, ctx.theme.palette.panel_divider, None, &guide);

        let dot = Circle::new((x as f64, point_y as f64), NOW_DOT_RADIUS);

        scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_bg, None, &dot);
        scene.stroke(&Stroke::new(2.0), Affine::IDENTITY, accent, None, &dot);
    }

    y + CURVE_LABEL_H + CURVE_H + CURVE_LABEL_H
}

/// Iconos del clima cada 4 horas, con su etiqueta. Devuelve el y final.
fn draw_hour_icons(scene: &mut Scene, area: Rect, y: f32, snapshot: &WeatherSnapshot, ctx: &mut RenderCtx<'_>) -> f32 {
    let slot_width = area.width / HOUR_SLOTS.len() as f32;
    let icon_size = ctx.theme.typography.size_base * 1.2;
    let label_size = ctx.theme.typography.size_base * 0.75;

    for (index, hour) in HOUR_SLOTS.iter().enumerate() {
        let slot = Rect::new(area.x + index as f32 * slot_width, y, slot_width, HOURS_ROW_ICON_H);

        if let Some(point) = snapshot.hourly.iter().find(|point| point.hour == *hour) {
            let icon = weather_icon(point.weather_code);
            let color = weather_icon_color(point.weather_code, &ctx.theme.palette);

            ctx.text
                .draw_centered(scene, icon, slot, TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, color));
        }

        let label = format!("{hour:02}");
        let label_slot = Rect::new(slot.x, y + HOURS_ROW_ICON_H, slot_width, HOURS_ROW_LABEL_H);

        ctx.text.draw_centered(
            scene,
            &label,
            label_slot,
            TextStyle::new(label_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );
    }

    y + HOURS_ROW_ICON_H + HOURS_ROW_LABEL_H
}

fn days_card_height() -> f32 {
    DAYS_CARD_PADDING * 2.0 + DAY_LABEL_H + DAY_ICON_H + DAY_MAX_H + DAY_MIN_H
}

/// Tarjeta con los próximos días; HOY va resaltado con borde accent.
fn draw_days_card(scene: &mut Scene, card: Rect, snapshot: &WeatherSnapshot, ctx: &mut RenderCtx<'_>) {
    let body =
        RoundedRect::new(card.x as f64, card.y as f64, (card.x + card.width) as f64, (card.y + card.height) as f64, DAYS_CARD_RADIUS);

    scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_inset, None, &body);

    let today = Local::now().date_naive();
    let days: Vec<_> = snapshot.daily.iter().take(VISIBLE_DAYS).collect();

    if days.is_empty() {
        return;
    }

    let cell_width = (card.width - DAYS_CARD_PADDING * 2.0) / days.len() as f32;
    let label_size = ctx.theme.typography.size_base * 0.72;
    let icon_size = ctx.theme.typography.size_base * 1.2;
    let max_size = ctx.theme.typography.size_base * 1.05;
    let min_size = ctx.theme.typography.size_base * 0.78;

    for (index, day) in days.iter().enumerate() {
        let cell = Rect::new(
            card.x + DAYS_CARD_PADDING + index as f32 * cell_width,
            card.y + DAYS_CARD_PADDING,
            cell_width,
            card.height - DAYS_CARD_PADDING * 2.0,
        );

        let is_today = day.date == today;

        if is_today {
            let highlight = RoundedRect::new(
                cell.x as f64,
                cell.y as f64,
                (cell.x + cell.width) as f64,
                (cell.y + cell.height) as f64,
                DAY_CELL_RADIUS,
            );

            scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_raised, None, &highlight);
            scene.stroke(&Stroke::new(1.0), Affine::IDENTITY, ctx.theme.palette.accent, None, &highlight);
        }

        let label = if is_today {
            "HOY".to_string()
        } else {
            WEEKDAY_ABBREVS[day.date.weekday().num_days_from_monday() as usize].to_uppercase()
        };

        let label_color = if is_today {
            ctx.theme.palette.accent
        } else {
            ctx.theme.palette.text_secondary
        };
        let mut y = cell.y;

        ctx.text.draw_centered(
            scene,
            &label,
            Rect::new(cell.x, y, cell.width, DAY_LABEL_H),
            TextStyle::new(label_size, ctx.theme.typography.font_family, label_color),
        );

        y += DAY_LABEL_H;

        let icon = weather_icon(day.weather_code);
        let icon_color = weather_icon_color(day.weather_code, &ctx.theme.palette);

        ctx.text.draw_centered(
            scene,
            icon,
            Rect::new(cell.x, y, cell.width, DAY_ICON_H),
            TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, icon_color),
        );

        y += DAY_ICON_H;

        ctx.text.draw_centered(
            scene,
            &format!("{}°", day.max_c.round() as i32),
            Rect::new(cell.x, y, cell.width, DAY_MAX_H),
            TextStyle::new(max_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
        );

        y += DAY_MAX_H;

        ctx.text.draw_centered(
            scene,
            &format!("{}°", day.min_c.round() as i32),
            Rect::new(cell.x, y, cell.width, DAY_MIN_H),
            TextStyle::new(min_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );
    }
}
