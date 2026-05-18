// ─── < Imports > ────────────────────────────────────────────────────

use smithay_client_toolkit::compositor::{CompositorState, Region};
use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::seat::SeatState;
use smithay_client_toolkit::shell::wlr_layer::LayerShell;
use smithay_client_toolkit::shm::Shm;
use wayland_client::{QueueHandle, protocol::wl_surface};

use crate::app::AppState;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InputRegionRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

pub(crate) struct WaylandState {
    pub(crate) registry_state: RegistryState,
    pub(crate) output_state: OutputState,
    pub(crate) seat_state: SeatState,
    pub(crate) shm_state: Shm,

    compositor_state: CompositorState,
    _layer_shell: LayerShell,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl InputRegionRect {
    pub(crate) fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    fn is_empty(self) -> bool {
        self.width <= 0 || self.height <= 0
    }
}

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

    pub(crate) fn apply_input_region(&self, surface: &wl_surface::WlSurface, rects: &[InputRegionRect]) {
        let Ok(region) = Region::new(&self.compositor_state) else {
            log::warn!("failed to create Wayland input region");
            return;
        };

        for rect in rects.iter().copied().filter(|rect| !rect.is_empty()) {
            region.add(rect.x, rect.y, rect.width, rect.height);
        }

        surface.set_input_region(Some(region.wl_region()));
        surface.commit();
    }
}
