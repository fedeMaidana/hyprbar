// ─── < Modules > ────────────────────────────────────────────────────

mod geometry;
mod listener;
mod mapper;
mod pill;
mod state;

// ─── < Public API > ────────────────────────────────────────────────────

pub use pill::WorkspacesPill;

// ─── < Tests > ────────────────────────────────────────────────────

#[doc(hidden)]
pub use mapper::parse_workspace_data;

#[doc(hidden)]
pub use state::{WorkspaceData, WorkspaceId};
