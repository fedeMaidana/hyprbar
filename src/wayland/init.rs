// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Context, Result};
use smithay_client_toolkit::compositor::CompositorState;
use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::seat::SeatState;
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::wlr_layer::{LayerShell, LayerSurface};
use smithay_client_toolkit::shm::Shm;
use wayland_client::{Connection, EventQueue, QueueHandle, globals::registry_queue_init};
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1;
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_v1::WpFractionalScaleV1;
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;
use wayland_protocols::wp::viewporter::client::wp_viewporter::WpViewporter;

use crate::app::AppState;

use super::layer_surface::LayerConfig;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct WaylandInit {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub seat_state: SeatState,
    pub shm_state: Shm,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,
    pub layer: LayerSurface,
    pub fractional: Option<WpFractionalScaleV1>,
    pub viewport: Option<WpViewport>,
    pub fractional_manager: Option<WpFractionalScaleManagerV1>,
    pub viewporter: Option<WpViewporter>,
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn init(conn: &Connection, config: LayerConfig) -> Result<(WaylandInit, EventQueue<AppState>)> {
    let (globals, event_queue) = registry_queue_init::<AppState>(conn).context("registry_queue_init failed")?;

    let qh: QueueHandle<AppState> = event_queue.handle();

    let registry_state = RegistryState::new(&globals);
    let output_state = OutputState::new(&globals, &qh);
    let seat_state = SeatState::new(&globals, &qh);
    let compositor_state = CompositorState::bind(&globals, &qh).context("wl_compositor no disponible")?;
    let shm_state = Shm::bind(&globals, &qh).context("wl_shm no disponible")?;
    let layer_shell =
        LayerShell::bind(&globals, &qh).context("zwlr_layer_shell_v1 no disponible — el compositor no soporta layer-shell")?;

    let surface = compositor_state.create_surface(&qh);
    let layer = layer_shell.create_layer_surface(&qh, surface, config.layer.into(), Some("hyprbar"), None);

    config.apply_to(&layer);
    layer.commit();

    // Fractional scale: the bar renders at the exact physical size and a viewport maps it
    // back to logical, instead of the blurry integer-scale-then-resample path.
    let viewporter: Option<WpViewporter> = globals.bind(&qh, 1..=1, ()).ok();
    let fractional_manager: Option<WpFractionalScaleManagerV1> = globals.bind(&qh, 1..=1, ()).ok();

    let (fractional, viewport) = match (&fractional_manager, &viewporter) {
        (Some(manager), Some(viewporter)) => (
            Some(manager.get_fractional_scale(layer.wl_surface(), &qh, ())),
            Some(viewporter.get_viewport(layer.wl_surface(), &qh, ())),
        ),
        _ => {
            log::info!("compositor sin fractional-scale; se usa escala entera");
            (None, None)
        }
    };

    Ok((
        WaylandInit {
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
        },
        event_queue,
    ))
}
