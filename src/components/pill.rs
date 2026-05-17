// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::Fill;

use crate::render::Rect;
use crate::theme::Theme;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct Pill;

// ─── < Implementations > ────────────────────────────────────────────────────

impl Pill {
    pub fn draw(scene: &mut Scene, bounds: Rect, theme: &Theme) {
        let radius = theme.tokens.pill_radius as f64;

        let shadow_offset = theme.tokens.shadow_offset_y as f64;
        let shadow = RoundedRect::new(
            bounds.x as f64,
            bounds.y as f64 + shadow_offset,
            (bounds.x + bounds.width) as f64,
            (bounds.y + bounds.height) as f64 + shadow_offset,
            radius,
        );
        scene.fill(Fill::NonZero, Affine::IDENTITY, theme.palette.shadow, None, &shadow);

        let body =
            RoundedRect::new(bounds.x as f64, bounds.y as f64, (bounds.x + bounds.width) as f64, (bounds.y + bounds.height) as f64, radius);
        scene.fill(Fill::NonZero, Affine::IDENTITY, theme.palette.pill_bg, None, &body);
    }
}
