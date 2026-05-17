// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Context, Result};
use calloop::{EventLoop, channel::channel};
use smithay_client_toolkit::shell::WaylandSurface;
use wayland_client::{Connection, EventQueue};

use crate::bar::default_bar;
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
        let theme = Theme::dark();
        let surface_height = top_bar_surface_height(&theme);
        let exclusive_zone = top_bar_exclusive_zone(&theme);

        let conn = Connection::connect_to_env().context("no se pudo conectar a Wayland")?;

        let (wl_init, mut event_queue) = wayland::init(&conn, LayerConfig::top_bar(surface_height, exclusive_zone))?;

        let mut event_loop: EventLoop<AppState> = EventLoop::try_new().context("calloop EventLoop::try_new")?;

        let (redraw_sender, redraw_channel) = channel::<()>();
        let bar = default_bar(redraw_sender);

        let mut app = AppState::new(wl_init, theme, bar);

        wait_until_configured(&mut event_queue, &mut app)?;

        log::info!("configured: {}x{}", app.surface.width, app.surface.height);

        create_render_surface(&conn, &mut app)?;

        sources::insert_sources(&mut event_loop, conn, event_queue, redraw_channel)?;

        run_main_loop(event_loop, app)
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn wait_until_configured(event_queue: &mut EventQueue<AppState>, app: &mut AppState) -> Result<()> {
    event_queue.roundtrip(app).context("roundtrip inicial falló")?;

    while !app.surface.configured {
        event_queue.blocking_dispatch(app).context("dispatch en espera de configure")?;
    }

    Ok(())
}

fn create_render_surface(conn: &Connection, app: &mut AppState) -> Result<()> {
    let wl_surface = app.layer_surface().wl_surface().clone();
    let handle = SurfaceHandle::new(conn, &wl_surface);

    app.render_ctx.create_surface(handle, app.surface.width, app.surface.height)?;

    app.surface.pending_resize = false;

    Ok(())
}

fn run_main_loop(mut event_loop: EventLoop<AppState>, mut app: AppState) -> Result<()> {
    loop {
        event_loop.dispatch(None, &mut app).context("event_loop dispatch")?;

        if app.should_close {
            log::info!("close requested");
            break;
        }

        if app.needs_redraw {
            if let Err(e) = app.render() {
                log::error!("render error: {e:?}");
            }

            app.needs_redraw = false;
        }
    }

    Ok(())
}

fn top_bar_surface_height(theme: &Theme) -> u32 {
    let height = theme.tokens.bar_margin_top
        + theme.tokens.pill_height
        + theme.tokens.dropdown_margin_top
        + theme.tokens.dropdown_height
        + theme.tokens.dropdown_margin_bottom;

    height.ceil().max(theme.tokens.bar_height).max(1.0) as u32
}

fn top_bar_exclusive_zone(theme: &Theme) -> i32 {
    theme.tokens.bar_height.ceil().max(1.0) as i32
}
