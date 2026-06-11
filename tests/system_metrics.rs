use hyprbar::bar::system::{
    CpuTimes, MemoryInfo, cpu_usage_percent, parse_cpu_times, parse_memory_info, parse_temperature_millidegrees, parse_uptime_seconds,
    short_kernel_version,
};

#[test]
fn parses_aggregate_cpu_times() {
    let stat = "cpu  100 0 50 800 50 0 0 0 0 0\ncpu0 50 0 25 400 25 0 0 0 0 0\n";

    let times = parse_cpu_times(stat).unwrap();

    assert_eq!(times, CpuTimes { idle: 850, total: 1000 });
}

#[test]
fn fails_when_aggregate_cpu_line_is_missing() {
    let stat = "cpu0 50 0 25 400 25 0 0 0 0 0\n";

    let error = parse_cpu_times(stat).unwrap_err();

    assert!(error.to_string().contains("missing aggregate cpu line"));
}

#[test]
fn computes_cpu_usage_percent_from_deltas() {
    let previous = CpuTimes { idle: 850, total: 1000 };
    let current = CpuTimes { idle: 900, total: 1100 };

    assert_eq!(cpu_usage_percent(previous, current), Some(50.0));
}

#[test]
fn returns_none_when_total_does_not_advance() {
    let times = CpuTimes { idle: 850, total: 1000 };

    assert_eq!(cpu_usage_percent(times, times), None);
}

#[test]
fn returns_none_when_counters_regress() {
    let previous = CpuTimes { idle: 900, total: 1100 };
    let current = CpuTimes { idle: 850, total: 1000 };

    assert_eq!(cpu_usage_percent(previous, current), None);
}

#[test]
fn parses_memory_info() {
    let meminfo = "MemTotal:       16384256 kB\nMemFree:         1024000 kB\nMemAvailable:    8192128 kB\n";

    let memory = parse_memory_info(meminfo).unwrap();

    assert_eq!(
        memory,
        MemoryInfo {
            total_kb: 16384256,
            available_kb: 8192128
        }
    );
    assert_eq!(memory.used_kb(), 8192128);
}

#[test]
fn fails_when_mem_available_is_missing() {
    let meminfo = "MemTotal:       16384256 kB\n";

    let error = parse_memory_info(meminfo).unwrap_err();

    assert!(error.to_string().contains("missing MemAvailable:"));
}

#[test]
fn parses_temperature_from_millidegrees() {
    let temperature = parse_temperature_millidegrees("54123\n").unwrap();

    assert!((temperature - 54.123).abs() < 0.001);
}

#[test]
fn parses_uptime_seconds() {
    assert_eq!(parse_uptime_seconds("12345.67 99999.00\n").unwrap(), 12345);
}

#[test]
fn shortens_arch_kernel_release() {
    assert_eq!(short_kernel_version("7.0.10-arch1-1\n"), "7.0.10");
}

#[test]
fn keeps_kernel_release_without_suffix() {
    assert_eq!(short_kernel_version("7.0.10"), "7.0.10");
}
