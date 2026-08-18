// ─── < Imports > ────────────────────────────────────────────────────

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow};
use calloop::{
    EventLoop,
    channel::{Channel, Event as ChannelEvent},
    timer::{TimeoutAction, Timer},
};
use calloop_wayland_source::WaylandSource;
use wayland_client::{Connection, EventQueue};

use super::state::AppState;

// ─── < Constants > ────────────────────────────────────────────────────

const CLOCK_TICK_SECONDS: u64 = 60;
const PALETTE_POLL_SECONDS: u64 = 1;
const SURFACE_RETRY_SECONDS: u64 = 5;

// ─── < Public Funtions > ────────────────────────────────────────────────────

pub(crate) fn insert_sources(
    event_loop: &mut EventLoop<AppState>,
    conn: Connection,
    event_queue: EventQueue<AppState>,
    redraw_channel: Channel<()>,
    shutdown_channel: Channel<()>,
) -> Result<()> {
    let loop_handle = event_loop.handle();

    insert_wayland_source(loop_handle.clone(), conn, event_queue)?;
    insert_redraw_source(loop_handle.clone(), redraw_channel)?;
    insert_shutdown_source(loop_handle.clone(), shutdown_channel)?;
    insert_clock_tick_source(loop_handle.clone())?;
    insert_palette_watch_source(loop_handle.clone())?;
    insert_surface_watch_source(loop_handle)?;

    Ok(())
}

/// Repintado periódico para el dropdown abierto que lo pida (el propio
/// componente declara su cadencia vía `Component::dropdown_tick`).
/// Se arma bajo demanda al abrir y se dropea solo cuando ningún dropdown
/// pide ticks, así una barra idle nunca se despierta de más.
pub(crate) fn insert_dropdown_tick_source(loop_handle: &calloop::LoopHandle<'static, AppState>, interval: Duration) -> Result<()> {
    let timer = Timer::from_duration(duration_until_next_tick(interval));

    loop_handle
        .insert_source(timer, |_event, _meta, app| {
            // Se re-consulta en cada disparo: si el usuario cambió a otro
            // dropdown con otra cadencia, el timer se adapta solo.
            let Some(interval) = app.bar.open_dropdown_tick(app.open_dropdown) else {
                app.dropdown_tick_armed = false;

                return TimeoutAction::Drop;
            };

            app.needs_redraw = true;

            TimeoutAction::ToDuration(duration_until_next_tick(interval))
        })
        .map_err(|e| anyhow!("no se pudo insertar el timer de tick del dropdown: {e:?}"))?;

    Ok(())
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn insert_wayland_source(
    loop_handle: calloop::LoopHandle<'_, AppState>,
    conn: Connection,
    event_queue: EventQueue<AppState>,
) -> Result<()> {
    WaylandSource::new(conn, event_queue)
        .insert(loop_handle)
        .map_err(|e| anyhow!("WaylandSource insert failed: {e:?}"))?;

    Ok(())
}

fn insert_redraw_source(loop_handle: calloop::LoopHandle<'_, AppState>, redraw_channel: Channel<()>) -> Result<()> {
    loop_handle
        .insert_source(redraw_channel, |event, _meta, app| {
            if let ChannelEvent::Msg(()) = event {
                app.needs_redraw = true;
            }
        })
        .map_err(|e| anyhow!("no se pudo insertar el canal de redraw: {e:?}"))?;

    Ok(())
}

/// El hilo de señales avisa por acá; `should_close` corta el loop
/// principal después del dispatch en curso.
fn insert_shutdown_source(loop_handle: calloop::LoopHandle<'_, AppState>, shutdown_channel: Channel<()>) -> Result<()> {
    loop_handle
        .insert_source(shutdown_channel, |event, _meta, app| {
            if let ChannelEvent::Msg(()) = event {
                app.should_close = true;
            }
        })
        .map_err(|e| anyhow!("no se pudo insertar el canal de cierre: {e:?}"))?;

    Ok(())
}

/// Watches the hyprcolor palette file and repaints the bar as soon as it
/// changes, so accent-tinted elements (active workspace, rings, meters)
/// follow every wallpaper or mode switch without waiting for other events.
/// This is the only place that re-reads the palette: the render path
/// never touches the filesystem.
fn insert_palette_watch_source(loop_handle: calloop::LoopHandle<'_, AppState>) -> Result<()> {
    let mut last_modified = crate::theme::hyprcolor::modified_time();

    let timer = Timer::from_duration(Duration::from_secs(PALETTE_POLL_SECONDS));

    loop_handle
        .insert_source(timer, move |_event, _meta, app| {
            let modified = crate::theme::hyprcolor::modified_time();

            if modified != last_modified {
                last_modified = modified;
                app.theme.refresh_dynamic_colors();
                app.needs_redraw = true;
            }

            TimeoutAction::ToDuration(Duration::from_secs(PALETTE_POLL_SECONDS))
        })
        .map_err(|e| anyhow!("no se pudo insertar el timer de la paleta: {e:?}"))?;

    Ok(())
}

/// Red de seguridad: la recreación normal de la superficie pasa en el
/// handler `closed()` (dirigida por evento); este timer solo reintenta
/// si aquella quedó a medio camino.
fn insert_surface_watch_source(loop_handle: calloop::LoopHandle<'_, AppState>) -> Result<()> {
    let timer = Timer::from_duration(Duration::from_secs(SURFACE_RETRY_SECONDS));

    loop_handle
        .insert_source(timer, |_event, _meta, app| {
            if app.surface.lost {
                app.recreate_surface();
            }

            TimeoutAction::ToDuration(Duration::from_secs(SURFACE_RETRY_SECONDS))
        })
        .map_err(|e| anyhow!("surface watch timer insert failed: {e:?}"))?;

    Ok(())
}

fn insert_clock_tick_source(loop_handle: calloop::LoopHandle<'_, AppState>) -> Result<()> {
    let timer = Timer::from_duration(duration_until_next_minute());

    loop_handle
        .insert_source(timer, |_event, _meta, app| {
            app.needs_redraw = true;

            TimeoutAction::ToDuration(duration_until_next_minute())
        })
        .map_err(|e| anyhow!("clock tick timer insert failed: {e:?}"))?;

    Ok(())
}

fn duration_until_next_minute() -> Duration {
    let Ok(elapsed) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return Duration::from_secs(CLOCK_TICK_SECONDS);
    };

    let elapsed_secs = elapsed.as_secs();
    let secs_until_next_minute = CLOCK_TICK_SECONDS - (elapsed_secs % CLOCK_TICK_SECONDS);

    Duration::from_secs(secs_until_next_minute.max(1))
}

/// Dispara alineado al próximo múltiplo del intervalo (para 1s, el
/// próximo segundo de reloj), así los ticks caen donde el ojo espera.
fn duration_until_next_tick(interval: Duration) -> Duration {
    let Ok(elapsed) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return interval;
    };

    let interval_nanos = interval.as_nanos().max(1);
    let until_next = interval_nanos - elapsed.as_nanos() % interval_nanos;

    Duration::from_nanos(until_next.max(1) as u64)
}
