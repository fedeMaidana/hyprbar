// ─── < Imports > ────────────────────────────────────────────────────

use std::sync::{Arc, Mutex};

// ─── < Constants > ────────────────────────────────────────────────────

const MIN_VISIBLE_WORKSPACES: i32 = 3;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceData {
    pub existing: Vec<i32>,
    pub active_id: i32,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceStore {
    inner: Arc<Mutex<WorkspaceData>>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl WorkspaceData {
    pub fn visible_count(&self) -> i32 {
        let max_id = self.existing.iter().copied().max().unwrap_or(0).max(self.active_id);

        max_id.max(MIN_VISIBLE_WORKSPACES)
    }
}

impl WorkspaceStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> WorkspaceData {
        self.inner.lock().unwrap().clone()
    }

    pub fn replace(&self, data: WorkspaceData) {
        *self.inner.lock().unwrap() = data;
    }
}
