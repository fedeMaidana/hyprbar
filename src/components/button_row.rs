// ─── < Imports > ────────────────────────────────────────────────────

use crate::render::Rect;

// ─── < Public Functions > ────────────────────────────────────────────────────

/// Splits a row into `N` equally sized buttons separated by `gap`.
pub fn evenly_spaced_rects<const N: usize>(x: f32, y: f32, width: f32, height: f32, gap: f32) -> [Rect; N] {
    let button_width = (width - gap * (N as f32 - 1.0)) / N as f32;

    std::array::from_fn(|index| Rect::new(x + index as f32 * (button_width + gap), y, button_width, height))
}
