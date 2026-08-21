// ─── < Imports > ────────────────────────────────────────────────────

use std::collections::VecDeque;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use calloop::channel::Sender;

use crate::app::{ShutdownToken, WorkerHandle};

use super::battery;
use super::metrics::{CpuTimes, cpu_usage_percent};
use super::net;
use super::sampler;
use super::state::{MetricsSnapshot, SystemStore};
use super::updates;

// ─── < Constants > ────────────────────────────────────────────────────

const OPEN_SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
const CLOSED_SAMPLE_INTERVAL: Duration = Duration::from_secs(5);
const UPDATES_PERIODIC_INTERVAL: Duration = Duration::from_secs(30 * 60);
const UPDATES_OPEN_DEBOUNCE: Duration = Duration::from_secs(10);
const WAIT_SLICE: Duration = Duration::from_millis(250);

/// Cadencia de las lecturas lentas (disco, IPs, wifi, mirror).
const SLOW_READ_INTERVAL: Duration = Duration::from_secs(30);

/// Paquetes que se muestran en la lista del panel de updates.
const VISIBLE_PACKAGES: usize = 5;

/// A quién le hacemos ping para latencia/jitter/pérdida.
const PING_TARGET: &str = "1.1.1.1";
const PING_INTERVAL: Duration = Duration::from_secs(2);
const PING_WINDOW: usize = 30;

// ─── < Structs > ────────────────────────────────────────────────────

/// Estado local del sampler entre ticks.
#[derive(Default)]
struct SamplerLocals {
    previous_cpu: Option<CpuTimes>,
    fan_path: Option<PathBuf>,
    battery_dir: Option<PathBuf>,
    adapter_dir: Option<PathBuf>,
    /// Interfaz de la ruta por defecto, refrescada por las lecturas lentas.
    interface: Option<String>,
    net_counters: Option<NetCounters>,
    session_base: Option<(u64, u64)>,
    max_temp_c: Option<f32>,
    last_slow_read: Option<Instant>,
    last_updates_check: Option<Instant>,
    previous_panel_open: bool,
}

