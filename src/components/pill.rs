// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, Rect as KurboRect, RoundedRect, Stroke};
use vello::peniko::{Color, Fill};

use crate::render::Rect;
use crate::theme::Theme;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct Pill;

// ─── < Implementations > ────────────────────────────────────────────────────

impl Pill {
    pub fn draw(scene: &mut Scene, bounds: Rect, theme: &Theme) {
        Self::draw_with_background(scene, bounds, theme, theme.palette.pill_bg);
    }

    pub fn draw_with_background(scene: &mut Scene, bounds: Rect, theme: &Theme, background: Color) {
        Self::draw_shape(scene, bounds, theme, background, theme.tokens.pill_radius as f64);
    }

    /// Same glass treatment, but fully round (radius = half the short side).
    pub fn draw_circular(scene: &mut Scene, bounds: Rect, theme: &Theme, background: Color) {
        Self::draw_shape(scene, bounds, theme, background, (bounds.width.min(bounds.height) / 2.0) as f64);
    }

    /// Shared background choice so every pill hovers and activates alike.
    pub fn background_for(active: bool, hovered: bool, theme: &Theme) -> Color {
        if active {
            theme.palette.slot_active_bg
        } else if hovered {
            theme.palette.pill_hover_bg
        } else {
            theme.palette.pill_bg
        }
    }

    fn draw_shape(scene: &mut Scene, bounds: Rect, theme: &Theme, background: Color, radius: f64) {
        let body_rect =
            KurboRect::new(bounds.x as f64, bounds.y as f64, (bounds.x + bounds.width) as f64, (bounds.y + bounds.height) as f64);

        let body = RoundedRect::from_rect(body_rect, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        let border = RoundedRect::new(body_rect.x0 + 0.5, body_rect.y0 + 0.5, body_rect.x1 - 0.5, body_rect.y1 - 0.5, radius - 0.5);

        scene.stroke(&Stroke::new(1.0), Affine::IDENTITY, theme.palette.pill_border, None, &border);
    }
}
