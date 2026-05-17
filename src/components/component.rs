// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;

use super::dropdown::DropdownId;

use crate::bar::workspaces::WorkspaceId;
use crate::render::{Rect, TextEngine};
use crate::theme::Theme;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct RenderCtx<'a> {
    pub theme: &'a Theme,
    pub text: &'a mut TextEngine,
    pub hovered_interaction: Option<Interaction>,
    pub open_dropdown: Option<DropdownId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interaction {
    Workspace(WorkspaceId),
    Dropdown(DropdownId),
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

// ─── < Traits > ────────────────────────────────────────────────────

pub trait Component {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32);

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>);

    fn hit_test(&self, _point: Point, _bounds: Rect, _theme: &Theme) -> Option<Interaction> {
        None
    }

    fn dropdown_id(&self) -> Option<DropdownId> {
        None
    }

    fn render_dropdown(&mut self, _scene: &mut Scene, _surface: Rect, _anchor: Rect, _ctx: &mut RenderCtx<'_>) {}
}
