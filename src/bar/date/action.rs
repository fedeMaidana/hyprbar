// ─── < Imports > ────────────────────────────────────────────────────

use crate::components::{ComponentAction, ComponentTag, Interaction};

// ─── < Constants > ────────────────────────────────────────────────────

const TAG: ComponentTag = ComponentTag::new("date");

// ─── < Enums > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarAction {
    PrevMonth,
    NextMonth,
    Today,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl CalendarAction {
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

    fn code(self) -> u16 {
        match self {
            Self::PrevMonth => 0,
            Self::NextMonth => 1,
            Self::Today => 2,
        }
    }

    fn from_code(code: u16) -> Option<Self> {
        match code {
            0 => Some(Self::PrevMonth),
            1 => Some(Self::NextMonth),
            2 => Some(Self::Today),
            _ => None,
        }
    }
}
