// ─── < Modules > ────────────────────────────────────────────────────

mod panel;
mod pill;
mod zones;

// ─── < Public API > ────────────────────────────────────────────────────

pub use panel::ClockPanel;
pub use pill::ClockPill;

// ─── < Tests > ────────────────────────────────────────────────────

#[doc(hidden)]
pub use zones::{WORLD_ZONES, day_tag, is_daytime, offset_label, utc_label, zone_display_name, zone_offset_minutes};
