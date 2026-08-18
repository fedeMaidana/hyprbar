// ─── < Imports > ────────────────────────────────────────────────────

use std::time::Duration;

use anyhow::Result;
use calloop::channel::Sender;

use crate::app::{ShutdownToken, WorkerHandle};

use super::config::WeatherConfig;
use super::location::{Coordinates, detect_location};
use super::mapper::parse_weather_snapshot;
use super::state::{WeatherSnapshot, WeatherStore};

// ─── < Constants > ────────────────────────────────────────────────────

const FORECAST_DAYS: u8 = 5;

/// Techo duro por request; sin esto un socket colgado retiene el hilo
/// (y el join del shutdown) indefinidamente.
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn spawn_fetcher(config: WeatherConfig, store: WeatherStore, redraw_signal: Sender<()>) -> Option<WorkerHandle> {
    match WorkerHandle::spawn("weather-fetcher", move |shutdown| fetcher_loop(config, store, redraw_signal, shutdown)) {
        Ok(worker) => Some(worker),
        Err(error) => {
            log::error!("no se pudo iniciar el fetcher del clima: {error}");
            None
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn fetcher_loop(config: WeatherConfig, store: WeatherStore, redraw: Sender<()>, shutdown: ShutdownToken) {
    let agent = http_agent();
    let mut coordinates = config.location.coordinates();

    while !shutdown.should_stop() {
        let current_coordinates = match coordinates {
            Some(coordinates) => coordinates,
            None => match detect_location() {
                Ok(location) => {
                    match &location.label {
                        Some(label) => log::info!("ubicación del clima detectada: {label}"),
                        None => {
                            log::info!(
                                "ubicación del clima detectada: {}, {}",
                                location.coordinates.latitude,
                                location.coordinates.longitude
                            )
                        }
                    }

                    if let Some(label) = location.label {
                        store.replace_location_label(label);
                        let _ = redraw.send(());
                    }

                    coordinates = Some(location.coordinates);

                    location.coordinates
                }
                Err(error) => {
                    log::warn!("no se pudo detectar la ubicación del clima: {error}");

                    if shutdown.sleep(config.location_retry_interval) {
                        break;
                    }

                    continue;
                }
            },
        };

        match fetch_once(&agent, current_coordinates) {
            Ok(snapshot) => {
                log::info!("clima: {}°C código={}", snapshot.temp_c.round() as i32, snapshot.weather_code);

                store.replace_snapshot(snapshot);
                let _ = redraw.send(());
            }
            Err(error) => {
                log::warn!("falló el fetch del clima: {error}");
            }
        }

        if shutdown.sleep(config.fetch_interval) {
            break;
        }
    }

    log::info!("fetcher del clima detenido");
}

fn http_agent() -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(HTTP_TIMEOUT)).build().into()
}

fn fetch_once(agent: &ureq::Agent, coordinates: Coordinates) -> Result<WeatherSnapshot> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,wind_speed_10m&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max&forecast_days={}&timezone=auto",
        coordinates.latitude, coordinates.longitude, FORECAST_DAYS
    );

    let mut response = agent.get(&url).call()?;
    let body = response.body_mut().read_to_string()?;

    parse_weather_snapshot(&body)
}
