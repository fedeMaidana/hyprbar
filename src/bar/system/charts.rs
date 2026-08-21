// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Point, RoundedRect, RoundedRectRadii, Stroke};
use vello::peniko::{Color, Fill, Gradient};

use crate::components::RenderCtx;
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::state::{HISTORY_LEN, History};

// ─── < Constants > ────────────────────────────────────────────────────

const LABEL_TEXT_SCALE: f32 = 0.72;
const BIG_VALUE_SCALE: f32 = 2.1;
const UNIT_TEXT_SCALE: f32 = 0.78;
const SUB_TEXT_SCALE: f32 = 0.78;

/// Fracción del slot que ocupa cada barra (el resto es aire).
const BAR_FILL_RATIO: f32 = 0.62;

/// Altura mínima visible de una barra/dash con valor > 0.
const MIN_MARK_HEIGHT: f32 = 1.5;

const DASH_HEIGHT: f32 = 3.5;

// Guiones del dash chart: uno por cada 2 muestras (promediadas), anchos,
// que se encienden a medida que se acercan al presente.
const DASH_SAMPLE_STRIDE: usize = 2;
const DASH_SLOTS: usize = HISTORY_LEN / DASH_SAMPLE_STRIDE;
const DASH_FILL_RATIO: f32 = 0.7;
const DASH_OLD_ALPHA: f32 = 0.35;

const CARD_RADIUS: f64 = 12.0;
pub(crate) const CARD_ROW_HEIGHT: f32 = 32.0;
const CARD_PADDING_X: f32 = 12.0;

// Glifo opcional al final de cada fila de una tarjeta (p. ej. copiar).
const TRAILING_ICON_SCALE: f32 = 0.8;
const TRAILING_ICON_GAP: f32 = 8.0;

// Degradados y resaltados de los gráficos.
const BAR_PEAK_ALPHA: f32 = 0.85;
const BAR_BASE_ALPHA: f32 = 0.3;
const BAR_NEWEST_BASE_ALPHA: f32 = 0.5;
const AREA_TOP_ALPHA: f32 = 0.38;
const AREA_BOTTOM_ALPHA: f32 = 0.05;
const CAP_WIDTH: f64 = 1.5;
const BASELINE_ALPHA: f32 = 0.16;
const GUIDE_ALPHA: f32 = 0.12;

// ─── < Public Functions: Texto > ────────────────────────────────────────────────────

