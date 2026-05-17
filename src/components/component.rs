use vello::Scene;

use crate::render::{Rect, TextEngine};
use crate::theme::Theme;

pub struct RenderCtx<'a> {
    pub theme: &'a Theme,
    pub text: &'a mut TextEngine,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interaction {
    Workspace(i32),
}

pub trait Component {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32);

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>);

    fn hit_test(&self, _point: Point, _bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        None
    }
}
