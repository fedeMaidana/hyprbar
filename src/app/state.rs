// ─── < Imports > ────────────────────────────────────────────────────

use smithay_client_toolkit::shell::wlr_layer::LayerSurface;
use wayland_client::{QueueHandle, protocol::wl_surface};

use crate::bar::Bar;
use crate::components::DropdownId;
use crate::render::{RenderContext, TextEngine};
use crate::theme::Theme;
use crate::wayland::init::WaylandInit;

use super::pointer::PointerState;
use super::surface::SurfaceState;
use super::wayland_state::WaylandState;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct AppState {
    pub(crate) wayland: WaylandState,
    pub(crate) surface: SurfaceState,
    pub(crate) pointer: PointerState,

    pub render_ctx: RenderContext,
    pub text_engine: TextEngine,
    pub theme: Theme,
    pub bar: Bar,

    pub(crate) open_dropdown: Option<DropdownId>,

    pub needs_redraw: bool,
    pub should_close: bool,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl AppState {
    pub fn new(wl_init: WaylandInit, theme: Theme, bar: Bar) -> Self {
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
        } = wl_init;

        Self {
            wayland: WaylandState::new(registry_state, output_state, seat_state, shm_state, compositor_state, layer_shell),
            surface: SurfaceState::new(layer, fractional, viewport),
            pointer: PointerState::new(),

            render_ctx: RenderContext::new(),
            text_engine: TextEngine::new(),
            theme,
            bar,

            open_dropdown: None,

            needs_redraw: true,
            should_close: false,
        }
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

        self.needs_redraw = true;
    }

    pub(crate) fn close_dropdown(&mut self) {
        if self.open_dropdown.take().is_some() {
            self.needs_redraw = true;
        }
    }
}
