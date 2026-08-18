// ─── < Imports > ────────────────────────────────────────────────────

use crate::components::{ComponentAction, ComponentTag, Interaction};

// ─── < Constants > ────────────────────────────────────────────────────

const TAG: ComponentTag = ComponentTag::new("notifications");

// ─── < Enums > ────────────────────────────────────────────────────

/// Acciones del panel de notificaciones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationAction {
    ClearHistory,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl NotificationAction {
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
            Self::ClearHistory => 0,
        }
    }

    fn from_code(code: u16) -> Option<Self> {
        match code {
            0 => Some(Self::ClearHistory),
            _ => None,
        }
    }
}
