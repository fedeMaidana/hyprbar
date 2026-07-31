// ─── < Imports > ────────────────────────────────────────────────────

use smithay_client_toolkit::shell::wlr_layer::LayerSurface;
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_v1::WpFractionalScaleV1;
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;

// ─── < Structs > ────────────────────────────────────────────────────

pub(crate) struct SurfaceState {
    pub(crate) layer: LayerSurface,
    pub(crate) configured: bool,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) scale: i32,
    /// Fractional scale in 1/120ths (wp_fractional_scale_v1); when present it wins over `scale`.
    pub(crate) scale120: Option<u32>,
    pub(crate) viewport: Option<WpViewport>,
    pub(crate) fractional: Option<WpFractionalScaleV1>,
    pub(crate) applied_buffer_scale: i32,
    pub(crate) pending_resize: bool,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl SurfaceState {
    pub(crate) fn new(layer: LayerSurface, fractional: Option<WpFractionalScaleV1>, viewport: Option<WpViewport>) -> Self {
        Self {
            layer,
            configured: false,
            width: 0,
            height: 0,
            scale: 1,
            scale120: None,
            viewport,
            fractional,
            applied_buffer_scale: 1,
            pending_resize: false,
        }
    }

    /// Physical pixels per logical pixel: fractional when the compositor provides it, integer otherwise.
    pub(crate) fn effective_scale(&self) -> f64 {
        match self.scale120 {
            Some(scale) => f64::from(scale) / 120.0,
            None => f64::from(self.scale.max(1)),
        }
    }

    pub(crate) fn physical_width(&self) -> u32 {
        physical_size(self.width, self.effective_scale())
    }

    pub(crate) fn physical_height(&self) -> u32 {
        physical_size(self.height, self.effective_scale())
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn physical_size(logical: u32, scale: f64) -> u32 {
    ((f64::from(logical) * scale).round() as u32).max(1)
}
