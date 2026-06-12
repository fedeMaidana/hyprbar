// ─── < Imports > ────────────────────────────────────────────────────

use std::process::Command;
use std::thread;

use anyhow::{Context, Result};

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn spawn_detached(program: &str, arguments: &[&str]) -> Result<()> {
    let mut child = Command::new(program)
        .args(arguments)
        .spawn()
        .with_context(|| format!("lanzando {program}"))?;

    thread::spawn(move || {
        let _ = child.wait();
    });

    Ok(())
}
