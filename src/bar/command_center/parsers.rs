// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Context, Result, anyhow, bail};

use super::state::AudioState;

// ─── < Constants > ────────────────────────────────────────────────────

const MUTED_SUFFIX: &str = "[MUTED]";
const MAX_VOLUME: f32 = 1.5;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn parse_wpctl_volume(output: &str) -> Result<AudioState> {
    let output = output.trim();

    let rest = output
        .strip_prefix("Volume:")
        .ok_or_else(|| anyhow!("unexpected wpctl output: {output}"))?
        .trim();

    let muted = rest.ends_with(MUTED_SUFFIX);

    let value = rest
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("wpctl output missing volume value"))?;

    let volume: f32 = value.parse().context("invalid wpctl volume value")?;

    Ok(AudioState {
        volume: volume.clamp(0.0, MAX_VOLUME),
        muted,
    })
}

pub fn parse_nmcli_radio(output: &str) -> Result<bool> {
    match output.trim() {
        "enabled" => Ok(true),
        "disabled" => Ok(false),
        other => bail!("unexpected nmcli radio output: {other}"),
    }
}

pub fn parse_active_ssid(output: &str) -> Option<String> {
    output
        .lines()
        .filter_map(|line| line.strip_prefix("yes:"))
        .map(str::trim)
        .find(|ssid| !ssid.is_empty())
        .map(str::to_string)
}
