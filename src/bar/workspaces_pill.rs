use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use calloop::channel::Sender;
use serde_json::Value;
use vello::{
    Scene,
    kurbo::{Affine, RoundedRect},
    peniko::Fill,
};

use crate::components::{Component, Pill, RenderCtx};
use crate::hyprland_ipc;
use crate::render::{Rect, TextStyle};

const MIN_VISIBLE_WORKSPACES: i32 = 3;

#[derive(Debug, Clone, Default)]
struct WorkspaceData {
    existing: Vec<i32>,
    active_id: i32,
}

impl WorkspaceData {
    fn visible_count(&self) -> i32 {
        let max_id = self
            .existing
            .iter()
            .copied()
            .max()
            .unwrap_or(0)
            .max(self.active_id);
        max_id.max(MIN_VISIBLE_WORKSPACES)
    }
}

pub struct WorkspacesPill {
    state: Arc<Mutex<WorkspaceData>>,
}

impl WorkspacesPill {
    pub fn new(redraw_signal: Sender<()>) -> Self {
        let state: Arc<Mutex<WorkspaceData>> = Arc::new(Mutex::new(WorkspaceData::default()));

        let state_clone = Arc::clone(&state);
        thread::spawn(move || listener_loop(state_clone, redraw_signal));

        Self { state }
    }

    fn snapshot(&self) -> WorkspaceData {
        self.state.lock().unwrap().clone()
    }
}

struct SlotGeometry {
    active_width: f32,
    active_height: f32,
    active_radius: f32,
    inactive_width: f32,
    inactive_height: f32,
    inactive_radius: f32,
    slot_box_height: f32,
    gap: f32,
    h_padding: f32,
}

impl SlotGeometry {
    fn from_theme(theme: &crate::theme::Theme) -> Self {
        let v_padding = theme.tokens.pill_padding_y;
        let cell_height = theme.tokens.pill_height - v_padding * 2.0;
        let active_height = cell_height;
        let active_width = active_height * 1.7;
        let active_radius = active_height * 0.5;

        let inactive_scale = 1.0;
        let inactive_height = active_height * inactive_scale;
        let inactive_width = inactive_height * 1.1;
        let inactive_radius = inactive_height * 0.35;

        Self {
            active_width,
            active_height,
            active_radius,
            inactive_width,
            inactive_height,
            inactive_radius,
            slot_box_height: cell_height,
            gap: 6.0,
            h_padding: theme.tokens.pill_padding_x,
        }
    }

    fn pill_width(&self, active_count: i32, inactive_count: i32) -> f32 {
        let total_slots = active_count + inactive_count;
        let inner_w = self.active_width * active_count as f32
            + self.inactive_width * inactive_count as f32
            + self.gap * (total_slots - 1).max(0) as f32;
        inner_w + self.h_padding * 2.0
    }
}

impl Component for WorkspacesPill {
    fn measure(&mut self, ctx: &mut RenderCtx<'_>) -> (f32, f32) {
        let geom = SlotGeometry::from_theme(ctx.theme);
        let data = self.snapshot();
        let count = data.visible_count();
        let active_visible = if data.active_id >= 1 && data.active_id <= count {
            1
        } else {
            0
        };
        let inactive_visible = count - active_visible;
        (
            geom.pill_width(active_visible, inactive_visible),
            ctx.theme.tokens.pill_height,
        )
    }

