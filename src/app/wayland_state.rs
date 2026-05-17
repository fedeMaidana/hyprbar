// ─── < Imports > ────────────────────────────────────────────────────

use smithay_client_toolkit::compositor::CompositorState;
use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::seat::SeatState;
use smithay_client_toolkit::shell::wlr_layer::LayerShell;
use smithay_client_toolkit::shm::Shm;
use wayland_client::{QueueHandle, protocol::wl_surface};

use crate::app::AppState;

// ─── < Structs > ────────────────────────────────────────────────────

pub(crate) struct WaylandState {
    pub(crate) registry_state: RegistryState,
    pub(crate) output_state: OutputState,
    pub(crate) seat_state: SeatState,
    pub(crate) shm_state: Shm,

    compositor_state: CompositorState,
    _layer_shell: LayerShell,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl WaylandState {
    pub(crate) fn new(
        registry_state: RegistryState,
        output_state: OutputState,
        seat_state: SeatState,
        shm_state: Shm,
        compositor_state: CompositorState,
        layer_shell: LayerShell,
    ) -> Self {
        Self {
            registry_state,
            output_state,
            seat_state,
            shm_state,
            compositor_state,
            _layer_shell: layer_shell,
        }
    }

    pub(crate) fn create_cursor_surface(&self, qh: &QueueHandle<AppState>) -> wl_surface::WlSurface {
        self.compositor_state.create_surface(qh)
    }
}
