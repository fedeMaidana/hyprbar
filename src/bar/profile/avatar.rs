use std::path::Path;
use std::sync::Arc;

use vello::peniko::{Blob, ImageAlphaType, ImageData, ImageFormat};

const AVATAR_RESOLUTION: u32 = 128;

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
