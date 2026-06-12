// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;

use crate::app::{ShutdownToken, WorkerHandle};

use super::config::WeatherConfig;
use super::location::{Coordinates, detect_location};
use super::mapper::parse_weather_snapshot;
use super::state::{WeatherSnapshot, WeatherStore};

// ─── < Constants > ────────────────────────────────────────────────────

const FORECAST_DAYS: u8 = 5;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn spawn_fetcher(config: WeatherConfig, store: WeatherStore) -> Option<WorkerHandle> {
    match WorkerHandle::spawn("weather-fetcher", move |shutdown| fetcher_loop(config, store, shutdown)) {
        Ok(worker) => Some(worker),
        Err(error) => {
            log::error!("weather fetcher spawn failed: {error}");
            None
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn fetcher_loop(config: WeatherConfig, store: WeatherStore, shutdown: ShutdownToken) {
    let mut coordinates = config.location.coordinates();

    while !shutdown.should_stop() {
        let current_coordinates = match coordinates {
            Some(coordinates) => coordinates,
            None => match detect_location() {
                Ok(location) => {
                    match &location.label {
                        Some(label) => log::info!("weather location detected: {label}"),
                        None => {
                            log::info!("weather location detected: {}, {}", location.coordinates.latitude, location.coordinates.longitude)
                        }
                    }

                    if let Some(label) = location.label {
                        store.replace_location_label(label);
                    }

                    coordinates = Some(location.coordinates);

                    location.coordinates
                }
                Err(error) => {
                    log::warn!("weather location detection failed: {error}");

                    if shutdown.sleep(config.location_retry_interval) {
                        break;
                    }

                    continue;
                }
            },
        };

        match fetch_once(current_coordinates) {
            Ok(snapshot) => {
                log::info!("weather: {}°C code={}", snapshot.temp_c.round() as i32, snapshot.weather_code);

                store.replace_snapshot(snapshot);
            }
            Err(error) => {
                log::warn!("weather fetch failed: {error}");
            }
        }

        if shutdown.sleep(config.fetch_interval) {
            break;
        }
    }

    log::info!("weather fetcher stopped");
}

fn fetch_once(coordinates: Coordinates) -> Result<WeatherSnapshot> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,wind_speed_10m&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max&forecast_days={}&timezone=auto",
        coordinates.latitude, coordinates.longitude, FORECAST_DAYS
    );

    let mut response = ureq::get(&url).call()?;
    let body = response.body_mut().read_to_string()?;

    parse_weather_snapshot(&body)
}
