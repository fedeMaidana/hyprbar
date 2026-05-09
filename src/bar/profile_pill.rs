//! Pill de perfil. Renderiza un avatar circular a partir de una imagen
//! cargada de disco al construir la pill.
//!
//! La imagen se pre-redimensiona al startup con un filtro Lanczos3 al
//! tamaño objetivo (con un poco de extra para soportar HiDPI). Esto evita
//! que vello tenga que hacer downscale en cada frame sin filter — los
//! resultados eran muy pixelados.

use std::path::Path;
use std::sync::Arc;

use vello::{
    Scene,
    kurbo::{Affine, Circle},
    peniko::{BlendMode, Blob, Compose, Fill, ImageAlphaType, ImageData, ImageFormat, Mix},
};

use crate::components::{Component, RenderCtx};
use crate::render::Rect;

/// Resolución a la que pre-redimensionamos el avatar. Un poco más alta
/// que el tamaño visible para tener margen ante HiDPI o si en el futuro
/// hacemos la pill más grande.
const AVATAR_RESOLUTION: u32 = 128;

pub struct ProfilePill {
    avatar: Option<ImageData>,
}

impl ProfilePill {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        match load_avatar(path) {
            Ok(img) => {
                log::info!("profile avatar cargado: {}", path.display());
                Self { avatar: Some(img) }
            }
            Err(e) => {
                log::warn!("no pude cargar {}: {e}", path.display());
                Self { avatar: None }
            }
        }
    }
}

impl Component for ProfilePill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let h = ctx.theme.tokens.pill_height;
        (h, h)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        let cx = bounds.x + bounds.width / 2.0;
        let cy = bounds.y + bounds.height / 2.0;
        // Borde: el círculo de fondo es un poco más grande que el del avatar.
        // Width controla el grosor del borde visible.
        let border_width = 2.5_f32;
        let outer_radius = (bounds.width.min(bounds.height) / 2.0) - 0.5;
        let inner_radius = outer_radius - border_width;

        let outer = Circle::new((cx as f64, cy as f64), outer_radius as f64);
        let inner = Circle::new((cx as f64, cy as f64), inner_radius as f64);

        // 1) Borde: círculo oscuro de fondo (mismo color que las pills,
        //    así se siente parte del lenguaje visual).
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            ctx.theme.palette.pill_bg,
            None,
            &outer,
        );

        match &self.avatar {
            Some(img) => {
                // 2) Clip al círculo interno y dibujar el avatar dentro.
                let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);
                scene.push_layer(Fill::NonZero, blend, 1.0, Affine::IDENTITY, &inner);

                // Cover scaling al diámetro del círculo interno
                let img_w = img.width as f32;
                let img_h = img.height as f32;
                let target = inner_radius * 2.0;
                let scale = target / img_w.min(img_h);
                let scaled_w = img_w * scale;
                let scaled_h = img_h * scale;
                let dx = cx - scaled_w / 2.0;
                let dy = cy - scaled_h / 2.0;

                let transform =
                    Affine::translate((dx as f64, dy as f64)) * Affine::scale(scale as f64);
                scene.draw_image(img, transform);

                scene.pop_layer();
            }
            None => {
                // Placeholder: círculo gris adentro del borde
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    ctx.theme.palette.text_secondary,
                    None,
                    &inner,
                );
            }
        }
    }
}

fn load_avatar(path: &Path) -> anyhow::Result<ImageData> {
    // Decodificamos y bajamos resolución con Lanczos3 al startup.
    // Esto evita downscale en cada frame de render con sampling crudo.
    let img = image::open(path)?
        .resize_to_fill(
            AVATAR_RESOLUTION,
            AVATAR_RESOLUTION,
            image::imageops::FilterType::Lanczos3,
        )
        .to_rgba8();
    let (width, height) = img.dimensions();
    let data = Arc::new(img.into_raw());
    Ok(ImageData {
        data: Blob::new(data),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width,
        height,
    })
}
