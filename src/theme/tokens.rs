// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Tokens {
    pub bar_height: f32,
    pub bar_margin_top: f32,
    pub bar_margin_x: f32,

    pub pill_height: f32,
    pub pill_gap: f32,
    pub pill_padding_x: f32,
    pub pill_padding_y: f32,
    pub pill_radius: f32,

    pub dropdown_width: f32,
    pub dropdown_height: f32,
    pub dropdown_margin_top: f32,
    pub dropdown_margin_bottom: f32,
    pub dropdown_padding_x: f32,
    pub dropdown_padding_y: f32,
    pub dropdown_item_height: f32,
    pub dropdown_item_gap: f32,
    pub dropdown_radius: f32,

    pub icon_scale: f32,

    pub weather_inner_gap: f32,

    pub workspace_slot_gap: f32,
    pub workspace_active_width_scale: f32,
    pub workspace_active_radius_scale: f32,
    pub workspace_inactive_width_scale: f32,
    pub workspace_inactive_height_scale: f32,
    pub workspace_inactive_radius_scale: f32,

    pub avatar_border_width: f32,
    pub avatar_outer_radius_offset: f32,

    pub notification_dot_radius: f32,
    pub notification_dot_x_overlap_scale: f32,
    pub notification_dot_y_icon_scale: f32,

    pub shadow_offset_y: f32,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Default for Tokens {
    fn default() -> Self {
        Self {
            bar_height: 40.0,
            bar_margin_top: 5.0,
            bar_margin_x: 12.0,

            pill_height: 29.0,
            pill_gap: 7.0,
            pill_padding_x: 11.0,
            pill_padding_y: 6.0,
            pill_radius: 13.0,

            dropdown_width: 220.0,
            dropdown_height: 150.0,
            dropdown_margin_top: 8.0,
            dropdown_margin_bottom: 8.0,
            dropdown_padding_x: 12.0,
            dropdown_padding_y: 10.0,
            dropdown_item_height: 34.0,
            dropdown_item_gap: 4.0,
            dropdown_radius: 14.0,

            icon_scale: 1.2,

            weather_inner_gap: 5.0,

            workspace_slot_gap: 7.0,
            workspace_active_width_scale: 1.7,
            workspace_active_radius_scale: 0.5,
            workspace_inactive_width_scale: 1.1,
            workspace_inactive_height_scale: 1.0,
            workspace_inactive_radius_scale: 0.35,

            avatar_border_width: 2.5,
            avatar_outer_radius_offset: 0.5,

            notification_dot_radius: 3.5,
            notification_dot_x_overlap_scale: 0.5,
            notification_dot_y_icon_scale: 0.35,

            shadow_offset_y: 1.0,
        }
    }
}