struct NetCounters {
    rx_bytes: u64,
    tx_bytes: u64,
    read_at: Instant,
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn spawn_sampler(store: SystemStore, redraw_signal: Sender<()>) -> Option<WorkerHandle> {
    match WorkerHandle::spawn("system-sampler", move |shutdown| sampler_loop(store, redraw_signal, shutdown)) {
        Ok(worker) => Some(worker),
        Err(error) => {
            log::error!("no se pudo iniciar el sampler de sistema: {error}");
            None
        }
    }
}

/// Sonda de red: ping periódico solo mientras el panel está abierto.
pub fn spawn_net_probe(store: SystemStore, redraw_signal: Sender<()>) -> Option<WorkerHandle> {
    match WorkerHandle::spawn("system-net-probe", move |shutdown| probe_loop(store, redraw_signal, shutdown)) {
        Ok(worker) => Some(worker),
        Err(error) => {
            log::error!("no se pudo iniciar la sonda de red: {error}");
            None
        }
    }
}

// ─── < Private Functions: Sampler > ────────────────────────────────────────────────────

fn sampler_loop(store: SystemStore, redraw: Sender<()>, shutdown: ShutdownToken) {
    let mut locals = SamplerLocals {
        fan_path: sampler::find_fan_input(),
        battery_dir: battery::find_battery_dir(),
        adapter_dir: battery::find_adapter_dir(),
        ..Default::default()
    };

    match sampler::read_kernel_version() {
        Ok(kernel) => store.update(|data| data.kernel = Some(kernel)),
        Err(error) => log::warn!("no se pudo leer la versión del kernel: {error}"),
    }

    store.update(|data| data.cores = std::thread::available_parallelism().ok().map(|cores| cores.get() as u32));

    while !shutdown.should_stop() {
        let panel_open = store.panel_open();
        let panel_just_opened = panel_open && !locals.previous_panel_open;
        locals.previous_panel_open = panel_open;

        sample_tick(&store, &mut locals);

        if panel_open {
            let _ = redraw.send(());
        }

        if updates_check_due(locals.last_updates_check, panel_just_opened) {
            locals.last_updates_check = Some(Instant::now());
            refresh_updates(&store);

            if store.panel_open() {
                let _ = redraw.send(());
            }
        }

        if wait_for_next_sample(&store, &shutdown) {
            break;
        }
    }

    log::info!("sampler de sistema detenido");
}

fn sample_tick(store: &SystemStore, locals: &mut SamplerLocals) {
    let metrics = read_metrics(&mut locals.previous_cpu);

    let load_avg = log_failure("loadavg", sampler::read_load_average());
    let swap_used_kb = log_failure("swap", sampler::read_swap_used_kb());
    let fan_rpm = locals.fan_path.as_deref().and_then(sampler::read_fan_rpm);

    if let Some(temp) = metrics.temperature_c {
        locals.max_temp_c = Some(locals.max_temp_c.map_or(temp, |max| max.max(temp)));
    }

    let battery = locals
        .battery_dir
        .as_deref()
        .map(|dir| battery::read_battery(dir, locals.adapter_dir.as_deref()));

    let slow_due = locals.last_slow_read.is_none_or(|last| last.elapsed() >= SLOW_READ_INTERVAL);

    let slow = if slow_due {
        locals.last_slow_read = Some(Instant::now());

        let sources = read_slow_sources();
        locals.interface = sources.interface.clone();

        Some(sources)
    } else {
        None
    };

    let rates = read_network_rates(locals);

    store.update(|data| {
        data.metrics = Some(metrics);
        data.load_avg = load_avg;
        data.swap_used_kb = swap_used_kb;
        data.fan_rpm = fan_rpm;
        data.session_max_temp_c = locals.max_temp_c;

        if let Some(cpu) = metrics.cpu_percent {
            data.cpu_history.push(cpu);
        }

        if let Some(memory) = metrics.memory {
            data.memory_history.push(memory.used_fraction());
        }

        if let Some(temp) = metrics.temperature_c {
            data.temp_history.push(temp);
        }

        if let Some((down, up, session_rx, session_tx)) = rates {
            data.network.down_rate_bps = Some(down);
            data.network.up_rate_bps = Some(up);
            data.network.down_history.push(down);
            data.network.session_rx_bytes = session_rx;
            data.network.session_tx_bytes = session_tx;
        }

        if let Some(mut fresh) = battery {
            // La historia de consumo vive acá, no en la lectura puntual.
            if let Some(previous) = data.battery.take() {
                fresh.power_history = previous.power_history;
            }

            if let Some(watts) = fresh.power_w {
                fresh.power_history.push(watts);
            }

            data.battery = Some(fresh);
        }

        if let Some(slow) = slow {
            if let Some(disk) = slow.disk {
                data.disk = Some(disk);
            }

            data.network.interface = slow.interface;
            data.network.gateway = slow.gateway;
            data.network.wifi = slow.wifi;
            data.network.ipv4 = slow.ipv4;
            data.network.dns = slow.dns;
            data.updates.synced_minutes_ago = slow.synced_minutes_ago;
            data.updates.mirror = slow.mirror;
        }
    });
}

struct SlowSources {
    disk: Option<super::state::DiskUsage>,
    interface: Option<String>,
    gateway: Option<String>,
    wifi: Option<super::state::WifiInfo>,
    ipv4: Option<String>,
    dns: Vec<String>,
    synced_minutes_ago: Option<u64>,
    mirror: Option<String>,
}

fn read_slow_sources() -> SlowSources {
    let disk = log_failure("disco", sampler::read_disk_usage());
    let route = log_failure("ruta por defecto", net::read_default_route());

    let (interface, gateway) = match route {
        Some((interface, gateway)) => (Some(interface), Some(gateway)),
        None => (None, None),
    };

    let wifi = interface
        .as_deref()
        .filter(|name| net::is_wireless(name))
        .and_then(net::read_wifi_info);

    let ipv4 = interface.as_deref().and_then(net::read_ipv4);

    SlowSources {
        disk,
        interface,
        gateway,
        wifi,
        ipv4,
        dns: net::read_nameservers(),
        synced_minutes_ago: updates::read_last_sync_minutes(),
        mirror: updates::read_mirror_host(),
    }
}

/// Rates en bytes/s desde los contadores de la interfaz por defecto.
/// Devuelve además los totales de la sesión (desde que arrancó la barra).
fn read_network_rates(locals: &mut SamplerLocals) -> Option<(f32, f32, u64, u64)> {
    let interface = locals.interface.clone()?;

    let (rx_bytes, tx_bytes) = net::read_interface_bytes(&interface).ok()?;
    let now = Instant::now();

    let session_base = *locals.session_base.get_or_insert((rx_bytes, tx_bytes));
    let session_rx = rx_bytes.saturating_sub(session_base.0);
    let session_tx = tx_bytes.saturating_sub(session_base.1);

    let rates = locals.net_counters.as_ref().map(|previous| {
        let dt = now.duration_since(previous.read_at).as_secs_f32().max(0.001);
        let down = rx_bytes.saturating_sub(previous.rx_bytes) as f32 / dt;
        let up = tx_bytes.saturating_sub(previous.tx_bytes) as f32 / dt;

        (down, up)
    });

    locals.net_counters = Some(NetCounters {
        rx_bytes,
        tx_bytes,
        read_at: now,
    });

    rates.map(|(down, up)| (down, up, session_rx, session_tx))
}

fn refresh_updates(store: &SystemStore) {
    match updates::check_pending_updates() {
        Ok(stdout) => {
            let count = updates::count_update_lines(&stdout);
            let mut packages = updates::parse_pending_packages(&stdout);

            packages.truncate(VISIBLE_PACKAGES);
            log::info!("updates pendientes: {count}");

            store.update(|data| {
                data.updates.pending = Some(count);
                data.updates.packages = packages;
            });
        }
        Err(error) => {
            log::warn!("falló el chequeo de updates: {error}");
        }
    }
}

fn read_metrics(previous_cpu: &mut Option<CpuTimes>) -> MetricsSnapshot {
    let current_cpu = log_failure("tiempos de cpu", sampler::read_cpu_times());

    let cpu_percent = match (*previous_cpu, current_cpu) {
        (Some(previous), Some(current)) => cpu_usage_percent(previous, current),
        _ => None,
    };

    if current_cpu.is_some() {
        *previous_cpu = current_cpu;
    }

    MetricsSnapshot {
        cpu_percent,
        memory: log_failure("memoria", sampler::read_memory_info()),
        temperature_c: sampler::read_cpu_temperature(),
        uptime_seconds: log_failure("uptime", sampler::read_uptime_seconds()),
    }
}

fn updates_check_due(last_check: Option<Instant>, panel_just_opened: bool) -> bool {
    let Some(checked_at) = last_check else {
        return true;
    };

    let elapsed = checked_at.elapsed();

    if panel_just_opened {
        elapsed >= UPDATES_OPEN_DEBOUNCE
    } else {
        elapsed >= UPDATES_PERIODIC_INTERVAL
    }
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

// ─── < Private Functions: Sonda de red > ────────────────────────────────────────────────────

fn probe_loop(store: SystemStore, redraw: Sender<()>, shutdown: ShutdownToken) {
    let mut window: VecDeque<Option<f32>> = VecDeque::new();

    while !shutdown.should_stop() {
        if !store.panel_open() {
            thread::sleep(WAIT_SLICE);
            continue;
        }

        let sample = net::ping_once(PING_TARGET);

        if window.len() >= PING_WINDOW {
            window.pop_front();
        }

        window.push_back(sample);

        let latency = sample;
        let jitter = jitter_of(&window);
        let loss = loss_of(&window);

        store.update(|data| {
            data.network.latency_ms = latency;
            data.network.jitter_ms = jitter;
            data.network.loss_percent = loss;

            if let Some(value) = latency {
                data.network.latency_history.push(value);
            }
        });

        if store.panel_open() {
            let _ = redraw.send(());
        }

        if shutdown.sleep(PING_INTERVAL) {
            break;
        }
    }

    log::info!("sonda de red detenida");
}

/// Promedio de las diferencias absolutas entre pings consecutivos.
fn jitter_of(window: &VecDeque<Option<f32>>) -> Option<f32> {
    let values: Vec<f32> = window.iter().flatten().copied().collect();

    if values.len() < 2 {
        return None;
    }

    let sum: f32 = values.windows(2).map(|pair| (pair[1] - pair[0]).abs()).sum();

    Some(sum / (values.len() - 1) as f32)
}

fn loss_of(window: &VecDeque<Option<f32>>) -> Option<f32> {
    if window.is_empty() {
        return None;
    }

    let lost = window.iter().filter(|sample| sample.is_none()).count();

    Some(lost as f32 / window.len() as f32 * 100.0)
}

fn log_failure<T>(what: &str, result: anyhow::Result<T>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            log::debug!("sampler de sistema: falló la lectura de {what}: {error}");
            None
        }
    }
}
