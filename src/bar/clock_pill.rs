//! Pill que muestra la hora en formato HH:MM.

use chrono::Local;
use vello::Scene;

use crate::components::{Component, Pill, RenderCtx};
use crate::render::Rect;

pub struct ClockPill;

impl ClockPill {
    pub fn new() -> Self {
        Self
    }

    fn current_text(&self) -> String {
        Local::now().format("%H:%M").to_string()
    }
}

impl Default for ClockPill {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for ClockPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let text = self.current_text();
        let (tw, _) = ctx.text.measure(
            &text,
            ctx.theme.typography.size_base,
            &ctx.theme.typography.font_family,
        );
        let w = tw + ctx.theme.tokens.pill_padding_x * 2.0;
        (w, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        Pill::draw(scene, bounds, ctx.theme);

        let text = self.current_text();
        let pad_x = ctx.theme.tokens.pill_padding_x;
        let size = ctx.theme.typography.size_base;

        ctx.text.draw_centered_v(
            scene,
            &text,
            bounds.x + pad_x,
            bounds.y,
            bounds.height,
            size,
            &ctx.theme.typography.font_family,
            ctx.theme.palette.text_primary,
        );
    }
}