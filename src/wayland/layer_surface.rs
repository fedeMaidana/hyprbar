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
    pub fn top_bar(height: u32) -> Self {
        Self {
            layer: LayerPosition::Top,
            anchor: Anchor::TOP_BAR,
            exclusive_zone: height as i32,
            initial_width: 0, // 0 = el compositor decide (full width gracias a anchor L+R)
            initial_height: height,
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
