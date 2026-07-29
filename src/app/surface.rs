// ─── < Imports > ────────────────────────────────────────────────────

use smithay_client_toolkit::shell::wlr_layer::LayerSurface;

// ─── < Structs > ────────────────────────────────────────────────────

pub(crate) struct SurfaceState {
    pub(crate) layer: LayerSurface,
    pub(crate) configured: bool,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) scale: i32,
    pub(crate) applied_buffer_scale: i32,
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
            scale: 1,
            applied_buffer_scale: 1,
            pending_resize: false,
        }
    }

    pub(crate) fn physical_width(&self) -> u32 {
        self.width.saturating_mul(self.scale.max(1) as u32)
    }

    pub(crate) fn physical_height(&self) -> u32 {
        self.height.saturating_mul(self.scale.max(1) as u32)
    }
}
