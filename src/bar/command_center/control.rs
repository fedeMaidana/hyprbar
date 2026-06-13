// ─── < Imports > ────────────────────────────────────────────────────

use std::fs;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::proc::spawn_detached;

use super::parsers::{
    parse_active_ssid, parse_bluetoothctl_powered, parse_brightnessctl_machine, parse_nmcli_radio, parse_playerctl_metadata,
    parse_wpctl_volume,
};
use super::state::{AudioState, BluetoothState, BrightnessState, MediaState, WifiState};

// ─── < Constants > ────────────────────────────────────────────────────

const SINK: &str = "@DEFAULT_AUDIO_SINK@";
const SOURCE: &str = "@DEFAULT_AUDIO_SOURCE@";
const PLAYERCTL_FORMAT: &str = "{{status}}\t{{title}}\t{{artist}}";
const MIN_BRIGHTNESS_PERCENT: u32 = 1;

const BLUETOOTH_SYSFS_DIR: &str = "/sys/class/bluetooth";
const BLUETOOTHCTL_TIMEOUT_SECS: &str = "1";

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
    let output = run_capture("brightnessctl", &["-c", "backlight", "-m"])?;

    Ok(BrightnessState {
        fraction: parse_brightnessctl_machine(&output)?,
    })
}

pub fn read_media() -> Option<MediaState> {
    let output = Command::new("playerctl")
        .args(["metadata", "--format", PLAYERCTL_FORMAT])
        .env("LC_ALL", "C")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    parse_playerctl_metadata(&String::from_utf8_lossy(&output.stdout))
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

pub fn read_bluetooth() -> Option<BluetoothState> {
    if !has_bluetooth_adapter() {
        return None;
    }

    let output = Command::new("timeout")
        .args([BLUETOOTHCTL_TIMEOUT_SECS, "bluetoothctl", "show"])
        .env("LC_ALL", "C")
        .output()
        .ok()?;

    let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));

    parse_bluetoothctl_powered(&text).map(|powered| BluetoothState { powered })
}

pub fn set_sink_volume(fraction: f32) -> Result<()> {
    let percent = format!("{}%", (fraction.clamp(0.0, 1.0) * 100.0).round() as u32);

    spawn_detached("wpctl", &["set-volume", "-l", "1", SINK, &percent])
}

pub fn set_brightness(fraction: f32) -> Result<()> {
    let percent = ((fraction.clamp(0.0, 1.0) * 100.0).round() as u32).max(MIN_BRIGHTNESS_PERCENT);

    spawn_detached("brightnessctl", &["-c", "backlight", "s", &format!("{percent}%")])
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

pub fn set_bluetooth_powered(powered: bool) -> Result<()> {
    spawn_detached("bluetoothctl", &["power", if powered { "on" } else { "off" }])
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

fn has_bluetooth_adapter() -> bool {
    fs::read_dir(BLUETOOTH_SYSFS_DIR)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}

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
