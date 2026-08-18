// ─── < Imports > ────────────────────────────────────────────────────

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

// ─── < Constants > ────────────────────────────────────────────────────

pub(crate) const MAX_VISIBLE_ROWS: usize = 8;

// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
pub enum NoteUrgency {
    Low,
    #[default]
    Normal,
    Critical,
}

// ─── < Structs > ────────────────────────────────────────────────────

/// El contrato `state.json` que escribe hyprnotify en su cache.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct HyprnotifyState {
    #[serde(default)]
    pub dnd: bool,
    #[serde(default)]
    pub history_count: usize,
}

/// Una entrada de `history.json` (contrato de hyprnotify).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct HistoryNote {
    #[serde(default)]
    pub app_name: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub urgency: NoteUrgency,
    #[serde(default)]
    pub closed_at_unix: u64,
}

/// Contratos de hyprnotify tal como los publica el worker.
#[derive(Debug, Clone, Default)]
pub(crate) struct NotificationsData {
    pub(crate) state: HyprnotifyState,
    /// Historial de más nueva a más vieja.
    pub(crate) notes: Vec<HistoryNote>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct NotificationsStore {
    inner: Arc<Mutex<NotificationsData>>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl NotificationsData {
    pub(crate) fn visible_rows(&self) -> usize {
        self.notes.len().min(MAX_VISIBLE_ROWS)
    }

    pub(crate) fn max_scroll(&self) -> usize {
        self.notes.len().saturating_sub(MAX_VISIBLE_ROWS)
    }
}

impl NotificationsStore {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn data(&self) -> NotificationsData {
        self.lock().clone()
    }

    pub(crate) fn state(&self) -> HyprnotifyState {
        self.lock().state
    }

    pub(crate) fn replace_state(&self, state: HyprnotifyState) {
        self.lock().state = state;
    }

    pub(crate) fn replace_notes(&self, notes: Vec<HistoryNote>) {
        self.lock().notes = notes;
    }

    /// Vacía el estado local tras pedirle a hyprnotify que limpie.
    pub(crate) fn clear_history(&self) {
        let mut data = self.lock();

        data.notes.clear();
        data.state.history_count = 0;
    }

    fn lock(&self) -> MutexGuard<'_, NotificationsData> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("el mutex del store de notificaciones estaba envenenado; se recupera el último valor");
                poisoned.into_inner()
            }
        }
    }
}

// ─── < Public Functions > ────────────────────────────────────────────────────

/// Edad compacta: "ahora", "5m", "3h", "2d".
pub fn age(seconds: u64) -> String {
    match seconds {
        0..60 => "ahora".to_owned(),
        60..3600 => format!("{}m", seconds / 60),
        3600..86400 => format!("{}h", seconds / 3600),
        _ => format!("{}d", seconds / 86400),
    }
}

pub(crate) fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
