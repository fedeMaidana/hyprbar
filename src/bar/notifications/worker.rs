// ─── < Imports > ────────────────────────────────────────────────────

use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use calloop::channel::Sender;
use serde::de::DeserializeOwned;

use crate::app::{ShutdownToken, WorkerHandle};

use super::state::{HistoryNote, HyprnotifyState, NotificationsStore};

// ─── < Constants > ────────────────────────────────────────────────────

/// Cada cuánto miramos los contratos de hyprnotify.
const POLL_INTERVAL: Duration = Duration::from_secs(1);

// ─── < Structs > ────────────────────────────────────────────────────

/// mtimes de la última lectura, para releer solo cuando algo cambió.
#[derive(Default)]
struct ContractWatch {
    state_modified: Option<SystemTime>,
    history_modified: Option<SystemTime>,
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn spawn_poller(store: NotificationsStore, redraw_signal: Sender<()>) -> Option<WorkerHandle> {
    match WorkerHandle::spawn("hyprnotify-poller", move |shutdown| poller_loop(store, redraw_signal, shutdown)) {
        Ok(worker) => Some(worker),
        Err(error) => {
            log::error!("no se pudo iniciar el poller de notificaciones: {error}");
            None
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn poller_loop(store: NotificationsStore, redraw: Sender<()>, shutdown: ShutdownToken) {
    let mut watch = ContractWatch::default();

    while !shutdown.should_stop() {
        if poll_contracts(&store, &mut watch) {
            let _ = redraw.send(());
        }

        if shutdown.sleep(POLL_INTERVAL) {
            break;
        }
    }

    log::info!("poller de notificaciones detenido");
}

/// Relee los contratos cuyo mtime cambió; devuelve si hubo novedades.
fn poll_contracts(store: &NotificationsStore, watch: &mut ContractWatch) -> bool {
    let mut changed = false;

    let state_path = cache_path("state.json");
    let modified = file_modified(&state_path);

    if modified != watch.state_modified {
        watch.state_modified = modified;

        let state: HyprnotifyState = read_json_file(&state_path).unwrap_or_default();

        store.replace_state(state);
        changed = true;
    }

    let history_path = cache_path("history.json");
    let modified = file_modified(&history_path);

    if modified != watch.history_modified {
        watch.history_modified = modified;

        let mut notes: Vec<HistoryNote> = read_json_file(&history_path).unwrap_or_default();

        notes.reverse(); // el archivo va de vieja a nueva

        store.replace_notes(notes);
        changed = true;
    }

    changed
}

fn file_modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok().and_then(|meta| meta.modified().ok())
}

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
