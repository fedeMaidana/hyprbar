//! Orchestrator de la app.
//!
//! `AppState` es el state combinado: campos de Wayland (registry, output,
//! layer surface) + campos de render (vello context, text engine, bar).
//! La razón por la que es un state único: `calloop_wayland_source` exige
//! que el state del calloop loop sea el mismo que dispatchea el EventQueue.

use anyhow::{anyhow, Context, Result};
use calloop::{
    channel::{channel, Event as ChannelEvent},
    timer::{TimeoutAction, Timer},
    EventLoop,
};
use calloop_wayland_source::WaylandSource;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
};
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    shell::{
        wlr_layer::{LayerShell, LayerSurface},
        WaylandSurface,
    },
};
use std::{ptr::NonNull, time::Duration};
use wayland_client::{Connection, Proxy};

use crate::bar::Bar;
use crate::components::RenderCtx;
use crate::render::{Rect, RenderContext, TextEngine};
use crate::theme::Theme;
use crate::wayland::{self, LayerConfig};

/// Wrapper para exponer el wl_surface como raw-window-handle.
pub struct SurfaceHandle {
    display_ptr: NonNull<std::ffi::c_void>,
    surface_ptr: NonNull<std::ffi::c_void>,
}

unsafe impl Send for SurfaceHandle {}
unsafe impl Sync for SurfaceHandle {}

impl SurfaceHandle {
    fn new(conn: &Connection, surface: &wayland_client::protocol::wl_surface::WlSurface) -> Self {
        let display_ptr = NonNull::new(conn.backend().display_ptr() as *mut _)
            .expect("display ptr no debería ser null");
        let surface_ptr = NonNull::new(surface.id().as_ptr() as *mut _)
            .expect("surface ptr no debería ser null");
        Self {
            display_ptr,
            surface_ptr,
        }
    }
}

impl HasDisplayHandle for SurfaceHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let h = WaylandDisplayHandle::new(self.display_ptr);
        Ok(unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Wayland(h)) })
    }
}

impl HasWindowHandle for SurfaceHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let h = WaylandWindowHandle::new(self.surface_ptr);
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Wayland(h)) })
    }
}

pub struct AppState {
    // === Wayland ===
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,
    pub layer: LayerSurface,
    pub configured: bool,
    pub width: u32,
    pub height: u32,
    pub pending_resize: bool,
    pub should_close: bool,

    // === Render / UI ===
    pub render_ctx: RenderContext,
    pub text_engine: TextEngine,
    pub theme: Theme,
    pub bar: Bar,
    pub needs_redraw: bool,
}

impl AppState {
    pub fn layer_surface(&self) -> &LayerSurface {
        &self.layer
    }

    fn render(&mut self) -> Result<()> {
        if self.pending_resize {
            self.render_ctx.resize(self.width, self.height);
            self.pending_resize = false;
        }

        let surface_rect = Rect::new(0.0, 0.0, self.width as f32, self.height as f32);

        let mut ctx = RenderCtx {
            theme: &self.theme,
            text: &mut self.text_engine,
        };
        self.bar
            .render(&mut self.render_ctx.scene, surface_rect, &self.theme, &mut ctx);

        self.render_ctx.render()?;
        Ok(())
    }
}

pub struct App;

impl App {
    pub fn run() -> Result<()> {
        let theme = Theme::dark();
        let height = theme.tokens.bar_height as u32;

        // === Wayland setup ===
        let conn = Connection::connect_to_env().context("no se pudo conectar a Wayland")?;
        let (wl_init, mut event_queue) = wayland::init(&conn, LayerConfig::top_bar(height))?;

        // === Calloop setup (necesitamos el handle/sender ya para construir Bar) ===
        let mut event_loop: EventLoop<AppState> =
            EventLoop::try_new().context("calloop EventLoop::try_new")?;
        let loop_handle = event_loop.handle();

        // Channel para que componentes con state externo (workspaces, futuro
        // notifications, etc) pidan redraw inmediato sin esperar al timer.
        let (redraw_sender, redraw_channel) = channel::<()>();

        // Construir el bar con el sender ya listo
        let bar = Bar::new(redraw_sender);

        // === AppState con todo armado ===
        let mut app = AppState {
            registry_state: wl_init.registry_state,
            output_state: wl_init.output_state,
            compositor_state: wl_init.compositor_state,
            layer_shell: wl_init.layer_shell,
            layer: wl_init.layer,
            configured: false,
            width: 0,
            height: 0,
            pending_resize: false,
            should_close: false,
            render_ctx: RenderContext::new(),
            text_engine: TextEngine::new(),
            theme,
            bar,
            needs_redraw: true,
        };

        // Esperar primer configure
        event_queue
            .roundtrip(&mut app)
            .context("roundtrip inicial falló")?;
        while !app.configured {
            event_queue
                .blocking_dispatch(&mut app)
                .context("dispatch en espera de configure")?;
        }
        log::info!("configured: {}x{}", app.width, app.height);

        // === Render setup ===
        let wl_surface = app.layer_surface().wl_surface().clone();
        let handle = SurfaceHandle::new(&conn, &wl_surface);
        app.render_ctx
            .create_surface(handle, app.width, app.height)?;
        app.pending_resize = false;

        // === Sources del calloop loop ===

        // Source: eventos del compositor de Wayland
        WaylandSource::new(conn, event_queue)
            .insert(loop_handle.clone())
            .map_err(|e| anyhow!("WaylandSource insert failed: {e:?}"))?;

        // Source: redraw channel. Cuando un componente externo (ej.
        // listener de Hyprland) manda señal, marcamos needs_redraw.
        loop_handle
            .insert_source(redraw_channel, |event, _meta, app| {
                if let ChannelEvent::Msg(()) = event {
                    app.needs_redraw = true;
                }
            })
            .map_err(|e| anyhow!("redraw channel insert failed: {e:?}"))?;

        // Source: timer cada 1s. Mantiene reloj/fecha actualizados.
        let timer = Timer::from_duration(Duration::from_secs(1));
        loop_handle
            .insert_source(timer, |_event, _meta, app| {
                app.needs_redraw = true;
                TimeoutAction::ToDuration(Duration::from_secs(1))
            })
            .map_err(|e| anyhow!("timer insert failed: {e:?}"))?;

        // === Main loop ===
        loop {
            event_loop
                .dispatch(None, &mut app)
                .context("event_loop dispatch")?;

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
}