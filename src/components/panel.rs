// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;

use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::component::{Interaction, Point, RenderCtx};
use super::dropdown::DropdownFrame;

// ─── < Constants > ────────────────────────────────────────────────────

/// Vertical share of the header taken by the title line.
const HEADER_TITLE_SHARE: f32 = 0.55;

// ─── < Structs > ────────────────────────────────────────────────────

/// Standard panel header: title on top, subtitle underneath.
#[derive(Debug, Clone, Copy)]
pub struct PanelHeader<'a> {
    pub title: &'a str,
    pub subtitle: &'a str,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl PanelHeader<'_> {
    pub fn draw(&self, scene: &mut Scene, x: f32, y: f32, ctx: &mut RenderCtx<'_>) {
        let header_height = ctx.theme.tokens.dropdown_header_height;
        let title_box = header_height * HEADER_TITLE_SHARE;

        let title_size = ctx.theme.typography.size_base * ctx.theme.tokens.dropdown_title_scale;
        let subtitle_size = ctx.theme.typography.size_base * ctx.theme.tokens.dropdown_subtitle_scale;

        ctx.text.draw_centered_v(
            scene,
            self.title,
            x,
            y,
            title_box,
            TextStyle::new(title_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
        );

        ctx.text.draw_centered_v(
            scene,
            self.subtitle,
            x,
            y + title_box,
            header_height - title_box,
            TextStyle::new(subtitle_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
        );
    }
}

// ─── < Traits > ────────────────────────────────────────────────────

/// Common surface for every dropdown panel: geometry, background and
/// hit-testing share one shape, so pills only wire up the content.
pub trait Panel {
    /// Frame (width + height) for the panel's current state.
    fn frame(&self, theme: &Theme) -> DropdownFrame;

    /// Draws the panel content; the background is already painted.
    fn draw_content(&self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>);

    /// Hit-tests the panel content against its resolved bounds.
    fn hit_test_content(&self, _point: Point, _bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        None
    }

    fn bounds(&self, surface: Rect, anchor: Rect, theme: &Theme) -> Rect {
        self.frame(theme).bounds(surface, anchor, theme)
    }

    fn render(&self, scene: &mut Scene, surface: Rect, anchor: Rect, ctx: &mut RenderCtx<'_>) {
        let frame = self.frame(ctx.theme);
        let bounds = frame.bounds(surface, anchor, ctx.theme);

        frame.draw_background(scene, bounds, ctx.theme);
        self.draw_content(scene, bounds, ctx);
    }

    fn hit_test(&self, point: Point, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Interaction> {
        self.hit_test_content(point, self.bounds(surface, anchor, theme), theme)
    }
}
