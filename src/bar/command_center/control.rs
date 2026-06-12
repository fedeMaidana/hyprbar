// ─── < Imports > ────────────────────────────────────────────────────

use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::proc::spawn_detached;

use super::parsers::{parse_brightnessctl_machine, parse_playerctl_metadata, parse_wpctl_volume};
use super::state::{AudioState, BrightnessState, MediaState};

// ─── < Constants > ────────────────────────────────────────────────────

const SINK: &str = "@DEFAULT_AUDIO_SINK@";
const SOURCE: &str = "@DEFAULT_AUDIO_SOURCE@";
const PLAYERCTL_FORMAT: &str = "{{status}}\t{{title}}\t{{artist}}";
const MIN_BRIGHTNESS_PERCENT: u32 = 1;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn read_sink() -> Result<AudioState> {
    let output = run_capture("wpctl", &["get-volume", SINK])?;

    parse_wpctl_volume(&output)
}

pub fn read_mic_muted() -> Result<bool> {
    let output = run_capture("wpctl", &["get-volume", SOURCE])?;

    Ok(parse_wpctl_volume(&output)?.muted)
}

pub fn read_brightness() -> Result<BrightnessState> {
    let output = run_capture("brightnessctl", &["-m"])?;

    Ok(BrightnessState {
        fraction: parse_brightnessctl_machine(&output)?,
    })
}

pub fn read_media() -> Option<MediaState> {
    let output = Command::new("playerctl")
        .args(["metadata", "--format", PLAYERCTL_FORMAT])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    parse_playerctl_metadata(&String::from_utf8_lossy(&output.stdout))
}

pub fn set_sink_volume(fraction: f32) -> Result<()> {
    let percent = format!("{}%", (fraction.clamp(0.0, 1.0) * 100.0).round() as u32);

    spawn_detached("wpctl", &["set-volume", "-l", "1", SINK, &percent])
}

pub fn set_brightness(fraction: f32) -> Result<()> {
    let percent = ((fraction.clamp(0.0, 1.0) * 100.0).round() as u32).max(MIN_BRIGHTNESS_PERCENT);

    spawn_detached("brightnessctl", &["s", &format!("{percent}%")])
}

pub fn toggle_sink_mute() -> Result<()> {
    spawn_detached("wpctl", &["set-mute", SINK, "toggle"])
}

pub fn toggle_mic_mute() -> Result<()> {
    spawn_detached("wpctl", &["set-mute", SOURCE, "toggle"])
}

pub fn media_play_pause() -> Result<()> {
    spawn_detached("playerctl", &["play-pause"])
}

pub fn media_previous() -> Result<()> {
    spawn_detached("playerctl", &["previous"])
}

pub fn media_next() -> Result<()> {
    spawn_detached("playerctl", &["next"])
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn run_capture(program: &str, arguments: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .with_context(|| format!("ejecutando {program}"))?;

    if !output.status.success() {
        bail!("{program} terminó con código {:?}: {}", output.status.code(), String::from_utf8_lossy(&output.stderr).trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
