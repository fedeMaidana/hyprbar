// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, Circle, RoundedRect, Stroke};
use vello::peniko::{Color, Fill, Gradient};

use crate::components::RenderCtx;
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::state::{AirQuality, UvInfo};

// ─── < Constants > ────────────────────────────────────────────────────

const LABEL_H: f32 = 18.0;
const BAR_H: f32 = 8.0;
const BAR_GAP: f32 = 8.0;
const SCALE_H: f32 = 16.0;
const ADVICE_H: f32 = 18.0;
const SECTION_GAP: f32 = 18.0;

const KNOB_RADIUS: f64 = 6.5;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn height(_theme: &Theme) -> f32 {
    section_height() + SECTION_GAP + section_height()
}

pub(crate) fn draw(scene: &mut Scene, area: Rect, uv: Option<&UvInfo>, air: Option<&AirQuality>, ctx: &mut RenderCtx<'_>) {
    let mut y = area.y;

    // ÍNDICE UV.
    let (uv_value, uv_level) = match uv {
        Some(uv) => (format!("{}", uv.current.round() as i32), uv_level_label(uv.current)),
        None => ("—".to_string(), ""),
    };

    draw_section_header(scene, area, y, "ÍNDICE UV", &uv_value, uv_level, ctx);
    y += LABEL_H + BAR_GAP;

    let uv_fraction = uv.map(|uv| (uv.current / 11.0).clamp(0.0, 1.0));

    draw_scale_bar(scene, Rect::new(area.x, y, area.width, BAR_H), uv_fraction, ctx);
    y += BAR_H + 4.0;

    draw_scale_labels(scene, area, y, &["0", "6", "11+"], ctx);
    y += SCALE_H;

    let uv_advice = match uv {
        Some(uv) => uv_advice(uv),
        None => "sin datos todavía".to_string(),
    };

    draw_advice(scene, area, y, &uv_advice, ctx);
    y += ADVICE_H + SECTION_GAP;

    // CALIDAD DEL AIRE.
    let (air_value, air_level) = match air {
        Some(air) => (air.aqi.to_string(), aqi_level_label(air.aqi)),
        None => ("—".to_string(), ""),
    };

    draw_section_header(scene, area, y, "CALIDAD DEL AIRE", &air_value, air_level, ctx);
    y += LABEL_H + BAR_GAP;

    let air_fraction = air.map(|air| (f32::from(air.aqi) / 150.0).clamp(0.0, 1.0));

    draw_scale_bar(scene, Rect::new(area.x, y, area.width, BAR_H), air_fraction, ctx);
    y += BAR_H + 4.0;

    draw_scale_labels(scene, area, y, &["Buena", "Moderada", "Mala"], ctx);
    y += SCALE_H;

    let air_advice = match air {
        Some(air) => aqi_advice(air.aqi),
        None => "sin datos todavía",
    };

    draw_advice(scene, area, y, air_advice, ctx);
}

/// "Bajo" / "Moderado" / "Alto" / "Muy alto" / "Extremo".
pub fn uv_level_label(uv: f32) -> &'static str {
    match uv.round() as i32 {
        i32::MIN..=2 => "Bajo",
        3..=5 => "Moderado",
        6..=7 => "Alto",
        8..=10 => "Muy alto",
        _ => "Extremo",
    }
}

