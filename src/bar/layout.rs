// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;

use crate::components::{Component, DropdownId, Interaction, InteractionOutcome, Point, RenderCtx};
use crate::render::Rect;
use crate::theme::Theme;

// ─── < Types > ────────────────────────────────────────────────────

type Components = Vec<Box<dyn Component>>;
type ComponentSizes = Vec<(f32, f32)>;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct Bar {
    left: Components,
    center: Components,
    right: Components,

    left_sizes: ComponentSizes,
    center_sizes: ComponentSizes,
    right_sizes: ComponentSizes,

    left_bounds: Vec<Rect>,
    center_bounds: Vec<Rect>,
    right_bounds: Vec<Rect>,

    last_surface: Option<Rect>,
}

#[derive(Debug, Clone, Copy)]
struct BarLayout {
    surface: Rect,
    pad_x: f32,
    pad_top: f32,
    gap: f32,
}

#[derive(Debug, Clone, Copy)]
struct SectionPlacement {
    x: f32,
    width: f32,
}

#[derive(Debug, Clone, Copy)]
struct CenterConstraints {
    layout: BarLayout,
    left: SectionPlacement,
    right: SectionPlacement,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Bar {
    pub fn new(left: Components, center: Components, right: Components) -> Self {
        let left_sizes = Vec::with_capacity(left.len());
        let center_sizes = Vec::with_capacity(center.len());
        let right_sizes = Vec::with_capacity(right.len());

        let left_bounds = Vec::with_capacity(left.len());
        let center_bounds = Vec::with_capacity(center.len());
        let right_bounds = Vec::with_capacity(right.len());

        Self {
            left,
            center,
            right,
            left_sizes,
            center_sizes,
            right_sizes,
            left_bounds,
            center_bounds,
            right_bounds,
            last_surface: None,
        }
    }

    pub fn render(&mut self, scene: &mut Scene, surface: Rect, theme: &Theme, ctx: &mut RenderCtx<'_>) {
        self.last_surface = Some(surface);

        let layout = BarLayout::new(surface, theme);

        measure_components_into(&mut self.left, &mut self.left_sizes, ctx);
        measure_components_into(&mut self.center, &mut self.center_sizes, ctx);
        measure_components_into(&mut self.right, &mut self.right_sizes, ctx);

        let left = left_placement(&self.left_sizes, layout);
        let right = right_placement(&self.right_sizes, layout);

        render_section(&mut self.left, &self.left_sizes, &mut self.left_bounds, scene, left.x, layout, ctx);

        render_section(&mut self.right, &self.right_sizes, &mut self.right_bounds, scene, right.x, layout, ctx);

        render_center_section(
            &mut self.center,
            &self.center_sizes,
            &mut self.center_bounds,
            scene,
            CenterConstraints { layout, left, right },
            ctx,
        );

        self.render_active_dropdown(scene, surface, ctx);
    }

    /// Tallest dropdown any component can open; sizes the bar surface.
    pub fn max_dropdown_height(&self, theme: &Theme) -> f32 {
        self.left
            .iter()
            .chain(self.center.iter())
            .chain(self.right.iter())
            .map(|component| component.dropdown_max_height(theme))
            .fold(0.0, f32::max)
    }

    pub fn hit_test(&self, point: Point, theme: &Theme, open_dropdown: Option<DropdownId>) -> Option<Interaction> {
        self.hit_test_open_dropdown(point, theme, open_dropdown)
            .or_else(|| hit_test_section(&self.left, &self.left_bounds, point, theme))
            .or_else(|| hit_test_section(&self.center, &self.center_bounds, point, theme))
            .or_else(|| hit_test_section(&self.right, &self.right_bounds, point, theme))
    }

    pub fn dropdown_contains_point(&self, point: Point, theme: &Theme, open_dropdown: Option<DropdownId>) -> bool {
        let Some(bounds) = self.open_dropdown_bounds(theme, open_dropdown) else {
            return false;
        };

        bounds.contains_point(point.x, point.y)
    }

    pub fn handle_interaction(&mut self, interaction: Interaction) -> Option<InteractionOutcome> {
        for component in self.left.iter_mut().chain(self.center.iter_mut()).chain(self.right.iter_mut()) {
            if let Some(outcome) = component.handle_interaction(interaction) {
                return Some(outcome);
            }
        }

        None
    }

    pub fn handle_drag(&mut self, interaction: Interaction, point: Point, theme: &Theme, open_dropdown: Option<DropdownId>) -> bool {
        let Some(surface) = self.last_surface else {
            return false;
        };

        let Some(dropdown_id) = open_dropdown else {
            return false;
        };

        let Some((component, anchor)) = self.dropdown_component_mut(dropdown_id) else {
            return false;
        };

        component.handle_drag(interaction, point, surface, anchor, theme)
    }

    pub fn end_drag(&mut self, interaction: Interaction, open_dropdown: Option<DropdownId>) {
        let Some(dropdown_id) = open_dropdown else {
            return;
        };

        if let Some((component, _anchor)) = self.dropdown_component_mut(dropdown_id) {
            component.end_drag(interaction);
        }
    }

    pub fn handle_scroll(&mut self, point: Point, delta: f64) -> bool {
        for (components, bounds) in [
            (&mut self.left, &self.left_bounds),
            (&mut self.center, &self.center_bounds),
            (&mut self.right, &self.right_bounds),
        ] {
            for (component, component_bounds) in components.iter_mut().zip(bounds.iter().copied()) {
                if component_bounds.contains_point(point.x, point.y) {
                    return component.handle_scroll(delta);
                }
            }
        }

        false
    }

    pub fn reset_scroll(&mut self) {
        for component in self.left.iter_mut().chain(self.center.iter_mut()).chain(self.right.iter_mut()) {
            component.reset_scroll();
        }
    }

    fn render_active_dropdown(&mut self, scene: &mut Scene, surface: Rect, ctx: &mut RenderCtx<'_>) {
        let Some(open_dropdown) = ctx.open_dropdown else {
            return;
        };

        let Some((component, anchor)) = self.dropdown_component_mut(open_dropdown) else {
            return;
        };

        component.render_dropdown(scene, surface, anchor, ctx);
    }

    fn dropdown_component_mut(&mut self, dropdown_id: DropdownId) -> Option<(&mut dyn Component, Rect)> {
        let sections = [
            (&mut self.left, &self.left_bounds),
            (&mut self.center, &self.center_bounds),
            (&mut self.right, &self.right_bounds),
        ];

        for (components, bounds) in sections {
            for (component, anchor) in components.iter_mut().zip(bounds.iter().copied()) {
                if component.dropdown_id() == Some(dropdown_id) {
                    return Some((component.as_mut(), anchor));
                }
            }
        }

        None
    }

    fn hit_test_open_dropdown(&self, point: Point, theme: &Theme, open_dropdown: Option<DropdownId>) -> Option<Interaction> {
        let surface = self.last_surface?;
        let dropdown_id = open_dropdown?;
        let (component, anchor) = self.dropdown_component(dropdown_id)?;

        component.hit_test_dropdown(point, surface, anchor, theme)
    }

    fn open_dropdown_bounds(&self, theme: &Theme, open_dropdown: Option<DropdownId>) -> Option<Rect> {
        let surface = self.last_surface?;
        let dropdown_id = open_dropdown?;
        let (component, anchor) = self.dropdown_component(dropdown_id)?;

        component.dropdown_bounds(surface, anchor, theme)
    }

    fn dropdown_component(&self, dropdown_id: DropdownId) -> Option<(&dyn Component, Rect)> {
        let sections = [
            (&self.left, &self.left_bounds),
            (&self.center, &self.center_bounds),
            (&self.right, &self.right_bounds),
        ];

        for (components, bounds) in sections {
            for (component, anchor) in components.iter().zip(bounds.iter().copied()) {
                if component.dropdown_id() == Some(dropdown_id) {
                    return Some((component.as_ref(), anchor));
                }
            }
        }

        None
    }
}

impl BarLayout {
    fn new(surface: Rect, theme: &Theme) -> Self {
        Self {
            surface,
            pad_x: theme.tokens.bar_margin_x,
            pad_top: theme.tokens.bar_margin_top,
            gap: theme.tokens.pill_gap,
        }
    }

