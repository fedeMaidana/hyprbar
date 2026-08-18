// ─── < Imports > ────────────────────────────────────────────────────

use serde::Deserialize;
use std::io::ErrorKind;
use std::{env, fs, path::PathBuf};
use vello::peniko::Color;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct HyprcolorPalette {
    pub accent: Color,
}

/// El contrato `colors.json` que escribe hyprcolors. Solo se lee lo que
/// la barra usa; cualquier otro campo del archivo se ignora.
#[derive(Debug, Deserialize)]
struct HyprcolorFile {
    #[serde(default)]
    accent: Option<String>,
}

// ─── < Public Functions > ────────────────────────────────────────────────────

/// Carga la paleta dinámica. Que el archivo no exista es normal (hyprcolors
/// no instalado, silencio); cualquier otra falla queda logueada. Corre solo
/// en arranque, toggle de theme y cambios del archivo — nunca por frame.
pub fn load() -> Option<HyprcolorPalette> {
    let path = colors_json_path()?;

    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return None,
        Err(error) => {
            log::warn!("no pude leer {}: {error}", path.display());
            return None;
        }
    };

    let file: HyprcolorFile = match serde_json::from_str(&content) {
        Ok(file) => file,
        Err(error) => {
            log::warn!("paleta inválida en {}: {error}", path.display());
            return None;
        }
    };

    let Some(raw_accent) = file.accent else {
        log::warn!("la paleta {} no trae 'accent'", path.display());
        return None;
    };

    let Some(accent) = parse_hex_color(&raw_accent) else {
        log::warn!("accent inválido en {}: {raw_accent:?}", path.display());
        return None;
    };

    Some(HyprcolorPalette { accent })
}

/// Last modification time of the palette file, to detect live changes.
pub fn modified_time() -> Option<std::time::SystemTime> {
    fs::metadata(colors_json_path()?).ok()?.modified().ok()
}

/// Parsea "#rrggbb". La longitud se mide en bytes, así que un char
/// multibyte adentro no es un color válido y además rompería los
/// slices: el chequeo `is_ascii` corta antes de que eso pase.
#[doc(hidden)]
pub fn parse_hex_color(value: &str) -> Option<Color> {
    let value = value.strip_prefix('#')?;

    if value.len() != 6 || !value.is_ascii() {
        return None;
    }

    let r = u8::from_str_radix(&value[0..2], 16).ok()?;
    let g = u8::from_str_radix(&value[2..4], 16).ok()?;
    let b = u8::from_str_radix(&value[4..6], 16).ok()?;

    Some(Color::from_rgba8(r, g, b, 0xff))
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn colors_json_path() -> Option<PathBuf> {
    if let Some(cache_home) = env::var_os("XDG_CACHE_HOME") {
        return Some(PathBuf::from(cache_home).join("hyprcolors/colors.json"));
    }

    env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache").join("hyprcolors").join("colors.json"))
}
