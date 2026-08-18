// ─── < Imports > ────────────────────────────────────────────────────

use crate::render::TextEngine;

// ─── < Constants > ────────────────────────────────────────────────────

pub const ELLIPSIS: &str = "…";

// ─── < Public Functions > ────────────────────────────────────────────────────

/// Truncates `text` with an ellipsis so it fits in `max_width`.
///
/// Binary-searches the cut point over char boundaries, so it shapes
/// O(log n) candidate strings instead of one per character.
pub fn truncate_to_width(text_engine: &mut TextEngine, text: &str, size: f32, family: &str, max_width: f32) -> String {
    let (full_width, _) = text_engine.measure(text, size, family);

    if full_width <= max_width {
        return text.to_owned();
    }

    let boundaries: Vec<usize> = text.char_indices().map(|(index, _)| index).collect();

    // Largest char count whose prefix + ellipsis still fits.
    let mut fits = 0;
    let mut low = 0;
    let mut high = boundaries.len();

    while low < high {
        let mid = low + (high - low).div_ceil(2);
        let candidate = format!("{}{ELLIPSIS}", prefix(text, &boundaries, mid));

        if text_engine.measure(&candidate, size, family).0 <= max_width {
            fits = mid;
            low = mid;
        } else {
            high = mid - 1;
        }
    }

    format!("{}{ELLIPSIS}", prefix(text, &boundaries, fits))
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn prefix<'a>(text: &'a str, boundaries: &[usize], chars: usize) -> &'a str {
    match boundaries.get(chars) {
        Some(&end) => &text[..end],
        None => text,
    }
}
