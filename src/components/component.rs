// ─── < Imports > ────────────────────────────────────────────────────

use std::time::Duration;

use vello::Scene;

use super::dropdown::DropdownId;

use crate::render::{Rect, TextEngine};
use crate::theme::Theme;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct RenderCtx<'a> {
    pub theme: &'a Theme,
    pub text: &'a mut TextEngine,
    pub hovered_interaction: Option<Interaction>,
    pub open_dropdown: Option<DropdownId>,
    /// Segundos desde el frame anterior, para avanzar transiciones.
    pub dt: f32,
    /// Los componentes lo prenden mientras tengan animaciones vivas;
    /// la app sigue pidiendo frames hasta que se apague.
    pub animating: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// Identifies which component owns an action, without the UI layer
/// knowing the concrete widget (e.g. `ComponentTag::new("command")`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentTag(&'static str);

/// An opaque, component-defined action. Each widget encodes its own
/// action enum into `id`/`value` and decodes it back in
/// `handle_interaction`, so the UI layer stays widget-agnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentAction {
    owner: ComponentTag,
    id: u16,
    value: i32,
    draggable: bool,
}

/// What the app should do after a component handled an interaction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InteractionOutcome {
    pub redraw: bool,
    pub close_dropdown: bool,
    pub toggle_theme: bool,
}

// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interaction {
    Dropdown(DropdownId),
    Action(ComponentAction),
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl ComponentTag {
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }
}

impl ComponentAction {
    pub const fn new(owner: ComponentTag, id: u16) -> Self {
        Self {
            owner,
            id,
            value: 0,
            draggable: false,
        }
    }

    pub const fn with_value(mut self, value: i32) -> Self {
        self.value = value;
        self
    }

    pub const fn draggable(mut self) -> Self {
        self.draggable = true;
        self
    }

    pub fn owner(self) -> ComponentTag {
        self.owner
    }

    pub fn id(self) -> u16 {
        self.id
    }

    pub fn value(self) -> i32 {
        self.value
    }

    pub fn is_draggable(self) -> bool {
        self.draggable
    }
}

impl Interaction {
    /// Whether pressing this interaction starts a drag gesture.
    pub fn is_draggable(self) -> bool {
        matches!(self, Self::Action(action) if action.is_draggable())
    }
}

impl InteractionOutcome {
    /// Handled, nothing else to do.
    pub fn quiet() -> Self {
        Self::default()
    }

    pub fn redraw() -> Self {
        Self {
            redraw: true,
            ..Self::default()
        }
    }

    pub fn close_dropdown() -> Self {
        Self {
            close_dropdown: true,
            ..Self::default()
        }
    }

    pub fn toggle_theme() -> Self {
        Self {
            toggle_theme: true,
            ..Self::default()
        }
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

    /// Tallest dropdown this component can open; sizes the bar surface.
    fn dropdown_max_height(&self, _theme: &Theme) -> f32 {
        0.0
    }

    /// Cadencia de repintado que necesita el dropdown de este componente
    /// mientras está abierto (p. ej. el reloj con sus segundos).
    /// `None` = no necesita ticks; la app no despierta por él.
    fn dropdown_tick(&self) -> Option<Duration> {
        None
    }

    fn render_dropdown(&mut self, _scene: &mut Scene, _surface: Rect, _anchor: Rect, _ctx: &mut RenderCtx<'_>) {}

    fn dropdown_bounds(&self, _surface: Rect, _anchor: Rect, _theme: &Theme) -> Option<Rect> {
        None
    }

    fn hit_test_dropdown(&self, _point: Point, _surface: Rect, _anchor: Rect, _theme: &Theme) -> Option<Interaction> {
        None
    }

    /// Handles an action owned by this component. Returns `None` when the
    /// action belongs to another component; otherwise the component runs
    /// any side effects itself and reports what the app should do.
    fn handle_interaction(&mut self, _interaction: Interaction) -> Option<InteractionOutcome> {
        None
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
