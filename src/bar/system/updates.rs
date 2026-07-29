// ─── < Imports > ────────────────────────────────────────────────────

use std::env;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::proc::spawn_detached;

// ─── < Constants > ────────────────────────────────────────────────────

/// Known terminals with the argument prefix needed to run a command in them.
const TERMINALS: [(&str, &[&str]); 5] = [
    ("kitty", &[]),
    ("ghostty", &["-e"]),
    ("alacritty", &["-e"]),
    ("foot", &[]),
    ("wezterm", &["start", "--"]),
];

const UPDATE_HELPERS: [&str; 2] = ["paru", "yay"];
const PACMAN_FALLBACK: &str = "sudo pacman -Syu";

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn check_pending_updates() -> Result<u32> {
    let output = Command::new("checkupdates")
        .output()
        .context("ejecutando checkupdates — ¿está instalado pacman-contrib?")?;

    match output.status.code() {
        Some(0) => Ok(count_update_lines(&String::from_utf8_lossy(&output.stdout))),
        Some(2) => Ok(0),
        code => bail!("checkupdates terminó con código {code:?}: {}", String::from_utf8_lossy(&output.stderr).trim()),
    }
}

pub fn count_update_lines(stdout: &str) -> u32 {
    stdout.lines().filter(|line| !line.trim().is_empty()).count() as u32
}

/// Opens a terminal running the system update, keeping the window open at the end.
pub fn launch_update() -> Result<()> {
    let (terminal, prefix) = find_terminal().context("no encontré una terminal conocida (kitty/ghostty/alacritty/foot/wezterm)")?;

    let shell_line = format!("{}; printf '\\n[hyprbar] update terminado — Enter para cerrar '; read _", update_command());

    let mut arguments: Vec<&str> = prefix.to_vec();
    arguments.extend(["sh", "-c", shell_line.as_str()]);

    spawn_detached(&terminal, &arguments)
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn update_command() -> String {
    UPDATE_HELPERS
        .into_iter()
        .find(|helper| is_in_path(helper))
        .map(|helper| format!("{helper} -Syu"))
        .unwrap_or_else(|| PACMAN_FALLBACK.to_string())
}

fn find_terminal() -> Option<(String, &'static [&'static str])> {
    if let Ok(preferred) = env::var("TERMINAL") {
        let base = Path::new(&preferred).file_name().map(|name| name.to_string_lossy().to_string());

        if let Some(base) = base
            && let Some((_, prefix)) = TERMINALS.into_iter().find(|(name, _)| *name == base)
        {
            return Some((preferred, prefix));
        }
    }

    TERMINALS
        .into_iter()
        .find(|(name, _)| is_in_path(name))
        .map(|(name, prefix)| (name.to_string(), prefix))
}

fn is_in_path(name: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&path).any(|dir| dir.join(name).is_file())
}
