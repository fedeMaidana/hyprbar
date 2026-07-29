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
pub use panel::{CommandPanel, PanelAvailability};
pub use pill::CommandCenterPill;

// ─── < Tests > ────────────────────────────────────────────────────

#[doc(hidden)]
pub use parsers::{parse_active_ssid, parse_nmcli_radio, parse_wpctl_volume};

#[doc(hidden)]
pub use state::{AudioState, CommandData, WifiState};
