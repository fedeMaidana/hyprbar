// ─── < Imports > ────────────────────────────────────────────────────

use chrono::{Local, Timelike};
use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Fill, ImageData};

use crate::components::{DropdownFrame, Interaction, Panel, Point, RenderCtx, evenly_spaced_rects};
use crate::render::{Rect, TextStyle};
use crate::theme::Theme;

use super::action::SessionAction;
use super::avatar::{AvatarCircle, draw_avatar_circle};
use super::session::greeting;

// ─── < Constants > ────────────────────────────────────────────────────

const HOST_TEXT_SCALE: f32 = 0.78;
const BUTTON_LABEL_SCALE: f32 = 0.85;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct ProfilePanel<'a> {
    pub avatar: Option<&'a ImageData>,
    pub user: &'a str,
    pub host: &'a str,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl ProfilePanel<'_> {
    pub fn height(theme: &Theme) -> f32 {
        let tokens = theme.tokens;

        tokens.profile_panel_padding_y * 2.0 + tokens.profile_avatar_size + tokens.profile_section_gap + tokens.profile_button_height
    }
}

impl Panel for ProfilePanel<'_> {
    fn frame(&self, theme: &Theme) -> DropdownFrame {
        DropdownFrame::new(theme.tokens.profile_panel_width, Self::height(theme))
    }

    fn draw_content(&self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        let theme = ctx.theme;
        let tokens = theme.tokens;

        let inner_x = bounds.x + tokens.profile_panel_padding_x;
        let y = bounds.y + tokens.profile_panel_padding_y;

        let title = format!("{}, {}", greeting(Local::now().hour()), self.user);

        draw_header(scene, inner_x, y, self.avatar, &title, self.host, ctx);

        let inner_width = bounds.width - tokens.profile_panel_padding_x * 2.0;
        let buttons_y = bounds.y + bounds.height - tokens.profile_panel_padding_y - tokens.profile_button_height;

        DropdownFrame::draw_divider(scene, inner_x, buttons_y - tokens.profile_section_gap / 2.0, inner_width, theme);

        draw_buttons(scene, bounds, ctx);
    }

    fn hit_test_content(&self, point: Point, bounds: Rect, theme: &Theme) -> Option<Interaction> {
        for (action, rect) in button_rects(bounds, theme) {
            if rect.contains_point(point.x, point.y) {
                return Some(action.interaction());
            }
        }

        None
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn draw_header(scene: &mut Scene, x: f32, y: f32, avatar: Option<&ImageData>, title: &str, host: &str, ctx: &mut RenderCtx<'_>) {
    let tokens = ctx.theme.tokens;
    let avatar_size = tokens.profile_avatar_size;
    let radius = avatar_size / 2.0;

    let circle = AvatarCircle::new(x + radius, y + radius, radius, tokens.profile_avatar_border_width);

    draw_avatar_circle(scene, avatar, &circle, ctx.theme.palette.panel_raised, ctx.theme.palette.text_secondary);

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

    let rects: [Rect; 2] = evenly_spaced_rects(inner_x, y, inner_width, tokens.profile_button_height, tokens.profile_button_gap);

    std::array::from_fn(|index| (SessionAction::ALL[index], rects[index]))
}

fn draw_buttons(scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
    let radius = ctx.theme.tokens.profile_button_radius as f64;
    let icon_size = ctx.theme.typography.size_base * ctx.theme.tokens.icon_scale;
    let label_size = ctx.theme.typography.size_base * BUTTON_LABEL_SCALE;

    for (action, rect) in button_rects(bounds, ctx.theme) {
        let is_hovered = ctx.hovered_interaction == Some(action.interaction());

        let background = if is_hovered {
            ctx.theme.palette.control_hover_bg
        } else {
            ctx.theme.palette.control_bg
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
