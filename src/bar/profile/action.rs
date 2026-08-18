// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;

use crate::components::{ComponentAction, ComponentTag, Interaction};
use crate::proc::spawn_detached;

// ─── < Constants > ────────────────────────────────────────────────────

const TAG: ComponentTag = ComponentTag::new("profile");

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

    pub fn interaction(self) -> Interaction {
        Interaction::Action(ComponentAction::new(TAG, self.code()))
    }

    pub fn from_interaction(interaction: Interaction) -> Option<Self> {
        let Interaction::Action(action) = interaction else {
            return None;
        };

        if action.owner() != TAG {
            return None;
        }

        Self::from_code(action.id())
    }

    pub fn execute(self) -> Result<()> {
        let (program, arguments) = match self {
            Self::Lock => LOCK_COMMAND,
            Self::Logout => LOGOUT_COMMAND,
        };

        spawn_detached(program, arguments)
    }

    fn code(self) -> u16 {
        match self {
            Self::Lock => 0,
            Self::Logout => 1,
        }
    }

    fn from_code(code: u16) -> Option<Self> {
        match code {
            0 => Some(Self::Lock),
            1 => Some(Self::Logout),
            _ => None,
        }
    }
}
