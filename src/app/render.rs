// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;

use crate::components::RenderCtx;
use crate::render::Rect;

use super::state::AppState;

// ─── < Implementations > ────────────────────────────────────────────────────

impl AppState {
    pub(crate) fn render(&mut self) -> Result<()> {
        if self.pending_resize {
            self.render_ctx.resize(self.width, self.height);
            self.pending_resize = false;
        }

        self.theme.refresh_dynamic_colors();

        let surface_rect = Rect::new(0.0, 0.0, self.width as f32, self.height as f32);

        let mut ctx = RenderCtx {
            theme: &self.theme,
            text: &mut self.text_engine,
        };

        self.bar.render(&mut self.render_ctx.scene, surface_rect, &self.theme, &mut ctx);

        self.render_ctx.render()?;

        Ok(())
    }
}
