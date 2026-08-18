// ─── < Imports > ────────────────────────────────────────────────────

use calloop::LoopHandle;
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::wlr_layer::LayerSurface;
use wayland_client::{Connection, QueueHandle, protocol::wl_surface};

use crate::bar::Bar;
use crate::components::DropdownId;
use crate::render::{RenderContext, TextEngine};
use crate::theme::Theme;
use crate::wayland::LayerConfig;
use crate::wayland::init::WaylandInit;

use super::pointer::PointerState;
use super::sources;
use super::surface::SurfaceState;
use super::wayland_state::WaylandState;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct AppState {
    pub(crate) wayland: WaylandState,
    pub(crate) surface: SurfaceState,
    pub(crate) pointer: PointerState,

    pub(crate) conn: Connection,
    pub(crate) qh: QueueHandle<AppState>,
    pub(crate) layer_config: LayerConfig,
    pub(crate) loop_handle: LoopHandle<'static, AppState>,
    /// The wgpu surface targets a dead wl_surface and must be rebuilt
    /// before the next render (set after recreating the layer surface).
    pub(crate) render_surface_stale: bool,

    pub render_ctx: RenderContext,
    pub text_engine: TextEngine,
    pub theme: Theme,
    pub bar: Bar,

    pub(crate) open_dropdown: Option<DropdownId>,
    /// Whether the open dropdown's periodic repaint timer is alive.
    pub(crate) dropdown_tick_armed: bool,

    pub needs_redraw: bool,
    pub should_close: bool,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl AppState {
    pub fn new(
        wl_init: WaylandInit,
        conn: Connection,
        qh: QueueHandle<Self>,
        layer_config: LayerConfig,
        theme: Theme,
        bar: Bar,
        loop_handle: LoopHandle<'static, AppState>,
    ) -> Self {
        let WaylandInit {
            registry_state,
            output_state,
            seat_state,
            shm_state,
            compositor_state,
            layer_shell,
            layer,
            fractional,
            viewport,
            fractional_manager,
            viewporter,
        } = wl_init;

        Self {
            wayland: WaylandState {
                registry_state,
                output_state,
                seat_state,
                shm_state,
                compositor_state,
                layer_shell,
                viewporter,
                fractional_manager,
            },
            surface: SurfaceState::new(layer, fractional, viewport),
            pointer: PointerState::new(),

            conn,
            qh,
            layer_config,
            loop_handle,
            render_surface_stale: false,

            render_ctx: RenderContext::new(),
            text_engine: TextEngine::new(),
            theme,
            bar,

            open_dropdown: None,
            dropdown_tick_armed: false,

            needs_redraw: true,
            should_close: false,
        }
    }

    /// The compositor closed the bar surface (output gone, e.g. the TV
    /// was turned off). Builds a fresh one and marks the GPU surface
    /// stale so the next render recreates it.
    pub(crate) fn recreate_surface(&mut self) {
        log::info!("recreando superficie de la barra");

        // The wgpu surface must die before the wl_surface it targets.
        self.render_ctx.drop_surface();

        if let Some(viewport) = self.surface.viewport.take() {
            viewport.destroy();
        }

        if let Some(fractional) = self.surface.fractional.take() {
            fractional.destroy();
        }

        let layer = self.wayland.create_bar_layer(&self.qh, &self.layer_config);
        let (fractional, viewport) = self.wayland.fractional_objects(layer.wl_surface(), &self.qh);

        self.surface = SurfaceState::new(layer, fractional, viewport);
        self.render_surface_stale = true;
        self.needs_redraw = true;
    }

    pub fn layer_surface(&self) -> &LayerSurface {
        &self.surface.layer
    }

    pub fn create_cursor_surface(&self, qh: &QueueHandle<Self>) -> wl_surface::WlSurface {
        self.wayland.create_cursor_surface(qh)
    }

    pub(crate) fn toggle_dropdown(&mut self, dropdown_id: DropdownId) {
        self.open_dropdown = if self.open_dropdown == Some(dropdown_id) {
            None
        } else {
            Some(dropdown_id)
        };

        self.arm_dropdown_tick_if_needed();

        self.needs_redraw = true;
    }

    /// Si el dropdown recién abierto pide ticks (lo declara su propio
    /// componente vía `dropdown_tick`), arma el timer; el timer se dropea
    /// solo cuando ningún dropdown abierto los pide.
    fn arm_dropdown_tick_if_needed(&mut self) {
        if self.dropdown_tick_armed {
            return;
        }

        let Some(interval) = self.bar.open_dropdown_tick(self.open_dropdown) else {
            return;
        };

        match sources::insert_dropdown_tick_source(&self.loop_handle, interval) {
            Ok(()) => self.dropdown_tick_armed = true,
            Err(error) => log::warn!("no se pudo armar el tick del dropdown: {error}"),
        }
    }

    pub(crate) fn close_dropdown(&mut self) {
        if self.open_dropdown.take().is_some() {
            self.needs_redraw = true;
        }
    }
}
