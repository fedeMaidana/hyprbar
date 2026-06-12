// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Context, Result, anyhow, bail};

use super::state::{AudioState, MediaState};

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

pub fn parse_brightnessctl_machine(line: &str) -> Result<f32> {
    let fields: Vec<&str> = line.trim().split(',').collect();

    if fields.len() < 5 {
        bail!("brightnessctl output too short: {line}");
    }

    let current: f32 = fields[2].parse().context("invalid brightnessctl current value")?;
    let max: f32 = fields[4].parse().context("invalid brightnessctl max value")?;

    if max <= 0.0 {
        bail!("brightnessctl max value is not positive");
    }

    Ok((current / max).clamp(0.0, 1.0))
}

pub fn parse_playerctl_metadata(output: &str) -> Option<MediaState> {
    let line = output.lines().next()?.trim_end();

    let mut parts = line.splitn(3, '\t');

    let status = parts.next()?;
    let title = parts.next().unwrap_or("").to_string();
    let artist = parts.next().unwrap_or("").to_string();

    Some(MediaState {
        playing: status.eq_ignore_ascii_case("playing"),
        title,
        artist,
    })
}
