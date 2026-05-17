// ─── < Modules > ────────────────────────────────────────────────────

mod config;
mod fetcher;
mod icons;
mod location;
mod mapper;
mod pill;
mod state;

// ─── < Public API > ────────────────────────────────────────────────────

pub use config::WeatherConfig;
pub use pill::WeatherPill;

// ─── < Tests > ────────────────────────────────────────────────────

#[doc(hidden)]
pub use icons::{UNKNOWN_WEATHER_ICON, weather_icon};

#[doc(hidden)]
pub use location::{Coordinates, parse_detected_location};

#[doc(hidden)]
pub use mapper::parse_weather_snapshot;

#[doc(hidden)]
pub use state::WeatherSnapshot;
