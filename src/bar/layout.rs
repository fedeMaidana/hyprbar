use vello::Scene;

use crate::components::{Component, RenderCtx};
use crate::render::Rect;
use crate::theme::Theme;

type Components = Vec<Box<dyn Component>>;

pub struct Bar {
    left: Components,
    center: Components,
    right: Components,
}

impl Bar {
    pub fn new(left: Components, center: Components, right: Components) -> Self {
        Self {
            left,
            center,
            right,
        }
    }

    pub fn render(
        &mut self,
        scene: &mut Scene,
        surface: Rect,
        theme: &Theme,
        ctx: &mut RenderCtx<'_>,
    ) {
        let pad_x = theme.tokens.bar_margin_x;
        let pad_top = theme.tokens.bar_margin_top;
        let gap = theme.tokens.pill_gap;

        render_left_section(&mut self.left, scene, surface, pad_x, pad_top, gap, ctx);
        render_center_section(&mut self.center, scene, surface, pad_top, gap, ctx);
        render_right_section(&mut self.right, scene, surface, pad_x, pad_top, gap, ctx);
    }
}

fn render_left_section(
    components: &mut [Box<dyn Component>],
    scene: &mut Scene,
    surface: Rect,
    pad_x: f32,
    pad_top: f32,
    gap: f32,
    ctx: &mut RenderCtx<'_>,
) {
    let mut x = surface.x + pad_x;

    for component in components {
        let (width, height) = component.measure(ctx);
        let bounds = Rect::new(x, surface.y + pad_top, width, height);

        component.render(scene, bounds, ctx);

        x += width + gap;
    }
}

fn render_center_section(
    components: &mut [Box<dyn Component>],
    scene: &mut Scene,
    surface: Rect,
    pad_top: f32,
    gap: f32,
    ctx: &mut RenderCtx<'_>,
) {
    if components.is_empty() {
        return;
    }

    let sizes = measure_components(components, ctx);
    let total_width = total_section_width(&sizes, gap);

    let mut x = surface.x + (surface.width - total_width) / 2.0;

    for (component, (width, height)) in components.iter_mut().zip(sizes.iter().copied()) {
        let bounds = Rect::new(x, surface.y + pad_top, width, height);

        component.render(scene, bounds, ctx);

        x += width + gap;
    }
}

fn render_right_section(
    components: &mut [Box<dyn Component>],
    scene: &mut Scene,
    surface: Rect,
    pad_x: f32,
    pad_top: f32,
    gap: f32,
    ctx: &mut RenderCtx<'_>,
) {
    let mut x = surface.x + surface.width - pad_x;

    for component in components.iter_mut().rev() {
        let (width, height) = component.measure(ctx);
        x -= width;

        let bounds = Rect::new(x, surface.y + pad_top, width, height);

        component.render(scene, bounds, ctx);

        x -= gap;
    }
}

fn measure_components(
    components: &mut [Box<dyn Component>],
    ctx: &mut RenderCtx<'_>,
) -> Vec<(f32, f32)> {
    components
        .iter_mut()
        .map(|component| component.measure(ctx))
        .collect()
}

fn total_section_width(sizes: &[(f32, f32)], gap: f32) -> f32 {
    let components_width: f32 = sizes.iter().map(|(width, _height)| width).sum();
    let gaps_width = gap * sizes.len().saturating_sub(1) as f32;

    components_width + gaps_width
}
