use std::time::Duration;

use anyhow::{Result, anyhow};
use calloop::{
    EventLoop,
    channel::{Channel, Event as ChannelEvent},
    timer::{TimeoutAction, Timer},
};
use calloop_wayland_source::WaylandSource;
use wayland_client::{Connection, EventQueue};

use super::state::AppState;

pub(crate) fn insert_sources(
    event_loop: &mut EventLoop<AppState>,
    conn: Connection,
    event_queue: EventQueue<AppState>,
    redraw_channel: Channel<()>,
) -> Result<()> {
    let loop_handle = event_loop.handle();

    insert_wayland_source(loop_handle.clone(), conn, event_queue)?;
    insert_redraw_source(loop_handle.clone(), redraw_channel)?;
    insert_timer_source(loop_handle)?;

    Ok(())
}

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

fn insert_timer_source(loop_handle: calloop::LoopHandle<'_, AppState>) -> Result<()> {
    let timer = Timer::from_duration(Duration::from_secs(1));

    loop_handle
        .insert_source(timer, |_event, _meta, app| {
            app.needs_redraw = true;
            TimeoutAction::ToDuration(Duration::from_secs(1))
        })
        .map_err(|e| anyhow!("timer insert failed: {e:?}"))?;

    Ok(())
}
