// ─── < Imports > ────────────────────────────────────────────────────

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use super::metrics::MemoryInfo;

// ─── < Constants > ────────────────────────────────────────────────────

/// Muestras que guarda cada gráfico de historia (~último minuto).
pub const HISTORY_LEN: usize = 60;

// ─── < Structs > ────────────────────────────────────────────────────

/// Ventana deslizante de muestras para los gráficos.
#[derive(Debug, Clone, Default)]
pub struct History {
    values: VecDeque<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetricsSnapshot {
    pub cpu_percent: Option<f32>,
    pub memory: Option<MemoryInfo>,
    pub temperature_c: Option<f32>,
    pub uptime_seconds: Option<u64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DiskUsage {
    pub used_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WifiInfo {
    pub ssid: String,
    pub interface: String,
    pub signal_dbm: Option<i32>,
    pub band: Option<&'static str>,
    pub channel: Option<u32>,
    pub security: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkData {
    pub interface: Option<String>,
    pub down_rate_bps: Option<f32>,
    pub up_rate_bps: Option<f32>,
    pub down_history: History,
    pub session_rx_bytes: u64,
    pub session_tx_bytes: u64,
    pub wifi: Option<WifiInfo>,
    pub ipv4: Option<String>,
    pub gateway: Option<String>,
    pub dns: Vec<String>,
    pub latency_ms: Option<f32>,
    pub latency_history: History,
    pub jitter_ms: Option<f32>,
    pub loss_percent: Option<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct BatteryData {
    pub percent: Option<u8>,
    /// Estado crudo de sysfs en minúsculas: "charging", "discharging", "full".
    pub status: Option<String>,
    pub minutes_left: Option<u32>,
    pub power_w: Option<f32>,
    pub power_history: History,
    pub charge_wh: Option<f32>,
    pub charge_full_wh: Option<f32>,
    pub voltage_v: Option<f32>,
    pub cycles: Option<u32>,
    pub health_percent: Option<u8>,
    pub technology: Option<String>,
    pub name: Option<String>,
    pub cell_temp_c: Option<f32>,
    pub adapter_online: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PendingPackage {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdatesData {
    pub pending: Option<u32>,
    /// Primeros paquetes del checkupdates, para la lista del panel.
    pub packages: Vec<PendingPackage>,
    pub synced_minutes_ago: Option<u64>,
    pub mirror: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SystemData {
    pub kernel: Option<String>,
    pub metrics: Option<MetricsSnapshot>,
    pub cores: Option<u32>,
    pub load_avg: Option<f32>,
    pub swap_used_kb: Option<u64>,
    pub cpu_history: History,
    pub memory_history: History,
    pub temp_history: History,
    pub fan_rpm: Option<u32>,
    pub session_max_temp_c: Option<f32>,
    pub disk: Option<DiskUsage>,
    pub network: NetworkData,
    pub battery: Option<BatteryData>,
    pub updates: UpdatesData,
}

#[derive(Debug, Clone, Default)]
pub struct SystemStore {
    inner: Arc<Mutex<SystemData>>,
    panel_open: Arc<AtomicBool>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl History {
    pub fn push(&mut self, value: f32) {
        if self.values.len() >= HISTORY_LEN {
            self.values.pop_front();
        }

        self.values.push_back(value);
    }

    pub fn iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.values.iter().copied()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn max(&self) -> Option<f32> {
        self.values.iter().copied().reduce(f32::max)
    }
}

impl SystemStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> SystemData {
        self.lock().clone()
    }

    /// Mutación en bloque: el worker arma todo el tick bajo un solo lock.
    pub fn update(&self, mutate: impl FnOnce(&mut SystemData)) {
        mutate(&mut self.lock());
    }

    pub fn set_panel_open(&self, open: bool) {
        self.panel_open.store(open, Ordering::Release);
    }

    pub fn panel_open(&self) -> bool {
        self.panel_open.load(Ordering::Acquire)
    }

    fn lock(&self) -> MutexGuard<'_, SystemData> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("el mutex del store de sistema estaba envenenado; se recupera el último valor");
                poisoned.into_inner()
            }
        }
    }
}
