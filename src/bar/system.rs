// ─── < Modules > ────────────────────────────────────────────────────

mod action;
mod battery;
mod charts;
mod metrics;
mod net;
mod panel;
mod pill;
mod power;
mod sampler;
mod state;
mod updates;
mod view_network;
mod view_power;
mod view_system;
mod view_updates;
mod worker;

// ─── < Public API > ────────────────────────────────────────────────────

pub use panel::SystemPanel;
pub use pill::ArchLogoPill;
pub use power::PowerAction;
pub use state::{MetricsSnapshot, SystemData};
pub use updates::launch_update;

// ─── < Tests > ────────────────────────────────────────────────────

#[doc(hidden)]
pub use metrics::{
    CpuTimes, MemoryInfo, cpu_usage_percent, parse_cpu_times, parse_df_bytes, parse_load_average, parse_memory_info, parse_swap_used_kb,
    parse_temperature_millidegrees, parse_uptime_seconds, short_kernel_version,
};

#[doc(hidden)]
pub use net::{
    LinkStatus, band_label, channel_for_freq, parse_active_security, parse_default_route, parse_interface_bytes, parse_ipv4_with_prefix,
    parse_iw_link, parse_nameservers, parse_ping_ms,
};

#[doc(hidden)]
pub use state::PendingPackage;

#[doc(hidden)]
pub use updates::{count_update_lines, parse_mirror_host, parse_pending_packages};
