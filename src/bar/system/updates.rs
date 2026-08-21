// ─── < Imports > ────────────────────────────────────────────────────

use std::path::Path;
use std::process::Command;
use std::time::SystemTime;
use std::{env, fs};

use anyhow::{Context, Result, bail};

use crate::proc::spawn_detached;

use super::state::PendingPackage;

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

pub fn check_pending_updates() -> Result<String> {
    let output = Command::new("checkupdates")
        .output()
        .context("ejecutando checkupdates — ¿está instalado pacman-contrib?")?;

    match output.status.code() {
        Some(0) => Ok(String::from_utf8_lossy(&output.stdout).into_owned()),
        Some(2) => Ok(String::new()),
        code => bail!("checkupdates terminó con código {code:?}: {}", String::from_utf8_lossy(&output.stderr).trim()),
    }
}

pub fn count_update_lines(stdout: &str) -> u32 {
    stdout.lines().filter(|line| !line.trim().is_empty()).count() as u32
}

/// Paquetes pendientes de "nombre viejo -> nuevo" (versión nueva).
pub fn parse_pending_packages(stdout: &str) -> Vec<PendingPackage> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let name = fields.next()?.to_owned();
            let version = fields.nth(2)?.to_owned();

            Some(PendingPackage { name, version })
        })
        .collect()
}

/// Minutos desde el último sync de la base de pacman (mtime más nuevo
/// de /var/lib/pacman/sync).
pub fn read_last_sync_minutes() -> Option<u64> {
    let entries = fs::read_dir("/var/lib/pacman/sync").ok()?;

    let newest = entries.flatten().filter_map(|entry| entry.metadata().ok()?.modified().ok()).max()?;

    let elapsed = SystemTime::now().duration_since(newest).ok()?;

    Some(elapsed.as_secs() / 60)
}

/// Host del primer mirror activo en la mirrorlist.
pub fn read_mirror_host() -> Option<String> {
    let content = fs::read_to_string("/etc/pacman.d/mirrorlist").ok()?;

    parse_mirror_host(&content)
}

pub fn parse_mirror_host(mirrorlist: &str) -> Option<String> {
    for line in mirrorlist.lines() {
        let line = line.trim();

        let Some(url) = line.strip_prefix("Server") else {
            continue;
        };

        let url = url.trim_start_matches([' ', '=']).trim();
        let without_scheme = url.split("://").nth(1).unwrap_or(url);

        return without_scheme.split('/').next().map(str::to_owned);
    }

    None
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
