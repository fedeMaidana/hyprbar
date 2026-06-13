// ─── < Imports > ────────────────────────────────────────────────────

use chrono::{Local, Timelike};
use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Fill, ImageData};

use crate::components::{DropdownFrame, Interaction, Point, RenderCtx};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::SessionAction;
use super::avatar::{AvatarCircle, draw_avatar_circle};
use super::session::greeting;

// ─── < Constants > ────────────────────────────────────────────────────

const HOST_TEXT_SCALE: f32 = 0.78;
const BUTTON_LABEL_SCALE: f32 = 0.85;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct ProfilePanel;

// ─── < Implementations > ────────────────────────────────────────────────────

impl ProfilePanel {
    pub fn height(theme: &Theme) -> f32 {
        let tokens = theme.tokens;

        tokens.profile_panel_padding_y * 2.0 + tokens.profile_avatar_size + tokens.profile_section_gap + tokens.profile_button_height
    }

    pub fn bounds(surface: Rect, anchor: Rect, theme: &Theme) -> Rect {
        Self::frame(theme).bounds(surface, anchor, theme)
    }

    pub fn hit_test(point: Point, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Interaction> {
        let bounds = Self::bounds(surface, anchor, theme);

        for (action, rect) in button_rects(bounds, theme) {
            if rect.contains_point(point.x, point.y) {
                return Some(Interaction::Session(action));
            }
        }

        None
    }

    pub fn draw(
        scene: &mut Scene,
        surface: Rect,
        anchor: Rect,
        avatar: Option<&ImageData>,
        user: &str,
        host: &str,
        ctx: &mut RenderCtx<'_>,
    ) {
        let theme = ctx.theme;
        let tokens = theme.tokens;

        let frame = Self::frame(theme);
        let bounds = frame.bounds(surface, anchor, theme);

        frame.draw_background(scene, bounds, theme);

        let inner_x = bounds.x + tokens.profile_panel_padding_x;
        let y = bounds.y + tokens.profile_panel_padding_y;

        let title = format!("{}, {}", greeting(Local::now().hour()), user);

        draw_header(scene, inner_x, y, avatar, &title, host, ctx);
        draw_buttons(scene, bounds, ctx);
    }

    fn frame(theme: &Theme) -> DropdownFrame {
        DropdownFrame::new(theme.tokens.profile_panel_width, Self::height(theme))
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn draw_header(scene: &mut Scene, x: f32, y: f32, avatar: Option<&ImageData>, title: &str, host: &str, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let avatar_size = tokens.profile_avatar_size;
    let radius = avatar_size / 2.0;

    let circle = AvatarCircle::new(x + radius, y + radius, radius, tokens.profile_avatar_border_width);

    draw_avatar_circle(scene, avatar, &circle, ctx.theme.palette.slot_inactive_bg, ctx.theme.palette.text_secondary);

    let text_x = x + avatar_size + tokens.profile_avatar_gap;

    let title_size = ctx.theme.typography.size_base;
    let host_size = title_size * HOST_TEXT_SCALE;

    ctx.text.draw_centered_v(
        scene,
        title,
        text_x,
        y,
        avatar_size * 0.5,
        TextStyle::new(title_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
    );

    ctx.text.draw_centered_v(
        scene,
        host,
        text_x,
        y + avatar_size * 0.5,
        avatar_size * 0.5,
        TextStyle::new(host_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_secondary),
    );
}

fn button_rects(bounds: Rect, theme: &Theme) -> [(SessionAction, Rect); 2] {
    let tokens = theme.tokens;

    let inner_x = bounds.x + tokens.profile_panel_padding_x;
    let inner_width = bounds.width - tokens.profile_panel_padding_x * 2.0;
    let y = bounds.y + bounds.height - tokens.profile_panel_padding_y - tokens.profile_button_height;

    let gap = tokens.profile_button_gap;
    let button_width = (inner_width - gap) / 2.0;

    let mut rects = [(SessionAction::Lock, Rect::new(0.0, 0.0, 0.0, 0.0)); 2];

    for (index, action) in SessionAction::ALL.into_iter().enumerate() {
        let x = inner_x + index as f32 * (button_width + gap);
        rects[index] = (action, Rect::new(x, y, button_width, tokens.profile_button_height));
    }

    rects
}

fn draw_buttons(scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
    let radius = ctx.theme.tokens.profile_button_radius as f64;
    let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
    let label_size = ctx.theme.typography.size_base * BUTTON_LABEL_SCALE;

    for (action, rect) in button_rects(bounds, ctx.theme) {
        let is_hovered = ctx.hovered_interaction == Some(Interaction::Session(action));

        let background = if is_hovered {
            ctx.theme.palette.slot_hover_bg
        } else {
            ctx.theme.palette.slot_inactive_bg
        };

        let body = RoundedRect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64, radius);

        scene.fill(Fill::NonZero, Affine::IDENTITY, background, None, &body);

        let glyph = action.glyph();
        let label = action.label();

        let (icon_width, _) = ctx.text.measure(glyph, icon_size, &ctx.theme.typography.icon_font_family);
        let (label_width, _) = ctx.text.measure(label, label_size, &ctx.theme.typography.font_family);

        let group_width = icon_width + ctx.theme.tokens.profile_button_inner_gap + label_width;
        let group_x = rect.x + (rect.width - group_width) / 2.0;

        ctx.text.draw_centered_v(
            scene,
            glyph,
            group_x,
            rect.y,
            rect.height,
            TextStyle::new(icon_size, &ctx.theme.typography.icon_font_family, ctx.theme.palette.text_primary),
        );

        ctx.text.draw_centered_v(
            scene,
            label,
            group_x + icon_width + ctx.theme.tokens.profile_button_inner_gap,
            rect.y,
            rect.height,
            TextStyle::new(label_size, &ctx.theme.typography.font_family, ctx.theme.palette.text_primary),
        );
    }
}
