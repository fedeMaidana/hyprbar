// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Rect as KurboRect, RoundedRect, Shape, Stroke};
use vello::peniko::{Brush, Color, Fill, Gradient};

use crate::render::Rect;
use crate::theme::Theme;

use super::shape::{squircle, squircle_inset};

// ─── < Constants > ────────────────────────────────────────────────────

/// Qué fracción de la altura recorre el brillo superior antes de morir.
const HIGHLIGHT_FADE_SHARE: f32 = 0.65;

/// Tolerancia al convertir formas circulares a path (px).
const PATH_TOLERANCE: f64 = 0.1;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct Pill;

// ─── < Implementations > ────────────────────────────────────────────────────

impl Pill {
    pub fn draw(scene: &mut Scene, bounds: Rect, theme: &Theme) {
        Self::draw_with_background(scene, bounds, theme, theme.palette.pill_bg);
    }

    pub fn draw_with_background(scene: &mut Scene, bounds: Rect, theme: &Theme, background: Color) {
        let radius = theme.tokens.pill_radius as f64;

        let body = squircle(bounds, radius);
        let border = squircle_inset(bounds, radius, 0.5);

        Self::draw_glass(scene, bounds, theme, background, &body, &border);
    }

    /// Same glass treatment, but fully round (radius = half the short side).
    pub fn draw_circular(scene: &mut Scene, bounds: Rect, theme: &Theme, background: Color) {
        let radius = (bounds.width.min(bounds.height) / 2.0) as f64;

        let body_rect =
            KurboRect::new(bounds.x as f64, bounds.y as f64, (bounds.x + bounds.width) as f64, (bounds.y + bounds.height) as f64);

        // Un círculo de verdad: acá el squircle desentonaría con el
        // recorte circular del avatar que va encima.
        let body = RoundedRect::from_rect(body_rect, radius).to_path(PATH_TOLERANCE);
        let border = RoundedRect::new(body_rect.x0 + 0.5, body_rect.y0 + 0.5, body_rect.x1 - 0.5, body_rect.y1 - 0.5, radius - 0.5)
            .to_path(PATH_TOLERANCE);

        Self::draw_glass(scene, bounds, theme, background, &body, &border);
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

    /// Cuerpo + hairline + brillo superior: el tratamiento de vidrio
    /// completo que comparten todas las formas de pill.
    fn draw_glass(scene: &mut Scene, bounds: Rect, theme: &Theme, background: Color, body: &BezPath, border: &BezPath) {
        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, body);

        scene.stroke(&Stroke::new(1.0), Affine::IDENTITY, theme.palette.pill_border, None, border);

        draw_top_highlight(scene, bounds, bounds.height * HIGHLIGHT_FADE_SHARE, theme, border);
    }
}

// ─── < Public Functions > ────────────────────────────────────────────────────

/// Línea de luz de 1px que cae desde el borde superior y se desvanece:
/// el detalle que hace que el vidrio se vea iluminado y no plano.
pub(crate) fn draw_top_highlight(scene: &mut Scene, bounds: Rect, fade_depth: f32, theme: &Theme, border: &BezPath) {
    let highlight = theme.palette.glass_highlight;

    let gradient = Gradient::new_linear((bounds.x as f64, bounds.y as f64), (bounds.x as f64, (bounds.y + fade_depth) as f64))
        .with_stops([(0.0, highlight), (1.0, highlight.with_alpha(0.0))]);

    scene.stroke(&Stroke::new(1.0), Affine::IDENTITY, &Brush::Gradient(gradient), None, border);
}
