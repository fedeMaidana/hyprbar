// ─── < Imports > ────────────────────────────────────────────────────

use std::sync::{Arc, Mutex};

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeatherSnapshot {
    pub temp_c: f32,
    pub weather_code: u32,
}

#[derive(Debug, Clone, Default)]
pub struct WeatherStore {
    inner: Arc<Mutex<Option<WeatherSnapshot>>>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl WeatherSnapshot {
    pub fn new(temp_c: f32, weather_code: u32) -> Self {
        Self { temp_c, weather_code }
    }
}

impl WeatherStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> Option<WeatherSnapshot> {
        *self.inner.lock().unwrap()
    }

    pub fn replace(&self, snapshot: WeatherSnapshot) {
        *self.inner.lock().unwrap() = Some(snapshot);
    }
}
