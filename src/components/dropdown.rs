// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::Fill;

use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::component::RenderCtx;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropdownId(&'static str);

#[derive(Debug, Clone, Copy)]
pub struct DropdownItem<'a> {
    title: &'a str,
    subtitle: Option<&'a str>,
}

pub struct Dropdown<'a> {
    width: f32,
    items: &'a [DropdownItem<'a>],
}

#[derive(Debug, Clone, Copy)]
pub struct DropdownFrame {
    pub width: f32,
    pub height: f32,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl DropdownId {
    pub const ARCH: Self = Self("arch");
    pub const DATE: Self = Self("date");
    pub const CLOCK: Self = Self("clock");
}

impl<'a> DropdownItem<'a> {
    pub const fn new(title: &'a str, subtitle: Option<&'a str>) -> Self {
        Self { title, subtitle }
    }
}

impl DropdownFrame {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn bounds(&self, surface: Rect, anchor: Rect, theme: &Theme) -> Rect {
        let margin = theme.tokens.bar_margin_x;
        let min_x = surface.x + margin;
        let max_x = (surface.x + surface.width - margin - self.width).max(min_x);

        let x = anchor.x.max(min_x).min(max_x);
        let y = anchor.y + anchor.height + theme.tokens.dropdown_margin_top;

        Rect::new(x, y, self.width, self.height)
    }

    pub fn draw_background(&self, scene: &mut Scene, bounds: Rect, theme: &Theme) {
        let radius = theme.tokens.dropdown_radius as f64;
        let shadow_offset = theme.tokens.shadow_offset_y as f64;

        let shadow = RoundedRect::new(
            bounds.x as f64,
            bounds.y as f64 + shadow_offset,
            (bounds.x + bounds.width) as f64,
            (bounds.y + bounds.height) as f64 + shadow_offset,
            radius,
        );

        scene.fill(Fill::NonZero, Affine::IDENTITY, theme.palette.shadow, None, &shadow);

        let body =
            RoundedRect::new(bounds.x as f64, bounds.y as f64, (bounds.x + bounds.width) as f64, (bounds.y + bounds.height) as f64, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, theme.palette.pill_bg, None, &body);
    }
}

impl<'a> Dropdown<'a> {
    pub fn new(width: f32, items: &'a [DropdownItem<'a>]) -> Self {
        Self { width, items }
    }

    pub fn draw(&self, scene: &mut Scene, surface: Rect, anchor: Rect, ctx: &mut RenderCtx<'_>) -> Rect {
        let frame = DropdownFrame::new(self.width, self.height(ctx));
        let bounds = frame.bounds(surface, anchor, ctx.theme);

        frame.draw_background(scene, bounds, ctx.theme);

        let mut y = bounds.y + ctx.theme.tokens.dropdown_padding_y;

        for item in self.items {
            draw_item(scene, bounds.x, y, item, ctx);
            y += ctx.theme.tokens.dropdown_item_height + ctx.theme.tokens.dropdown_item_gap;
        }

        bounds
    }

    fn height(&self, ctx: &RenderCtx<'_>) -> f32 {
        let item_count = self.items.len() as f32;
        let gap_count = self.items.len().saturating_sub(1) as f32;

        ctx.theme.tokens.dropdown_padding_y * 2.0
            + item_count * ctx.theme.tokens.dropdown_item_height
            + gap_count * ctx.theme.tokens.dropdown_item_gap
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn draw_item(scene: &mut Scene, x: f32, y: f32, item: &DropdownItem<'_>, ctx: &mut RenderCtx<'_>) {
    let text_x = x + ctx.theme.tokens.dropdown_padding_x;
    let item_height = ctx.theme.tokens.dropdown_item_height;
    let title_size = ctx.theme.typography.size_base;
    let subtitle_size = ctx.theme.typography.size_base * 0.78;

    match item.subtitle {
        Some(subtitle) => {
            ctx.text.draw_centered_v(
                scene,
                item.title,
                text_x,
                y,
                item_height * 0.55,
                TextStyle::new(title_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
            );

            ctx.text.draw_centered_v(
                scene,
                subtitle,
                text_x,
                y + item_height * 0.45,
                item_height * 0.45,
                TextStyle::new(subtitle_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
            );
        }
        None => {
            ctx.text.draw_centered_v(
                scene,
                item.title,
                text_x,
                y,
                item_height,
                TextStyle::new(title_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
            );
        }
    }
}
