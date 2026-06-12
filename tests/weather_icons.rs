use hyprbar::bar::weather::{UNKNOWN_WEATHER_ICON, weather_description, weather_icon};

#[test]
fn maps_clear_sky_icon() {
    assert_eq!(weather_icon(0), "\u{e30d}");
}

#[test]
fn maps_unknown_weather_code_to_fallback_icon() {
    assert_eq!(weather_icon(250), UNKNOWN_WEATHER_ICON);
}

#[test]
fn maps_weather_descriptions() {
    assert_eq!(weather_description(0), "Despejado");
    assert_eq!(weather_description(2), "Parcialmente nublado");
    assert_eq!(weather_description(95), "Tormenta");
    assert_eq!(weather_description(250), "Desconocido");
}
