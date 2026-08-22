// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;

use crate::components::{Interaction, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::{CopyField, SystemAction};
use super::charts::{
    card_height, card_row_rects, draw_bar_chart, draw_big_value, draw_card_background, draw_dash_chart, draw_info_card, draw_row_value,
    draw_section_label, draw_sub_row, format_bytes, format_rate, section_card, section_card_height,
};
use super::state::{SystemData, WifiInfo};

// ─── < Constants > ────────────────────────────────────────────────────

const VALUE_PLACEHOLDER: &str = "—";

const WIFI_GLYPH: &str = "\u{f05a9}";
const STRENGTH_GLYPH: &str = "\u{f0928}";
const COPY_GLYPH: &str = "\u{f018f}";

const LABEL_H: f32 = 14.0;
const MAIN_H: f32 = 30.0;
const SUB_H: f32 = 14.0;
const CHART_H: f32 = 26.0;
const INNER_GAP: f32 = 4.0;

/// Aire entre tarjetas de sección.
const CARD_GAP: f32 = 10.0;

const UP_COLUMN_X: f32 = 108.0;

/// El histograma de bajada va a todo el ancho, gemelo del de cpu.
const NET_CHART_H: f32 = 36.0;

const WIFI_CARD_H: f32 = 48.0;
const CARD_PADDING_X: f32 = 12.0;

const INFO_ROWS: usize = 3;

/// A quién le medimos la latencia (mostrado en el panel).
pub(crate) const PING_TARGET_LABEL: &str = "1.1.1.1";

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn max_height(_theme: &Theme) -> f32 {
    rates_height() + CARD_GAP + WIFI_CARD_H + CARD_GAP + latency_height() + CARD_GAP + card_height(INFO_ROWS)
}

pub(crate) fn height(data: &SystemData, _theme: &Theme) -> f32 {
    rates_height() + CARD_GAP + wifi_block_height(data) + latency_height() + CARD_GAP + card_height(INFO_ROWS)
}

pub(crate) fn draw(scene: &mut Scene, area: Rect, data: &SystemData, ctx: &mut RenderCtx<'_>) {
    let accent = ctx.theme.palette.accent;
    let network = &data.network;
    let mut y = area.y;

    // DOWN / UP con el histograma a todo el ancho debajo.
    let card = section_card(scene, Rect::new(area.x, y, area.width, rates_height()), ctx.theme);
    let mut row = card.y;

    draw_section_label(scene, card.x, row, LABEL_H, "DOWN", ctx);
    draw_section_label(scene, card.x + UP_COLUMN_X, row, LABEL_H, "UP", ctx);
    row += LABEL_H + INNER_GAP;

    let (down_value, down_unit) = network.down_rate_bps.map(format_rate).unwrap_or((VALUE_PLACEHOLDER.into(), ""));
    let (up_value, up_unit) = network.up_rate_bps.map(format_rate).unwrap_or((VALUE_PLACEHOLDER.into(), ""));

    draw_big_value(scene, card.x, row, MAIN_H, &down_value, down_unit, ctx);
    draw_big_value(scene, card.x + UP_COLUMN_X, row, MAIN_H, &up_value, up_unit, ctx);
    row += MAIN_H + INNER_GAP;

    draw_bar_chart(scene, Rect::new(card.x, row, card.width, NET_CHART_H), &network.down_history, 1.0, accent);
    row += NET_CHART_H + INNER_GAP;

    let session = format!("session {} ↓ · {} ↑", format_bytes(network.session_rx_bytes), format_bytes(network.session_tx_bytes),);

    draw_sub_row(scene, Rect::new(card.x, row, card.width, SUB_H), "last 60s", &session, ctx);
    y += rates_height() + CARD_GAP;

    // Tarjeta de wifi, solo con una conexión inalámbrica activa.
    if let Some(wifi) = &network.wifi {
        draw_wifi_card(scene, Rect::new(area.x, y, area.width, WIFI_CARD_H), wifi, ctx);
        y += WIFI_CARD_H + CARD_GAP;
    }

    // LATENCY.
    let card = section_card(scene, Rect::new(area.x, y, area.width, latency_height()), ctx.theme);
    let mut row = card.y;

    let latency_value = network
        .latency_ms
        .map(|value| format!("{value:.0} ms"))
        .unwrap_or_else(|| VALUE_PLACEHOLDER.to_string());

    draw_section_label(scene, card.x, row, LABEL_H, "LATENCY", ctx);
    draw_row_value(scene, card.x, row, card.width, LABEL_H, &latency_value, ctx);
    row += LABEL_H + INNER_GAP;

    draw_dash_chart(scene, Rect::new(card.x, row, card.width, CHART_H), &network.latency_history, accent);
    row += CHART_H + INNER_GAP;

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

    draw_sub_row(scene, Rect::new(card.x, row, card.width, SUB_H), &quality, PING_TARGET_LABEL, ctx);
    y += latency_height() + CARD_GAP;

    // IPV4 / GATEWAY / DNS, con el glifo de copiar en cada fila.
    let rows = info_rows(data);
    draw_info_card(scene, Rect::new(area.x, y, area.width, card_height(INFO_ROWS)), &rows, Some(COPY_GLYPH), ctx);
}

/// Las filas de la tarjeta copian su valor al portapapeles.
pub(crate) fn hit_test(point: Point, area: Rect, data: &SystemData, _theme: &Theme) -> Option<Interaction> {
    let card_y = area.y + rates_height() + CARD_GAP + wifi_block_height(data) + latency_height() + CARD_GAP;
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
    section_card_height(LABEL_H + INNER_GAP + MAIN_H + INNER_GAP + NET_CHART_H + INNER_GAP + SUB_H)
}

fn latency_height() -> f32 {
    section_card_height(LABEL_H + INNER_GAP + CHART_H + INNER_GAP + SUB_H)
}

/// Lo que ocupa la tarjeta de wifi (con su gap) si corresponde mostrarla.
fn wifi_block_height(data: &SystemData) -> f32 {
    if data.network.wifi.is_some() { WIFI_CARD_H + CARD_GAP } else { 0.0 }
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
