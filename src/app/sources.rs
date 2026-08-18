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

use crate::bar::clock::CLOCK_DROPDOWN;

use super::state::AppState;

// ─── < Constants > ────────────────────────────────────────────────────

const CLOCK_TICK_SECONDS: u64 = 60;
const NANOS_PER_SECOND: u64 = 1_000_000_000;
const PALETTE_POLL_SECONDS: u64 = 1;
const SURFACE_RETRY_SECONDS: u64 = 1;

// ─── < Public Funtions > ────────────────────────────────────────────────────

pub(crate) fn insert_sources(
    event_loop: &mut EventLoop<AppState>,
    conn: Connection,
    event_queue: EventQueue<AppState>,
    redraw_channel: Channel<()>,
) -> Result<()> {
    let loop_handle = event_loop.handle();

    insert_wayland_source(loop_handle.clone(), conn, event_queue)?;
    insert_redraw_source(loop_handle.clone(), redraw_channel)?;
    insert_clock_tick_source(loop_handle.clone())?;
    insert_palette_watch_source(loop_handle.clone())?;
    insert_surface_watch_source(loop_handle)?;

    Ok(())
}

/// Per-second repaint for the open clock panel. Armed on demand when the
/// dropdown opens and drops itself when it closes, so an idle bar never
/// wakes up once a second for nothing.
pub(crate) fn insert_panel_seconds_tick_source(loop_handle: &calloop::LoopHandle<'static, AppState>) -> Result<()> {
    let timer = Timer::from_duration(duration_until_next_second());

    loop_handle
        .insert_source(timer, |_event, _meta, app| {
            if app.open_dropdown == Some(CLOCK_DROPDOWN) {
                app.needs_redraw = true;

                return TimeoutAction::ToDuration(duration_until_next_second());
            }

            app.seconds_timer_armed = false;

            TimeoutAction::Drop
        })
        .map_err(|e| anyhow!("panel seconds timer insert failed: {e:?}"))?;

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
        .map_err(|e| anyhow!("redraw channel insert failed: {e:?}"))?;

    Ok(())
}

/// Watches the hyprcolor palette file and repaints the bar as soon as it
/// changes, so accent-tinted elements (active workspace, rings, meters)
/// follow every wallpaper or mode switch without waiting for other events.
fn insert_palette_watch_source(loop_handle: calloop::LoopHandle<'_, AppState>) -> Result<()> {
    let mut last_modified = crate::theme::hyprcolor::modified_time();

    let timer = Timer::from_duration(Duration::from_secs(PALETTE_POLL_SECONDS));

    loop_handle
        .insert_source(timer, move |_event, _meta, app| {
            let modified = crate::theme::hyprcolor::modified_time();

            if modified != last_modified {
                last_modified = modified;
                app.needs_redraw = true;
            }

            TimeoutAction::ToDuration(Duration::from_secs(PALETTE_POLL_SECONDS))
        })
        .map_err(|e| anyhow!("palette watch timer insert failed: {e:?}"))?;

    Ok(())
}

/// Rebuilds the bar surface after the compositor closes it (output
/// unplugged or TV powered off), retrying until a configure arrives.
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

fn duration_until_next_second() -> Duration {
    let Ok(elapsed) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return Duration::from_secs(1);
    };

    let nanos_until_next_second = NANOS_PER_SECOND - elapsed.subsec_nanos() as u64;

    Duration::from_nanos(nanos_until_next_second.max(1))
}
