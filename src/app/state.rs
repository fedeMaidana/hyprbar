use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    seat::{
        SeatState,
        pointer::{CursorIcon, ThemedPointer},
    },
    shell::wlr_layer::{LayerShell, LayerSurface},
    shm::Shm,
};
use wayland_client::{QueueHandle, protocol::wl_surface};

use crate::bar::Bar;
use crate::components::Point;
use crate::render::{RenderContext, TextEngine};
use crate::theme::Theme;
use crate::wayland::init::WaylandInit;

pub struct AppState {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub seat_state: SeatState,
    pub shm_state: Shm,

    compositor_state: CompositorState,
    _layer_shell: LayerShell,

    pub layer: LayerSurface,
    pub configured: bool,
    pub width: u32,
    pub height: u32,
    pub pending_resize: bool,
    pub should_close: bool,

    pub render_ctx: RenderContext,
    pub text_engine: TextEngine,
    pub theme: Theme,
    pub bar: Bar,
    pub needs_redraw: bool,

    pub(crate) themed_pointer: Option<ThemedPointer>,
    pub(crate) pointer_position: Option<Point>,
    pub(crate) cursor_icon: CursorIcon,
}

impl AppState {
    pub fn new(wl_init: WaylandInit, theme: Theme, bar: Bar) -> Self {
        Self {
            registry_state: wl_init.registry_state,
            output_state: wl_init.output_state,
            seat_state: wl_init.seat_state,
            shm_state: wl_init.shm_state,
            compositor_state: wl_init.compositor_state,
            _layer_shell: wl_init.layer_shell,
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
            themed_pointer: None,
            pointer_position: None,
            cursor_icon: CursorIcon::Default,
        }
    }

    pub fn layer_surface(&self) -> &LayerSurface {
        &self.layer
    }

    pub fn create_cursor_surface(&self, qh: &QueueHandle<Self>) -> wl_surface::WlSurface {
        self.compositor_state.create_surface(qh)
    }
}
