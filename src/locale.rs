// ─── < Constants > ────────────────────────────────────────────────────

pub const MONTH_NAMES: [&str; 12] = [
    "enero",
    "febrero",
    "marzo",
    "abril",
    "mayo",
    "junio",
    "julio",
    "agosto",
    "septiembre",
    "octubre",
    "noviembre",
    "diciembre",
];

pub const WEEKDAY_NAMES: [&str; 7] = ["lunes", "martes", "miércoles", "jueves", "viernes", "sábado", "domingo"];

pub const WEEKDAY_ABBREVS: [&str; 7] = ["lun", "mar", "mié", "jue", "vie", "sáb", "dom"];

pub const WEEKDAY_HEADERS: [&str; 7] = ["L", "M", "X", "J", "V", "S", "D"];

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn month_name(month: u32) -> &'static str {
    month
        .checked_sub(1)
        .and_then(|index| MONTH_NAMES.get(index as usize))
        .copied()
        .unwrap_or("?")
}

pub fn weekday_name(monday_index: usize) -> &'static str {
    WEEKDAY_NAMES.get(monday_index).copied().unwrap_or("?")
}

pub fn weekday_abbrev(monday_index: usize) -> &'static str {
    WEEKDAY_ABBREVS.get(monday_index).copied().unwrap_or("?")
}
