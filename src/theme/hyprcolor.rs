use std::{env, fs, path::PathBuf};

use serde_json::Value;
use vello::peniko::Color;

#[derive(Debug, Clone, Copy)]
pub struct HyprcolorPalette {
    pub accent: Color,
    pub foreground: Color,
}

pub fn load() -> Option<HyprcolorPalette> {
    let path = colors_json_path()?;
    let content = fs::read_to_string(path).ok()?;
    let json: Value = serde_json::from_str(&content).ok()?;

    Some(HyprcolorPalette {
        accent: parse_json_color(&json, "accent")?,
        foreground: parse_json_color(&json, "foreground")?,
    })
}

fn colors_json_path() -> Option<PathBuf> {
    if let Some(cache_home) = env::var_os("XDG_CACHE_HOME") {
        return Some(PathBuf::from(cache_home).join("hyprcolors/colors.json"));
    }

    env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache").join("hyprcolors").join("colors.json"))
}

fn parse_json_color(json: &Value, key: &str) -> Option<Color> {
    let value = json.get(key)?.as_str()?;
    parse_hex_color(value)
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
