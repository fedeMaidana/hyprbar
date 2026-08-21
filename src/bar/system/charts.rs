// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Color, Fill};

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

const DASH_HEIGHT: f32 = 2.5;

const CARD_RADIUS: f64 = 12.0;
pub(crate) const CARD_ROW_HEIGHT: f32 = 32.0;
const CARD_PADDING_X: f32 = 12.0;

const AREA_FILL_ALPHA: f32 = 0.35;

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

/// Histograma de barras (lo más nuevo a la derecha).
pub(crate) fn draw_bar_chart(scene: &mut Scene, rect: Rect, history: &History, max_hint: f32, color: Color) {
    let ceiling = history.max().unwrap_or(0.0).max(max_hint).max(0.001);
    let slot = rect.width / HISTORY_LEN as f32;
    let bar_width = (slot * BAR_FILL_RATIO).max(1.0);

    for_each_sample(rect, history, |x, value| {
        let height = ((value / ceiling) * rect.height).clamp(MIN_MARK_HEIGHT, rect.height);
        let bar = RoundedRect::new(
            x as f64,
            (rect.y + rect.height - height) as f64,
            (x + bar_width) as f64,
            (rect.y + rect.height) as f64,
            (bar_width / 2.0) as f64,
        );

        scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &bar);
    });
}

/// Área rellena con borde superior más brillante (memoria del mockup).
pub(crate) fn draw_area_chart(scene: &mut Scene, rect: Rect, history: &History, max_hint: f32, color: Color) {
    let ceiling = history.max().unwrap_or(0.0).max(max_hint).max(0.001);
    let slot = rect.width / HISTORY_LEN as f32;
    let fill = color.with_alpha(AREA_FILL_ALPHA);

    for_each_sample(rect, history, |x, value| {
        let height = ((value / ceiling) * rect.height).clamp(MIN_MARK_HEIGHT, rect.height);
        let top = rect.y + rect.height - height;

        let body = RoundedRect::new(x as f64, top as f64, (x + slot) as f64, (rect.y + rect.height) as f64, 0.0);
        scene.fill(Fill::NonZero, Affine::IDENTITY, fill, None, &body);

        let cap = RoundedRect::new(x as f64, top as f64, (x + slot) as f64, (top + 1.5) as f64, 0.0);
        scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &cap);
    });
}

/// Línea punteada (temperatura/latencia): un guión por muestra, con la
/// escala autoajustada al rango de la ventana.
pub(crate) fn draw_dash_chart(scene: &mut Scene, rect: Rect, history: &History, color: Color) {
    let (mut low, mut high) = (f32::MAX, f32::MIN);

    for value in history.iter() {
        low = low.min(value);
        high = high.max(value);
    }

    if history.is_empty() {
        return;
    }

    // Margen para que una línea plana quede al medio y no pegada al piso.
    let span = (high - low).max(1.0);
    let low = low - span * 0.25;
    let range = span * 1.5;

    let slot = rect.width / HISTORY_LEN as f32;
    let dash_width = (slot * BAR_FILL_RATIO).max(1.5);

    for_each_sample(rect, history, |x, value| {
        let fraction = ((value - low) / range).clamp(0.0, 1.0);
        let center = rect.y + rect.height - fraction * rect.height;

        let dash = RoundedRect::new(
            x as f64,
            (center - DASH_HEIGHT / 2.0) as f64,
            (x + dash_width) as f64,
            (center + DASH_HEIGHT / 2.0) as f64,
            (DASH_HEIGHT / 2.0) as f64,
        );

        scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &dash);
    });
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

/// Tarjeta de pares etiqueta/valor con divisores entre filas.
pub(crate) fn draw_info_card(scene: &mut Scene, rect: Rect, rows: &[(&str, String)], ctx: &mut RenderCtx<'_>) {
    draw_card_background(scene, rect, ctx.theme);

    let label_size = ctx.theme.typography.size_base * LABEL_TEXT_SCALE;
    let value_size = ctx.theme.typography.size_base * 0.9;

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
            rect.x + rect.width - CARD_PADDING_X - value_width,
            row_y,
            CARD_ROW_HEIGHT,
            TextStyle::new(value_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
        );
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

/// "2h ago" / "35m ago" para el sync de pacman.
pub(crate) fn format_minutes_ago(minutes: u64) -> String {
    if minutes >= 60 {
        format!("{}h ago", minutes / 60)
    } else {
        format!("{minutes}m ago")
    }
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

/// Recorre la historia dándole a cada muestra su x (lo nuevo a la derecha).
fn for_each_sample(rect: Rect, history: &History, mut draw: impl FnMut(f32, f32)) {
    let slot = rect.width / HISTORY_LEN as f32;
    let count = history.len();

    for (index, value) in history.iter().enumerate() {
        let position = HISTORY_LEN - count + index;
        let x = rect.x + position as f32 * slot;

        draw(x, value);
    }
}
