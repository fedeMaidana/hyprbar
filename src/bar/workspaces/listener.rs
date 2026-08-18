// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;
use calloop::channel::Sender;
use std::time::Duration;

use crate::app::{ShutdownToken, WorkerHandle};
use crate::hyprland_ipc::{self, EventStreamRead};

use super::mapper::parse_workspace_data;
use super::state::WorkspaceStore;

// ─── < Constants > ────────────────────────────────────────────────────

/// Backoff entre reintentos de conexión: rápido primero, hasta un techo.
const RECONNECT_DELAYS: [Duration; 4] = [
    Duration::from_secs(1),
    Duration::from_secs(2),
    Duration::from_secs(5),
    Duration::from_secs(10),
];

// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionEnd {
    /// El worker debe terminar.
    Shutdown,
    /// Nunca llegó a conectar (Hyprland todavía no está listo).
    NeverConnected,
    /// Estuvo conectado y el socket se cerró (restart de Hyprland).
    Lost,
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn spawn_listener(store: WorkspaceStore, redraw_signal: Sender<()>) -> Option<WorkerHandle> {
    match WorkerHandle::spawn("hyprland-workspaces-listener", move |shutdown| listener_loop(store, redraw_signal, shutdown)) {
        Ok(worker) => Some(worker),
        Err(error) => {
            log::error!("no se pudo iniciar el listener de hyprland: {error}");
            None
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

/// Reconecta para siempre: un restart de Hyprland o una carrera en el
/// arranque solo cuestan un reintento, nunca un worker muerto.
fn listener_loop(store: WorkspaceStore, redraw: Sender<()>, shutdown: ShutdownToken) {
    let mut failed_attempts: usize = 0;
    // Un solo warn por caída: mientras hyprland siga sin responder, los
    // reintentos loguean en debug para no llenar el journal.
    let mut announced_down = false;

    while !shutdown.should_stop() {
        let announce = !announced_down;
        let end = run_session(&store, &redraw, &shutdown, announce);

        let delay_index = match end {
            SessionEnd::Shutdown => break,
            SessionEnd::Lost => {
                // Estuvo andando: reintento rápido y con aviso.
                announced_down = false;
                failed_attempts = 0;
                0
            }
            SessionEnd::NeverConnected => {
                let index = failed_attempts.min(RECONNECT_DELAYS.len() - 1);
                failed_attempts += 1;
                index
            }
        };

        let delay = RECONNECT_DELAYS[delay_index];

        if end == SessionEnd::NeverConnected && !announce {
            log::debug!("workspaces: reintento de conexión a hyprland en {delay:?}");
        } else {
            log::warn!("workspaces: reintento de conexión a hyprland en {delay:?}");
        }

        if end == SessionEnd::NeverConnected {
            announced_down = true;
        }

        if shutdown.sleep(delay) {
            break;
        }
    }

    log::info!("listener de workspaces detenido");
}

/// Conecta el stream, hace el fetch inicial y procesa eventos hasta que
/// el socket se cierre o pidan apagar. `announce` decide si las fallas
/// de conexión merecen warn o ya son el reintento silencioso de siempre.
fn run_session(store: &WorkspaceStore, redraw: &Sender<()>, shutdown: &ShutdownToken, announce: bool) -> SessionEnd {
    // El stream se abre antes del fetch inicial para no perder eventos
    // que lleguen entre uno y otro.
    let mut stream = match hyprland_ipc::event_stream() {
        Ok(stream) => stream,
        Err(error) => {
            if announce {
                log::warn!("no se pudo abrir el event stream de hyprland: {error}");
            } else {
                log::debug!("no se pudo abrir el event stream de hyprland: {error}");
            }

            return SessionEnd::NeverConnected;
        }
    };

    if let Err(error) = refresh(store) {
        if announce {
            log::warn!("falló el fetch inicial de workspaces: {error}");
        } else {
            log::debug!("falló el fetch inicial de workspaces: {error}");
        }

        return SessionEnd::NeverConnected;
    }

    let _ = redraw.send(());
    log::info!("workspaces: conectado a hyprland");

    loop {
        if shutdown.should_stop() {
            return SessionEnd::Shutdown;
        }

        match stream.next_event() {
            Ok(EventStreamRead::Event(event)) => {
                log::debug!("evento de hyprland: {}>>{}", event.name, event.data);

                if should_refresh_after_event(&event.name) {
                    refresh_or_log(store, redraw, &event.name);
                }
            }
            Ok(EventStreamRead::Timeout) => {}
            Ok(EventStreamRead::Closed) => {
                log::warn!("el event stream de hyprland se cerró (¿restart del compositor?)");
                return SessionEnd::Lost;
            }
            Err(error) => {
                log::warn!("evento malformado: {error}");
            }
        }
    }
}

fn refresh_or_log(store: &WorkspaceStore, redraw: &Sender<()>, event_name: &str) {
    if let Err(error) = refresh(store) {
        log::warn!("falló el refresh tras el evento {event_name}: {error}");
        return;
    }

    let _ = redraw.send(());
}

fn refresh(store: &WorkspaceStore) -> Result<()> {
    let workspaces_json = hyprland_ipc::query("j/workspaces")?;
    let active_json = hyprland_ipc::query("j/activeworkspace")?;

    let data = parse_workspace_data(&workspaces_json, &active_json)?;

    store.replace(data);

    Ok(())
}

fn should_refresh_after_event(event_name: &str) -> bool {
    matches!(event_name, "workspace" | "createworkspace" | "destroyworkspace" | "focusedmon")
}
