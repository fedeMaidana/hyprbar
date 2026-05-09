//! Pill con el logo de Arch Linux.

use vello::Scene;

use crate::components::{Component, Pill, RenderCtx};
use crate::render::Rect;

const ARCH_GLYPH: &str = "\u{f08c7}";

pub struct ArchLogoPill;

impl ArchLogoPill {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ArchLogoPill {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for ArchLogoPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let size = ctx.theme.typography.size_base * 1.2;
        let (tw, _) = ctx
            .text
            .measure(ARCH_GLYPH, size, &ctx.theme.typography.icon_font_family);
        let w = tw + ctx.theme.tokens.pill_padding_x * 2.0;
        (w, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        Pill::draw(scene, bounds, ctx.theme);

        let pad_x = ctx.theme.tokens.pill_padding_x;
        let size = ctx.theme.typography.size_base * 1.2;

        ctx.text.draw_centered_v(
            scene,
            ARCH_GLYPH,
            bounds.x + pad_x,
            bounds.y,
            bounds.height,
            size,
            &ctx.theme.typography.icon_font_family,
            ctx.theme.palette.text_primary,
        );
    }
}