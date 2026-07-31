// ─── < Imports > ────────────────────────────────────────────────────

use smithay_client_toolkit::compositor::{CompositorState, Region};
use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::seat::SeatState;
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::wlr_layer::{LayerShell, LayerSurface};
use smithay_client_toolkit::shm::Shm;
use wayland_client::{QueueHandle, protocol::wl_surface};
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1;
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_v1::WpFractionalScaleV1;
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;
use wayland_protocols::wp::viewporter::client::wp_viewporter::WpViewporter;

use crate::app::AppState;
use crate::wayland::LayerConfig;

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
    layer_shell: LayerShell,
    viewporter: Option<WpViewporter>,
    fractional_manager: Option<WpFractionalScaleManagerV1>,
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
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        registry_state: RegistryState,
        output_state: OutputState,
        seat_state: SeatState,
        shm_state: Shm,
        compositor_state: CompositorState,
        layer_shell: LayerShell,
        viewporter: Option<WpViewporter>,
        fractional_manager: Option<WpFractionalScaleManagerV1>,
    ) -> Self {
        Self {
            registry_state,
            output_state,
            seat_state,
            shm_state,
            compositor_state,
            layer_shell,
            viewporter,
            fractional_manager,
        }
    }

    /// Creates a fresh bar layer surface with the given config applied.
    pub(crate) fn create_bar_layer(&self, qh: &QueueHandle<AppState>, config: &LayerConfig) -> LayerSurface {
        let surface = self.compositor_state.create_surface(qh);

        let layer = self
            .layer_shell
            .create_layer_surface(qh, surface, config.layer.into(), Some("hyprbar"), None);

        config.apply_to(&layer);
        layer.commit();

        layer
    }

    /// Fractional-scale + viewport objects for a surface, when the compositor supports both.
    pub(crate) fn fractional_objects(
        &self,
        surface: &wl_surface::WlSurface,
        qh: &QueueHandle<AppState>,
    ) -> (Option<WpFractionalScaleV1>, Option<WpViewport>) {
        match (&self.fractional_manager, &self.viewporter) {
            (Some(manager), Some(viewporter)) => (
                Some(manager.get_fractional_scale(surface, qh, ())),
                Some(viewporter.get_viewport(surface, qh, ())),
            ),
            _ => (None, None),
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
