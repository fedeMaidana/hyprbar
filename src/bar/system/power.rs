// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;

use crate::proc::spawn_detached;

// ─── < Constants > ────────────────────────────────────────────────────

const SUSPEND_COMMAND: (&str, &[&str]) = ("systemctl", &["suspend"]);
const HIBERNATE_COMMAND: (&str, &[&str]) = ("systemctl", &["hibernate"]);
const REBOOT_COMMAND: (&str, &[&str]) = ("systemctl", &["reboot"]);
const SHUTDOWN_COMMAND: (&str, &[&str]) = ("systemctl", &["poweroff"]);

// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    Suspend,
    Hibernate,
    Reboot,
    Shutdown,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl PowerAction {
    pub const ALL: [Self; 4] = [Self::Suspend, Self::Hibernate, Self::Reboot, Self::Shutdown];

    pub(crate) fn glyph(self) -> &'static str {
        match self {
            Self::Suspend => "\u{f0594}",
            Self::Hibernate => "\u{f0717}",
            Self::Reboot => "\u{f0450}",
            Self::Shutdown => "\u{f0425}",
        }
    }

    /// Título del modal de confirmación.
    pub(crate) fn confirm_title(self) -> &'static str {
        match self {
            Self::Suspend => "¿Suspender el equipo?",
            Self::Hibernate => "¿Hibernar el equipo?",
            Self::Reboot => "¿Reiniciar el equipo?",
            Self::Shutdown => "¿Apagar el equipo?",
        }
    }

    /// Las que cortan la sesión llevan el confirmar en rojo.
    pub(crate) fn is_destructive(self) -> bool {
        matches!(self, Self::Reboot | Self::Shutdown)
    }

    pub fn execute(self) -> Result<()> {
        let (program, arguments) = self.command();

        spawn_detached(program, arguments)
    }

    fn command(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::Suspend => SUSPEND_COMMAND,
            Self::Hibernate => HIBERNATE_COMMAND,
            Self::Reboot => REBOOT_COMMAND,
            Self::Shutdown => SHUTDOWN_COMMAND,
        }
    }
}
