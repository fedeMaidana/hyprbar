// ─── < Imports > ────────────────────────────────────────────────────

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow};

use super::metrics::{
    CpuTimes, MemoryInfo, parse_cpu_times, parse_df_bytes, parse_load_average, parse_memory_info, parse_swap_used_kb,
    parse_temperature_millidegrees, parse_uptime_seconds,
};
use super::state::DiskUsage;

// ─── < Constants > ────────────────────────────────────────────────────

const CPU_SENSOR_NAMES: [&str; 4] = ["k10temp", "zenpower", "coretemp", "cpu_thermal"];
const HWMON_DIR: &str = "/sys/class/hwmon";
const THERMAL_ZONE_FALLBACK: &str = "/sys/class/thermal/thermal_zone0/temp";

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn read_cpu_times() -> Result<CpuTimes> {
    let stat = fs::read_to_string("/proc/stat").context("leyendo /proc/stat")?;

    parse_cpu_times(&stat)
}

pub fn read_memory_info() -> Result<MemoryInfo> {
    let meminfo = fs::read_to_string("/proc/meminfo").context("leyendo /proc/meminfo")?;

    parse_memory_info(&meminfo)
}

pub fn read_uptime_seconds() -> Result<u64> {
    let uptime = fs::read_to_string("/proc/uptime").context("leyendo /proc/uptime")?;

    parse_uptime_seconds(&uptime)
}

pub fn read_kernel_version() -> Result<String> {
    let release = fs::read_to_string("/proc/sys/kernel/osrelease").context("leyendo /proc/sys/kernel/osrelease")?;

    Ok(release.trim().to_string())
}

pub fn read_cpu_temperature() -> Option<f32> {
    hwmon_cpu_temperature().or_else(thermal_zone_temperature)
}

pub fn read_load_average() -> Result<f32> {
    let content = fs::read_to_string("/proc/loadavg").context("leyendo /proc/loadavg")?;

    parse_load_average(&content)
}

pub fn read_swap_used_kb() -> Result<u64> {
    let meminfo = fs::read_to_string("/proc/meminfo").context("leyendo /proc/meminfo")?;

    parse_swap_used_kb(&meminfo)
}

/// Primer sensor de ventilador que aparezca en hwmon.
pub fn find_fan_input() -> Option<PathBuf> {
    let entries = fs::read_dir(HWMON_DIR).ok()?;

    for entry in entries.flatten() {
        let path = entry.path().join("fan1_input");

        if path.exists() {
            return Some(path);
        }
    }

    None
}

pub fn read_fan_rpm(path: &Path) -> Option<u32> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// Uso del filesystem raíz vía `df` (no hay statvfs en std).
pub fn read_disk_usage() -> Result<DiskUsage> {
    let output = Command::new("df")
        .args(["-B1", "--output=used,size", "/"])
        .output()
        .context("ejecutando df")?;

    if !output.status.success() {
        return Err(anyhow!("df terminó con {:?}", output.status.code()));
    }

    let (used_bytes, total_bytes) = parse_df_bytes(&String::from_utf8_lossy(&output.stdout))?;

    Ok(DiskUsage { used_bytes, total_bytes })
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn hwmon_cpu_temperature() -> Option<f32> {
    let entries = fs::read_dir(HWMON_DIR).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        let Ok(name) = fs::read_to_string(path.join("name")) else {
            continue;
        };

        if !CPU_SENSOR_NAMES.contains(&name.trim()) {
            continue;
        }

        if let Some(temperature) = read_temp_input(&path.join("temp1_input")) {
            return Some(temperature);
        }
    }

    None
}

fn thermal_zone_temperature() -> Option<f32> {
    read_temp_input(Path::new(THERMAL_ZONE_FALLBACK))
}

fn read_temp_input(path: &Path) -> Option<f32> {
    let content = fs::read_to_string(path).ok()?;

    parse_temperature_millidegrees(&content).ok()
}
