use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    shell::wlr_layer::{LayerShell, LayerSurface},
};

use crate::bar::Bar;
use crate::render::{RenderContext, TextEngine};
use crate::theme::Theme;
use crate::wayland::init::WaylandInit;

pub struct AppState {
    pub registry_state: RegistryState,
    pub output_state: OutputState,

    _compositor_state: CompositorState,
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
}

impl AppState {
    pub fn new(wl_init: WaylandInit, theme: Theme, bar: Bar) -> Self {
        Self {
            registry_state: wl_init.registry_state,
            output_state: wl_init.output_state,
            _compositor_state: wl_init.compositor_state,
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
        }
    }

    pub fn layer_surface(&self) -> &LayerSurface {
        &self.layer
    }
}
