// ─── < Imports > ────────────────────────────────────────────────────

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioState {
    pub volume: f32,
    pub muted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiState {
    pub enabled: bool,
    pub ssid: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandData {
    pub sink: Option<AudioState>,
    pub mic_muted: Option<bool>,
    pub wifi: Option<WifiState>,
}

#[derive(Debug, Clone, Default)]
pub struct CommandStore {
    inner: Arc<Mutex<CommandData>>,
    panel_open: Arc<AtomicBool>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl CommandStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data(&self) -> CommandData {
        self.lock().clone()
    }

    pub fn replace(&self, data: CommandData) {
        *self.lock() = data;
    }

    pub fn update(&self, mutate: impl FnOnce(&mut CommandData)) {
        mutate(&mut self.lock());
    }

    pub fn set_panel_open(&self, open: bool) {
        self.panel_open.store(open, Ordering::Release);
    }

    pub fn panel_open(&self) -> bool {
        self.panel_open.load(Ordering::Acquire)
    }

    fn lock(&self) -> MutexGuard<'_, CommandData> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("el mutex del store del command center estaba envenenado; se recupera el último valor");
                poisoned.into_inner()
            }
        }
    }
}
