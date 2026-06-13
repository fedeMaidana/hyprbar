// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;

use crate::proc::spawn_detached;

// ─── < Constants > ────────────────────────────────────────────────────

const LOCK_COMMAND: (&str, &[&str]) = ("hyprlock", &[]);
const LOGOUT_COMMAND: (&str, &[&str]) = ("hyprctl", &["dispatch", "exit"]);

// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAction {
    Lock,
    Logout,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl SessionAction {
    pub const ALL: [Self; 2] = [Self::Lock, Self::Logout];

    pub(crate) fn glyph(self) -> &'static str {
        match self {
            Self::Lock => "\u{f033e}",
            Self::Logout => "\u{f0343}",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Lock => "Bloquear",
            Self::Logout => "Salir",
        }
    }

    pub fn execute(self) -> Result<()> {
        let (program, arguments) = match self {
            Self::Lock => LOCK_COMMAND,
            Self::Logout => LOGOUT_COMMAND,
        };

        spawn_detached(program, arguments)
    }
}
