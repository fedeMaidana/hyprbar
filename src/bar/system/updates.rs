// ─── < Imports > ────────────────────────────────────────────────────

use std::process::Command;

use anyhow::{Context, Result, bail};

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
