//! Pill: rectángulo redondeado con fondo glassy y sombra.
//! Es el chasis visual de TODOS los componentes de la barra.
//!
//! Uso: el componente concreto (date, clock, weather) llama a `Pill::draw`
//! para el fondo, y después dibuja su contenido encima.

use vello::{
    Scene,
    kurbo::{Affine, RoundedRect},
    peniko::Fill
};

use crate::render::Rect;
use crate::theme::Theme;

pub struct Pill;

impl Pill {
    /// Dibuja el fondo de la pill (con sombra fake como rect adicional debajo).
    pub fn draw(scene: &mut Scene, bounds: Rect, theme: &Theme) {
        let radius = theme.tokens.pill_radius as f64;

        // Sombra: rect ligeramente más grande, desplazado, color shadow.
        // (Vello no tiene blur built-in, esto es una sombra "fake" sólida.
        // Para blur real habría que pre-renderizar a una textura y aplicar shader.)
        let shadow_offset = theme.tokens.shadow_offset_y as f64;
        let shadow = RoundedRect::new(
            bounds.x as f64,
            bounds.y as f64 + shadow_offset,
            (bounds.x + bounds.width) as f64,
            (bounds.y + bounds.height) as f64 + shadow_offset,
            radius,
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            theme.palette.shadow,
            None,
            &shadow,
        );

        // Cuerpo de la pill
        let body = RoundedRect::new(
            bounds.x as f64,
            bounds.y as f64,
            (bounds.x + bounds.width) as f64,
            (bounds.y + bounds.height) as f64,
            radius,
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            theme.palette.pill_bg,
            None,
            &body,
        );
    }
}
