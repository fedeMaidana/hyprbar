// ─── < Modules > ────────────────────────────────────────────────────

pub mod context;
pub mod text;

// ─── < Imports > ────────────────────────────────────────────────────

pub use context::RenderContext;
pub use text::{TextEngine, TextStyle};

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}
