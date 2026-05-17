use std::time::Duration;

#[derive(Debug, Clone)]
pub struct WeatherConfig {
    pub latitude: f64,
    pub longitude: f64,
    pub fetch_interval: Duration,
}

impl WeatherConfig {
    pub fn mar_del_plata() -> Self {
        Self {
            latitude: -38.0023,
            longitude: -57.5575,
            fetch_interval: Duration::from_secs(600),
        }
    }
}
