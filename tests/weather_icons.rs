use hyprbar::bar::weather::{UNKNOWN_WEATHER_ICON, weather_icon};

#[test]
fn maps_clear_sky_icon() {
    assert_eq!(weather_icon(0), "\u{e30d}");
}

#[test]
fn maps_unknown_weather_code_to_fallback_icon() {
    assert_eq!(weather_icon(999), UNKNOWN_WEATHER_ICON);
}
