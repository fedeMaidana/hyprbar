// ─── < Modules > ────────────────────────────────────────────────────

mod action;
mod alarms;
mod panel;
mod pill;
mod state;
mod view_alarms;
mod view_clock;
mod view_stopwatch;
mod view_timer;
mod worker;
mod zones;

// ─── < Public API > ────────────────────────────────────────────────────

pub use panel::ClockPanel;
pub use pill::ClockPill;

// ─── < Tests > ────────────────────────────────────────────────────

#[doc(hidden)]
pub use state::{Alarm, AlarmEditor, Repeat, StopwatchData, TimerData};

#[doc(hidden)]
pub use view_stopwatch::format_stopwatch;

#[doc(hidden)]
pub use view_timer::format_timer;

#[doc(hidden)]
pub use zones::{WORLD_ZONES, is_daytime, offset_label, utc_label, zone_display_name, zone_offset_minutes};
