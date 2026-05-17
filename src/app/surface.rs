// ─── < Imports > ────────────────────────────────────────────────────

use smithay_client_toolkit::shell::wlr_layer::LayerSurface;

// ─── < Structs > ────────────────────────────────────────────────────

pub(crate) struct SurfaceState {
    pub(crate) layer: LayerSurface,
    pub(crate) configured: bool,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pending_resize: bool,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl SurfaceState {
    pub(crate) fn new(layer: LayerSurface) -> Self {
        Self {
            layer,
            configured: false,
            width: 0,
            height: 0,
            pending_resize: false,
        }
    }
}
