// ─── < Modules > ────────────────────────────────────────────────────

mod action;
mod config;
mod fetcher;
mod icons;
mod location;
mod mapper;
mod panel;
mod pill;
mod state;
mod view_air;
mod view_forecast;
mod view_sea;

// ─── < Public API > ────────────────────────────────────────────────────

pub use config::WeatherConfig;
pub use panel::WeatherPanel;
pub use pill::WeatherPill;

// ─── < Tests > ────────────────────────────────────────────────────

#[doc(hidden)]
pub use icons::{UNKNOWN_WEATHER_ICON, weather_description, weather_icon};

#[doc(hidden)]
pub use location::{Coordinates, parse_detected_location};

#[doc(hidden)]
pub use mapper::{cardinal_label, parse_air_quality, parse_sea_info, parse_weather_snapshot};

#[doc(hidden)]
pub use view_air::{aqi_level_label, uv_level_label};

#[doc(hidden)]
pub use state::{AirQuality, DailyForecast, HourlyPoint, SeaInfo, Tide, UvInfo, WeatherSnapshot};
