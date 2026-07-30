// ─── < Imports > ────────────────────────────────────────────────────

use chrono::Local;
use vello::Scene;
use vello::peniko::Color;

use crate::components::{Component, DropdownId, Interaction, Pill, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::panel::ClockPanel;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct ClockPill {
    frame_text: Option<String>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl ClockPill {
    pub fn new() -> Self {
        Self { frame_text: None }
    }

    fn current_text(&self) -> String {
        Local::now().format("%H:%M").to_string()
    }

    fn take_frame_text(&mut self) -> String {
        self.frame_text.take().unwrap_or_else(|| self.current_text())
    }

    fn is_active(&self, ctx: &RenderCtx<'_>) -> bool {
        ctx.open_dropdown == Some(DropdownId::CLOCK)
    }

    fn background_color(&self, ctx: &RenderCtx<'_>) -> Color {
        let hovered = ctx.hovered_interaction == Some(Interaction::Dropdown(DropdownId::CLOCK));

        Pill::background_for(self.is_active(ctx), hovered, ctx.theme)
    }

    fn text_color(&self, ctx: &RenderCtx<'_>) -> Color {
        if self.is_active(ctx) {
            ctx.theme.palette.slot_active_text
        } else {
            ctx.theme.palette.text_primary
        }
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

        let (tw, _) = ctx
            .text
            .measure(&text, ctx.theme.typography.size_base, &ctx.theme.typography.font_family);

        self.frame_text = Some(text);

        let w = tw + ctx.theme.tokens.pill_padding_x * 2.0;

        (w, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        Pill::draw_with_background(scene, bounds, ctx.theme, self.background_color(ctx));

        let text = self.take_frame_text();
        let pad_x = ctx.theme.tokens.pill_padding_x;
        let size = ctx.theme.typography.size_base;

        ctx.text.draw_centered_v(
            scene,
            &text,
            bounds.x + pad_x,
            bounds.y,
            bounds.height,
            TextStyle::new(size, &ctx.theme.typography.font_family, self.text_color(ctx)),
        );
    }

    fn hit_test(&self, _point: Point, _bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        Some(Interaction::Dropdown(DropdownId::CLOCK))
    }

    fn dropdown_id(&self) -> Option<DropdownId> {
        Some(DropdownId::CLOCK)
    }

    fn render_dropdown(&mut self, scene: &mut Scene, surface: Rect, anchor: Rect, ctx: &mut RenderCtx<'_>) {
        ClockPanel::draw(scene, surface, anchor, ctx);
    }

    fn dropdown_bounds(&self, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Rect> {
        Some(ClockPanel::bounds(surface, anchor, theme))
    }
}
