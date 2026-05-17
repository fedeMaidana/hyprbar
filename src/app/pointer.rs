// ─── < Imports > ────────────────────────────────────────────────────

use smithay_client_toolkit::seat::pointer::{CursorIcon, ThemedPointer};

use crate::components::Point;

// ─── < Structs > ────────────────────────────────────────────────────

pub(crate) struct PointerState {
    pub(crate) themed_pointer: Option<ThemedPointer>,
    pub(crate) position: Option<Point>,
    pub(crate) icon: CursorIcon,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl PointerState {
    pub(crate) fn new() -> Self {
        Self {
            themed_pointer: None,
            position: None,
            icon: CursorIcon::Default,
        }
    }
}
