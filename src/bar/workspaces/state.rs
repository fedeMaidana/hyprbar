// ─── < Imports > ────────────────────────────────────────────────────

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

    pub fn replace(&self, data: WorkspaceData) {
        let mut guard = self.lock();

        *guard = data;
    }

    fn lock(&self) -> MutexGuard<'_, WorkspaceData> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("workspace store mutex was poisoned; recovering latest value");
                poisoned.into_inner()
            }
        }
    }
}
