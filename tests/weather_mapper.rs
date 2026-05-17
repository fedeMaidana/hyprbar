use hyprbar::bar::weather::{WeatherSnapshot, parse_weather_snapshot};

#[test]
fn parses_current_weather_snapshot() {
    let body = r#"
        {
            "current": {
                "temperature_2m": 18.7,
                "weather_code": 3
            }
        }
    "#;

    let snapshot = parse_weather_snapshot(body).unwrap();

    assert_eq!(snapshot, WeatherSnapshot::new(18.7, 3));
}

#[test]
fn fails_when_current_object_is_missing() {
    let body = r#"{ "daily": {} }"#;

    let error = parse_weather_snapshot(body).unwrap_err();

    assert!(error.to_string().contains("missing current weather object"));
}

#[test]
fn fails_when_temperature_is_missing() {
    let body = r#"
        {
            "current": {
                "weather_code": 3
            }
        }
    "#;

    let error = parse_weather_snapshot(body).unwrap_err();

    assert!(error.to_string().contains("missing temperature_2m"));
}

#[test]
fn fails_when_weather_code_is_missing() {
    let body = r#"
        {
            "current": {
                "temperature_2m": 18.7
            }
        }
    "#;

    let error = parse_weather_snapshot(body).unwrap_err();

    assert!(error.to_string().contains("missing weather_code"));
}

#[test]
fn fails_when_weather_code_is_out_of_range() {
    let body = r#"
        {
            "current": {
                "temperature_2m": 18.7,
                "weather_code": 999999999999
            }
        }
    "#;

    let error = parse_weather_snapshot(body).unwrap_err();

    assert!(error.to_string().contains("weather_code out of range"));
}
