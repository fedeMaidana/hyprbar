// ─── < Imports > ────────────────────────────────────────────────────

use chrono::NaiveDate;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct WeatherSnapshot {
    pub temp_c: f32,
    pub weather_code: u8,
    pub feels_like_c: Option<f32>,
    pub humidity_percent: Option<u8>,
    pub wind_kmh: Option<f32>,
    pub precipitation_percent: Option<u8>,
    pub daily: Vec<DailyForecast>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DailyForecast {
    pub date: NaiveDate,
    pub weather_code: u8,
    pub max_c: f32,
    pub min_c: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WeatherData {
    pub snapshot: Option<WeatherSnapshot>,
    pub location_label: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct WeatherStore {
    inner: Arc<Mutex<WeatherData>>,
    /// Sube con cada escritura; los lectores clonan solo cuando cambió.
    generation: Arc<AtomicU64>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl WeatherSnapshot {
    pub fn new(temp_c: f32, weather_code: u8) -> Self {
        Self {
            temp_c,
            weather_code,
            feels_like_c: None,
            humidity_percent: None,
            wind_kmh: None,
            precipitation_percent: None,
            daily: Vec::new(),
        }
    }
}

impl WeatherStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data(&self) -> WeatherData {
        self.lock().clone()
    }

    /// Número de escrituras acumuladas: si no cambió desde la última
    /// lectura, no hace falta volver a clonar los datos.
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub fn replace_snapshot(&self, snapshot: WeatherSnapshot) {
        self.lock().snapshot = Some(snapshot);
        self.generation.fetch_add(1, Ordering::Release);
    }

    pub fn replace_location_label(&self, label: String) {
        self.lock().location_label = Some(label);
        self.generation.fetch_add(1, Ordering::Release);
    }

    fn lock(&self) -> MutexGuard<'_, WeatherData> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("el mutex del store del clima estaba envenenado; se recupera el último valor");
                poisoned.into_inner()
            }
        }
    }
}
