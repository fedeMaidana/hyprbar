// ─── < Imports > ────────────────────────────────────────────────────

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use super::metrics::MemoryInfo;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetricsSnapshot {
    pub cpu_percent: Option<f32>,
    pub memory: Option<MemoryInfo>,
    pub temperature_c: Option<f32>,
    pub uptime_seconds: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SystemData {
    pub kernel: Option<String>,
    pub metrics: Option<MetricsSnapshot>,
    pub pending_updates: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct SystemStore {
    inner: Arc<Mutex<SystemData>>,
    panel_open: Arc<AtomicBool>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl SystemStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> SystemData {
        self.lock().clone()
    }

    pub fn replace_metrics(&self, metrics: MetricsSnapshot) {
        self.lock().metrics = Some(metrics);
    }

    pub fn replace_kernel(&self, kernel: String) {
        self.lock().kernel = Some(kernel);
    }

    pub fn replace_pending_updates(&self, count: u32) {
        self.lock().pending_updates = Some(count);
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
