// ─── < Imports > ────────────────────────────────────────────────────

use std::time::Duration;

use super::location::Coordinates;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WeatherConfig {
    pub location: WeatherLocation,
    pub fetch_interval: Duration,
    pub location_retry_interval: Duration,
}

#[derive(Debug, Clone, Copy)]
pub enum WeatherLocation {
    Auto,
    Coordinates(Coordinates),
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl WeatherConfig {
    pub fn auto_detect() -> Self {
        Self {
            location: WeatherLocation::Auto,
            fetch_interval: Duration::from_secs(600),
            location_retry_interval: Duration::from_secs(60),
        }
    }

    pub fn from_coordinates(latitude: f64, longitude: f64) -> Self {
        Self {
            location: WeatherLocation::Coordinates(Coordinates::new(latitude, longitude)),
            fetch_interval: Duration::from_secs(600),
            location_retry_interval: Duration::from_secs(60),
        }
    }
}

impl WeatherLocation {
    pub(crate) fn coordinates(self) -> Option<Coordinates> {
        match self {
            Self::Auto => None,
            Self::Coordinates(coordinates) => Some(coordinates),
        }
    }
}
