// ─── < Imports > ────────────────────────────────────────────────────

use vello::Scene;

use crate::components::{Component, Interaction, Point, RenderCtx};
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
    center_sizes: ComponentSizes,
    left_bounds: Vec<Rect>,
    center_bounds: Vec<Rect>,
    right_bounds: Vec<Rect>,
}

#[derive(Debug, Clone, Copy)]
struct BarLayout {
    surface: Rect,
    pad_x: f32,
    pad_top: f32,
    gap: f32,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Bar {
    pub fn new(left: Components, center: Components, right: Components) -> Self {
        let center_sizes = Vec::with_capacity(center.len());
        let left_bounds = Vec::with_capacity(left.len());
        let center_bounds = Vec::with_capacity(center.len());
        let right_bounds = Vec::with_capacity(right.len());

        Self {
            left,
            center,
            right,
            center_sizes,
            left_bounds,
            center_bounds,
            right_bounds,
        }
    }

    pub fn render(&mut self, scene: &mut Scene, surface: Rect, theme: &Theme, ctx: &mut RenderCtx<'_>) {
        let layout = BarLayout::new(surface, theme);

        render_left_section(&mut self.left, &mut self.left_bounds, scene, layout, ctx);

        render_center_section(&mut self.center, &mut self.center_sizes, &mut self.center_bounds, scene, layout, ctx);

        render_right_section(&mut self.right, &mut self.right_bounds, scene, layout, ctx);
    }

    pub fn hit_test(&self, point: Point, theme: &Theme) -> Option<Interaction> {
        hit_test_section(&self.left, &self.left_bounds, point, theme)
            .or_else(|| hit_test_section(&self.center, &self.center_bounds, point, theme))
            .or_else(|| hit_test_section(&self.right, &self.right_bounds, point, theme))
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

// ─── < Private Functions > ────────────────────────────────────────────────────

fn render_left_section(
    components: &mut [Box<dyn Component>],
    bounds_buffer: &mut Vec<Rect>,
    scene: &mut Scene,
    layout: BarLayout,
    ctx: &mut RenderCtx<'_>,
) {
    bounds_buffer.clear();

    let mut x = layout.surface.x + layout.pad_x;

    for component in components {
        let (width, height) = component.measure(ctx);
        let bounds = Rect::new(x, layout.y(), width, height);

        bounds_buffer.push(bounds);
        component.render(scene, bounds, ctx);

        x += width + layout.gap;
    }
}

fn render_center_section(
    components: &mut [Box<dyn Component>],
    sizes: &mut ComponentSizes,
    bounds_buffer: &mut Vec<Rect>,
    scene: &mut Scene,
    layout: BarLayout,
    ctx: &mut RenderCtx<'_>,
) {
    bounds_buffer.clear();

    if components.is_empty() {
        return;
    }

    measure_components_into(components, sizes, ctx);

    let total_width = total_section_width(sizes, layout.gap);
    let mut x = layout.surface.x + (layout.surface.width - total_width) / 2.0;

    for (component, (width, height)) in components.iter_mut().zip(sizes.iter().copied()) {
        let bounds = Rect::new(x, layout.y(), width, height);

        bounds_buffer.push(bounds);
        component.render(scene, bounds, ctx);

        x += width + layout.gap;
    }
}

fn render_right_section(
    components: &mut [Box<dyn Component>],
    bounds_buffer: &mut Vec<Rect>,
    scene: &mut Scene,
    layout: BarLayout,
    ctx: &mut RenderCtx<'_>,
) {
    bounds_buffer.clear();

    let mut x = layout.surface.x + layout.surface.width - layout.pad_x;

    for component in components.iter_mut().rev() {
        let (width, height) = component.measure(ctx);

        x -= width;

        let bounds = Rect::new(x, layout.y(), width, height);

        bounds_buffer.push(bounds);
        component.render(scene, bounds, ctx);

        x -= layout.gap;
    }

    bounds_buffer.reverse();
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
    let components_width: f32 = sizes.iter().map(|(width, _height)| width).sum();
    let gaps_width = gap * sizes.len().saturating_sub(1) as f32;

    components_width + gaps_width
}
