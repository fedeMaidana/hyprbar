// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;

use crate::components::{Component, Pill, RenderCtx};
use crate::render::{Rect, TextStyle};

// ─── < Constants > ────────────────────────────────────────────────────

const ARCH_GLYPH: &str = "\u{f08c7}";

// ─── < Structs > ────────────────────────────────────────────────────

pub struct ArchLogoPill;

// ─── < Implementations > ────────────────────────────────────────────────────

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
        let size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
        let (tw, _) = ctx.text.measure(ARCH_GLYPH, size, &ctx.theme.typography.icon_font_family);
        let w = tw + ctx.theme.tokens.pill_padding_x * 2.0;

        (w, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        Pill::draw(scene, bounds, ctx.theme);

        let pad_x = ctx.theme.tokens.pill_padding_x;
        let size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;

        ctx.text.draw_centered_v(
            scene,
            ARCH_GLYPH,
            bounds.x + pad_x,
            bounds.y,
            bounds.height,
            TextStyle::new(size, &ctx.theme.typography.icon_font_family, ctx.theme.palette.text_primary),
        );
    }
}