    fn render(&mut self, scene: &mut Scene, bounds: Rect, ctx: &mut RenderCtx<'_>) {
        Pill::draw(scene, bounds, ctx.theme);

        let data = self.snapshot();
        let geom = SlotGeometry::from_theme(ctx.theme);
        let palette = &ctx.theme.palette;

        let mut x = bounds.x + geom.h_padding;
        let box_y = bounds.y + (bounds.height - geom.slot_box_height) / 2.0;

        for slot_id in 1..=data.visible_count() {
            let exists = data.existing.contains(&slot_id);
            let is_active = slot_id == data.active_id;

            let (bg_color, slot_w, slot_h, slot_r) = if is_active {
                (
                    palette.slot_active_bg,
                    geom.active_width,
                    geom.active_height,
                    geom.active_radius,
                )
            } else if exists {
                (
                    palette.slot_inactive_bg,
                    geom.inactive_width,
                    geom.inactive_height,
                    geom.inactive_radius,
                )
            } else {
                (
                    palette.slot_empty_bg,
                    geom.inactive_width,
                    geom.inactive_height,
                    geom.inactive_radius,
                )
            };

            let slot_y = box_y + (geom.slot_box_height - slot_h) / 2.0;

            let slot_rect = RoundedRect::new(
                x as f64,
                slot_y as f64,
                (x + slot_w) as f64,
                (slot_y + slot_h) as f64,
                slot_r as f64,
            );
            scene.fill(Fill::NonZero, Affine::IDENTITY, bg_color, None, &slot_rect);

            if is_active {
                let label = slot_id.to_string();
                let size = ctx.theme.typography.size_base;
                let (tw, _) = ctx
                    .text
                    .measure(&label, size, &ctx.theme.typography.font_family);
                let text_x = x + (slot_w - tw) / 2.0;
                ctx.text.draw_centered_v(
                    scene,
                    &label,
                    text_x,
                    slot_y,
                    slot_h,
                    TextStyle::new(
                        size,
                        &ctx.theme.typography.font_family,
                        palette.slot_active_text,
                    ),
                );
            }

            x += slot_w + geom.gap;
        }
    }
}

fn listener_loop(state: Arc<Mutex<WorkspaceData>>, redraw: Sender<()>) {
    let mut retries = 0;
    loop {
        match refresh(&state) {
            Ok(()) => {
                let _ = redraw.send(());
                log::info!("hyprland workspaces: fetch inicial OK");
                break;
            }
            Err(e) => {
                retries += 1;
                if retries > 5 {
                    log::error!("hyprland fetch falló 5 veces: {e}");
                    return;
                }
                log::warn!("hyprland fetch (retry {retries}/5): {e}");
                thread::sleep(Duration::from_millis(500));
            }
        }
    }

    let stream = match hyprland_ipc::event_stream() {
        Ok(s) => s,
        Err(e) => {
            log::error!("hyprland event_stream: {e}");
            return;
        }
    };

    for event in stream {
        match event {
            Ok(ev) => {
                log::debug!("hyprland event: {}>>{}", ev.name, ev.data);

                match ev.name.as_str() {
                    "workspace" | "createworkspace" | "destroyworkspace" | "focusedmon" => {
                        if let Err(e) = refresh(&state) {
                            log::warn!("refresh tras evento {}: {e}", ev.name);
                        } else {
                            let _ = redraw.send(());
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => log::warn!("event parse: {e}"),
        }
    }
    log::warn!("hyprland event stream terminó (socket cerrado)");
}

fn refresh(state: &Arc<Mutex<WorkspaceData>>) -> anyhow::Result<()> {
    let workspaces_json = hyprland_ipc::query("j/workspaces")?;
    let active_json = hyprland_ipc::query("j/activeworkspace")?;

    let workspaces: Value = serde_json::from_str(&workspaces_json)?;
    let active: Value = serde_json::from_str(&active_json)?;

    let mut existing: Vec<i32> = workspaces
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("workspaces no es array"))?
        .iter()
        .filter_map(|w| {
            let id = w.get("id")?.as_i64()? as i32;
            if id < 0 {
                return None;
            }
            Some(id)
        })
        .collect();
    existing.sort_unstable();

    let active_id = active
        .get("id")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| anyhow::anyhow!("activeworkspace sin id"))? as i32;

    *state.lock().unwrap() = WorkspaceData {
        existing,
        active_id,
    };
    Ok(())
}
