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

    fn draw_shape(scene: &mut Scene, bounds: Rect, theme: &Theme, background: Color, radius: f64) {
        let body_rect =
            KurboRect::new(bounds.x as f64, bounds.y as f64, (bounds.x + bounds.width) as f64, (bounds.y + bounds.height) as f64);

        let shadow_offset = theme.tokens.shadow_offset_y as f64;
        let shadow_blur = theme.tokens.pill_shadow_blur as f64;
        let shadow_spread = shadow_blur * 2.5;

        // Clipped at the pill's top edge: the shadow falls below, never above.
        let shadow_clip = KurboRect::new(
            body_rect.x0 - shadow_spread,
            body_rect.y0,
            body_rect.x1 + shadow_spread,
            body_rect.y1 + shadow_offset + shadow_spread,
        );

        scene.draw_blurred_rounded_rect_in(
            &shadow_clip,
            Affine::IDENTITY,
            KurboRect::new(body_rect.x0, body_rect.y0 + shadow_offset, body_rect.x1, body_rect.y1 + shadow_offset),
            theme.palette.shadow,
            radius,
            shadow_blur,
        );

        let body = RoundedRect::from_rect(body_rect, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        let border = RoundedRect::new(body_rect.x0 + 0.5, body_rect.y0 + 0.5, body_rect.x1 - 0.5, body_rect.y1 - 0.5, radius - 0.5);

        scene.stroke(&Stroke::new(1.0), Affine::IDENTITY, theme.palette.pill_border, None, &border);
    }
}
