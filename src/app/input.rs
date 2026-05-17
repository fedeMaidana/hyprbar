// ─── < Imports > ────────────────────────────────────────────────────

use smithay_client_toolkit::seat::pointer::CursorIcon;
use wayland_client::Connection;

use super::state::AppState;
use crate::components::{Interaction, Point};
use crate::hyprland_ipc;

// ─── < Implementations > ────────────────────────────────────────────────────

impl AppState {
    pub fn handle_pointer_enter_or_motion(&mut self, conn: &Connection, point: Point, force_cursor_reload: bool) {
        self.pointer.position = Some(point);

        let hovered_interaction = self.bar.hit_test(point, &self.theme);

        if self.pointer.hovered_interaction != hovered_interaction {
            self.pointer.hovered_interaction = hovered_interaction;
            self.needs_redraw = true;
        }

        let cursor_icon = if hovered_interaction.is_some() {
            CursorIcon::Pointer
        } else {
            CursorIcon::Default
        };

        self.set_cursor_icon(conn, cursor_icon, force_cursor_reload);
    }

    pub fn handle_pointer_leave(&mut self, conn: &Connection) {
        self.pointer.position = None;

        if self.pointer.hovered_interaction.take().is_some() {
            self.needs_redraw = true;
        }

        self.set_cursor_icon(conn, CursorIcon::Default, true);
    }

    pub fn handle_pointer_press(&mut self) {
        let Some(point) = self.pointer.position else {
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
        if !force_reload && self.pointer.icon == icon {
            return;
        }

        let Some(pointer) = self.pointer.themed_pointer.as_mut() else {
            return;
        };

        if let Err(error) = pointer.set_cursor(conn, icon) {
            log::warn!("set cursor failed: {error}");
            return;
        }

        self.pointer.icon = icon;
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn activate_workspace(workspace_id: i32) {
    match hyprland_ipc::dispatch_workspace(workspace_id) {
        Ok(()) => {
            log::info!("workspace {workspace_id} dispatch sent");
        }
        Err(error) => {
            log::warn!("workspace dispatch failed for {workspace_id}: {error}");
        }
    }
}
