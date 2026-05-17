// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::peniko::Color;

use crate::components::{Component, Dropdown, DropdownId, DropdownItem, Interaction, Pill, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

// ─── < Constants > ────────────────────────────────────────────────────

const ARCH_GLYPH: &str = "\u{f08c7}";

const ARCH_DROPDOWN_ITEMS: [DropdownItem<'static>; 3] = [
    DropdownItem::new("Arch Linux", Some("System menu")),
    DropdownItem::new("Hyprbar", Some("Wayland status bar")),
    DropdownItem::new("Session", Some("Hyprland")),
];

// ─── < Structs > ────────────────────────────────────────────────────

pub struct ArchLogoPill;

// ─── < Implementations > ────────────────────────────────────────────────────

impl ArchLogoPill {
    pub fn new() -> Self {
        Self
    }

    fn is_active(&self, ctx: &RenderCtx<'_>) -> bool {
        ctx.open_dropdown == Some(DropdownId::ARCH)
    }

    fn background_color(&self, ctx: &RenderCtx<'_>) -> Color {
        if self.is_active(ctx) {
            ctx.theme.palette.slot_active_bg
        } else {
            ctx.theme.palette.pill_bg
        }
    }

    fn icon_color(&self, ctx: &RenderCtx<'_>) -> Color {
        if self.is_active(ctx) {
            ctx.theme.palette.slot_active_text
        } else {
            ctx.theme.palette.text_primary
        }
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
        Pill::draw_with_background(scene, bounds, ctx.theme, self.background_color(ctx));

        let pad_x = ctx.theme.tokens.pill_padding_x;
        let size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;

        ctx.text.draw_centered_v(
            scene,
            ARCH_GLYPH,
            bounds.x + pad_x,
            bounds.y,
            bounds.height,
            TextStyle::new(size, &ctx.theme.typography.icon_font_family, self.icon_color(ctx)),
        );
    }

    fn hit_test(&self, _point: Point, _bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        Some(Interaction::Dropdown(DropdownId::ARCH))
    }

    fn dropdown_id(&self) -> Option<DropdownId> {
        Some(DropdownId::ARCH)
    }

    fn render_dropdown(&mut self, scene: &mut Scene, surface: Rect, anchor: Rect, ctx: &mut RenderCtx<'_>) {
        let dropdown = Dropdown::new(ctx.theme.tokens.dropdown_width, &ARCH_DROPDOWN_ITEMS);

        dropdown.draw(scene, surface, anchor, ctx);
    }
}
