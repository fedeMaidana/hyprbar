// ─── < Imports > ────────────────────────────────────────────────────

use std::time::Duration;

use calloop::channel::Sender;
use chrono::{Datelike, Local, Timelike};

use crate::app::{ShutdownToken, WorkerHandle};
use crate::proc::spawn_detached;

use super::state::{Alarm, ClockStore};

// ─── < Constants > ────────────────────────────────────────────────────

/// El vigilante corre siempre: las alarmas suenan con el panel cerrado.
const WATCH_INTERVAL: Duration = Duration::from_millis(500);

/// Sonido estándar de freedesktop; si no existe, paplay falla y solo
/// queda la notificación.
const ALARM_SOUND: &str = "/usr/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga";

// ─── < Public Functions > ────────────────────────────────────────────────────

pub(crate) fn spawn_clock_watcher(store: ClockStore, redraw: Sender<()>) -> Option<WorkerHandle> {
    match WorkerHandle::spawn("clock-watcher", move |shutdown| watcher_loop(store, redraw, shutdown)) {
        Ok(worker) => Some(worker),
        Err(error) => {
            log::error!("no se pudo iniciar el vigilante del reloj: {error}");
            None
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn watcher_loop(store: ClockStore, redraw: Sender<()>, shutdown: ShutdownToken) {
    // (día del año, hora, minuto) del último disparo, para no repetir.
    let mut last_fired: Option<(u32, u32, u32)> = None;

    loop {
        if shutdown.sleep(WATCH_INTERVAL) {
            break;
        }

        check_alarms(&store, &mut last_fired);

        if check_timer(&store) {
            let _ = redraw.send(());
        }
    }

    log::info!("vigilante del reloj detenido");
}

fn check_alarms(store: &ClockStore, last_fired: &mut Option<(u32, u32, u32)>) {
    let now = Local::now();
    let key = (now.ordinal(), now.hour(), now.minute());

    if *last_fired == Some(key) {
        return;
    }

    let due: Vec<Alarm> = store
        .snapshot()
        .alarms
        .into_iter()
        .filter(|alarm| {
            alarm.enabled
                && u32::from(alarm.hour) == now.hour()
                && u32::from(alarm.minute) == now.minute()
                && alarm.repeat.matches(now.weekday())
        })
        .collect();

    if due.is_empty() {
        return;
    }

    *last_fired = Some(key);

    for alarm in due {
        log::info!("alarma {} disparada", alarm.time_text());
        notify(&format!("Alarma {}", alarm.time_text()), &alarm.subtitle());
    }
}

/// Marca el temporizador vencido (una sola vez) y avisa.
fn check_timer(store: &ClockStore) -> bool {
    let expired_minutes = store.update(|data| {
        let timer = &mut data.timer;

        if timer.running() && timer.remaining() == Duration::ZERO {
            timer.started_at = None;
            timer.remaining_at_pause = Duration::ZERO;
            timer.finished = true;

            Some(timer.duration.as_secs() / 60)
        } else {
            None
        }
    });

    let Some(minutes) = expired_minutes else {
        return false;
    };

    log::info!("temporizador de {minutes} min terminado");
    notify("Temporizador terminado", &format!("{minutes} min cumplidos"));

    true
}

/// Notificación crítica + sonido; si el sonido falla, queda el aviso.
fn notify(title: &str, body: &str) {
    if let Err(error) = spawn_detached("notify-send", &["-u", "critical", "-a", "hyprbar", title, body]) {
        log::warn!("no se pudo notificar «{title}»: {error}");
    }

    if let Err(error) = spawn_detached("paplay", &[ALARM_SOUND]) {
        log::debug!("sin sonido para «{title}»: {error}");
    }
}
