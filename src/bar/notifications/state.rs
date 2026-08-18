// ─── < Imports > ────────────────────────────────────────────────────

use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde::Deserialize;
use serde::de::DeserializeOwned;

// ─── < Constants > ────────────────────────────────────────────────────

/// Cada cuánto miramos el contrato de hyprnotify como mucho.
const POLL_INTERVAL: Duration = Duration::from_secs(2);

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

/// Estado local del pill: contratos de hyprnotify + scroll del panel.
pub(crate) struct NotificationsState {
    pub(crate) state: HyprnotifyState,
    state_checked: Option<Instant>,
    state_modified: Option<SystemTime>,
    /// Historial de más nueva a más vieja, cargado con el panel abierto.
    pub(crate) notes: Vec<HistoryNote>,
    notes_modified: Option<SystemTime>,
    /// Scroll discreto, en filas.
    pub(crate) scroll: usize,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl NotificationsState {
    pub(crate) fn new() -> Self {
        Self {
            state: HyprnotifyState::default(),
            state_checked: None,
            state_modified: None,
            notes: Vec::new(),
            notes_modified: None,
            scroll: 0,
        }
    }

    /// Relee `state.json` solo si pasó el intervalo y cambió el mtime.
    pub(crate) fn refresh_state(&mut self) {
        let now = Instant::now();

        if self.state_checked.is_some_and(|checked| now - checked < POLL_INTERVAL) {
            return;
        }

        self.state_checked = Some(now);

        let path = cache_path("state.json");
        let modified = fs::metadata(&path).ok().and_then(|meta| meta.modified().ok());

        if modified == self.state_modified {
            return;
        }

        self.state_modified = modified;
        self.state = read_json_file(&path).unwrap_or_default();
    }

    /// Relee `history.json` cuando cambia; solo corre con el panel abierto.
    pub(crate) fn refresh_notes(&mut self) {
        let path = cache_path("history.json");
        let modified = fs::metadata(&path).ok().and_then(|meta| meta.modified().ok());

        if modified == self.notes_modified {
            return;
        }

        self.notes_modified = modified;

        let mut notes: Vec<HistoryNote> = read_json_file(&path).unwrap_or_default();

        notes.reverse(); // el archivo va de vieja a nueva

        self.notes = notes;
        self.scroll = self.scroll.min(self.max_scroll());
    }

    /// Vacía el estado local tras pedirle a hyprnotify que limpie.
    pub(crate) fn clear(&mut self) {
        self.notes.clear();
        self.state.history_count = 0;
        self.scroll = 0;
    }

    pub(crate) fn visible_rows(&self) -> usize {
        self.notes.len().min(MAX_VISIBLE_ROWS)
    }

    pub(crate) fn max_scroll(&self) -> usize {
        self.notes.len().saturating_sub(MAX_VISIBLE_ROWS)
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

// ─── < Private Functions > ────────────────────────────────────────────────────

/// Lee y parsea un contrato JSON; un archivo ausente es normal (sin log),
/// pero un archivo corrupto o ilegible queda registrado.
fn read_json_file<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return None,
        Err(error) => {
            log::warn!("no pude leer {}: {error}", path.display());
            return None;
        }
    };

    match serde_json::from_str(&content) {
        Ok(value) => Some(value),
        Err(error) => {
            log::warn!("contrato inválido en {}: {error}", path.display());
            None
        }
    }
}

fn cache_path(file: &str) -> PathBuf {
    let base = if let Some(cache_home) = env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(cache_home)
    } else {
        env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".cache")
    };

    base.join("hyprnotify").join(file)
}
