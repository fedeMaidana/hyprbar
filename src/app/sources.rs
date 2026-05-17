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
    insert_clock_tick_source(loop_handle)?;

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
