// ─── < Imports > ────────────────────────────────────────────────────

use serde::Deserialize;
use std::{env, fs, path::PathBuf};
use vello::peniko::Color;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct HyprcolorPalette {
    pub accent: Color,
    pub foreground: Color,
}

#[derive(Debug, Deserialize)]
struct HyprcolorFile {
    accent: String,
    foreground: String,
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn load() -> Option<HyprcolorPalette> {
    let path = colors_json_path()?;
    let content = fs::read_to_string(path).ok()?;
    let colors: HyprcolorFile = serde_json::from_str(&content).ok()?;

    Some(HyprcolorPalette {
        accent: parse_hex_color(&colors.accent)?,
        foreground: parse_hex_color(&colors.foreground)?,
    })
}

/// Last modification time of the palette file, to detect live changes.
pub fn modified_time() -> Option<std::time::SystemTime> {
    fs::metadata(colors_json_path()?).ok()?.modified().ok()
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn colors_json_path() -> Option<PathBuf> {
    if let Some(cache_home) = env::var_os("XDG_CACHE_HOME") {
        return Some(PathBuf::from(cache_home).join("hyprcolors/colors.json"));
    }

    env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache").join("hyprcolors").join("colors.json"))
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let value = value.strip_prefix('#')?;

    if value.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&value[0..2], 16).ok()?;
    let g = u8::from_str_radix(&value[2..4], 16).ok()?;
    let b = u8::from_str_radix(&value[4..6], 16).ok()?;

    Some(Color::from_rgba8(r, g, b, 0xff))
}
