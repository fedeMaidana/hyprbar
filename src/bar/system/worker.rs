// ─── < Imports > ────────────────────────────────────────────────────

use std::thread;
use std::time::{Duration, Instant};

use calloop::channel::Sender;

use crate::app::{ShutdownToken, WorkerHandle};

use super::metrics::{CpuTimes, cpu_usage_percent};
use super::sampler;
use super::state::{MetricsSnapshot, SystemStore};
use super::updates;

// ─── < Constants > ────────────────────────────────────────────────────

const OPEN_SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
const CLOSED_SAMPLE_INTERVAL: Duration = Duration::from_secs(5);
const UPDATES_CHECK_INTERVAL: Duration = Duration::from_secs(30 * 60);
const WAIT_SLICE: Duration = Duration::from_millis(250);

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn spawn_sampler(store: SystemStore, redraw_signal: Sender<()>) -> Option<WorkerHandle> {
    match WorkerHandle::spawn("system-sampler", move |shutdown| sampler_loop(store, redraw_signal, shutdown)) {
        Ok(worker) => Some(worker),
        Err(error) => {
            log::error!("system sampler spawn failed: {error}");
            None
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn sampler_loop(store: SystemStore, redraw: Sender<()>, shutdown: ShutdownToken) {
    match sampler::read_kernel_version() {
        Ok(kernel) => store.replace_kernel(kernel),
        Err(error) => log::warn!("kernel version read failed: {error}"),
    }

    let mut previous_cpu: Option<CpuTimes> = None;
    let mut last_updates_check: Option<Instant> = None;

    while !shutdown.should_stop() {
        if updates_check_due(last_updates_check) {
            last_updates_check = Some(Instant::now());

            match updates::check_pending_updates() {
                Ok(count) => {
                    log::info!("pending updates: {count}");
                    store.replace_pending_updates(count);
                }
                Err(error) => {
                    log::warn!("updates check failed: {error}");
                }
            }
        }

        let snapshot = read_metrics(&mut previous_cpu);
        store.replace_metrics(snapshot);

        if store.panel_open() {
            let _ = redraw.send(());
        }

        if wait_for_next_sample(&store, &shutdown) {
            break;
        }
    }

    log::info!("system sampler stopped");
}

fn read_metrics(previous_cpu: &mut Option<CpuTimes>) -> MetricsSnapshot {
    let current_cpu = log_failure("cpu times", sampler::read_cpu_times());

    let cpu_percent = match (*previous_cpu, current_cpu) {
        (Some(previous), Some(current)) => cpu_usage_percent(previous, current),
        _ => None,
    };

    if current_cpu.is_some() {
        *previous_cpu = current_cpu;
    }

    MetricsSnapshot {
        cpu_percent,
        memory: log_failure("memory info", sampler::read_memory_info()),
        temperature_c: sampler::read_cpu_temperature(),
        uptime_seconds: log_failure("uptime", sampler::read_uptime_seconds()),
    }
}

fn updates_check_due(last_check: Option<Instant>) -> bool {
    last_check.is_none_or(|checked_at| checked_at.elapsed() >= UPDATES_CHECK_INTERVAL)
}

fn wait_for_next_sample(store: &SystemStore, shutdown: &ShutdownToken) -> bool {
    let started_at = Instant::now();
    let was_open = store.panel_open();

    loop {
        if shutdown.should_stop() {
            return true;
        }

        let open = store.panel_open();

        if open && !was_open {
            return false;
        }

        let interval = if open { OPEN_SAMPLE_INTERVAL } else { CLOSED_SAMPLE_INTERVAL };

        if started_at.elapsed() >= interval {
            return false;
        }

        thread::sleep(WAIT_SLICE);
    }
}

fn log_failure<T>(what: &str, result: anyhow::Result<T>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            log::debug!("system sampler: {what} read failed: {error}");
            None
        }
    }
}
