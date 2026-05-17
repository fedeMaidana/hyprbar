use std::thread;

use anyhow::Result;

use super::config::WeatherConfig;
use super::mapper::parse_weather_snapshot;
use super::state::{WeatherSnapshot, WeatherStore};

pub fn spawn_fetcher(config: WeatherConfig, store: WeatherStore) {
    match thread::Builder::new()
        .name("weather-fetcher".to_string())
        .spawn(move || fetcher_loop(config, store))
    {
        Ok(_handle) => {}
        Err(error) => log::error!("weather fetcher spawn failed: {error}"),
    }
}

fn fetcher_loop(config: WeatherConfig, store: WeatherStore) {
    loop {
        match fetch_once(&config) {
            Ok(snapshot) => {
                log::info!("weather: {}°C code={}", snapshot.temp_c.round() as i32, snapshot.weather_code);

                store.replace(snapshot);
            }
            Err(error) => {
                log::warn!("weather fetch failed: {error}");
            }
        }

        thread::sleep(config.fetch_interval);
    }
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