pub fn aqi_level_label(aqi: u16) -> &'static str {
    match aqi {
        0..=50 => "Buena",
        51..=100 => "Moderada",
        _ => "Mala",
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn section_height() -> f32 {
    LABEL_H + BAR_GAP + BAR_H + 4.0 + SCALE_H + ADVICE_H
}

fn draw_section_header(scene: &mut Scene, area: Rect, y: f32, title: &str, value: &str, level: &str, ctx: &mut RenderCtx<'_>) {
    let title_size = ctx.theme.typography.size_base * 0.72;
    let value_size = ctx.theme.typography.size_base * 1.05;
    let level_size = ctx.theme.typography.size_base * 0.85;

    ctx.text.draw_centered_v(
        scene,
        title,
        area.x,
        y,
        LABEL_H,
        TextStyle::new(title_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    let (level_width, _) = ctx.text.measure(level, level_size, ctx.theme.typography.font_family);
    let (value_width, _) = ctx.text.measure(value, value_size, ctx.theme.typography.font_family);

    let level_x = area.x + area.width - level_width;
    let value_x = level_x - if level.is_empty() { 0.0 } else { 6.0 } - value_width;

    ctx.text.draw_centered_v(
        scene,
        value,
        value_x,
        y,
        LABEL_H,
        TextStyle::new(value_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    ctx.text.draw_centered_v(
        scene,
        level,
        level_x,
        y,
        LABEL_H,
        TextStyle::new(level_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
}

/// Barra degradada verde → amarillo → rojo con el knob en la posición.
fn draw_scale_bar(scene: &mut Scene, bar: Rect, fraction: Option<f32>, ctx: &mut RenderCtx<'_>) {
    let radius = (bar.height / 2.0) as f64;
    let body = RoundedRect::new(bar.x as f64, bar.y as f64, (bar.x + bar.width) as f64, (bar.y + bar.height) as f64, radius);

    let gradient = Gradient::new_linear((bar.x as f64, bar.y as f64), ((bar.x + bar.width) as f64, bar.y as f64)).with_stops([
        ctx.theme.palette.positive,
        ctx.theme.palette.meter_warning,
        ctx.theme.palette.meter_critical,
    ]);

    scene.fill(Fill::NonZero, Affine::IDENTITY, &gradient, None, &body);

    let Some(fraction) = fraction else {
        return;
    };

    let knob_x = bar.x + bar.width * fraction;
    let knob = Circle::new((knob_x as f64, (bar.y + bar.height / 2.0) as f64), KNOB_RADIUS);

    scene.fill(Fill::NonZero, Affine::IDENTITY, Color::WHITE, None, &knob);
    scene.stroke(&Stroke::new(1.5), Affine::IDENTITY, ctx.theme.palette.panel_bg, None, &knob);
}

fn draw_scale_labels(scene: &mut Scene, area: Rect, y: f32, labels: &[&str; 3], ctx: &mut RenderCtx<'_>) {
    let size = ctx.theme.typography.size_base * 0.72;
    let style = TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary);

    ctx.text.draw_centered_v(scene, labels[0], area.x, y, SCALE_H, style);

    let (middle_width, _) = ctx.text.measure(labels[1], size, ctx.theme.typography.font_family);
    ctx.text
        .draw_centered_v(scene, labels[1], area.x + (area.width - middle_width) / 2.0, y, SCALE_H, style);

    let (last_width, _) = ctx.text.measure(labels[2], size, ctx.theme.typography.font_family);
    ctx.text
        .draw_centered_v(scene, labels[2], area.x + area.width - last_width, y, SCALE_H, style);
}

fn draw_advice(scene: &mut Scene, area: Rect, y: f32, advice: &str, ctx: &mut RenderCtx<'_>) {
    let size = ctx.theme.typography.size_base * 0.78;

    ctx.text.draw_centered_v(
        scene,
        advice,
        area.x,
        y,
        ADVICE_H,
        TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
}

fn uv_advice(uv: &UvInfo) -> String {
    let base = match uv.current.round() as i32 {
        i32::MIN..=2 => "Sin protección necesaria",
        3..=5 => "Usá protección al mediodía",
        _ => "Evitá el sol del mediodía",
    };

    if uv.peak > uv.current {
        format!("{base}. El pico fue {} a las {}:00.", uv.peak.round() as i32, uv.peak_hour)
    } else {
        format!("{base}.")
    }
}

fn aqi_advice(aqi: u16) -> &'static str {
    match aqi {
        0..=50 => "Buena para actividad al aire libre.",
        51..=100 => "Sensibles: moderá el ejercicio afuera.",
        _ => "Mejor actividades adentro.",
    }
}
