// ─── < Imports > ────────────────────────────────────────────────────

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::state::Alarm;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize, Deserialize)]
struct AlarmsFile {
    alarms: Vec<Alarm>,
}

// ─── < Public Functions > ────────────────────────────────────────────────────

/// Carga las alarmas guardadas; sin archivo (o roto) arranca vacío.
pub(crate) fn load() -> Vec<Alarm> {
    let Some(path) = alarms_path() else {
        return Vec::new();
    };

    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => {
            log::warn!("no se pudieron leer las alarmas de {path:?}: {error}");
            return Vec::new();
        }
    };

    match serde_json::from_str::<AlarmsFile>(&contents) {
        Ok(file) => file.alarms,
        Err(error) => {
            log::warn!("alarmas ilegibles en {path:?}: {error}");
            Vec::new()
        }
    }
}

/// Persiste las alarmas; los errores se loguean y la sesión sigue.
pub(crate) fn save(alarms: &[Alarm]) {
    let Some(path) = alarms_path() else {
        log::warn!("sin directorio de estado; las alarmas no se guardan");
        return;
    };

    if let Some(parent) = path.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        log::warn!("no se pudo crear {parent:?}: {error}");
        return;
    }

    let file = AlarmsFile { alarms: alarms.to_vec() };

    let contents = match serde_json::to_string_pretty(&file) {
        Ok(contents) => contents,
        Err(error) => {
            log::warn!("no se pudieron serializar las alarmas: {error}");
            return;
        }
    };

    if let Err(error) = fs::write(&path, contents) {
        log::warn!("no se pudieron guardar las alarmas en {path:?}: {error}");
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

/// `$XDG_STATE_HOME/hyprbar/alarms.json`, con `~/.local/state` de respaldo.
fn alarms_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))?;

    Some(base.join("hyprbar/alarms.json"))
}
