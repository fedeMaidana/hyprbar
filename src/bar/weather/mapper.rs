// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Result, anyhow};
use serde_json::Value;

use super::state::WeatherSnapshot;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn parse_weather_snapshot(body: &str) -> Result<WeatherSnapshot> {
    let json: Value = serde_json::from_str(body)?;

    let current = json.get("current").ok_or_else(|| anyhow!("missing current weather object"))?;

    let temp_c = current
        .get("temperature_2m")
        .and_then(Value::as_f64)
        .ok_or_else(|| anyhow!("missing temperature_2m"))? as f32;

    let weather_code = current
        .get("weather_code")
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("missing weather_code"))?;

    let weather_code = u32::try_from(weather_code).map_err(|_| anyhow!("weather_code out of range"))?;

    Ok(WeatherSnapshot::new(temp_c, weather_code))
}
