// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Result, anyhow};
use chrono::NaiveDate;
use serde::Deserialize;

use super::state::{AirQuality, DailyForecast, HourlyPoint, SeaInfo, Tide, UvInfo, WeatherSnapshot};

// ─── < Constants > ────────────────────────────────────────────────────

const DATE_FORMAT: &str = "%Y-%m-%d";

/// Puntos cardinales cada 45°, arrancando en el norte.
const CARDINALS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SO", "O", "NO"];

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    current: Option<CurrentWeather>,
    daily: Option<DailyWeather>,
    hourly: Option<HourlyWeather>,
}

#[derive(Debug, Default, Deserialize)]
struct HourlyWeather {
    #[serde(default)]
    time: Vec<String>,
    #[serde(default)]
    temperature_2m: Vec<Option<f64>>,
    #[serde(default)]
    weather_code: Vec<Option<u64>>,
    #[serde(default)]
    uv_index: Vec<Option<f64>>,
}

#[derive(Debug, Deserialize)]
struct AirResponse {
    current: Option<AirCurrent>,
}

#[derive(Debug, Deserialize)]
struct AirCurrent {
    us_aqi: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct MarineResponse {
    current: Option<MarineCurrent>,
    hourly: Option<MarineHourly>,
}

#[derive(Debug, Deserialize)]
struct MarineCurrent {
    sea_surface_temperature: Option<f64>,
    wave_height: Option<f64>,
    wave_direction: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
struct MarineHourly {
    #[serde(default)]
    time: Vec<String>,
    #[serde(default)]
    sea_level_height_msl: Vec<Option<f64>>,
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
    let hourly = parse_hourly(&response.hourly.unwrap_or_default());

    Ok(WeatherSnapshot {
        temp_c,
        weather_code,
        feels_like_c: current.apparent_temperature.map(|value| value as f32),
        humidity_percent: current.relative_humidity_2m.and_then(to_percent),
        wind_kmh: current.wind_speed_10m.map(|value| value as f32),
        precipitation_percent: first_probability(&daily),
        daily: parse_daily_forecast(&daily),
        uv: uv_info(&hourly),
        hourly: hourly.points,
    })
}

pub fn parse_air_quality(body: &str) -> Result<AirQuality> {
    let response: AirResponse = serde_json::from_str(body)?;

    let aqi = response
        .current
        .and_then(|current| current.us_aqi)
        .ok_or_else(|| anyhow!("la respuesta no trae us_aqi"))?;

    Ok(AirQuality {
        aqi: aqi.round().clamp(0.0, 500.0) as u16,
    })
}

pub fn parse_sea_info(body: &str, current_hour: u8) -> Result<SeaInfo> {
    let response: MarineResponse = serde_json::from_str(body)?;

    let current = response.current.ok_or_else(|| anyhow!("la respuesta marina no trae current"))?;
    let (tide_high, tide_low) = tides(&response.hourly.unwrap_or_default(), current_hour);

    Ok(SeaInfo {
        water_temp_c: current.sea_surface_temperature.map(|value| value as f32),
        wave_height_m: current.wave_height.map(|value| value as f32),
        wave_direction: current.wave_direction.map(cardinal_label),
        tide_high,
        tide_low,
    })
}

/// "SE" para 135°; los cuadrantes van cada 45° centrados en el punto.
pub fn cardinal_label(degrees: f64) -> &'static str {
    let index = ((degrees.rem_euclid(360.0) + 22.5) / 45.0) as usize % CARDINALS.len();

    CARDINALS[index]
}

// ─── < Private Functions > ────────────────────────────────────────────────────

/// Hourly parseado + los índices UV crudos, que comparten serie.
struct ParsedHourly {
    points: Vec<HourlyPoint>,
    uv_by_hour: Vec<(u8, f32)>,
}

fn parse_hourly(hourly: &HourlyWeather) -> ParsedHourly {
    let mut points = Vec::with_capacity(hourly.time.len());
    let mut uv_by_hour = Vec::with_capacity(hourly.time.len());

    // Solo el día calendario de hoy: las primeras 24 entradas.
    for (index, time) in hourly.time.iter().enumerate().take(24) {
        // "2026-08-22T14:00" → 14.
        let Some(hour) = time.get(11..13).and_then(|raw| raw.parse::<u8>().ok()) else {
            continue;
        };

        if let Some(uv) = value_at(&hourly.uv_index, index) {
            uv_by_hour.push((hour, uv as f32));
        }

        let Some(temp) = value_at(&hourly.temperature_2m, index) else {
            continue;
        };

        let Some(code) = value_at(&hourly.weather_code, index).and_then(|code| u8::try_from(code).ok()) else {
            continue;
        };

        points.push(HourlyPoint {
            hour,
            temp_c: temp as f32,
            weather_code: code,
        });
    }

    ParsedHourly { points, uv_by_hour }
}

fn uv_info(hourly: &ParsedHourly) -> Option<UvInfo> {
    let now_hour = chrono::Local::now().format("%H").to_string().parse::<u8>().ok()?;

    let current = hourly.uv_by_hour.iter().find(|(hour, _)| *hour == now_hour).map(|(_, uv)| *uv)?;

    let (peak_hour, peak) = hourly.uv_by_hour.iter().copied().max_by(|a, b| a.1.total_cmp(&b.1))?;

    Some(UvInfo { current, peak, peak_hour })
}

/// Marea alta y baja del día: el pico y el valle de la serie horaria
/// del nivel del mar.
fn tides(hourly: &MarineHourly, _current_hour: u8) -> (Option<Tide>, Option<Tide>) {
    let mut high: Option<(usize, f64)> = None;
    let mut low: Option<(usize, f64)> = None;

    for (index, level) in hourly.sea_level_height_msl.iter().enumerate() {
        let Some(level) = level else { continue };

        if high.is_none_or(|(_, best)| *level > best) {
            high = Some((index, *level));
        }

        if low.is_none_or(|(_, best)| *level < best) {
            low = Some((index, *level));
        }
    }

    let tide_at = |entry: Option<(usize, f64)>| {
        entry.and_then(|(index, level)| {
            let time = hourly.time.get(index)?.get(11..16)?.to_string();

            Some(Tide {
                time,
                height_m: level as f32,
            })
        })
    };

    (tide_at(high), tide_at(low))
}

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
