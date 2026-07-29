// ─── < Imports > ────────────────────────────────────────────────────

use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::proc::spawn_detached;

use super::parsers::{parse_active_ssid, parse_nmcli_radio, parse_wpctl_volume};
use super::state::{AudioState, WifiState};

// ─── < Constants > ────────────────────────────────────────────────────

const SINK: &str = "@DEFAULT_AUDIO_SINK@";
const SOURCE: &str = "@DEFAULT_AUDIO_SOURCE@";

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn read_sink() -> Result<AudioState> {
    let output = run_capture("wpctl", &["get-volume", SINK])?;

    parse_wpctl_volume(&output)
}

pub fn read_mic_muted() -> Result<bool> {
    let output = run_capture("wpctl", &["get-volume", SOURCE])?;

    Ok(parse_wpctl_volume(&output)?.muted)
}

pub fn read_wifi() -> Option<WifiState> {
    let radio = run_capture("nmcli", &["radio", "wifi"]).ok()?;
    let enabled = parse_nmcli_radio(&radio).ok()?;

    let ssid = if enabled {
        run_capture("nmcli", &["-t", "-f", "ACTIVE,SSID", "dev", "wifi", "list", "--rescan", "no"])
            .ok()
            .and_then(|output| parse_active_ssid(&output))
    } else {
        None
    };

    Some(WifiState { enabled, ssid })
}

pub fn set_sink_volume(fraction: f32) -> Result<()> {
    let percent = format!("{}%", (fraction.clamp(0.0, 1.0) * 100.0).round() as u32);

    spawn_detached("wpctl", &["set-volume", "-l", "1", SINK, &percent])
}

pub fn toggle_sink_mute() -> Result<()> {
    spawn_detached("wpctl", &["set-mute", SINK, "toggle"])
}

pub fn toggle_mic_mute() -> Result<()> {
    spawn_detached("wpctl", &["set-mute", SOURCE, "toggle"])
}

pub fn set_wifi_enabled(enabled: bool) -> Result<()> {
    spawn_detached("nmcli", &["radio", "wifi", if enabled { "on" } else { "off" }])
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn run_capture(program: &str, arguments: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(arguments)
        .env("LC_ALL", "C")
        .output()
        .with_context(|| format!("ejecutando {program}"))?;

    if !output.status.success() {
        bail!("{program} terminó con código {:?}: {}", output.status.code(), String::from_utf8_lossy(&output.stderr).trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
