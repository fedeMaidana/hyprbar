//! Trait base de componente. Todo lo que se dibuja en la barra lo implementa.

use vello::Scene;

use crate::render::{Rect, TextEngine};
use crate::theme::Theme;

/// Contexto que se pasa a `render` y `measure`. Contiene tema y motor de texto.
pub struct RenderCtx<'a> {
    pub theme: &'a Theme,
    pub text: &'a mut TextEngine,
}

pub trait Component {
    /// Tamaño intrínseco del componente (sin restricciones). El layout puede pedir más.
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32);

    /// Renderiza el componente dentro de `bounds`.
    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>);
}
