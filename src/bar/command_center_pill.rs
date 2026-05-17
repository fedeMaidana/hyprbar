// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;

use crate::components::{Component, Pill, RenderCtx};
use crate::render::{Rect, TextStyle};

// ─── < Constants > ────────────────────────────────────────────────────

const TOGGLE_GLYPH: &str = "\u{f1542}";

// ─── < Structs > ────────────────────────────────────────────────────

pub struct CommandCenterPill;

// ─── < Implementations > ────────────────────────────────────────────────────

impl CommandCenterPill {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CommandCenterPill {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for CommandCenterPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let icon_size = ctx.theme.typography.size_base * 1.2;
        let (iw, _) = ctx.text.measure(TOGGLE_GLYPH, icon_size, &ctx.theme.typography.icon_font_family);
        let w = iw + ctx.theme.tokens.pill_padding_x * 2.0;
        (w, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        Pill::draw(scene, bounds, ctx.theme);

        let pad_x = ctx.theme.tokens.pill_padding_x;
        let icon_size = ctx.theme.typography.size_base * 1.2;

        ctx.text.draw_centered_v(
            scene,
            TOGGLE_GLYPH,
            bounds.x + pad_x,
            bounds.y,
            bounds.height,
            TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, ctx.theme.palette.text_primary),
        );
    }
}
