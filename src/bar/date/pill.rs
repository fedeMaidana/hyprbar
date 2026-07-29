// ─── < Imports > ────────────────────────────────────────────────────

use chrono::Local;
use vello::Scene;
use vello::peniko::Color;

use crate::components::{Component, DropdownId, Interaction, Pill, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::CalendarAction;
use super::panel::DatePanel;

// ─── < Constants > ────────────────────────────────────────────────────

const MAX_MONTH_OFFSET: i32 = 1200;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct DatePill {
    frame_text: Option<String>,
    month_offset: i32,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl DatePill {
    pub fn new() -> Self {
        Self {
            frame_text: None,
            month_offset: 0,
        }
    }

    fn current_text(&self) -> String {
        Local::now().format("%d/%m").to_string()
    }

    fn take_frame_text(&mut self) -> String {
        self.frame_text.take().unwrap_or_else(|| self.current_text())
    }

    fn is_active(&self, ctx: &RenderCtx<'_>) -> bool {
        ctx.open_dropdown == Some(DropdownId::DATE)
    }

    fn background_color(&self, ctx: &RenderCtx<'_>) -> Color {
        if self.is_active(ctx) {
            ctx.theme.palette.slot_active_bg
        } else {
            ctx.theme.palette.pill_bg
        }
    }

    fn text_color(&self, ctx: &RenderCtx<'_>) -> Color {
        if self.is_active(ctx) {
            ctx.theme.palette.slot_active_text
        } else {
            ctx.theme.palette.text_primary
        }
    }
}

impl Default for DatePill {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for DatePill {
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
        if !self.is_active(ctx) {
            self.month_offset = 0;
        }

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
        Some(Interaction::Dropdown(DropdownId::DATE))
    }

    fn dropdown_id(&self) -> Option<DropdownId> {
        Some(DropdownId::DATE)
    }

    fn render_dropdown(&mut self, scene: &mut Scene, surface: Rect, anchor: Rect, ctx: &mut RenderCtx<'_>) {
        DatePanel::draw(scene, surface, anchor, self.month_offset, ctx);
    }

    fn dropdown_bounds(&self, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Rect> {
        Some(DatePanel::bounds(surface, anchor, self.month_offset, theme))
    }

    fn hit_test_dropdown(&self, point: Point, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Interaction> {
        DatePanel::hit_test(point, surface, anchor, self.month_offset, theme)
    }

    fn handle_interaction(&mut self, interaction: Interaction) -> bool {
        let Interaction::Calendar(action) = interaction else {
            return false;
        };

        match action {
            CalendarAction::PrevMonth => {
                self.month_offset = (self.month_offset - 1).max(-MAX_MONTH_OFFSET);
                true
            }
            CalendarAction::NextMonth => {
                self.month_offset = (self.month_offset + 1).min(MAX_MONTH_OFFSET);
                true
            }
            CalendarAction::Today => {
                if self.month_offset == 0 {
                    return false;
                }

                self.month_offset = 0;
                true
            }
        }
    }
}
