// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;

use crate::app::{ShutdownToken, WorkerHandle};

use super::config::WeatherConfig;
use super::mapper::parse_weather_snapshot;
use super::state::{WeatherSnapshot, WeatherStore};

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
    while !shutdown.should_stop() {
        match fetch_once(&config) {
            Ok(snapshot) => {
                log::info!("weather: {}°C code={}", snapshot.temp_c.round() as i32, snapshot.weather_code);

                store.replace(snapshot);
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

fn fetch_once(config: &WeatherConfig) -> Result<WeatherSnapshot> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code",
        config.latitude, config.longitude
    );

    let mut response = ureq::get(&url).call()?;
    let body = response.body_mut().read_to_string()?;

    parse_weather_snapshot(&body)
}
