// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, Rect as KurboRect, Stroke};
use vello::peniko::Fill;

use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::component::RenderCtx;
use super::pill::draw_top_highlight;
use super::shape::{squircle, squircle_inset};

// ─── < Constants > ────────────────────────────────────────────────────

/// Profundidad del brillo superior en los panels (px).
const PANEL_HIGHLIGHT_DEPTH: f32 = 30.0;

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
    /// Each widget declares its own id (e.g. `DropdownId::new("clock")`),
    /// so adding a component never touches this file.
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }
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

        let centered_x = anchor.x + (anchor.width - self.width) / 2.0;
        let x = centered_x.max(min_x).min(max_x);
        let y = anchor.y + anchor.height + theme.tokens.dropdown_margin_top;

        Rect::new(x, y, self.width, self.height)
    }

    pub fn draw_background(&self, scene: &mut Scene, bounds: Rect, theme: &Theme) {
        let tokens = theme.tokens;
        let radius = tokens.dropdown_radius as f64;

        // Sombra difusa debajo del panel: elevación real, sin hacks.
        let shadow_rect = KurboRect::new(
            bounds.x as f64,
            (bounds.y + tokens.dropdown_shadow_offset_y) as f64,
            (bounds.x + bounds.width) as f64,
            (bounds.y + bounds.height + tokens.dropdown_shadow_offset_y) as f64,
        );

        scene.draw_blurred_rounded_rect(
            Affine::IDENTITY,
            shadow_rect,
            theme.palette.panel_shadow,
            radius,
            tokens.dropdown_shadow_std_dev as f64,
        );

        let body = squircle(bounds, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, theme.palette.panel_bg, None, &body);

        // Hairline border keeps the panel edge readable over any wallpaper.
        let border_width = tokens.dropdown_border_width as f64;
        let border = squircle_inset(bounds, radius, (border_width / 2.0) as f32);

        scene.stroke(&Stroke::new(border_width), Affine::IDENTITY, theme.palette.panel_border, None, &border);

        draw_top_highlight(scene, bounds, PANEL_HIGHLIGHT_DEPTH, theme, &border);
    }

    pub fn draw_divider(scene: &mut Scene, x: f32, y: f32, width: f32, theme: &Theme) {
        let line = KurboRect::new(x as f64, (y - 0.5) as f64, (x + width) as f64, (y + 0.5) as f64);

        scene.fill(Fill::NonZero, Affine::IDENTITY, theme.palette.panel_divider, None, &line);
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
                TextStyle::new(title_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
            );

            ctx.text.draw_centered_v(
                scene,
                subtitle,
                text_x,
                y + item_height * 0.45,
                item_height * 0.45,
                TextStyle::new(subtitle_size, ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
            );
        }
        None => {
            ctx.text.draw_centered_v(
                scene,
                item.title,
                text_x,
                y,
                item_height,
                TextStyle::new(title_size, ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
            );
        }
    }
}
