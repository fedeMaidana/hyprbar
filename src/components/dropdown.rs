// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;
use vello::kurbo::{Affine, Rect as KurboRect, RoundedRect, Stroke};
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
    pub const WEATHER: Self = Self("weather");
    pub const COMMAND: Self = Self("command");
    pub const PROFILE: Self = Self("profile");
    pub const NOTIFICATIONS: Self = Self("notifications");
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

        let body_rect =
            KurboRect::new(bounds.x as f64, bounds.y as f64, (bounds.x + bounds.width) as f64, (bounds.y + bounds.height) as f64);

        let body = RoundedRect::from_rect(body_rect, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, theme.palette.panel_bg, None, &body);

        // Hairline border keeps the panel edge readable over any wallpaper.
        let border_width = tokens.dropdown_border_width as f64;
        let inset = border_width / 2.0;
        let border =
            RoundedRect::new(body_rect.x0 + inset, body_rect.y0 + inset, body_rect.x1 - inset, body_rect.y1 - inset, radius - inset);

        scene.stroke(&Stroke::new(border_width), Affine::IDENTITY, theme.palette.panel_border, None, &border);
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
