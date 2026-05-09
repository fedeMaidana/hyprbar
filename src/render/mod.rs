//! Pipeline de rendering: wgpu como backend, vello como 2D scene API,
//! parley para layout de texto.

pub mod context;
pub mod text;

pub use context::RenderContext;
pub use text::TextEngine;

/// Rectángulo en coordenadas de pantalla (px).
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}
