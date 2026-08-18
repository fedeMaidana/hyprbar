// ─── < Modules > ────────────────────────────────────────────────────

mod panel;
mod pill;
mod zones;

// ─── < Public API > ────────────────────────────────────────────────────

pub use panel::ClockPanel;
pub use pill::ClockPill;

pub(crate) use pill::CLOCK_DROPDOWN;

// ─── < Tests > ────────────────────────────────────────────────────

#[doc(hidden)]
pub use zones::{WORLD_ZONES, is_daytime, offset_label, utc_label, zone_display_name, zone_offset_minutes};