    fn y(&self) -> f32 {
        self.surface.y + self.pad_top
    }
}

impl SectionPlacement {
    fn end_x(self) -> f32 {
        self.x + self.width
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn left_placement(sizes: &[(f32, f32)], layout: BarLayout) -> SectionPlacement {
    SectionPlacement {
        x: layout.surface.x + layout.pad_x,
        width: total_section_width(sizes, layout.gap),
    }
}

fn right_placement(sizes: &[(f32, f32)], layout: BarLayout) -> SectionPlacement {
    let width = total_section_width(sizes, layout.gap);

    SectionPlacement {
        x: layout.surface.x + layout.surface.width - layout.pad_x - width,
        width,
    }
}

fn render_center_section(
    components: &mut [Box<dyn Component>],
    sizes: &[(f32, f32)],
    bounds_buffer: &mut Vec<Rect>,
    scene: &mut Scene,
    constraints: CenterConstraints,
    ctx: &mut RenderCtx<'_>,
) {
    bounds_buffer.clear();

    if components.is_empty() {
        return;
    }

    let layout = constraints.layout;
    let center_width = total_section_width(sizes, layout.gap);

    if center_width <= 0.0 {
        return;
    }

    let safe_left = constraints.left.end_x() + layout.gap;
    let safe_right = constraints.right.x - layout.gap;
    let available_width = safe_right - safe_left;

    if center_width > available_width {
        log::warn!("center section hidden: required_width={} available_width={}", center_width, available_width);

        return;
    }

    let ideal_x = layout.surface.x + (layout.surface.width - center_width) / 2.0;
    let x = ideal_x.clamp(safe_left, safe_right - center_width);

    render_section(components, sizes, bounds_buffer, scene, x, layout, ctx);
}

fn render_section(
    components: &mut [Box<dyn Component>],
    sizes: &[(f32, f32)],
    bounds_buffer: &mut Vec<Rect>,
    scene: &mut Scene,
    start_x: f32,
    layout: BarLayout,
    ctx: &mut RenderCtx<'_>,
) {
    bounds_buffer.clear();

    let mut x = start_x;

    for (component, (width, height)) in components.iter_mut().zip(sizes.iter().copied()) {
        let bounds = Rect::new(x, layout.y(), width, height);

        bounds_buffer.push(bounds);
        component.render(scene, bounds, ctx);

        x += width + layout.gap;
    }
}

fn hit_test_section(components: &[Box<dyn Component>], bounds: &[Rect], point: Point, theme: &Theme) -> Option<Interaction> {
    for (component, bounds) in components.iter().zip(bounds.iter().copied()) {
        if bounds.contains_point(point.x, point.y)
            && let Some(interaction) = component.hit_test(point, bounds, theme)
        {
            return Some(interaction);
        }
    }

    None
}

fn measure_components_into(components: &mut [Box<dyn Component>], sizes: &mut ComponentSizes, ctx: &mut RenderCtx<'_>) {
    sizes.clear();

    sizes.extend(components.iter_mut().map(|component| component.measure(ctx)));
}

fn total_section_width(sizes: &[(f32, f32)], gap: f32) -> f32 {
    if sizes.is_empty() {
        return 0.0;
    }

    let components_width: f32 = sizes.iter().map(|(width, _height)| width).sum();
    let gaps_width = gap * sizes.len().saturating_sub(1) as f32;

    components_width + gaps_width
}
