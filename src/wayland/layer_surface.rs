// ─── < Imports > ────────────────────────────────────────────────────

use smithay_client_toolkit::shell::wlr_layer::{Anchor as SctkAnchor, KeyboardInteractivity, Layer as SctkLayer, LayerSurface};

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Anchor {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct LayerConfig {
    pub layer: LayerPosition,
    pub anchor: Anchor,
    pub exclusive_zone: i32,
    pub initial_width: u32,
    pub initial_height: u32,
    pub margin_top: i32,
    pub margin_bottom: i32,
    pub margin_left: i32,
    pub margin_right: i32,
}

// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum LayerPosition {
    Background,
    Bottom,
    Top,
    Overlay,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl From<LayerPosition> for SctkLayer {
    fn from(p: LayerPosition) -> Self {
        match p {
            LayerPosition::Background => SctkLayer::Background,
            LayerPosition::Bottom => SctkLayer::Bottom,
            LayerPosition::Top => SctkLayer::Top,
            LayerPosition::Overlay => SctkLayer::Overlay,
        }
    }
}

impl Anchor {
    pub const TOP_BAR: Self = Self {
        top: true,
        bottom: false,
        left: true,
        right: true,
    };

    fn to_sctk(self) -> SctkAnchor {
        let mut a = SctkAnchor::empty();

        if self.top {
            a |= SctkAnchor::TOP;
        }

        if self.bottom {
            a |= SctkAnchor::BOTTOM;
        }

        if self.left {
            a |= SctkAnchor::LEFT;
        }

        if self.right {
            a |= SctkAnchor::RIGHT;
        }

        a
    }
}

impl LayerConfig {
    pub fn top_bar(surface_height: u32, exclusive_zone: i32) -> Self {
        Self {
            layer: LayerPosition::Top,
            anchor: Anchor::TOP_BAR,
            exclusive_zone,
            initial_width: 0,
            initial_height: surface_height,
            margin_top: 0,
            margin_bottom: 0,
            margin_left: 0,
            margin_right: 0,
        }
    }

    /// Overlay a pantalla completa (modales de confirmación): anclado a
    /// los cuatro bordes y sin reservar lugar.
    pub fn fullscreen_overlay() -> Self {
        Self {
            layer: LayerPosition::Overlay,
            anchor: Anchor {
                top: true,
                bottom: true,
                left: true,
                right: true,
            },
            exclusive_zone: -1,
            initial_width: 0,
            initial_height: 0,
            margin_top: 0,
            margin_bottom: 0,
            margin_left: 0,
            margin_right: 0,
        }
    }

    pub fn apply_to(&self, layer: &LayerSurface) {
        layer.set_anchor(self.anchor.to_sctk());
        layer.set_size(self.initial_width, self.initial_height);
        layer.set_exclusive_zone(self.exclusive_zone);
        layer.set_margin(self.margin_top, self.margin_right, self.margin_bottom, self.margin_left);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    }
}
