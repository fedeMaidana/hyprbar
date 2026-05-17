// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;
use calloop::channel::Sender;
use std::time::Duration;

use crate::app::{ShutdownToken, WorkerHandle};
use crate::hyprland_ipc::{self, EventStreamRead};

use super::mapper::parse_workspace_data;
use super::state::WorkspaceStore;

// ─── < Constants > ────────────────────────────────────────────────────

const MAX_INITIAL_FETCH_RETRIES: u8 = 5;
const INITIAL_FETCH_RETRY_DELAY: Duration = Duration::from_millis(500);

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn spawn_listener(store: WorkspaceStore, redraw_signal: Sender<()>) -> Option<WorkerHandle> {
    match WorkerHandle::spawn("hyprland-workspaces-listener", move |shutdown| listener_loop(store, redraw_signal, shutdown)) {
        Ok(worker) => Some(worker),
        Err(error) => {
            log::error!("hyprland listener spawn failed: {error}");
            None
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn listener_loop(store: WorkspaceStore, redraw: Sender<()>, shutdown: ShutdownToken) {
    if !wait_for_initial_refresh(&store, &redraw, &shutdown) {
        return;
    }

    let mut stream = match hyprland_ipc::event_stream() {
        Ok(stream) => stream,
        Err(error) => {
            log::error!("hyprland event_stream: {error}");
            return;
        }
    };

    loop {
        if shutdown.should_stop() {
            break;
        }

        match stream.next_event() {
            Ok(EventStreamRead::Event(event)) => {
                log::debug!("hyprland event: {}>>{}", event.name, event.data);

                if should_refresh_after_event(&event.name) {
                    refresh_or_log(&store, &redraw, &event.name);
                }
            }
            Ok(EventStreamRead::Timeout) => {}
            Ok(EventStreamRead::Closed) => {
                log::warn!("hyprland event stream terminó (socket cerrado)");
                break;
            }
            Err(error) => {
                log::warn!("event parse: {error}");
            }
        }
    }

    log::info!("hyprland workspaces listener stopped");
}

fn wait_for_initial_refresh(store: &WorkspaceStore, redraw: &Sender<()>, shutdown: &ShutdownToken) -> bool {
    let mut retries = 0;

    loop {
        if shutdown.should_stop() {
            return false;
        }

        match refresh(store) {
            Ok(()) => {
                let _ = redraw.send(());
                log::info!("hyprland workspaces: fetch inicial OK");
                return true;
            }
            Err(error) => {
                retries += 1;

                if retries > MAX_INITIAL_FETCH_RETRIES {
                    log::error!("hyprland fetch falló {MAX_INITIAL_FETCH_RETRIES} veces: {error}");
                    return false;
                }

                log::warn!("hyprland fetch (retry {retries}/{MAX_INITIAL_FETCH_RETRIES}): {error}");

                if shutdown.sleep(INITIAL_FETCH_RETRY_DELAY) {
                    return false;
                }
            }
        }
    }
}

fn refresh_or_log(store: &WorkspaceStore, redraw: &Sender<()>, event_name: &str) {
    if let Err(error) = refresh(store) {
        log::warn!("refresh tras evento {event_name}: {error}");
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
