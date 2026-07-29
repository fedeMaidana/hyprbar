// ─── < Imports > ────────────────────────────────────────────────────

use vello::peniko::Color;

use crate::theme::Palette;

// ─── < Constants > ────────────────────────────────────────────────────

pub const UNKNOWN_WEATHER_ICON: &str = "\u{e374}";

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn weather_icon(code: u8) -> &'static str {
    match code {
        0 => "\u{e30d}",
        1 | 2 => "\u{e302}",
        3 => "\u{e312}",
        45 | 48 => "\u{e313}",
        51..=57 => "\u{e319}",
        61..=67 => "\u{e318}",
        71..=77 => "\u{e31a}",
        80..=82 => "\u{e318}",
        85 | 86 => "\u{e31a}",
        95..=99 => "\u{e31d}",
        _ => UNKNOWN_WEATHER_ICON,
    }
}

/// Condition-based tint so the forecast reads at a glance.
pub fn weather_icon_color(code: u8, palette: &Palette) -> Color {
    match code {
        0..=2 => palette.clock_day,
        3 | 45 | 48 => palette.text_secondary,
        51..=57 | 61..=67 | 80..=82 => palette.clock_night,
        71..=77 | 85 | 86 => palette.text_primary,
        95..=99 => palette.meter_warning,
        _ => palette.text_secondary,
    }
}

pub fn weather_description(code: u8) -> &'static str {
    match code {
        0 => "Despejado",
        1 => "Mayormente despejado",
        2 => "Parcialmente nublado",
        3 => "Nublado",
        45 | 48 => "Niebla",
        51..=57 => "Llovizna",
        61..=67 => "Lluvia",
        71..=77 => "Nieve",
        80..=82 => "Chubascos",
        85 | 86 => "Chubascos de nieve",
        95..=99 => "Tormenta",
        _ => "Desconocido",
    }
}
