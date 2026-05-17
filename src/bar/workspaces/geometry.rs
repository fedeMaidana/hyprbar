// ─── < Imports > ────────────────────────────────────────────────────

use crate::theme::Theme;

use super::state::WorkspaceData;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct SlotGeometry {
    pub active_width: f32,
    pub active_height: f32,
    pub active_radius: f32,
    pub inactive_width: f32,
    pub inactive_height: f32,
    pub inactive_radius: f32,
    pub slot_box_height: f32,
    pub gap: f32,
    pub h_padding: f32,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl SlotGeometry {
    pub fn from_theme(theme: &Theme) -> Self {
        let tokens = theme.tokens;

        let v_padding = tokens.pill_padding_y;
        let cell_height = tokens.pill_height - v_padding * 2.0;

        let active_height = cell_height;
        let active_width = active_height * tokens.workspace_active_width_scale;
        let active_radius = active_height * tokens.workspace_active_radius_scale;

        let inactive_height = active_height * tokens.workspace_inactive_height_scale;
        let inactive_width = inactive_height * tokens.workspace_inactive_width_scale;
        let inactive_radius = inactive_height * tokens.workspace_inactive_radius_scale;

        Self {
            active_width,
            active_height,
            active_radius,
            inactive_width,
            inactive_height,
            inactive_radius,
            slot_box_height: cell_height,
            gap: tokens.workspace_slot_gap,
            h_padding: tokens.pill_padding_x,
        }
    }

    pub fn pill_width(&self, data: &WorkspaceData) -> f32 {
        let count = data.visible_count();

        let active_count = if data.active_id >= 1 && data.active_id <= count { 1 } else { 0 };

        let inactive_count = count - active_count;
        let total_slots = active_count + inactive_count;

        let inner_width = self.active_width * active_count as f32
            + self.inactive_width * inactive_count as f32
            + self.gap * (total_slots - 1).max(0) as f32;

        inner_width + self.h_padding * 2.0
    }
}
