// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;

use crate::components::{ComponentAction, ComponentTag, Interaction};

use super::power::PowerAction;
use super::updates::launch_update;

// ─── < Constants > ────────────────────────────────────────────────────

const TAG: ComponentTag = ComponentTag::new("system");

const UPDATES_CODE: u16 = 3;

// ─── < Enums > ────────────────────────────────────────────────────

/// Everything clickable inside the system panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemAction {
    Power(PowerAction),
    Updates,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl SystemAction {
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
        match self {
            Self::Power(action) => action.execute(),
            Self::Updates => launch_update(),
        }
    }

    fn code(self) -> u16 {
        match self {
            Self::Power(PowerAction::Suspend) => 0,
            Self::Power(PowerAction::Reboot) => 1,
            Self::Power(PowerAction::Shutdown) => 2,
            Self::Updates => UPDATES_CODE,
        }
    }

    fn from_code(code: u16) -> Option<Self> {
        match code {
            0 => Some(Self::Power(PowerAction::Suspend)),
            1 => Some(Self::Power(PowerAction::Reboot)),
            2 => Some(Self::Power(PowerAction::Shutdown)),
            UPDATES_CODE => Some(Self::Updates),
            _ => None,
        }
    }
}
