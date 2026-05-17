// ─── < Imports > ────────────────────────────────────────────────────

use smithay_client_toolkit::seat::pointer::CursorIcon;
use wayland_client::Connection;

use super::state::AppState;
use crate::components::{Interaction, Point};
use crate::hyprland_ipc;

// ─── < Implementations > ────────────────────────────────────────────────────

impl AppState {
    pub fn handle_pointer_enter_or_motion(&mut self, conn: &Connection, point: Point, force_cursor_reload: bool) {
        self.pointer_position = Some(point);

        let cursor_icon = if self.bar.hit_test(point, &self.theme).is_some() {
            CursorIcon::Pointer
        } else {
            CursorIcon::Default
        };

        self.set_cursor_icon(conn, cursor_icon, force_cursor_reload);
    }

    pub fn handle_pointer_leave(&mut self, conn: &Connection) {
        self.pointer_position = None;
        self.set_cursor_icon(conn, CursorIcon::Default, true);
    }

    pub fn handle_pointer_press(&mut self) {
        let Some(point) = self.pointer_position else {
            return;
        };

        let Some(interaction) = self.bar.hit_test(point, &self.theme) else {
            return;
        };

        match interaction {
            Interaction::Workspace(workspace_id) => activate_workspace(workspace_id),
        }
    }

    fn set_cursor_icon(&mut self, conn: &Connection, icon: CursorIcon, force_reload: bool) {
        if !force_reload && self.cursor_icon == icon {
            return;
        }

        let Some(pointer) = self.themed_pointer.as_mut() else {
            return;
        };

        if let Err(error) = pointer.set_cursor(conn, icon) {
            log::warn!("set cursor failed: {error}");
            return;
        }

        self.cursor_icon = icon;
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn activate_workspace(workspace_id: i32) {
    let command = format!(r#"/dispatch hl.dsp.focus({{ workspace = "{workspace_id}" }})"#);

    match hyprland_ipc::query(&command) {
        Ok(response) => {
            let response = response.trim();

            if response.is_empty() {
                log::info!("workspace {workspace_id} dispatch sent");
            } else {
                log::info!("workspace {workspace_id} dispatch response: {response}");
            }
        }
        Err(error) => {
            log::warn!("workspace dispatch failed for {workspace_id}: {error}");
        }
    }
}
