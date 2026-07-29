// ─── < Imports > ────────────────────────────────────────────────────

use std::path::Path;

use vello::Scene;
use vello::kurbo::{Affine, Circle, Stroke};
use vello::peniko::ImageData;

use crate::components::{Component, DropdownId, Interaction, Pill, Point, RenderCtx};
use crate::render::Rect;
use crate::theme::Theme;

use super::avatar::{AvatarCircle, draw_avatar_circle, load_avatar};
use super::panel::ProfilePanel;
use super::session;

// ─── < Constants > ────────────────────────────────────────────────────

const ACTIVE_RING_WIDTH: f32 = 2.0;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct ProfilePill {
    avatar: Option<ImageData>,
    user: String,
    host: String,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl ProfilePill {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        let avatar = match load_avatar(path) {
            Ok(avatar) => {
                log::info!("profile avatar cargado: {}", path.display());
                Some(avatar)
            }
            Err(error) => {
                log::warn!("no pude cargar {}: {error}", path.display());
                None
            }
        };

        Self {
            avatar,
            user: session::user_name(),
            host: session::host_name(),
        }
    }

    fn is_active(&self, ctx: &RenderCtx<'_>) -> bool {
        ctx.open_dropdown == Some(DropdownId::PROFILE)
    }
}

impl Component for ProfilePill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let height = ctx.theme.tokens.pill_height;

        (height, height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        let tokens = ctx.theme.tokens;

        // Same glass treatment as every other pill; the avatar sits inset.
        Pill::draw_circular(scene, bounds, ctx.theme, ctx.theme.palette.pill_bg);

        let center_x = bounds.x + bounds.width / 2.0;
        let center_y = bounds.y + bounds.height / 2.0;
        let half = bounds.width.min(bounds.height) / 2.0;
        let outer_radius = half - tokens.avatar_outer_radius_offset;

        let circle = AvatarCircle::new(center_x, center_y, outer_radius, tokens.avatar_border_width);

        draw_avatar_circle(scene, self.avatar.as_ref(), &circle, ctx.theme.palette.panel_raised, ctx.theme.palette.text_secondary);

        // Active state: thin accent ring at the edge, same thickness always.
        if self.is_active(ctx) {
            let ring_radius = half - ACTIVE_RING_WIDTH / 2.0;
            let ring = Circle::new((center_x as f64, center_y as f64), ring_radius as f64);

            scene.stroke(&Stroke::new(ACTIVE_RING_WIDTH as f64), Affine::IDENTITY, ctx.theme.palette.accent, None, &ring);
        }
    }

    fn hit_test(&self, _point: Point, _bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        Some(Interaction::Dropdown(DropdownId::PROFILE))
    }

    fn dropdown_id(&self) -> Option<DropdownId> {
        Some(DropdownId::PROFILE)
    }

    fn render_dropdown(&mut self, scene: &mut Scene, surface: Rect, anchor: Rect, ctx: &mut RenderCtx<'_>) {
        ProfilePanel::draw(scene, surface, anchor, self.avatar.as_ref(), &self.user, &self.host, ctx);
    }

    fn dropdown_bounds(&self, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Rect> {
        Some(ProfilePanel::bounds(surface, anchor, theme))
    }

    fn hit_test_dropdown(&self, point: Point, surface: Rect, anchor: Rect, theme: &Theme) -> Option<Interaction> {
        ProfilePanel::hit_test(point, surface, anchor, theme)
    }
}
