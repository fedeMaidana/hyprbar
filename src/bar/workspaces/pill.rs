// ─── < Imports > ────────────────────────────────────────────────────

use calloop::channel::Sender;
use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Color, Fill};

use crate::app::WorkerHandle;
use crate::components::{Component, ComponentAction, ComponentTag, Interaction, InteractionOutcome, Pill, Point, RenderCtx};
use crate::hyprland_ipc::WorkspaceTarget;
use crate::render::{Rect, TextStyle};

use super::dispatcher::spawn_dispatcher;
use super::geometry::SlotGeometry;
use super::listener::spawn_listener;
use super::state::{WorkspaceData, WorkspaceId, WorkspaceStore};

// ─── < Constants > ────────────────────────────────────────────────────

const TAG: ComponentTag = ComponentTag::new("workspaces");

const SCROLL_THRESHOLD: f64 = 10.0;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct WorkspacesPill {
    store: WorkspaceStore,
    _listener: Option<WorkerHandle>,
    _dispatcher: Option<WorkerHandle>,
    dispatch: std::sync::mpsc::Sender<WorkspaceTarget>,
    measured_data: Option<WorkspaceData>,
    rendered_data: Option<WorkspaceData>,
    scroll_accumulator: f64,
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
        let listener = spawn_listener(store.clone(), redraw_signal);
        let (dispatcher, dispatch) = spawn_dispatcher();

        Self {
            store,
            _listener: listener,
            _dispatcher: dispatcher,
            dispatch,
            measured_data: None,
            rendered_data: None,
            scroll_accumulator: 0.0,
        }
    }

    fn snapshot(&self) -> WorkspaceData {
        self.store.snapshot()
    }

    /// Encola el dispatch; el worker hace la parte bloqueante.
    fn send_dispatch(&self, target: WorkspaceTarget) {
        if self.dispatch.send(target).is_err() {
            log::warn!("el dispatcher de workspaces no está corriendo; se pierde {target:?}");
        }
    }
}

/// The click action for a workspace slot, with the id as payload.
fn workspace_interaction(workspace_id: WorkspaceId) -> Interaction {
    Interaction::Action(ComponentAction::new(TAG, 0).with_value(workspace_id as i32))
}

fn workspace_from_interaction(interaction: Interaction) -> Option<WorkspaceId> {
    let Interaction::Action(action) = interaction else {
        return None;
    };

    if action.owner() != TAG {
        return None;
    }

    WorkspaceId::try_from(action.value()).ok()
}

impl Component for WorkspacesPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let geometry = SlotGeometry::from_theme(ctx.theme);
        let data = self.snapshot();

        let width = geometry.pill_width(&data);

        self.measured_data = Some(data);

        (width, ctx.theme.tokens.pill_height)
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        Pill::draw(scene, bounds, ctx.theme);

        let data = self.measured_data.take().unwrap_or_else(|| self.snapshot());
        let geometry = SlotGeometry::from_theme(ctx.theme);

        self.rendered_data = Some(data.clone());

        render_workspace_slots(scene, bounds, ctx, &data, &geometry);
    }

    fn hit_test(&self, point: Point, bounds: Rect, theme: &crate::theme::Theme) -> Option<Interaction> {
        let data = self.rendered_data.as_ref().cloned().unwrap_or_else(|| self.snapshot());
        let geometry = SlotGeometry::from_theme(theme);

        let mut x = bounds.x + geometry.h_padding;
        let box_y = bounds.y + (bounds.height - geometry.slot_box_height) / 2.0;

        for slot_id in 1..=data.visible_count() {
            let slot = SlotHitBox::from_workspace(slot_id, &data, &geometry);
            let slot_y = box_y + (geometry.slot_box_height - slot.height) / 2.0;
            let slot_bounds = Rect::new(x, slot_y, slot.width, slot.height);

            if slot_bounds.contains_point(point.x, point.y) {
                return Some(workspace_interaction(slot_id));
            }

            x += slot.width + geometry.gap;
        }

        None
    }

    fn handle_interaction(&mut self, interaction: Interaction) -> Option<InteractionOutcome> {
        let workspace_id = workspace_from_interaction(interaction)?;

        self.send_dispatch(WorkspaceTarget::Id(workspace_id));

        Some(InteractionOutcome::close_dropdown())
    }

    fn handle_scroll(&mut self, delta: f64) -> bool {
        self.scroll_accumulator += delta;

        let target = if self.scroll_accumulator >= SCROLL_THRESHOLD {
            Some(WorkspaceTarget::Next)
        } else if self.scroll_accumulator <= -SCROLL_THRESHOLD {
            Some(WorkspaceTarget::Previous)
        } else {
            None
        };

        let Some(target) = target else {
            return false;
        };

        self.scroll_accumulator = 0.0;
        self.send_dispatch(target);

        true
    }

    fn reset_scroll(&mut self) {
        self.scroll_accumulator = 0.0;
    }
}

impl SlotVisual {
    fn from_workspace(slot_id: WorkspaceId, data: &WorkspaceData, geometry: &SlotGeometry, ctx: &RenderCtx<'_>) -> Self {
        let exists = data.existing.contains(&slot_id);
        let is_active = slot_id == data.active_id;
        let is_hovered = ctx.hovered_interaction == Some(workspace_interaction(slot_id));

        if is_active {
            return Self {
                is_active,
                background: ctx.theme.palette.slot_active_bg,
                width: geometry.active_width,
                height: geometry.active_height,
                radius: geometry.active_radius,
            };
        }

        if is_hovered {
            return Self {
                is_active,
                background: ctx.theme.palette.slot_hover_bg,
                width: geometry.inactive_width,
                height: geometry.inactive_height,
                radius: geometry.inactive_radius,
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
    fn from_workspace(slot_id: WorkspaceId, data: &WorkspaceData, geometry: &SlotGeometry) -> Self {
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

fn draw_active_label(scene: &mut Scene, ctx: &mut RenderCtx<'_>, slot_id: WorkspaceId, slot_x: f32, slot_y: f32, slot: &SlotVisual) {
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
