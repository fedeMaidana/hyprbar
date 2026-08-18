// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Result, anyhow};
use chrono::NaiveDate;
use serde::Deserialize;

use super::state::{DailyForecast, WeatherSnapshot};

// ─── < Constants > ────────────────────────────────────────────────────

const DATE_FORMAT: &str = "%Y-%m-%d";

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    current: Option<CurrentWeather>,
    daily: Option<DailyWeather>,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    temperature_2m: Option<f64>,
    weather_code: Option<u64>,
    apparent_temperature: Option<f64>,
    relative_humidity_2m: Option<f64>,
    wind_speed_10m: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
struct DailyWeather {
    #[serde(default)]
    time: Vec<String>,
    #[serde(default)]
    weather_code: Vec<Option<u64>>,
    #[serde(default)]
    temperature_2m_max: Vec<Option<f64>>,
    #[serde(default)]
    temperature_2m_min: Vec<Option<f64>>,
    #[serde(default)]
    precipitation_probability_max: Vec<Option<u64>>,
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn parse_weather_snapshot(body: &str) -> Result<WeatherSnapshot> {
    let response: WeatherResponse = serde_json::from_str(body)?;

    let current = response.current.ok_or_else(|| anyhow!("la respuesta no trae el objeto current"))?;

    let temp_c = current
        .temperature_2m
        .ok_or_else(|| anyhow!("la respuesta no trae temperature_2m"))? as f32;

    let weather_code = current.weather_code.ok_or_else(|| anyhow!("la respuesta no trae weather_code"))?;

    let weather_code = u8::try_from(weather_code).map_err(|_| anyhow!("weather_code fuera de rango"))?;

    let daily = response.daily.unwrap_or_default();

    Ok(WeatherSnapshot {
        temp_c,
        weather_code,
        feels_like_c: current.apparent_temperature.map(|value| value as f32),
        humidity_percent: current.relative_humidity_2m.and_then(to_percent),
        wind_kmh: current.wind_speed_10m.map(|value| value as f32),
        precipitation_percent: first_probability(&daily),
        daily: parse_daily_forecast(&daily),
    })
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn parse_daily_forecast(daily: &DailyWeather) -> Vec<DailyForecast> {
    let mut forecast = Vec::with_capacity(daily.time.len());

    for (index, time) in daily.time.iter().enumerate() {
        let Ok(date) = NaiveDate::parse_from_str(time, DATE_FORMAT) else {
            continue;
        };

        let Some(code) = value_at(&daily.weather_code, index) else {
            continue;
        };

        let Ok(weather_code) = u8::try_from(code) else {
            continue;
        };

        let Some(max_c) = value_at(&daily.temperature_2m_max, index) else {
            continue;
        };

        let Some(min_c) = value_at(&daily.temperature_2m_min, index) else {
            continue;
        };

        forecast.push(DailyForecast {
            date,
            weather_code,
            max_c: max_c as f32,
            min_c: min_c as f32,
        });
    }

    forecast
}

fn first_probability(daily: &DailyWeather) -> Option<u8> {
    daily
        .precipitation_probability_max
        .first()
        .copied()
        .flatten()
        .map(|value| value.min(100) as u8)
}

fn to_percent(value: f64) -> Option<u8> {
    let rounded = value.round();

    if !(0.0..=100.0).contains(&rounded) {
        return None;
    }

    Some(rounded as u8)
}

fn value_at<T: Copy>(values: &[Option<T>], index: usize) -> Option<T> {
    values.get(index).copied().flatten()
}
