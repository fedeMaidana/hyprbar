// ─── < Imports > ────────────────────────────────────────────────────

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

// ─── < Types > ────────────────────────────────────────────────────

pub type WorkspaceId = u8;

// ─── < Constants > ────────────────────────────────────────────────────

const MIN_VISIBLE_WORKSPACES: WorkspaceId = 3;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceData {
    pub existing: Vec<WorkspaceId>,
    pub active_id: WorkspaceId,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceStore {
    inner: Arc<Mutex<WorkspaceData>>,
    /// Sube con cada escritura; los lectores clonan solo cuando cambió.
    generation: Arc<AtomicU64>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl WorkspaceData {
    pub fn visible_count(&self) -> WorkspaceId {
        let max_id = self.existing.iter().copied().max().unwrap_or(0).max(self.active_id);

        max_id.max(MIN_VISIBLE_WORKSPACES)
    }
}

impl WorkspaceStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> WorkspaceData {
        self.lock().clone()
    }

    /// Número de escrituras acumuladas: si no cambió desde la última
    /// lectura, no hace falta volver a clonar los datos.
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub fn replace(&self, data: WorkspaceData) {
        *self.lock() = data;

        self.generation.fetch_add(1, Ordering::Release);
    }

    fn lock(&self) -> MutexGuard<'_, WorkspaceData> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("el mutex del store de workspaces estaba envenenado; se recupera el último valor");
                poisoned.into_inner()
            }
        }
    }
}
