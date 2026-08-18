// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Context, Result, anyhow, bail};

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuTimes {
    pub idle: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryInfo {
    pub total_kb: u64,
    pub available_kb: u64,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl MemoryInfo {
    pub fn used_kb(self) -> u64 {
        self.total_kb.saturating_sub(self.available_kb)
    }

    pub fn used_fraction(self) -> f32 {
        if self.total_kb == 0 {
            return 0.0;
        }

        self.used_kb() as f32 / self.total_kb as f32
    }
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn parse_cpu_times(stat: &str) -> Result<CpuTimes> {
    let line = stat
        .lines()
        .find(|line| line.starts_with("cpu "))
        .ok_or_else(|| anyhow!("falta la línea agregada de cpu en /proc/stat"))?;

    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .map(str::parse)
        .collect::<Result<_, _>>()
        .context("valor de tiempo de cpu inválido")?;

    if values.len() < 4 {
        bail!("línea agregada de cpu demasiado corta");
    }

    let idle = values[3] + values.get(4).copied().unwrap_or(0);
    let total: u64 = values.iter().take(8).sum();

    Ok(CpuTimes { idle, total })
}

pub fn cpu_usage_percent(previous: CpuTimes, current: CpuTimes) -> Option<f32> {
    let total_delta = current.total.checked_sub(previous.total)?;
    let idle_delta = current.idle.checked_sub(previous.idle)?;

    if total_delta == 0 {
        return None;
    }

    let busy = total_delta.saturating_sub(idle_delta);

    Some((busy as f32 / total_delta as f32) * 100.0)
}

pub fn parse_memory_info(meminfo: &str) -> Result<MemoryInfo> {
    let total_kb = parse_meminfo_field(meminfo, "MemTotal:")?;
    let available_kb = parse_meminfo_field(meminfo, "MemAvailable:")?;

    Ok(MemoryInfo { total_kb, available_kb })
}

pub fn parse_temperature_millidegrees(content: &str) -> Result<f32> {
    let millidegrees: i64 = content.trim().parse().context("valor de temperatura inválido")?;

    Ok(millidegrees as f32 / 1000.0)
}

pub fn parse_uptime_seconds(content: &str) -> Result<u64> {
    let seconds: f64 = content
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("contenido de uptime vacío"))?
        .parse()
        .context("valor de uptime inválido")?;

    Ok(seconds as u64)
}

pub fn short_kernel_version(release: &str) -> String {
    let release = release.trim();

    release.split('-').next().unwrap_or(release).to_string()
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn parse_meminfo_field(meminfo: &str, field: &str) -> Result<u64> {
    let line = meminfo
        .lines()
        .find(|line| line.starts_with(field))
        .ok_or_else(|| anyhow!("falta {field} en meminfo"))?;

    let value = line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow!("línea de {field} malformada"))?;

    value.parse().with_context(|| format!("valor de {field} inválido"))
}
