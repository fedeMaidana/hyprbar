use anyhow::{Result, anyhow};
use serde_json::Value;

use super::state::WeatherSnapshot;

pub fn parse_weather_snapshot(body: &str) -> Result<WeatherSnapshot> {
    let json: Value = serde_json::from_str(body)?;

    let current = json
        .get("current")
        .ok_or_else(|| anyhow!("missing current weather object"))?;

    let temp_c = current
        .get("temperature_2m")
        .and_then(Value::as_f64)
        .ok_or_else(|| anyhow!("missing temperature_2m"))? as f32;

    let weather_code = current
        .get("weather_code")
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("missing weather_code"))?;

    let weather_code =
        u32::try_from(weather_code).map_err(|_| anyhow!("weather_code out of range"))?;

    Ok(WeatherSnapshot::new(temp_c, weather_code))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
