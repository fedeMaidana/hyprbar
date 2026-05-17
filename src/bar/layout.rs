use vello::Scene;

use crate::components::{Component, RenderCtx};
use crate::render::Rect;
use crate::theme::Theme;

type Components = Vec<Box<dyn Component>>;
type ComponentSizes = Vec<(f32, f32)>;

pub struct Bar {
    left: Components,
    center: Components,
    right: Components,
    center_sizes: ComponentSizes,
}

impl Bar {
    pub fn new(left: Components, center: Components, right: Components) -> Self {
        let center_sizes = Vec::with_capacity(center.len());

        Self {
            left,
            center,
            right,
            center_sizes,
        }
    }

    pub fn render(
        &mut self,
        scene: &mut Scene,
        surface: Rect,
        theme: &Theme,
        ctx: &mut RenderCtx<'_>,
    ) {
        let layout = BarLayout::new(surface, theme);

        render_left_section(&mut self.left, scene, layout, ctx);

        render_center_section(&mut self.center, &mut self.center_sizes, scene, layout, ctx);

        render_right_section(&mut self.right, scene, layout, ctx);
    }
}

#[derive(Debug, Clone, Copy)]
struct BarLayout {
    surface: Rect,
    pad_x: f32,
    pad_top: f32,
    gap: f32,
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

fn render_left_section(
    components: &mut [Box<dyn Component>],
    scene: &mut Scene,
    layout: BarLayout,
    ctx: &mut RenderCtx<'_>,
) {
    let mut x = layout.surface.x + layout.pad_x;

    for component in components {
        let (width, height) = component.measure(ctx);
        let bounds = Rect::new(x, layout.y(), width, height);

        component.render(scene, bounds, ctx);

        x += width + layout.gap;
    }
}

fn render_center_section(
    components: &mut [Box<dyn Component>],
    sizes: &mut ComponentSizes,
    scene: &mut Scene,
    layout: BarLayout,
    ctx: &mut RenderCtx<'_>,
) {
    if components.is_empty() {
        return;
    }

    measure_components_into(components, sizes, ctx);

    let total_width = total_section_width(sizes, layout.gap);
    let mut x = layout.surface.x + (layout.surface.width - total_width) / 2.0;

    for (component, (width, height)) in components.iter_mut().zip(sizes.iter().copied()) {
        let bounds = Rect::new(x, layout.y(), width, height);

        component.render(scene, bounds, ctx);

        x += width + layout.gap;
    }
}

fn render_right_section(
    components: &mut [Box<dyn Component>],
    scene: &mut Scene,
    layout: BarLayout,
    ctx: &mut RenderCtx<'_>,
) {
    let mut x = layout.surface.x + layout.surface.width - layout.pad_x;

    for component in components.iter_mut().rev() {
        let (width, height) = component.measure(ctx);

        x -= width;

        let bounds = Rect::new(x, layout.y(), width, height);

        component.render(scene, bounds, ctx);

        x -= layout.gap;
    }
}

fn measure_components_into(
    components: &mut [Box<dyn Component>],
    sizes: &mut ComponentSizes,
    ctx: &mut RenderCtx<'_>,
) {
    sizes.clear();

    sizes.extend(
        components
            .iter_mut()
            .map(|component| component.measure(ctx)),
    );
}

fn total_section_width(sizes: &[(f32, f32)], gap: f32) -> f32 {
    let components_width: f32 = sizes.iter().map(|(width, _height)| width).sum();
    let gaps_width = gap * sizes.len().saturating_sub(1) as f32;

    components_width + gaps_width
}
