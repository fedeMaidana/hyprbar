use anyhow::{Context, Result};
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::{
        WaylandSurface,
        wlr_layer::{LayerShell, LayerSurface},
    },
    shm::Shm,
};
use wayland_client::{Connection, EventQueue, QueueHandle, globals::registry_queue_init};

use crate::app::AppState;

use super::layer_surface::LayerConfig;

pub struct WaylandInit {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub seat_state: SeatState,
    pub shm_state: Shm,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,
    pub layer: LayerSurface,
}

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

    Ok((
        WaylandInit {
            registry_state,
            output_state,
            seat_state,
            shm_state,
            compositor_state,
            layer_shell,
            layer,
        },
        event_queue,
    ))
}