/// Etiqueta de sección en mayúsculas chicas ("PROCESSOR").
pub(crate) fn draw_section_label(scene: &mut Scene, x: f32, y: f32, height: f32, text: &str, ctx: &mut RenderCtx<'_>) {
    let size = ctx.theme.typography.size_base * LABEL_TEXT_SCALE;

    ctx.text.draw_centered_v(
        scene,
        text,
        x,
        y,
        height,
        TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
}

/// Valor grande + unidad chica ("2.2" + "%"). Devuelve el x final.
pub(crate) fn draw_big_value(
    scene: &mut Scene,
    x: f32,
    box_y: f32,
    box_height: f32,
    value: &str,
    unit: &str,
    ctx: &mut RenderCtx<'_>,
) -> f32 {
    let value_size = ctx.theme.typography.size_base * BIG_VALUE_SCALE;
    let unit_size = ctx.theme.typography.size_base * UNIT_TEXT_SCALE;

    let (value_width, _) = ctx.text.measure(value, value_size, ctx.theme.typography.font_family);

    ctx.text.draw_centered_v(
        scene,
        value,
        x,
        box_y,
        box_height,
        TextStyle::new(value_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    if unit.is_empty() {
        return x + value_width;
    }

    let unit_x = x + value_width + 3.0;

    ctx.text.draw_centered_v(
        scene,
        unit,
        unit_x,
        box_y,
        box_height,
        TextStyle::new(unit_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    let (unit_width, _) = ctx.text.measure(unit, unit_size, ctx.theme.typography.font_family);

    unit_x + unit_width
}

/// Renglón chico bajo un gráfico: texto a la izquierda y a la derecha.
pub(crate) fn draw_sub_row(scene: &mut Scene, rect: Rect, left: &str, right: &str, ctx: &mut RenderCtx<'_>) {
    let size = ctx.theme.typography.size_base * SUB_TEXT_SCALE;
    let style = TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary);

    ctx.text.draw_centered_v(scene, left, rect.x, rect.y, rect.height, style);

    let (right_width, _) = ctx.text.measure(right, size, ctx.theme.typography.font_family);

    ctx.text
        .draw_centered_v(scene, right, rect.x + rect.width - right_width, rect.y, rect.height, style);
}

/// Valor alineado a la derecha en el renglón de la etiqueta.
pub(crate) fn draw_row_value(scene: &mut Scene, x: f32, y: f32, width: f32, height: f32, text: &str, ctx: &mut RenderCtx<'_>) {
    let size = ctx.theme.typography.size_base;
    let (text_width, _) = ctx.text.measure(text, size, ctx.theme.typography.font_family);

    ctx.text.draw_centered_v(
        scene,
        text,
        x + width - text_width,
        y,
        height,
        TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );
}

// ─── < Public Functions: Gráficos > ────────────────────────────────────────────────────

/// Histograma de barras (lo más nuevo a la derecha), con degradado vertical
/// y la última muestra resaltada.
pub(crate) fn draw_bar_chart(scene: &mut Scene, rect: Rect, history: &History, max_hint: f32, color: Color) {
    draw_baseline(scene, rect, color);

    let ceiling = history.max().unwrap_or(0.0).max(max_hint).max(0.001);
    let slot = rect.width / HISTORY_LEN as f32;
    let bar_width = (slot * BAR_FILL_RATIO).max(1.0);
    let bottom = (rect.y + rect.height) as f64;

    for_each_sample(rect, history, |x, value, is_newest| {
        let height = ((value / ceiling) * rect.height).clamp(MIN_MARK_HEIGHT, rect.height);
        let top = (rect.y + rect.height - height) as f64;
        let radius = (bar_width / 2.0) as f64;

        // Esquinas redondeadas solo arriba: la barra "sale" de la base.
        let bar = RoundedRect::new(x as f64, top, (x + bar_width) as f64, bottom, RoundedRectRadii::new(radius, radius, 0.0, 0.0));

        let (peak, base) = if is_newest {
            (1.0, BAR_NEWEST_BASE_ALPHA)
        } else {
            (BAR_PEAK_ALPHA, BAR_BASE_ALPHA)
        };

        let gradient =
            Gradient::new_linear((x as f64, top), (x as f64, bottom)).with_stops([color.with_alpha(peak), color.with_alpha(base)]);

        scene.fill(Fill::NonZero, Affine::IDENTITY, &gradient, None, &bar);
    });
}

/// Área continua con degradado vertical y borde superior brillante
/// (memoria del mockup).
pub(crate) fn draw_area_chart(scene: &mut Scene, rect: Rect, history: &History, max_hint: f32, color: Color) {
    draw_baseline(scene, rect, color);

    if history.is_empty() {
        return;
    }

    let ceiling = history.max().unwrap_or(0.0).max(max_hint).max(0.001);
    let slot = rect.width / HISTORY_LEN as f32;
    let bottom = (rect.y + rect.height) as f64;

    // Un punto por muestra (al centro de su slot): el borde superior como
    // línea y el área cerrada contra el piso del gráfico.
    let mut cap = BezPath::new();
    let mut area = BezPath::new();
    let mut last_x = rect.x as f64;

    for_each_sample(rect, history, |x, value, _| {
        let height = ((value / ceiling) * rect.height).clamp(MIN_MARK_HEIGHT, rect.height);
        let point = Point::new((x + slot * 0.5) as f64, (rect.y + rect.height - height) as f64);

        if cap.elements().is_empty() {
            cap.move_to(point);
            area.move_to(Point::new(point.x, bottom));
        } else {
            cap.line_to(point);
        }

        area.line_to(point);
        last_x = point.x;
    });

    area.line_to(Point::new(last_x, bottom));
    area.close_path();

    let gradient = Gradient::new_linear((rect.x as f64, rect.y as f64), (rect.x as f64, bottom))
        .with_stops([color.with_alpha(AREA_TOP_ALPHA), color.with_alpha(AREA_BOTTOM_ALPHA)]);

    scene.fill(Fill::NonZero, Affine::IDENTITY, &gradient, None, &area);
    scene.stroke(&Stroke::new(CAP_WIDTH), Affine::IDENTITY, color, None, &cap);
}

/// Línea punteada (temperatura/latencia): guiones anchos que se apagan
/// hacia el pasado, con la escala autoajustada al rango de la ventana.
pub(crate) fn draw_dash_chart(scene: &mut Scene, rect: Rect, history: &History, color: Color) {
    // Guía sutil al medio como referencia visual.
    let guide_y = (rect.y + rect.height / 2.0) as f64;
    let guide = vello::kurbo::Rect::new(rect.x as f64, guide_y - 0.5, (rect.x + rect.width) as f64, guide_y + 0.5);

    scene.fill(Fill::NonZero, Affine::IDENTITY, color.with_alpha(GUIDE_ALPHA), None, &guide);

    if history.is_empty() {
        return;
    }

    // Un guión por cada DASH_SAMPLE_STRIDE muestras, promediadas; si al
    // final queda un resto (lo más nuevo), también dibuja el suyo.
    let mut dashes = [0.0_f32; DASH_SLOTS];
    let mut count = 0;
    let mut sum = 0.0;
    let mut in_chunk = 0;

    for value in history.iter() {
        sum += value;
        in_chunk += 1;

        if in_chunk == DASH_SAMPLE_STRIDE {
            dashes[count] = sum / DASH_SAMPLE_STRIDE as f32;
            count += 1;
            sum = 0.0;
            in_chunk = 0;
        }
    }

    if in_chunk > 0 {
        dashes[count] = sum / in_chunk as f32;
        count += 1;
    }

    let (mut low, mut high) = (f32::MAX, f32::MIN);

    for &value in &dashes[..count] {
        low = low.min(value);
        high = high.max(value);
    }

    // Margen para que una línea plana quede al medio y no pegada al piso.
    let span = (high - low).max(1.0);
    let low = low - span * 0.25;
    let range = span * 1.5;

    let slot = rect.width / DASH_SLOTS as f32;
    let dash_width = (slot * DASH_FILL_RATIO).max(1.5);

    for (index, &value) in dashes[..count].iter().enumerate() {
        // Lo más nuevo a la derecha.
        let position = DASH_SLOTS - count + index;
        let x = rect.x + position as f32 * slot;

        let fraction = ((value - low) / range).clamp(0.0, 1.0);
        let center = rect.y + rect.height - fraction * rect.height;

        // Se encienden a medida que se acercan al presente.
        let age = (index + 1) as f32 / count as f32;
        let alpha = DASH_OLD_ALPHA + (1.0 - DASH_OLD_ALPHA) * age;

        let dash = RoundedRect::new(
            x as f64,
            (center - DASH_HEIGHT / 2.0) as f64,
            (x + dash_width) as f64,
            (center + DASH_HEIGHT / 2.0) as f64,
            (DASH_HEIGHT / 2.0) as f64,
        );

        scene.fill(Fill::NonZero, Affine::IDENTITY, color.with_alpha(alpha), None, &dash);
    }
}

/// Barra de progreso redondeada sobre pista sutil.
pub(crate) fn draw_progress(scene: &mut Scene, rect: Rect, fraction: f32, color: Color, theme: &Theme) {
    let radius = (rect.height / 2.0) as f64;

    let track = RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, radius);
    scene.fill(Fill::NonZero, Affine::IDENTITY, theme.palette.panel_raised, None, &track);

    let fill_width = (rect.width * fraction.clamp(0.0, 1.0)).max(rect.height);
    let fill = RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + fill_width) as f64, (rect.y + rect.height) as f64, radius);
    scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &fill);
}

