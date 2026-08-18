// ─── < Imports > ────────────────────────────────────────────────────

use std::time::Instant;

use anyhow::Result;
use smithay_client_toolkit::shell::WaylandSurface;
use vello::Scene;
use vello::kurbo::Affine;

use crate::app::wayland_state::InputRegionRect;
use crate::components::RenderCtx;
use crate::render::Rect;

use super::state::AppState;
use super::surface_handle::SurfaceHandle;

// ─── < Constants > ────────────────────────────────────────────────────

/// dt máximo que ven las animaciones (evita saltos tras una pausa larga).
const MAX_FRAME_DT: f32 = 0.05;

/// dt asumido para el primer frame.
const FALLBACK_FRAME_DT: f32 = 1.0 / 60.0;

// ─── < Implementations > ────────────────────────────────────────────────────

impl AppState {
    pub(crate) fn render(&mut self) -> Result<()> {
        if !self.surface.configured || self.surface.lost {
            return Ok(());
        }

        // After recreating the layer surface, the wgpu surface still
        // points at the dead wl_surface: rebuild it first.
        if self.render_surface_stale {
            let wl_surface = self.layer_surface().wl_surface().clone();
            let handle = SurfaceHandle::new(&self.conn, &wl_surface);

            self.render_ctx
                .create_surface(handle, self.surface.physical_width(), self.surface.physical_height())?;

            self.render_surface_stale = false;
            self.surface.pending_resize = false;
        }

        if self.surface.pending_resize {
            self.render_ctx
                .resize(self.surface.physical_width(), self.surface.physical_height());
            self.surface.pending_resize = false;
        }

        self.apply_current_input_region();
        self.apply_buffer_scale();

        let surface_rect = Rect::new(0.0, 0.0, self.surface.width as f32, self.surface.height as f32);

        let now = Instant::now();
        let dt = self
            .last_render
            .map(|previous| now.duration_since(previous).as_secs_f32().min(MAX_FRAME_DT))
            .unwrap_or(FALLBACK_FRAME_DT);
        self.last_render = Some(now);

        let mut ctx = RenderCtx {
            theme: &self.theme,
            text: &mut self.text_engine,
            hovered_interaction: self.pointer.hovered_interaction,
            open_dropdown: self.open_dropdown,
            dt,
            animating: false,
        };

        let scale = self.surface.effective_scale();

        if (scale - 1.0).abs() < f64::EPSILON {
            self.bar.render(&mut self.render_ctx.scene, surface_rect, &self.theme, &mut ctx);
        } else {
            let mut logical_scene = Scene::new();

            self.bar.render(&mut logical_scene, surface_rect, &self.theme, &mut ctx);

            self.render_ctx.scene.append(&logical_scene, Some(Affine::scale(scale)));
        }

        self.animating = ctx.animating;

        if let Some(viewport) = &self.surface.viewport {
            viewport.set_destination(self.surface.width.max(1) as i32, self.surface.height.max(1) as i32);
        }

        self.render_ctx.render()?;

        Ok(())
    }

    fn apply_current_input_region(&self) {
        let surface = self.layer_surface().wl_surface();

        let rects = if self.open_dropdown.is_some() {
            vec![InputRegionRect::new(0, 0, self.surface.width as i32, self.surface.height as i32)]
        } else {
            vec![InputRegionRect::new(
                0,
                0,
                self.surface.width as i32,
                self.theme.tokens.bar_height.ceil() as i32,
            )]
        };

        self.wayland.apply_input_region(surface, &rects);
    }

    fn apply_buffer_scale(&mut self) {
        // With a viewport the buffer stays at scale 1 and the compositor maps physical -> logical.
        if self.surface.fractional.is_some() {
            return;
        }

        if self.surface.applied_buffer_scale == self.surface.scale {
            return;
        }

        self.layer_surface().wl_surface().set_buffer_scale(self.surface.scale);
        self.surface.applied_buffer_scale = self.surface.scale;
    }
}
