// ─── < Imports > ────────────────────────────────────────────────────

use chrono::NaiveDate;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

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
    /// Las 24 horas del día calendario, para la curva del pronóstico.
    pub hourly: Vec<HourlyPoint>,
    pub uv: Option<UvInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DailyForecast {
    pub date: NaiveDate,
    pub weather_code: u8,
    pub max_c: f32,
    pub min_c: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HourlyPoint {
    pub hour: u8,
    pub temp_c: f32,
    pub weather_code: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvInfo {
    pub current: f32,
    pub peak: f32,
    pub peak_hour: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AirQuality {
    /// US AQI: hasta 50 buena, hasta 100 moderada, más es mala.
    pub aqi: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeaInfo {
    pub water_temp_c: Option<f32>,
    pub wave_height_m: Option<f32>,
    pub wave_direction: Option<&'static str>,
    pub tide_high: Option<Tide>,
    pub tide_low: Option<Tide>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tide {
    /// "18:40".
    pub time: String,
    pub height_m: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WeatherData {
    pub snapshot: Option<WeatherSnapshot>,
    pub location_label: Option<String>,
    pub air: Option<AirQuality>,
    pub sea: Option<SeaInfo>,
    /// Cuándo llegó el último snapshot ("Actualizado hace N min").
    pub updated_at: Option<Instant>,
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
            hourly: Vec::new(),
            uv: None,
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
        let mut data = self.lock();

        data.snapshot = Some(snapshot);
        data.updated_at = Some(Instant::now());
        drop(data);

        self.generation.fetch_add(1, Ordering::Release);
    }

    pub fn replace_location_label(&self, label: String) {
        self.lock().location_label = Some(label);
        self.generation.fetch_add(1, Ordering::Release);
    }

    pub fn replace_air(&self, air: AirQuality) {
        self.lock().air = Some(air);
        self.generation.fetch_add(1, Ordering::Release);
    }

    pub fn replace_sea(&self, sea: SeaInfo) {
        self.lock().sea = Some(sea);
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