// ─── < Public Functions: Tarjetas > ────────────────────────────────────────────────────

pub(crate) fn card_height(rows: usize) -> f32 {
    rows as f32 * CARD_ROW_HEIGHT
}

pub(crate) fn draw_card_background(scene: &mut Scene, rect: Rect, theme: &Theme) {
    let body = RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, CARD_RADIUS);

    scene.fill(Fill::NonZero, Affine::IDENTITY, theme.palette.panel_raised, None, &body);
}

/// Rects de cada fila de una tarjeta (para hit-testing de copiar).
pub(crate) fn card_row_rects(rect: Rect, rows: usize) -> Vec<Rect> {
    (0..rows)
        .map(|index| Rect::new(rect.x, rect.y + index as f32 * CARD_ROW_HEIGHT, rect.width, CARD_ROW_HEIGHT))
        .collect()
}

/// Tarjeta de pares etiqueta/valor con divisores entre filas. Si viene
/// `trailing_glyph`, cada fila lo lleva al final (p. ej. el de copiar).
pub(crate) fn draw_info_card(
    scene: &mut Scene,
    rect: Rect,
    rows: &[(&str, String)],
    trailing_glyph: Option<&str>,
    ctx: &mut RenderCtx<'_>,
) {
    draw_card_background(scene, rect, ctx.theme);

    let label_size = ctx.theme.typography.size_base * LABEL_TEXT_SCALE;
    let value_size = ctx.theme.typography.size_base * 0.9;

    let icon_size = ctx.theme.typography.size_base * TRAILING_ICON_SCALE;

    let icon_slot = trailing_glyph
        .map(|glyph| ctx.text.measure(glyph, icon_size, ctx.theme.typography.icon_font_family).0 + TRAILING_ICON_GAP)
        .unwrap_or(0.0);

    for (index, (label, value)) in rows.iter().enumerate() {
        let row_y = rect.y + index as f32 * CARD_ROW_HEIGHT;

        if index > 0 {
            let line = vello::kurbo::Rect::new(
                (rect.x + CARD_PADDING_X) as f64,
                (row_y - 0.5) as f64,
                (rect.x + rect.width - CARD_PADDING_X) as f64,
                (row_y + 0.5) as f64,
            );

            scene.fill(Fill::NonZero, Affine::IDENTITY, ctx.theme.palette.panel_divider, None, &line);
        }

        ctx.text.draw_centered_v(
            scene,
            label,
            rect.x + CARD_PADDING_X,
            row_y,
            CARD_ROW_HEIGHT,
            TextStyle::new(label_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );

        let (value_width, _) = ctx.text.measure(value, value_size, ctx.theme.typography.font_family);

        ctx.text.draw_centered_v(
            scene,
            value,
            rect.x + rect.width - CARD_PADDING_X - icon_slot - value_width,
            row_y,
            CARD_ROW_HEIGHT,
            TextStyle::new(value_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
        );

        if let Some(glyph) = trailing_glyph {
            ctx.text.draw_centered_v(
                scene,
                glyph,
                rect.x + rect.width - CARD_PADDING_X - icon_slot + TRAILING_ICON_GAP,
                row_y,
                CARD_ROW_HEIGHT,
                TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, ctx.theme.palette.text_secondary),
            );
        }
    }
}

// ─── < Public Functions: Formato > ────────────────────────────────────────────────────

/// Rate legible: ("12", "MB/s") o ("640", "kB/s").
pub(crate) fn format_rate(bytes_per_second: f32) -> (String, &'static str) {
    let mbps = bytes_per_second / 1_000_000.0;

    if mbps >= 10.0 {
        (format!("{mbps:.0}"), "MB/s")
    } else if mbps >= 1.0 {
        (format!("{mbps:.1}"), "MB/s")
    } else {
        (format!("{:.0}", (bytes_per_second / 1000.0).max(0.0)), "kB/s")
    }
}

/// Bytes acumulados legibles ("4.1 GiB", "612 MiB").
pub(crate) fn format_bytes(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;

    let bytes = bytes as f64;

    if bytes >= GIB {
        format!("{:.1} GiB", bytes / GIB)
    } else {
        format!("{:.0} MiB", bytes / MIB)
    }
}

/// Tamaño de disco compacto en unidades decimales ("214", "476G").
pub(crate) fn format_disk(bytes: u64) -> String {
    format!("{:.0}", bytes as f64 / 1_000_000_000.0)
}

/// "3h 12m" para el tiempo restante de batería.
pub(crate) fn format_minutes(minutes: u32) -> String {
    if minutes >= 60 {
        format!("{}h {}m", minutes / 60, minutes % 60)
    } else {
        format!("{minutes}m")
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

/// Recorre la historia dándole a cada muestra su x (lo nuevo a la derecha)
/// y avisando cuál es la más reciente.
fn for_each_sample(rect: Rect, history: &History, mut draw: impl FnMut(f32, f32, bool)) {
    let slot = rect.width / HISTORY_LEN as f32;
    let count = history.len();

    for (index, value) in history.iter().enumerate() {
        let position = HISTORY_LEN - count + index;
        let x = rect.x + position as f32 * slot;

        draw(x, value, index + 1 == count);
    }
}

/// Línea de base tenue que ancla el gráfico.
fn draw_baseline(scene: &mut Scene, rect: Rect, color: Color) {
    let line = vello::kurbo::Rect::new(
        rect.x as f64,
        (rect.y + rect.height - 1.0) as f64,
        (rect.x + rect.width) as f64,
        (rect.y + rect.height) as f64,
    );

    scene.fill(Fill::NonZero, Affine::IDENTITY, color.with_alpha(BASELINE_ALPHA), None, &line);
}
