// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;
use smithay_client_toolkit::shell::WaylandSurface;

use crate::app::wayland_state::InputRegionRect;
use crate::components::RenderCtx;
use crate::render::Rect;

use super::state::AppState;

// ─── < Implementations > ────────────────────────────────────────────────────

impl AppState {
    pub(crate) fn render(&mut self) -> Result<()> {
        if self.surface.pending_resize {
            self.render_ctx.resize(self.surface.width, self.surface.height);
            self.surface.pending_resize = false;
        }

        self.apply_current_input_region();

        self.theme.refresh_dynamic_colors();

        let surface_rect = Rect::new(0.0, 0.0, self.surface.width as f32, self.surface.height as f32);

        let mut ctx = RenderCtx {
            theme: &self.theme,
            text: &mut self.text_engine,
            hovered_interaction: self.pointer.hovered_interaction,
            open_dropdown: self.open_dropdown,
        };

        self.bar.render(&mut self.render_ctx.scene, surface_rect, &self.theme, &mut ctx);

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
}
