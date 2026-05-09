//! Setup inicial de Wayland: bind de globals y creación del layer surface.
//!
//! Devuelve los componentes que `AppState` necesita absorber. La razón de
//! por qué AppState los absorbe (en vez de tener una struct WaylandState
//! separada): `calloop_wayland_source::WaylandSource::insert` exige que el
//! state del calloop loop sea el mismo D que dispatchea el EventQueue.
//! Como necesitamos que el loop tenga acceso a render_ctx, bar, etc.,
//! todo eso vive junto en AppState.

use anyhow::{Context, Result};
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    shell::{
        wlr_layer::{LayerShell, LayerSurface},
        WaylandSurface,
    },
};
use wayland_client::{globals::registry_queue_init, Connection, EventQueue, QueueHandle};

use crate::app::AppState;

use super::layer_surface::LayerConfig;

/// Componentes inicializados que AppState absorbe.
pub struct WaylandInit {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,
    pub layer: LayerSurface,
}

pub fn init(
    conn: &Connection,
    config: LayerConfig,
) -> Result<(WaylandInit, EventQueue<AppState>)> {
    let (globals, event_queue) =
        registry_queue_init::<AppState>(conn).context("registry_queue_init failed")?;
    let qh: QueueHandle<AppState> = event_queue.handle();

    let registry_state = RegistryState::new(&globals);
    let output_state = OutputState::new(&globals, &qh);
    let compositor_state =
        CompositorState::bind(&globals, &qh).context("wl_compositor no disponible")?;
    let layer_shell = LayerShell::bind(&globals, &qh)
        .context("zwlr_layer_shell_v1 no disponible — el compositor no soporta layer-shell")?;

    let surface = compositor_state.create_surface(&qh);
    let layer = layer_shell.create_layer_surface(
        &qh,
        surface,
        config.layer.into(),
        Some("hyprbar"),
        None,
    );

    config.apply_to(&layer);
    layer.commit();

    Ok((
        WaylandInit {
            registry_state,
            output_state,
            compositor_state,
            layer_shell,
            layer,
        },
        event_queue,
    ))
}