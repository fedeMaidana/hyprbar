// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;

use crate::components::RenderCtx;
use crate::render::Rect;
use crate::theme::Theme;

use super::charts::{
    draw_area_chart, draw_bar_chart, draw_big_value, draw_dash_chart, draw_progress, draw_row_value, draw_section_label, draw_sub_row,
    format_bytes, format_disk, section_card, section_card_height,
};
use super::state::SystemData;

// ─── < Constants > ────────────────────────────────────────────────────

const VALUE_PLACEHOLDER: &str = "—";

const LABEL_H: f32 = 16.0;
const MAIN_H: f32 = 34.0;
const SUB_H: f32 = 16.0;
const CHART_H: f32 = 26.0;
const INNER_GAP: f32 = 6.0;

/// Aire entre tarjetas de sección.
const CARD_GAP: f32 = 12.0;

/// Procesador y temperatura llevan gráficos más altos que el resto.
const CPU_CHART_H: f32 = 40.0;
const TEMP_CHART_H: f32 = 40.0;

const DISK_ROW_H: f32 = 20.0;
const DISK_BAR_H: f32 = 6.0;
const DISK_LABEL_SLOT: f32 = 52.0;
const DISK_VALUE_SLOT: f32 = 86.0;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn height(_data: &SystemData, theme: &Theme) -> f32 {
    max_height(theme)
}

pub(crate) fn max_height(_theme: &Theme) -> f32 {
    // PROCESSOR + MEMORY + TEMPERATURE + DISK, cada una en su tarjeta.
    processor_height() + CARD_GAP + memory_height() + CARD_GAP + temperature_height() + CARD_GAP + disk_height()
}

pub(crate) fn draw(scene: &mut Scene, area: Rect, data: &SystemData, ctx: &mut RenderCtx<'_>) {
    let accent = ctx.theme.palette.accent;
    let mut y = area.y;

    // PROCESSOR: valor grande y el histograma a todo el ancho debajo.
    let card = section_card(scene, Rect::new(area.x, y, area.width, processor_height()), ctx.theme);
    let mut row = card.y;

    draw_section_label(scene, card.x, row, LABEL_H, "PROCESSOR", ctx);
    row += LABEL_H + INNER_GAP;

    let cpu_text = data
        .metrics
        .and_then(|metrics| metrics.cpu_percent)
        .map(|value| format!("{value:.1}"))
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    draw_big_value(scene, card.x, row, MAIN_H, &cpu_text, "%", ctx);
    row += MAIN_H + INNER_GAP;

    draw_bar_chart(scene, Rect::new(card.x, row, card.width, CPU_CHART_H), &data.cpu_history, 100.0, accent);
    row += CPU_CHART_H + INNER_GAP;

    let cores_load = format!(
        "{} cores · load {}",
        data.cores
            .map(|cores| cores.to_string())
            .unwrap_or_else(|| VALUE_PLACEHOLDER.into()),
        data.load_avg
            .map(|load| format!("{load:.2}"))
            .unwrap_or_else(|| VALUE_PLACEHOLDER.into()),
    );

    draw_sub_row(scene, Rect::new(card.x, row, card.width, SUB_H), "last 60s", &cores_load, ctx);
    y += processor_height() + CARD_GAP;

    // MEMORY: valor a la derecha + área rellena.
    let card = section_card(scene, Rect::new(area.x, y, area.width, memory_height()), ctx.theme);
    let mut row = card.y;

    let memory = data.metrics.and_then(|metrics| metrics.memory);

    let memory_value = memory
        .map(
            |memory| format!("{:.1} / {:.0} GiB", memory.used_kb() as f64 / (1024.0 * 1024.0), memory.total_kb as f64 / (1024.0 * 1024.0),),
        )
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    draw_section_label(scene, card.x, row, LABEL_H, "MEMORY", ctx);
    draw_row_value(scene, card.x, row, card.width, LABEL_H, &memory_value, ctx);
    row += LABEL_H + INNER_GAP;

    draw_area_chart(scene, Rect::new(card.x, row, card.width, CHART_H), &data.memory_history, 1.0, accent);
    row += CHART_H + INNER_GAP;

    let swap = match data.swap_used_kb {
        Some(0) | None => "swap 0 B".to_string(),
        Some(kb) => format!("swap {}", format_bytes(kb * 1024)),
    };

    let used_percent = memory
        .map(|memory| format!("{:.0}% used", memory.used_fraction() * 100.0))
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    draw_sub_row(scene, Rect::new(card.x, row, card.width, SUB_H), &swap, &used_percent, ctx);
    y += memory_height() + CARD_GAP;

    // TEMPERATURE: valor a la derecha + línea punteada.
    let card = section_card(scene, Rect::new(area.x, y, area.width, temperature_height()), ctx.theme);
    let mut row = card.y;

    let temperature = data.metrics.and_then(|metrics| metrics.temperature_c);

    let temperature_value = temperature
        .map(|value| format!("{value:.0} °C"))
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    draw_section_label(scene, card.x, row, LABEL_H, "TEMPERATURE", ctx);
    draw_row_value(scene, card.x, row, card.width, LABEL_H, &temperature_value, ctx);
    row += LABEL_H + INNER_GAP;

    draw_dash_chart(scene, Rect::new(card.x, row, card.width, TEMP_CHART_H), &data.temp_history, accent);
    row += TEMP_CHART_H + INNER_GAP;

    let fan = data.fan_rpm.map(|rpm| format!("fan {rpm} rpm")).unwrap_or_default();
    let max_temp = data.session_max_temp_c.map(|value| format!("max {value:.0}°C")).unwrap_or_default();

    draw_sub_row(scene, Rect::new(card.x, row, card.width, SUB_H), &fan, &max_temp, ctx);
    y += temperature_height() + CARD_GAP;

    // DISK: etiqueta + barra + valor en una sola fila.
    let card = section_card(scene, Rect::new(area.x, y, area.width, disk_height()), ctx.theme);

    draw_section_label(scene, card.x, card.y, DISK_ROW_H, "DISK", ctx);

    if let Some(disk) = data.disk {
        let bar = Rect::new(
            card.x + DISK_LABEL_SLOT,
            card.y + (DISK_ROW_H - DISK_BAR_H) / 2.0,
            card.width - DISK_LABEL_SLOT - DISK_VALUE_SLOT,
            DISK_BAR_H,
        );

        let fraction = if disk.total_bytes > 0 {
            disk.used_bytes as f32 / disk.total_bytes as f32
        } else {
            0.0
        };

        draw_progress(scene, bar, fraction, accent, ctx.theme);

        let value = format!("{} / {}G", format_disk(disk.used_bytes), format_disk(disk.total_bytes));
        draw_row_value(scene, card.x, card.y, card.width, DISK_ROW_H, &value, ctx);
    } else {
        draw_row_value(scene, card.x, card.y, card.width, DISK_ROW_H, VALUE_PLACEHOLDER, ctx);
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn processor_height() -> f32 {
    section_card_height(LABEL_H + INNER_GAP + MAIN_H + INNER_GAP + CPU_CHART_H + INNER_GAP + SUB_H)
}

fn memory_height() -> f32 {
    section_card_height(LABEL_H + INNER_GAP + CHART_H + INNER_GAP + SUB_H)
}

fn temperature_height() -> f32 {
    section_card_height(LABEL_H + INNER_GAP + TEMP_CHART_H + INNER_GAP + SUB_H)
}

fn disk_height() -> f32 {
    section_card_height(DISK_ROW_H)
}
