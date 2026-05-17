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

    left_sizes: ComponentSizes,
    center_sizes: ComponentSizes,
    right_sizes: ComponentSizes,

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
        }
    }

    pub fn render(&mut self, scene: &mut Scene, surface: Rect, theme: &Theme, ctx: &mut RenderCtx<'_>) {
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
