pub const UNKNOWN_WEATHER_ICON: &str = "\u{e374}";

pub fn weather_icon(code: u32) -> &'static str {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_clear_sky_icon() {
        assert_eq!(weather_icon(0), "\u{e30d}");
    }

    #[test]
    fn maps_unknown_weather_code_to_fallback_icon() {
        assert_eq!(weather_icon(999), UNKNOWN_WEATHER_ICON);
    }
}
