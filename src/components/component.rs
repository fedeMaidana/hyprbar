use vello::Scene;

use crate::render::{Rect, TextEngine};
use crate::theme::Theme;

pub struct RenderCtx<'a> {
    pub theme: &'a Theme,
    pub text: &'a mut TextEngine,
}

pub trait Component {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32);

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>);
}
