// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;

use crate::components::RenderCtx;
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::charts::{
    card_height, draw_bar_chart, draw_big_value, draw_info_card, draw_progress, draw_row_value, draw_section_label, draw_sub_row,
    format_minutes, section_card, section_card_height,
};
use super::state::{BatteryData, SystemData};

// ─── < Constants > ────────────────────────────────────────────────────

const VALUE_PLACEHOLDER: &str = "—";

const BATTERY_GLYPH: &str = "\u{f0079}";

const LABEL_H: f32 = 14.0;
const MAIN_H: f32 = 30.0;
const SUB_H: f32 = 14.0;
const CHART_H: f32 = 26.0;
const BAR_H: f32 = 6.0;
const INNER_GAP: f32 = 4.0;
const BAR_GAP: f32 = 6.0;

/// Aire entre tarjetas de sección.
const CARD_GAP: f32 = 10.0;

const INFO_ROWS: usize = 4;

/// Debajo de este porcentaje, la barra de batería pasa a crítica.
const LOW_BATTERY_PERCENT: u8 = 20;

const EMPTY_HEIGHT: f32 = 44.0;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn max_height(_theme: &Theme) -> f32 {
    battery_block_height() + CARD_GAP + draw_block_height() + CARD_GAP + card_height(INFO_ROWS)
}

pub(crate) fn height(data: &SystemData, theme: &Theme) -> f32 {
    if data.battery.is_some() { max_height(theme) } else { EMPTY_HEIGHT }
}

pub(crate) fn draw(scene: &mut Scene, area: Rect, data: &SystemData, ctx: &mut RenderCtx<'_>) {
    let Some(battery) = &data.battery else {
        let size = ctx.theme.typography.size_base * 0.9;

        ctx.text.draw_centered_v(
            scene,
            "sin batería detectada",
            area.x,
            area.y,
            EMPTY_HEIGHT,
            TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );

        return;
    };

    let accent = ctx.theme.palette.accent;
    let mut y = area.y;

    // BATTERY: porcentaje grande + tiempo restante + barra.
    let card = section_card(scene, Rect::new(area.x, y, area.width, battery_block_height()), ctx.theme);
    let mut row = card.y;

    draw_section_label(scene, card.x, row, LABEL_H, "BATTERY", ctx);
    row += LABEL_H + INNER_GAP;

    let percent_text = battery
        .percent
        .map(|value| value.to_string())
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    let end_x = draw_big_value(scene, card.x, row, MAIN_H, &percent_text, "%", ctx);

    if let Some(minutes) = battery.minutes_left {
        let detail = format!("· {} left", format_minutes(minutes));
        let size = ctx.theme.typography.size_base * 0.82;

        ctx.text.draw_centered_v(
            scene,
            &detail,
            end_x + 8.0,
            row,
            MAIN_H,
            TextStyle::new(size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );
    }

    let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
    let (icon_width, _) = ctx.text.measure(BATTERY_GLYPH, icon_size, ctx.theme.typography.icon_font_family);

    ctx.text.draw_centered_v(
        scene,
        BATTERY_GLYPH,
        card.x + card.width - icon_width,
        row,
        MAIN_H,
        TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, ctx.theme.palette.text_secondary),
    );

    row += MAIN_H + BAR_GAP;

    let fraction = battery.percent.map(|value| value as f32 / 100.0).unwrap_or(0.0);

    let bar_color = match battery.percent {
        Some(percent) if percent <= LOW_BATTERY_PERCENT => ctx.theme.palette.meter_critical,
        _ => ctx.theme.palette.positive,
    };

    draw_progress(scene, Rect::new(card.x, row, card.width, BAR_H), fraction, bar_color, ctx.theme);
    row += BAR_H + BAR_GAP;

    let status = battery.status.clone().unwrap_or_else(|| VALUE_PLACEHOLDER.into());

    let identity = match (&battery.name, &battery.technology) {
        (Some(name), Some(tech)) => format!("{name} · {tech}"),
        (Some(name), None) => name.clone(),
        _ => VALUE_PLACEHOLDER.to_string(),
    };

    draw_sub_row(scene, Rect::new(card.x, row, card.width, SUB_H), &status, &identity, ctx);
    y += battery_block_height() + CARD_GAP;

    // POWER DRAW.
    let card = section_card(scene, Rect::new(area.x, y, area.width, draw_block_height()), ctx.theme);
    let mut row = card.y;

    let draw_value = battery
        .power_w
        .map(|watts| format!("{watts:.1} W"))
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    draw_section_label(scene, card.x, row, LABEL_H, "POWER DRAW", ctx);
    draw_row_value(scene, card.x, row, card.width, LABEL_H, &draw_value, ctx);
    row += LABEL_H + INNER_GAP;

    draw_bar_chart(scene, Rect::new(card.x, row, card.width, CHART_H), &battery.power_history, 1.0, accent);
    row += CHART_H + INNER_GAP;

    let adapter = match battery.adapter_online {
        Some(true) => "adapter connected",
        Some(false) => "adapter disconnected",
        None => "",
    };

    let cell = battery.cell_temp_c.map(|value| format!("cell {value:.0} °C")).unwrap_or_default();

    draw_sub_row(scene, Rect::new(card.x, row, card.width, SUB_H), adapter, &cell, ctx);
    y += draw_block_height() + CARD_GAP;

    // Tarjeta de detalles.
    let rows = info_rows(battery);
    draw_info_card(scene, Rect::new(area.x, y, area.width, card_height(INFO_ROWS)), &rows, None, ctx);
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn battery_block_height() -> f32 {
    section_card_height(LABEL_H + INNER_GAP + MAIN_H + BAR_GAP + BAR_H + BAR_GAP + SUB_H)
}

fn draw_block_height() -> f32 {
    section_card_height(LABEL_H + INNER_GAP + CHART_H + INNER_GAP + SUB_H)
}

fn info_rows(battery: &BatteryData) -> [(&'static str, String); INFO_ROWS] {
    let charge = match (battery.charge_wh, battery.charge_full_wh) {
        (Some(now), Some(full)) => format!("{now:.1} / {full:.1} Wh"),
        _ => VALUE_PLACEHOLDER.to_string(),
    };

    let health = match battery.health_percent {
        Some(percent) => format!("{percent}% · {}", health_word(percent)),
        None => VALUE_PLACEHOLDER.to_string(),
    };

    [
        ("CHARGE", charge),
        (
            "VOLTAGE",
            battery
                .voltage_v
                .map(|v| format!("{v:.1} V"))
                .unwrap_or_else(|| VALUE_PLACEHOLDER.into()),
        ),
        ("CYCLES", battery.cycles.map(|c| c.to_string()).unwrap_or_else(|| VALUE_PLACEHOLDER.into())),
        ("HEALTH", health),
    ]
}

fn health_word(percent: u8) -> &'static str {
    match percent {
        80.. => "good",
        60..=79 => "fair",
        _ => "poor",
    }
}
