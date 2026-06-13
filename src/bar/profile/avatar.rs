// ─── < Imports > ────────────────────────────────────────────────────

use std::path::Path;
use std::sync::Arc;

use vello::Scene;
use vello::kurbo::{Affine, Circle};
use vello::peniko::{BlendMode, Blob, Color, Compose, Fill, ImageAlphaType, ImageData, ImageFormat, Mix};

// ─── < Constants > ────────────────────────────────────────────────────

const AVATAR_RESOLUTION: u32 = 128;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct AvatarCircle {
    center_x: f32,
    center_y: f32,
    outer_radius: f32,
    inner_radius: f32,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl AvatarCircle {
    pub fn new(center_x: f32, center_y: f32, outer_radius: f32, border_width: f32) -> Self {
        Self {
            center_x,
            center_y,
            outer_radius,
            inner_radius: (outer_radius - border_width).max(0.0),
        }
    }

    fn outer(&self) -> Circle {
        Circle::new((self.center_x as f64, self.center_y as f64), self.outer_radius as f64)
    }

    fn inner(&self) -> Circle {
        Circle::new((self.center_x as f64, self.center_y as f64), self.inner_radius as f64)
    }
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn load_avatar(path: &Path) -> anyhow::Result<ImageData> {
    let image = image::open(path)?
        .resize_to_fill(AVATAR_RESOLUTION, AVATAR_RESOLUTION, image::imageops::FilterType::Lanczos3)
        .to_rgba8();

    let (width, height) = image.dimensions();
    let data = Arc::new(image.into_raw());

    Ok(ImageData {
        data: Blob::new(data),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width,
        height,
    })
}

pub fn draw_avatar_circle(
    scene: &mut Scene,
    avatar: Option<&ImageData>,
    circle: &AvatarCircle,
    border_color: Color,
    placeholder_color: Color,
) {
    scene.fill(Fill::NonZero, Affine::IDENTITY, border_color, None, &circle.outer());

    match avatar {
        Some(avatar) => draw_avatar_image(scene, avatar, circle),
        None => {
            scene.fill(Fill::NonZero, Affine::IDENTITY, placeholder_color, None, &circle.inner());
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn draw_avatar_image(scene: &mut Scene, avatar: &ImageData, circle: &AvatarCircle) {
    let inner_circle = circle.inner();
    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);

    scene.push_layer(Fill::NonZero, blend, 1.0, Affine::IDENTITY, &inner_circle);

    let transform = avatar_cover_transform(avatar, circle);
    scene.draw_image(avatar, transform);

    scene.pop_layer();
}

fn avatar_cover_transform(avatar: &ImageData, circle: &AvatarCircle) -> Affine {
    let image_width = avatar.width as f32;
    let image_height = avatar.height as f32;

    let target_size = circle.inner_radius * 2.0;
    let scale = target_size / image_width.min(image_height);

    let scaled_width = image_width * scale;
    let scaled_height = image_height * scale;

    let dx = circle.center_x - scaled_width / 2.0;
    let dy = circle.center_y - scaled_height / 2.0;

    Affine::translate((dx as f64, dy as f64)) * Affine::scale(scale as f64)
}
