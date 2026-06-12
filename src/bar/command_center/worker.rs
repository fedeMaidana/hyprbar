// ─── < Imports > ────────────────────────────────────────────────────

use std::thread;
use std::time::{Duration, Instant};

use calloop::channel::Sender;

use crate::app::{ShutdownToken, WorkerHandle};

use super::control;
use super::state::{CommandData, CommandStore};

// ─── < Constants > ────────────────────────────────────────────────────

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const WAIT_SLICE: Duration = Duration::from_millis(250);

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn spawn_poller(store: CommandStore, redraw_signal: Sender<()>) -> Option<WorkerHandle> {
    match WorkerHandle::spawn("command-center-poller", move |shutdown| poll_loop(store, redraw_signal, shutdown)) {
        Ok(worker) => Some(worker),
        Err(error) => {
            log::error!("command center poller spawn failed: {error}");
            None
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn poll_loop(store: CommandStore, redraw: Sender<()>, shutdown: ShutdownToken) {
    while !shutdown.should_stop() {
        if !store.panel_open() {
            thread::sleep(WAIT_SLICE);
            continue;
        }

        store.replace(read_data());

        let _ = redraw.send(());

        sleep_poll_interval(&store, &shutdown);
    }

    log::info!("command center poller stopped");
}

fn read_data() -> CommandData {
    CommandData {
        sink: log_failure("sink volume", control::read_sink()),
        mic_muted: log_failure("mic mute", control::read_mic_muted()),
        brightness: log_failure("brightness", control::read_brightness()),
        media: control::read_media(),
    }
}

fn sleep_poll_interval(store: &CommandStore, shutdown: &ShutdownToken) {
    let started_at = Instant::now();

    while started_at.elapsed() < POLL_INTERVAL {
        if shutdown.should_stop() || !store.panel_open() {
            return;
        }

        thread::sleep(WAIT_SLICE);
    }
}

fn log_failure<T>(what: &str, result: anyhow::Result<T>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            log::debug!("command center: {what} read failed: {error}");
            None
        }
    }
}
