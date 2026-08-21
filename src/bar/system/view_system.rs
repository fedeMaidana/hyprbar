// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;

use crate::components::RenderCtx;
use crate::render::Rect;
use crate::theme::Theme;

use super::charts::{
    draw_area_chart, draw_bar_chart, draw_big_value, draw_dash_chart, draw_progress, draw_row_value, draw_section_label, draw_sub_row,
    format_bytes, format_disk,
};
use super::state::SystemData;

// ─── < Constants > ────────────────────────────────────────────────────

const VALUE_PLACEHOLDER: &str = "—";

const LABEL_H: f32 = 16.0;
const MAIN_H: f32 = 34.0;
const SUB_H: f32 = 16.0;
const CHART_H: f32 = 26.0;
const INNER_GAP: f32 = 6.0;
const SECTION_GAP: f32 = 18.0;

const DISK_ROW_H: f32 = 20.0;
const DISK_BAR_H: f32 = 6.0;
const DISK_LABEL_SLOT: f32 = 52.0;
const DISK_VALUE_SLOT: f32 = 86.0;

/// La barra de cpu arranca a la derecha del valor grande.
const CPU_CHART_OFFSET: f32 = 112.0;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn height(_data: &SystemData, _theme: &Theme) -> f32 {
    max_height(_theme)
}

pub(crate) fn max_height(_theme: &Theme) -> f32 {
    // PROCESSOR + MEMORY + TEMPERATURE + DISK, con sus gaps.
    processor_height() + SECTION_GAP + section_height() + SECTION_GAP + section_height() + SECTION_GAP + DISK_ROW_H
}

pub(crate) fn draw(scene: &mut Scene, area: Rect, data: &SystemData, ctx: &mut RenderCtx<'_>) {
    let accent = ctx.theme.palette.accent;
    let mut y = area.y;

    // PROCESSOR: valor grande + histograma al lado.
    draw_section_label(scene, area.x, y, LABEL_H, "PROCESSOR", ctx);
    y += LABEL_H + INNER_GAP;

    let cpu_text = data
        .metrics
        .and_then(|metrics| metrics.cpu_percent)
        .map(|value| format!("{value:.1}"))
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    draw_big_value(scene, area.x, y, MAIN_H, &cpu_text, "%", ctx);

    let cpu_chart = Rect::new(area.x + CPU_CHART_OFFSET, y + MAIN_H - CHART_H, area.width - CPU_CHART_OFFSET, CHART_H);
    draw_bar_chart(scene, cpu_chart, &data.cpu_history, 100.0, accent);

    y += MAIN_H + INNER_GAP;

    let cores_load = format!(
        "{} cores · load {}",
        data.cores
            .map(|cores| cores.to_string())
            .unwrap_or_else(|| VALUE_PLACEHOLDER.into()),
        data.load_avg
            .map(|load| format!("{load:.2}"))
            .unwrap_or_else(|| VALUE_PLACEHOLDER.into()),
    );

    draw_sub_row(scene, Rect::new(area.x, y, area.width, SUB_H), "last 60s", &cores_load, ctx);
    y += SUB_H + SECTION_GAP;

    // MEMORY: valor a la derecha + área rellena.
    let memory = data.metrics.and_then(|metrics| metrics.memory);

    let memory_value = memory
        .map(
            |memory| format!("{:.1} / {:.0} GiB", memory.used_kb() as f64 / (1024.0 * 1024.0), memory.total_kb as f64 / (1024.0 * 1024.0),),
        )
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    draw_section_label(scene, area.x, y, LABEL_H, "MEMORY", ctx);
    draw_row_value(scene, area.x, y, area.width, LABEL_H, &memory_value, ctx);
    y += LABEL_H + INNER_GAP;

    let memory_chart = Rect::new(area.x, y, area.width, CHART_H);
    draw_area_chart(scene, memory_chart, &data.memory_history, 1.0, accent);
    y += CHART_H + INNER_GAP;

    let swap = match data.swap_used_kb {
        Some(0) | None => "swap 0 B".to_string(),
        Some(kb) => format!("swap {}", format_bytes(kb * 1024)),
    };

    let used_percent = memory
        .map(|memory| format!("{:.0}% used", memory.used_fraction() * 100.0))
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    draw_sub_row(scene, Rect::new(area.x, y, area.width, SUB_H), &swap, &used_percent, ctx);
    y += SUB_H + SECTION_GAP;

    // TEMPERATURE: valor a la derecha + línea punteada.
    let temperature = data.metrics.and_then(|metrics| metrics.temperature_c);

    let temperature_value = temperature
        .map(|value| format!("{value:.0} °C"))
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    draw_section_label(scene, area.x, y, LABEL_H, "TEMPERATURE", ctx);
    draw_row_value(scene, area.x, y, area.width, LABEL_H, &temperature_value, ctx);
    y += LABEL_H + INNER_GAP;

    let temp_chart = Rect::new(area.x, y, area.width, CHART_H);
    draw_dash_chart(scene, temp_chart, &data.temp_history, accent);
    y += CHART_H + INNER_GAP;

    let fan = data.fan_rpm.map(|rpm| format!("fan {rpm} rpm")).unwrap_or_default();
    let max_temp = data.session_max_temp_c.map(|value| format!("max {value:.0}°C")).unwrap_or_default();

    draw_sub_row(scene, Rect::new(area.x, y, area.width, SUB_H), &fan, &max_temp, ctx);
    y += SUB_H + SECTION_GAP;

    // DISK: etiqueta + barra + valor.
    draw_section_label(scene, area.x, y, DISK_ROW_H, "DISK", ctx);

    if let Some(disk) = data.disk {
        let bar = Rect::new(
            area.x + DISK_LABEL_SLOT,
            y + (DISK_ROW_H - DISK_BAR_H) / 2.0,
            area.width - DISK_LABEL_SLOT - DISK_VALUE_SLOT,
            DISK_BAR_H,
        );

        let fraction = if disk.total_bytes > 0 {
            disk.used_bytes as f32 / disk.total_bytes as f32
        } else {
            0.0
        };

        draw_progress(scene, bar, fraction, accent, ctx.theme);

        let value = format!("{} / {}G", format_disk(disk.used_bytes), format_disk(disk.total_bytes));
        draw_row_value(scene, area.x, y, area.width, DISK_ROW_H, &value, ctx);
    } else {
        draw_row_value(scene, area.x, y, area.width, DISK_ROW_H, VALUE_PLACEHOLDER, ctx);
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn processor_height() -> f32 {
    LABEL_H + INNER_GAP + MAIN_H + INNER_GAP + SUB_H
}

fn section_height() -> f32 {
    LABEL_H + INNER_GAP + CHART_H + INNER_GAP + SUB_H
}
