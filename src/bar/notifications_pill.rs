//! Pill de notificaciones (estática por ahora).
//!
//! Muestra un ícono de campana y un badge cuando haya notificaciones.
//! El contador está hardcoded en 0; cuando implementemos el daemon de
//! notificaciones, reemplazar `count()` por una lectura de estado
//! compartido (mismo patrón que workspaces).

use vello::{
    kurbo::{Affine, Circle},
    peniko::Fill,
    Scene,
};

use crate::components::{Component, Pill, RenderCtx};
use crate::render::Rect;

/// Glyph de campana (Nerd Font, Material Design Icons): 󰂚
const BELL_GLYPH: &str = "\u{f009a}";

pub struct NotificationsPill;

impl NotificationsPill {
    pub fn new() -> Self {
        Self
    }

    /// Cantidad de notificaciones pendientes. Hardcoded en 0 hasta que
    /// implementemos el daemon. Cuando exista, esto va a leer de un
    /// `Arc<Mutex<usize>>` igual que workspaces.
    fn count(&self) -> usize {
        0
    }
}

impl Default for NotificationsPill {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for NotificationsPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let size = ctx.theme.typography.size_base * 1.2;
        let (iw, _) = ctx
            .text
            .measure(BELL_GLYPH, size, &ctx.theme.typography.icon_font_family);
        let w = iw + ctx.theme.tokens.pill_padding_x * 2.0;
        (w, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        Pill::draw(scene, bounds, ctx.theme);

        let pad_x = ctx.theme.tokens.pill_padding_x;
        let icon_size = ctx.theme.typography.size_base * 1.2;

        ctx.text.draw_centered_v(
            scene,
            BELL_GLYPH,
            bounds.x + pad_x,
            bounds.y,
            bounds.height,
            icon_size,
            &ctx.theme.typography.icon_font_family,
            ctx.theme.palette.text_primary,
        );

        // Badge (dot de color) en esquina superior-derecha del ícono.
        // Solo visible cuando hay notificaciones pendientes.
        let count = self.count();
        if count > 0 {
            let (iw, _) = ctx
                .text
                .measure(BELL_GLYPH, icon_size, &ctx.theme.typography.icon_font_family);

            let dot_radius = 3.5_f32;
            let dot_cx = bounds.x + pad_x + iw - dot_radius * 0.5;
            let dot_cy = bounds.y + (bounds.height / 2.0) - icon_size * 0.35;

            let dot = Circle::new((dot_cx as f64, dot_cy as f64), dot_radius as f64);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                ctx.theme.palette.accent,
                None,
                &dot,
            );
        }
    }
}