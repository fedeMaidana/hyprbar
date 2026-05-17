// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Result, anyhow};
use serde::Deserialize;

use super::state::WeatherSnapshot;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    current: Option<CurrentWeather>,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    temperature_2m: Option<f64>,
    weather_code: Option<u64>,
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn parse_weather_snapshot(body: &str) -> Result<WeatherSnapshot> {
    let response: WeatherResponse = serde_json::from_str(body)?;

    let current = response.current.ok_or_else(|| anyhow!("missing current weather object"))?;

    let temp_c = current.temperature_2m.ok_or_else(|| anyhow!("missing temperature_2m"))? as f32;

    let weather_code = current.weather_code.ok_or_else(|| anyhow!("missing weather_code"))?;

    let weather_code = u32::try_from(weather_code).map_err(|_| anyhow!("weather_code out of range"))?;

    Ok(WeatherSnapshot::new(temp_c, weather_code))
}
