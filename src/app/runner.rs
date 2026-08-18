// ─── < Imports > ────────────────────────────────────────────────────

use std::thread;

use anyhow::{Context, Result};
use calloop::{
    EventLoop,
    channel::{Sender, channel},
};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use smithay_client_toolkit::shell::WaylandSurface;
use wayland_client::{Connection, EventQueue};

use crate::bar::{Bar, default_bar};

use crate::theme::Theme;

use crate::wayland::{self, LayerConfig};

use super::sources;
use super::state::AppState;
use super::surface_handle::SurfaceHandle;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct App;

// ─── < Implementations > ────────────────────────────────────────────────────

impl App {
    pub fn run() -> Result<()> {
        let theme = Theme::preferred();

        // The bar is built first so every component can report its
        // tallest dropdown before the surface is sized.
        let (redraw_sender, redraw_channel) = channel::<()>();
        let bar = default_bar(redraw_sender);

        let surface_height = top_bar_surface_height(&theme, &bar);
        let exclusive_zone = top_bar_exclusive_zone(&theme);

        let conn = Connection::connect_to_env().context("no se pudo conectar a Wayland")?;

        let layer_config = LayerConfig::top_bar(surface_height, exclusive_zone);
        let (wl_init, mut event_queue) = wayland::init(&conn, layer_config)?;

        let mut event_loop: EventLoop<'static, AppState> = EventLoop::try_new().context("calloop EventLoop::try_new")?;

        let qh = event_queue.handle();
        let mut app = AppState::new(wl_init, conn.clone(), qh, layer_config, theme, bar, event_loop.handle());

        wait_until_configured(&mut event_queue, &mut app)?;

        log::info!("superficie configurada: {}x{}", app.surface.width, app.surface.height);

        create_render_surface(&conn, &mut app)?;

        let (shutdown_sender, shutdown_channel) = channel::<()>();
        spawn_signal_listener(shutdown_sender);

        sources::insert_sources(&mut event_loop, conn, event_queue, redraw_channel, shutdown_channel)?;

        run_main_loop(event_loop, app)
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn wait_until_configured(event_queue: &mut EventQueue<AppState>, app: &mut AppState) -> Result<()> {
    event_queue.roundtrip(app).context("roundtrip inicial falló")?;

    while !app.surface.configured {
        // Output gone during startup (e.g. TV off): retry until it returns.
        if app.surface.lost {
            std::thread::sleep(std::time::Duration::from_millis(500));
            app.recreate_surface();
        }

        event_queue.blocking_dispatch(app).context("dispatch en espera de configure")?;
    }

    Ok(())
}

fn create_render_surface(conn: &Connection, app: &mut AppState) -> Result<()> {
    let wl_surface = app.layer_surface().wl_surface().clone();
    let handle = SurfaceHandle::new(conn, &wl_surface);

    app.render_ctx
        .create_surface(handle, app.surface.physical_width(), app.surface.physical_height())?;

    app.surface.pending_resize = false;
    app.render_surface_stale = false;

    Ok(())
}

/// Registra SIGINT/SIGTERM y despierta el loop por canal: el flag
/// `should_close` corta el loop y, al dropear `AppState`, cada
/// `WorkerHandle` pide el apagado y se joinea.
fn spawn_signal_listener(sender: Sender<()>) {
    let mut signals = match Signals::new([SIGINT, SIGTERM]) {
        Ok(signals) => signals,
        Err(error) => {
            log::warn!("no se pudieron registrar las señales de cierre: {error}");
            return;
        }
    };

    let spawned = thread::Builder::new().name("signal-listener".to_string()).spawn(move || {
        for signal in signals.forever() {
            log::info!("señal {signal} recibida; se pide el cierre");

            if sender.send(()).is_err() {
                break;
            }
        }
    });

    if let Err(error) = spawned {
        log::warn!("no se pudo iniciar el hilo de señales: {error}");
    }
}

fn run_main_loop(mut event_loop: EventLoop<AppState>, mut app: AppState) -> Result<()> {
    loop {
        event_loop.dispatch(None, &mut app).context("event_loop dispatch")?;

        if app.should_close {
            log::info!("cierre pedido; se apagan los workers");
            break;
        }

        if app.needs_redraw && app.surface.configured && !app.surface.lost {
            if let Err(e) = app.render() {
                log::error!("error de render: {e:?}");
            }

            app.needs_redraw = false;
        }
    }

    Ok(())
}

fn top_bar_surface_height(theme: &Theme, bar: &Bar) -> u32 {
    let dropdown_height = theme.tokens.dropdown_height.max(bar.max_dropdown_height(theme));

    let height = theme.tokens.bar_margin_top
        + theme.tokens.pill_height
        + theme.tokens.dropdown_margin_top
        + dropdown_height
        + theme.tokens.dropdown_margin_bottom;

    height.ceil().max(theme.tokens.bar_height).max(1.0) as u32
}

fn top_bar_exclusive_zone(theme: &Theme) -> i32 {
    theme.tokens.bar_height.ceil().max(1.0) as i32
}
