// ─── < Imports > ────────────────────────────────────────────────────

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioState {
    pub volume: f32,
    pub muted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrightnessState {
    pub fraction: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaState {
    pub playing: bool,
    pub title: String,
    pub artist: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandData {
    pub sink: Option<AudioState>,
    pub mic_muted: Option<bool>,
    pub brightness: Option<BrightnessState>,
    pub media: Option<MediaState>,
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
                log::warn!("command store mutex was poisoned; recovering latest value");
                poisoned.into_inner()
            }
        }
    }
}
