// ─── < Imports > ────────────────────────────────────────────────────

use smithay_client_toolkit::seat::pointer::CursorIcon;
use wayland_client::Connection;

use super::state::AppState;
use crate::bar::profile::SessionAction;
use crate::bar::system::PowerAction;
use crate::components::{Interaction, Point};
use crate::hyprland_ipc;

// ─── < Implementations > ────────────────────────────────────────────────────

impl AppState {
    pub fn handle_pointer_enter_or_motion(&mut self, conn: &Connection, point: Point, force_cursor_reload: bool) {
        self.pointer.position = Some(point);

        if let Some(drag) = self.pointer.dragging {
            if self.bar.handle_drag(drag, point, &self.theme, self.open_dropdown) {
                self.needs_redraw = true;
            }

            return;
        }

        let hovered_interaction = self.bar.hit_test(point, &self.theme, self.open_dropdown);

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

        if let Some(drag) = self.pointer.dragging.take() {
            self.bar.end_drag(drag, self.open_dropdown);
            self.needs_redraw = true;
        }

        if self.pointer.hovered_interaction.take().is_some() {
            self.needs_redraw = true;
        }

        self.bar.reset_scroll();

        self.set_cursor_icon(conn, CursorIcon::Default, true);
    }

    pub fn handle_pointer_press(&mut self) {
        let Some(point) = self.pointer.position else {
            return;
        };

        let Some(interaction) = self.bar.hit_test(point, &self.theme, self.open_dropdown) else {
            if !self.bar.dropdown_contains_point(point, &self.theme, self.open_dropdown) {
                self.close_dropdown();
            }

            return;
        };

        match interaction {
            Interaction::Workspace(workspace_id) => {
                self.close_dropdown();
                activate_workspace(workspace_id);
            }
            Interaction::Dropdown(dropdown_id) => {
                self.toggle_dropdown(dropdown_id);
            }
            Interaction::Power(action) => {
                self.close_dropdown();
                run_power_action(action);
            }
            Interaction::Session(action) => {
                self.close_dropdown();
                run_session_action(action);
            }
            Interaction::Calendar(_) => {
                if self.bar.handle_interaction(interaction) {
                    self.needs_redraw = true;
                }
            }
            Interaction::Command(action) => {
                if action.is_slider() {
                    self.pointer.dragging = Some(interaction);

                    if self.bar.handle_drag(interaction, point, &self.theme, self.open_dropdown) {
                        self.needs_redraw = true;
                    }
                } else if self.bar.handle_interaction(interaction) {
                    self.needs_redraw = true;
                }
            }
        }
    }

    pub fn handle_pointer_release(&mut self) {
        if let Some(drag) = self.pointer.dragging.take() {
            self.bar.end_drag(drag, self.open_dropdown);
            self.needs_redraw = true;
        }
    }

    pub fn handle_pointer_scroll(&mut self, delta: f64) {
        let Some(point) = self.pointer.position else {
            return;
        };

        if self.pointer.dragging.is_some() {
            return;
        }

        if self.bar.handle_scroll(point, delta) {
            self.needs_redraw = true;
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

fn activate_workspace(workspace_id: crate::bar::workspaces::WorkspaceId) {
    match hyprland_ipc::dispatch_workspace(workspace_id) {
        Ok(()) => {
            log::info!("workspace {workspace_id} dispatch sent");
        }
        Err(error) => {
            log::warn!("workspace dispatch failed for {workspace_id}: {error}");
        }
    }
}

fn run_power_action(action: PowerAction) {
    match action.execute() {
        Ok(()) => {
            log::info!("power action {action:?} launched");
        }
        Err(error) => {
            log::warn!("power action {action:?} failed: {error}");
        }
    }
}

fn run_session_action(action: SessionAction) {
    match action.execute() {
        Ok(()) => {
            log::info!("session action {action:?} launched");
        }
        Err(error) => {
            log::warn!("session action {action:?} failed: {error}");
        }
    }
}
