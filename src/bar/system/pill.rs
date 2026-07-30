// ─── < Imports > ────────────────────────────────────────────────────

use calloop::channel::Sender;
use vello::Scene;
use vello::peniko::Color;

use crate::app::WorkerHandle;
use crate::components::{Component, DropdownId, Interaction, Pill, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::panel::SystemPanel;
use super::state::SystemStore;
use super::worker::spawn_sampler;

// ─── < Constants > ────────────────────────────────────────────────────

const ARCH_GLYPH: &str = "\u{f08c7}";

// ─── < Structs > ────────────────────────────────────────────────────

pub struct ArchLogoPill {
    store: SystemStore,
    _sampler: Option<WorkerHandle>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl ArchLogoPill {
    pub fn new(redraw_signal: Sender<()>) -> Self {
        let store = SystemStore::new();
        let sampler = spawn_sampler(store.clone(), redraw_signal);

        Self { store, _sampler: sampler }
    }

    fn is_active(&self, ctx: &RenderCtx<'_>) -> bool {
        ctx.open_dropdown == Some(DropdownId::ARCH)
    }

    fn background_color(&self, ctx: &RenderCtx<'_>) -> Color {
        let hovered = ctx.hovered_interaction == Some(Interaction::Dropdown(DropdownId::ARCH));

        Pill::background_for(self.is_active(ctx), hovered, ctx.theme)
    }

    fn icon_color(&self, ctx: &RenderCtx<'_>) -> Color {
        if self.is_active(ctx) {
            ctx.theme.palette.slot_active_text
        } else {
            ctx.theme.palette.text_primary
        }
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
        self.store.set_panel_open(self.is_active(ctx));

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
        let data = self.store.snapshot();

        SystemPanel::draw(scene, surface, anchor, &data, ctx);
    }

    fn dropdown_bounds(&self, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Rect> {
        Some(SystemPanel::bounds(surface, anchor, theme))
    }

    fn hit_test_dropdown(&self, point: Point, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Interaction> {
        SystemPanel::hit_test(point, surface, anchor, theme)
    }
}
