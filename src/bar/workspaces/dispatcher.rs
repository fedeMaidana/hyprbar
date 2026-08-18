// ─── < Imports > ────────────────────────────────────────────────────

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use crate::app::{ShutdownToken, WorkerHandle};
use crate::hyprland_ipc::{self, WorkspaceTarget};

// ─── < Constants > ────────────────────────────────────────────────────

/// Granularidad con la que el worker revisa si le pidieron apagarse.
const DISPATCH_POLL: Duration = Duration::from_millis(250);

// ─── < Public Functions > ────────────────────────────────────────────────────

/// Worker que serializa los dispatches a Hyprland fuera del hilo del
/// loop: un click o un scroll solo encolan; el connect/write/read
/// bloqueante pasa acá.
pub(crate) fn spawn_dispatcher() -> (Option<WorkerHandle>, Sender<WorkspaceTarget>) {
    let (sender, receiver) = mpsc::channel();

    match WorkerHandle::spawn("hyprland-workspace-dispatcher", move |shutdown| dispatcher_loop(receiver, shutdown)) {
        Ok(worker) => (Some(worker), sender),
        Err(error) => {
            log::error!("no se pudo iniciar el dispatcher de workspaces: {error}");
            (None, sender)
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn dispatcher_loop(receiver: Receiver<WorkspaceTarget>, shutdown: ShutdownToken) {
    while !shutdown.should_stop() {
        match receiver.recv_timeout(DISPATCH_POLL) {
            Ok(target) => dispatch_or_log(target),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    log::info!("dispatcher de workspaces detenido");
}

fn dispatch_or_log(target: WorkspaceTarget) {
    match hyprland_ipc::dispatch_workspace_target(target) {
        Ok(()) => log::info!("dispatch de workspace enviado: {target:?}"),
        Err(error) => log::warn!("falló el dispatch de workspace ({target:?}): {error}"),
    }
}
