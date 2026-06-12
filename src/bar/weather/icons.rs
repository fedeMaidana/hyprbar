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
