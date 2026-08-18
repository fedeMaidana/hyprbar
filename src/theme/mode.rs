// ─── < Imports > ────────────────────────────────────────────────────

use std::path::PathBuf;
use std::{env, fs};

use crate::proc::spawn_detached;

// ─── < Constants > ────────────────────────────────────────────────────

const STATE_DIR: &str = "hyprbar";
const STATE_FILE: &str = "theme";
const HOOK_FILE: &str = "theme-hook.sh";

// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl ThemeMode {
    pub fn toggled(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    /// Parsea el valor persistido, tolerando mayúsculas y espacios.
    #[doc(hidden)]
    pub fn from_persisted(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "dark" => Some(Self::Dark),
            "light" => Some(Self::Light),
            _ => None,
        }
    }
}

// ─── < Public Functions > ────────────────────────────────────────────────────

/// Reads the persisted theme mode, defaulting to dark. A missing file is
/// normal (primer arranque); un contenido irreconocible queda logueado.
pub fn load_preferred() -> ThemeMode {
    let Some(content) = state_file_path().and_then(|path| fs::read_to_string(path).ok()) else {
        return ThemeMode::default();
    };

    match ThemeMode::from_persisted(&content) {
        Some(mode) => mode,
        None => {
            log::warn!("modo de theme persistido no reconocido: {:?}; se usa dark", content.trim());
            ThemeMode::default()
        }
    }
}

/// Persists the theme mode so it survives restarts.
pub fn persist(mode: ThemeMode) {
    let Some(path) = state_file_path() else {
        return;
    };

    if let Some(parent) = path.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        log::warn!("no se pudo crear el directorio de estado del theme: {error}");
        return;
    }

    if let Err(error) = fs::write(&path, mode.as_str()) {
        log::warn!("no se pudo guardar el modo del theme: {error}");
    }
}

/// Runs the user hook (if present) with the new mode as its argument.
pub fn run_hook(mode: ThemeMode) {
    let Some(path) = hook_file_path() else {
        return;
    };

    if !path.is_file() {
        return;
    }

    let Some(program) = path.to_str() else {
        return;
    };

    if let Err(error) = spawn_detached(program, &[mode.as_str()]) {
        log::warn!("falló el hook del theme: {error}");
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn state_file_path() -> Option<PathBuf> {
    let base = env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("state")))?;

    Some(base.join(STATE_DIR).join(STATE_FILE))
}

fn hook_file_path() -> Option<PathBuf> {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;

    Some(base.join(STATE_DIR).join(HOOK_FILE))
}
