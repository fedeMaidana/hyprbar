// ─── < Modules > ────────────────────────────────────────────────────

mod action;
mod control;
mod panel;
mod parsers;
mod pill;
mod state;
mod worker;

// ─── < Public API > ────────────────────────────────────────────────────

pub use action::CommandAction;
pub use panel::CommandPanel;
pub use pill::CommandCenterPill;

// ─── < Tests > ────────────────────────────────────────────────────

#[doc(hidden)]
pub use parsers::{parse_brightnessctl_machine, parse_playerctl_metadata, parse_wpctl_volume};

#[doc(hidden)]
pub use state::{AudioState, BrightnessState, CommandData, MediaState};
