// ─── < Imports > ────────────────────────────────────────────────────

use calloop::channel::Sender;
use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Color, Fill};

use crate::components::{Component, Interaction, Pill, Point, RenderCtx};
use crate::render::{Rect, TextStyle};

use super::geometry::SlotGeometry;
use super::listener::spawn_listener;
use super::state::{WorkspaceData, WorkspaceStore};

// ─── < Structs > ────────────────────────────────────────────────────

pub struct WorkspacesPill {
    store: WorkspaceStore,
}

struct SlotVisual {
    is_active: bool,
    background: Color,
    width: f32,
    height: f32,
    radius: f32,
}

struct SlotHitBox {
    width: f32,
    height: f32,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl WorkspacesPill {
    pub fn new(redraw_signal: Sender<()>) -> Self {
        let store = WorkspaceStore::new();

        spawn_listener(store.clone(), redraw_signal);

        Self { store }
    }

    fn snapshot(&self) -> WorkspaceData {
        self.store.snapshot()
    }
}

impl Component for WorkspacesPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let geometry = SlotGeometry::from_theme(ctx.theme);
        let data = self.snapshot();

        (geometry.pill_width(&data), ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        Pill::draw(scene, bounds, ctx.theme);

        let data = self.snapshot();
        let geometry = SlotGeometry::from_theme(ctx.theme);

        render_workspace_slots(scene, bounds, ctx, &data, &geometry);
    }

    fn hit_test(&self, point: Point, bounds: Rect, theme: &crate::theme::Theme) -> Option<Interaction> {
        let data = self.snapshot();
        let geometry = SlotGeometry::from_theme(theme);

        let mut x = bounds.x + geometry.h_padding;
        let box_y = bounds.y + (bounds.height - geometry.slot_box_height) / 2.0;

        for slot_id in 1..=data.visible_count() {
            let slot = SlotHitBox::from_workspace(slot_id, &data, &geometry);
            let slot_y = box_y + (geometry.slot_box_height - slot.height) / 2.0;
            let slot_bounds = Rect::new(x, slot_y, slot.width, slot.height);

            if slot_bounds.contains_point(point.x, point.y) {
                return Some(Interaction::Workspace(slot_id));
            }

            x += slot.width + geometry.gap;
        }

        None
    }
}

impl SlotVisual {
    fn from_workspace(slot_id: i32, data: &WorkspaceData, geometry: &SlotGeometry, ctx: &RenderCtx<'_>) -> Self {
        let exists = data.existing.contains(&slot_id);
        let is_active = slot_id == data.active_id;

        if is_active {
            return Self {
                is_active,
                background: ctx.theme.palette.slot_active_bg,
                width: geometry.active_width,
                height: geometry.active_height,
                radius: geometry.active_radius,
            };
        }

        if exists {
            return Self {
                is_active,
                background: ctx.theme.palette.slot_inactive_bg,
                width: geometry.inactive_width,
                height: geometry.inactive_height,
                radius: geometry.inactive_radius,
            };
        }

        Self {
            is_active,
            background: ctx.theme.palette.slot_empty_bg,
            width: geometry.inactive_width,
            height: geometry.inactive_height,
            radius: geometry.inactive_radius,
        }
    }
}

impl SlotHitBox {
    fn from_workspace(slot_id: i32, data: &WorkspaceData, geometry: &SlotGeometry) -> Self {
        if slot_id == data.active_id {
            return Self {
                width: geometry.active_width,
                height: geometry.active_height,
            };
        }

        Self {
            width: geometry.inactive_width,
            height: geometry.inactive_height,
        }
    }
}

// ─── < Functions Private > ────────────────────────────────────────────────────

fn render_workspace_slots(scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>, data: &WorkspaceData, geometry: &SlotGeometry) {
    let mut x = bounds.x + geometry.h_padding;
    let box_y = bounds.y + (bounds.height - geometry.slot_box_height) / 2.0;

    for slot_id in 1..=data.visible_count() {
        let slot = SlotVisual::from_workspace(slot_id, data, geometry, ctx);
        let slot_y = box_y + (geometry.slot_box_height - slot.height) / 2.0;

        draw_slot(scene, x, slot_y, &slot);

        if slot.is_active {
            draw_active_label(scene, ctx, slot_id, x, slot_y, &slot);
        }

        x += slot.width + geometry.gap;
    }
}

fn draw_slot(scene: &mut Scene, x: f32, y: f32, slot: &SlotVisual) {
    let slot_rect = RoundedRect::new(x as f64, y as f64, (x + slot.width) as f64, (y + slot.height) as f64, slot.radius as f64);

    scene.fill(Fill::NonZero, Affine::IDENTITY, slot.background, None, &slot_rect);
}

fn draw_active_label(scene: &mut Scene, ctx: &mut RenderCtx<'_>, slot_id: i32, slot_x: f32, slot_y: f32, slot: &SlotVisual) {
    let label = slot_id.to_string();
    let size = ctx.theme.typography.size_base;

    let (text_width, _) = ctx.text.measure(&label, size, &ctx.theme.typography.font_family);

    let text_x = slot_x + (slot.width - text_width) / 2.0;

    ctx.text.draw_centered_v(
        scene,
        &label,
        text_x,
        slot_y,
        slot.height,
        TextStyle::new(size, &ctx.theme.typography.font_family, ctx.theme.palette.slot_active_text),
    );
}
