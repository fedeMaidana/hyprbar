use std::path::Path;

use vello::{
    Scene,
    kurbo::{Affine, Circle},
    peniko::{BlendMode, Compose, Fill, ImageData, Mix},
};

use crate::components::{Component, RenderCtx};
use crate::render::Rect;

use super::avatar::load_avatar;

const BORDER_WIDTH: f32 = 2.5;
const OUTER_RADIUS_OFFSET: f32 = 0.5;

pub struct ProfilePill {
    avatar: Option<ImageData>,
}

impl ProfilePill {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        match load_avatar(path) {
            Ok(avatar) => {
                log::info!("profile avatar cargado: {}", path.display());

                Self {
                    avatar: Some(avatar),
                }
            }
            Err(error) => {
                log::warn!("no pude cargar {}: {error}", path.display());

                Self { avatar: None }
            }
        }
    }
}

impl Component for ProfilePill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let height = ctx.theme.tokens.pill_height;

        (height, height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        let layout = AvatarLayout::from_bounds(bounds);

        draw_border(scene, ctx, &layout);

        match &self.avatar {
            Some(avatar) => draw_avatar(scene, avatar, &layout),
            None => draw_placeholder(scene, ctx, &layout),
        }
    }
}

struct AvatarLayout {
    center_x: f32,
    center_y: f32,
    outer_radius: f32,
    inner_radius: f32,
}

impl AvatarLayout {
    fn from_bounds(bounds: Rect) -> Self {
        let center_x = bounds.x + bounds.width / 2.0;
        let center_y = bounds.y + bounds.height / 2.0;
        let outer_radius = (bounds.width.min(bounds.height) / 2.0) - OUTER_RADIUS_OFFSET;
        let inner_radius = outer_radius - BORDER_WIDTH;

        Self {
            center_x,
            center_y,
            outer_radius,
            inner_radius,
        }
    }

    fn outer_circle(&self) -> Circle {
        Circle::new(
            (self.center_x as f64, self.center_y as f64),
            self.outer_radius as f64,
        )
    }

    fn inner_circle(&self) -> Circle {
        Circle::new(
            (self.center_x as f64, self.center_y as f64),
            self.inner_radius as f64,
        )
    }
}

fn draw_border(scene: &mut Scene, ctx: &RenderCtx<'_>, layout: &AvatarLayout) {
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        ctx.theme.palette.pill_bg,
        None,
        &layout.outer_circle(),
    );
}

fn draw_avatar(scene: &mut Scene, avatar: &ImageData, layout: &AvatarLayout) {
    let inner_circle = layout.inner_circle();
    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);

    scene.push_layer(Fill::NonZero, blend, 1.0, Affine::IDENTITY, &inner_circle);

    let transform = avatar_cover_transform(avatar, layout);
    scene.draw_image(avatar, transform);

    scene.pop_layer();
}

fn draw_placeholder(scene: &mut Scene, ctx: &RenderCtx<'_>, layout: &AvatarLayout) {
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        ctx.theme.palette.text_secondary,
        None,
        &layout.inner_circle(),
    );
}

fn avatar_cover_transform(avatar: &ImageData, layout: &AvatarLayout) -> Affine {
    let image_width = avatar.width as f32;
    let image_height = avatar.height as f32;

    let target_size = layout.inner_radius * 2.0;
    let scale = target_size / image_width.min(image_height);

    let scaled_width = image_width * scale;
    let scaled_height = image_height * scale;

    let dx = layout.center_x - scaled_width / 2.0;
    let dy = layout.center_y - scaled_height / 2.0;

    Affine::translate((dx as f64, dy as f64)) * Affine::scale(scale as f64)
}
