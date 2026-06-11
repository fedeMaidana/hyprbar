// ─── < Modules > ────────────────────────────────────────────────────

mod metrics;
mod panel;
mod pill;
mod power;
mod sampler;
mod state;
mod updates;
mod worker;

// ─── < Public API > ────────────────────────────────────────────────────

pub use panel::SystemPanel;
pub use pill::ArchLogoPill;
pub use power::PowerAction;
pub use state::{MetricsSnapshot, SystemData};

// ─── < Tests > ────────────────────────────────────────────────────

#[doc(hidden)]
pub use metrics::{
    CpuTimes, MemoryInfo, cpu_usage_percent, parse_cpu_times, parse_memory_info, parse_temperature_millidegrees, parse_uptime_seconds,
    short_kernel_version,
};

#[doc(hidden)]
pub use updates::count_update_lines;
