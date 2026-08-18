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

    let program = program.to_owned();

    // El reaper además avisa si el comando falló: sin esto, un
    // hyprlock que sale con error es indistinguible de uno exitoso.
    thread::spawn(move || match child.wait() {
        Ok(status) if status.success() => {}
        Ok(status) => log::warn!("{program} terminó mal ({status})"),
        Err(error) => log::warn!("no se pudo esperar a {program}: {error}"),
    });

    Ok(())
}
