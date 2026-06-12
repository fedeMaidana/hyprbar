use chrono::NaiveDate;
use hyprbar::bar::weather::{WeatherSnapshot, parse_weather_snapshot};

#[test]
fn parses_minimal_current_snapshot() {
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
fn parses_extended_current_fields() {
    let body = r#"
        {
            "current": {
                "temperature_2m": 12.3,
                "weather_code": 2,
                "apparent_temperature": 10.1,
                "relative_humidity_2m": 72.4,
                "wind_speed_10m": 17.8
            }
        }
    "#;

    let snapshot = parse_weather_snapshot(body).unwrap();

    assert_eq!(snapshot.feels_like_c, Some(10.1));
    assert_eq!(snapshot.humidity_percent, Some(72));
    assert_eq!(snapshot.wind_kmh, Some(17.8));
}

#[test]
fn parses_daily_forecast_and_precipitation() {
    let body = r#"
        {
            "current": {
                "temperature_2m": 12.3,
                "weather_code": 2
            },
            "daily": {
                "time": ["2026-06-11", "2026-06-12"],
                "weather_code": [3, 61],
                "temperature_2m_max": [14.2, 11.0],
                "temperature_2m_min": [8.1, 7.4],
                "precipitation_probability_max": [35, 80]
            }
        }
    "#;

    let snapshot = parse_weather_snapshot(body).unwrap();

    assert_eq!(snapshot.precipitation_percent, Some(35));
    assert_eq!(snapshot.daily.len(), 2);

    let first = snapshot.daily[0];

    assert_eq!(first.date, NaiveDate::from_ymd_opt(2026, 6, 11).unwrap());
    assert_eq!(first.weather_code, 3);
    assert_eq!(first.max_c, 14.2);
    assert_eq!(first.min_c, 8.1);
}

#[test]
fn skips_malformed_daily_entries() {
    let body = r#"
        {
            "current": {
                "temperature_2m": 12.3,
                "weather_code": 2
            },
            "daily": {
                "time": ["2026-06-11", "2026-06-12", "not-a-date"],
                "weather_code": [3, null, 0],
                "temperature_2m_max": [14.2, 11.0, 9.0],
                "temperature_2m_min": [8.1, 7.4, 4.0]
            }
        }
    "#;

    let snapshot = parse_weather_snapshot(body).unwrap();

    assert_eq!(snapshot.daily.len(), 1);
    assert_eq!(snapshot.daily[0].weather_code, 3);
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
