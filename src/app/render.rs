// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;
use smithay_client_toolkit::shell::WaylandSurface;
use vello::Scene;
use vello::kurbo::Affine;

use crate::app::wayland_state::InputRegionRect;
use crate::components::RenderCtx;
use crate::render::Rect;

use super::state::AppState;

// ─── < Implementations > ────────────────────────────────────────────────────

impl AppState {
    pub(crate) fn render(&mut self) -> Result<()> {
        if self.surface.pending_resize {
            self.render_ctx
                .resize(self.surface.physical_width(), self.surface.physical_height());
            self.surface.pending_resize = false;
        }

        self.apply_current_input_region();
        self.apply_buffer_scale();

        self.theme.refresh_dynamic_colors();

        let surface_rect = Rect::new(0.0, 0.0, self.surface.width as f32, self.surface.height as f32);

        let mut ctx = RenderCtx {
            theme: &self.theme,
            text: &mut self.text_engine,
            hovered_interaction: self.pointer.hovered_interaction,
            open_dropdown: self.open_dropdown,
        };

        let scale = self.surface.scale.max(1);

        if scale == 1 {
            self.bar.render(&mut self.render_ctx.scene, surface_rect, &self.theme, &mut ctx);
        } else {
            let mut logical_scene = Scene::new();

            self.bar.render(&mut logical_scene, surface_rect, &self.theme, &mut ctx);

            self.render_ctx.scene.append(&logical_scene, Some(Affine::scale(scale as f64)));
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
        if self.surface.applied_buffer_scale == self.surface.scale {
            return;
        }

        self.layer_surface().wl_surface().set_buffer_scale(self.surface.scale);
        self.surface.applied_buffer_scale = self.surface.scale;
    }
}
