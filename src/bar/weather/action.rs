// ─── < Imports > ────────────────────────────────────────────────────

use crate::components::{ComponentAction, ComponentTag, Interaction};

// ─── < Constants > ────────────────────────────────────────────────────

const TAG: ComponentTag = ComponentTag::new("weather");

// ─── < Enums > ────────────────────────────────────────────────────

/// Pestañas del panel del clima.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeatherTab {
    #[default]
    Forecast,
    AirUv,
    Sea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherAction {
    SelectTab(WeatherTab),
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl WeatherTab {
    pub const ALL: [Self; 3] = [Self::Forecast, Self::AirUv, Self::Sea];

    pub fn label(self) -> &'static str {
        match self {
            Self::Forecast => "Pronóstico",
            Self::AirUv => "Aire y UV",
            Self::Sea => "Mar",
        }
    }

    fn code(self) -> u16 {
        match self {
            Self::Forecast => 0,
            Self::AirUv => 1,
            Self::Sea => 2,
        }
    }

    fn from_code(code: u16) -> Option<Self> {
        match code {
            0 => Some(Self::Forecast),
            1 => Some(Self::AirUv),
            2 => Some(Self::Sea),
            _ => None,
        }
    }
}

impl WeatherAction {
    pub fn interaction(self) -> Interaction {
        let Self::SelectTab(tab) = self;

        Interaction::Action(ComponentAction::new(TAG, tab.code()))
    }

    pub fn from_interaction(interaction: Interaction) -> Option<Self> {
        let Interaction::Action(action) = interaction else {
            return None;
        };

        if action.owner() != TAG {
            return None;
        }

        WeatherTab::from_code(action.id()).map(Self::SelectTab)
    }
}
