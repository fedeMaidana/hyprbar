// ─── < Modules > ────────────────────────────────────────────────────

pub mod clock_pill;
pub mod command_center_pill;
pub mod date;
pub mod factory;
pub mod layout;
pub mod notifications_pill;
pub mod profile;
pub mod system;
pub mod weather;
pub mod workspaces;

// ─── < Public API > ────────────────────────────────────────────────────

pub use factory::default_bar;
pub use layout::Bar;
