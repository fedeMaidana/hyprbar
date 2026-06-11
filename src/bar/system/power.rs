// ─── < Imports > ────────────────────────────────────────────────────

use std::process::Command;
use std::thread;

use anyhow::{Context, Result};

// ─── < Constants > ────────────────────────────────────────────────────

const LOCK_COMMAND: (&str, &[&str]) = ("hyprlock", &[]);
const SUSPEND_COMMAND: (&str, &[&str]) = ("systemctl", &["suspend"]);
const REBOOT_COMMAND: (&str, &[&str]) = ("systemctl", &["reboot"]);
const SHUTDOWN_COMMAND: (&str, &[&str]) = ("systemctl", &["poweroff"]);

// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    Lock,
    Suspend,
    Reboot,
    Shutdown,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl PowerAction {
    pub const ALL: [Self; 4] = [Self::Lock, Self::Suspend, Self::Reboot, Self::Shutdown];

    pub(crate) fn glyph(self) -> &'static str {
        match self {
            Self::Lock => "\u{f033e}",
            Self::Suspend => "\u{f0904}",
            Self::Reboot => "\u{f0450}",
            Self::Shutdown => "\u{f0425}",
        }
    }

    pub fn execute(self) -> Result<()> {
        let (program, arguments) = self.command();

        let mut child = Command::new(program)
            .args(arguments)
            .spawn()
            .with_context(|| format!("lanzando {program} para {self:?}"))?;

        thread::spawn(move || {
            let _ = child.wait();
        });

        Ok(())
    }

    fn command(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::Lock => LOCK_COMMAND,
            Self::Suspend => SUSPEND_COMMAND,
            Self::Reboot => REBOOT_COMMAND,
            Self::Shutdown => SHUTDOWN_COMMAND,
        }
    }
}
