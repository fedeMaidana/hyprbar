// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;

use super::dropdown::DropdownId;

use crate::bar::command_center::CommandAction;
use crate::bar::date::CalendarAction;
use crate::bar::profile::SessionAction;
use crate::bar::system::PowerAction;
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
    Power(PowerAction),
    Calendar(CalendarAction),
    Command(CommandAction),
    Session(SessionAction),
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

    fn dropdown_bounds(&self, _surface: Rect, _anchor: Rect, _theme: &Theme) -> Option<Rect> {
        None
    }

    fn hit_test_dropdown(&self, _point: Point, _surface: Rect, _anchor: Rect, _theme: &Theme) -> Option<Interaction> {
        None
    }

    fn handle_interaction(&mut self, _interaction: Interaction) -> bool {
        false
    }

    fn handle_drag(&mut self, _interaction: Interaction, _point: Point, _surface: Rect, _anchor: Rect, _theme: &Theme) -> bool {
        false
    }

    fn end_drag(&mut self, _interaction: Interaction) {}

    fn handle_scroll(&mut self, _delta: f64) -> bool {
        false
    }

    fn reset_scroll(&mut self) {}
}
