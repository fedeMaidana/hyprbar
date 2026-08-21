// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;

use crate::components::{Interaction, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::{CopyField, SystemAction};
use super::charts::{
    card_height, card_row_rects, draw_bar_chart, draw_big_value, draw_card_background, draw_dash_chart, draw_info_card, draw_row_value,
    draw_section_label, draw_sub_row, format_bytes, format_rate,
};
use super::state::{SystemData, WifiInfo};

// ─── < Constants > ────────────────────────────────────────────────────

const VALUE_PLACEHOLDER: &str = "—";

const WIFI_GLYPH: &str = "\u{f05a9}";
const STRENGTH_GLYPH: &str = "\u{f0928}";

const LABEL_H: f32 = 16.0;
const MAIN_H: f32 = 34.0;
const SUB_H: f32 = 16.0;
const CHART_H: f32 = 26.0;
const INNER_GAP: f32 = 6.0;
const SECTION_GAP: f32 = 16.0;

const UP_COLUMN_X: f32 = 108.0;
const RATE_CHART_OFFSET: f32 = 208.0;

const WIFI_CARD_H: f32 = 54.0;
const CARD_PADDING_X: f32 = 12.0;

const INFO_ROWS: usize = 3;

/// A quién le medimos la latencia (mostrado en el panel).
pub(crate) const PING_TARGET_LABEL: &str = "1.1.1.1";

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn max_height(_theme: &Theme) -> f32 {
    rates_height() + SECTION_GAP + WIFI_CARD_H + SECTION_GAP + latency_height() + SECTION_GAP + card_height(INFO_ROWS)
}

pub(crate) fn height(data: &SystemData, theme: &Theme) -> f32 {
    max_height(theme) - wifi_block_height(data)
}

pub(crate) fn draw(scene: &mut Scene, area: Rect, data: &SystemData, ctx: &mut RenderCtx<'_>) {
    let accent = ctx.theme.palette.accent;
    let network = &data.network;
    let mut y = area.y;

    // DOWN / UP con histograma al costado.
    draw_section_label(scene, area.x, y, LABEL_H, "DOWN", ctx);
    draw_section_label(scene, area.x + UP_COLUMN_X, y, LABEL_H, "UP", ctx);
    y += LABEL_H + INNER_GAP;

    let (down_value, down_unit) = network.down_rate_bps.map(format_rate).unwrap_or((VALUE_PLACEHOLDER.into(), ""));
    let (up_value, up_unit) = network.up_rate_bps.map(format_rate).unwrap_or((VALUE_PLACEHOLDER.into(), ""));

    draw_big_value(scene, area.x, y, MAIN_H, &down_value, down_unit, ctx);
    draw_big_value(scene, area.x + UP_COLUMN_X, y, MAIN_H, &up_value, up_unit, ctx);

    let chart = Rect::new(area.x + RATE_CHART_OFFSET, y + MAIN_H - CHART_H, area.width - RATE_CHART_OFFSET, CHART_H);
    draw_bar_chart(scene, chart, &network.down_history, 1.0, accent);

    y += MAIN_H + INNER_GAP;

    let session = format!("session {} ↓ · {} ↑", format_bytes(network.session_rx_bytes), format_bytes(network.session_tx_bytes),);

    draw_sub_row(scene, Rect::new(area.x, y, area.width, SUB_H), "last 60s", &session, ctx);
    y += SUB_H + SECTION_GAP;

    // Tarjeta de wifi, solo con una conexión inalámbrica activa.
    if let Some(wifi) = &data.network.wifi {
        draw_wifi_card(scene, Rect::new(area.x, y, area.width, WIFI_CARD_H), wifi, ctx);
        y += WIFI_CARD_H + SECTION_GAP;
    }

    // LATENCY.
    let latency_value = network
        .latency_ms
        .map(|value| format!("{value:.0} ms"))
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    draw_section_label(scene, area.x, y, LABEL_H, "LATENCY", ctx);
    draw_row_value(scene, area.x, y, area.width, LABEL_H, &latency_value, ctx);
    y += LABEL_H + INNER_GAP;

    draw_dash_chart(scene, Rect::new(area.x, y, area.width, CHART_H), &network.latency_history, accent);
    y += CHART_H + INNER_GAP;

    let quality = format!(
        "jitter {} · {} loss",
        network
            .jitter_ms
            .map(|v| format!("{v:.0} ms"))
            .unwrap_or_else(|| VALUE_PLACEHOLDER.into()),
        network
            .loss_percent
            .map(|v| format!("{v:.0}%"))
            .unwrap_or_else(|| VALUE_PLACEHOLDER.into()),
    );

    draw_sub_row(scene, Rect::new(area.x, y, area.width, SUB_H), &quality, PING_TARGET_LABEL, ctx);
    y += SUB_H + SECTION_GAP;

    // IPV4 / GATEWAY / DNS.
    let rows = info_rows(data);
    draw_info_card(scene, Rect::new(area.x, y, area.width, card_height(INFO_ROWS)), &rows, ctx);
}

/// Las filas de la tarjeta copian su valor al portapapeles.
pub(crate) fn hit_test(point: Point, area: Rect, data: &SystemData, _theme: &Theme) -> Option<Interaction> {
    let card_y = area.y + rates_height() + SECTION_GAP + wifi_block_height(data) + latency_height() + SECTION_GAP;
    let card = Rect::new(area.x, card_y, area.width, card_height(INFO_ROWS));

    let fields = [CopyField::Ipv4, CopyField::Gateway, CopyField::Dns];

    for (rect, field) in card_row_rects(card, INFO_ROWS).into_iter().zip(fields) {
        if rect.contains_point(point.x, point.y) {
            return Some(SystemAction::Copy(field).interaction());
        }
    }

    None
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn rates_height() -> f32 {
    LABEL_H + INNER_GAP + MAIN_H + INNER_GAP + SUB_H
}

/// Lo que ocupa la tarjeta de wifi (con su gap) si corresponde mostrarla.
fn wifi_block_height(data: &SystemData) -> f32 {
    if data.network.wifi.is_some() {
        WIFI_CARD_H + SECTION_GAP
    } else {
        0.0
    }
}

fn latency_height() -> f32 {
    LABEL_H + INNER_GAP + CHART_H + INNER_GAP + SUB_H
}

fn info_rows(data: &SystemData) -> [(&'static str, String); INFO_ROWS] {
    let network = &data.network;

    let dns = if network.dns.is_empty() {
        VALUE_PLACEHOLDER.to_string()
    } else {
        network.dns.join(" · ")
    };

    [
        ("IPV4", network.ipv4.clone().unwrap_or_else(|| VALUE_PLACEHOLDER.into())),
        ("GATEWAY", network.gateway.clone().unwrap_or_else(|| VALUE_PLACEHOLDER.into())),
        ("DNS", dns),
    ]
}

fn draw_wifi_card(scene: &mut Scene, card: Rect, wifi: &WifiInfo, ctx: &mut RenderCtx<'_>) {
    draw_card_background(scene, card, ctx.theme);

    let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;

    ctx.text.draw_centered_v(
        scene,
        WIFI_GLYPH,
        card.x + CARD_PADDING_X,
        card.y,
        card.height,
        TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, ctx.theme.palette.accent),
    );

    let text_x = card.x + CARD_PADDING_X + 26.0;
    let ssid_size = ctx.theme.typography.size_base * 0.95;
    let detail_size = ctx.theme.typography.size_base * 0.75;

    ctx.text.draw_centered_v(
        scene,
        &wifi.ssid,
        text_x,
        card.y + 6.0,
        card.height * 0.5 - 6.0,
        TextStyle::new(ssid_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    let mut details: Vec<String> = vec![wifi.interface.clone()];

    if let Some(security) = &wifi.security {
        details.push(security.clone());
    }

    if let Some(band) = wifi.band {
        match wifi.channel {
            Some(channel) => details.push(format!("{band} ch {channel}")),
            None => details.push(band.to_string()),
        }
    }

    ctx.text.draw_centered_v(
        scene,
        &details.join(" · "),
        text_x,
        card.y + card.height * 0.5,
        card.height * 0.5 - 6.0,
        TextStyle::new(detail_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );

    // Señal a la derecha: glyph + dBm.
    let signal = wifi
        .signal_dbm
        .map(|dbm| format!("{dbm} dBm"))
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    let signal_size = ctx.theme.typography.size_base * 0.85;
    let (signal_width, _) = ctx.text.measure(&signal, signal_size, ctx.theme.typography.font_family);
    let signal_x = card.x + card.width - CARD_PADDING_X - signal_width;

    ctx.text.draw_centered_v(
        scene,
        &signal,
        signal_x,
        card.y,
        card.height,
        TextStyle::new(signal_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    let (glyph_width, _) = ctx.text.measure(STRENGTH_GLYPH, icon_size, ctx.theme.typography.icon_font_family);

    ctx.text.draw_centered_v(
        scene,
        STRENGTH_GLYPH,
        signal_x - glyph_width - 8.0,
        card.y,
        card.height,
        TextStyle::new(icon_size, ctx.theme.typography.icon_font_family, ctx.theme.palette.text_secondary),
    );
}
