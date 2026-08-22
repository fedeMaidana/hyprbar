// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::Result;

use crate::components::{ComponentAction, ComponentTag, Interaction};

use super::power::PowerAction;
use super::updates::launch_update;

// ─── < Constants > ────────────────────────────────────────────────────

const TAG: ComponentTag = ComponentTag::new("system");

const RUN_UPDATE_CODE: u16 = 4;
const TAB_CODE_BASE: u16 = 10;
const COPY_CODE_BASE: u16 = 20;
const CONFIRMED_POWER_CODE_BASE: u16 = 30;

// ─── < Enums > ────────────────────────────────────────────────────

/// Pestañas del panel de sistema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SystemTab {
    #[default]
    System,
    Network,
    Power,
    Updates,
}

/// Valores copiables al portapapeles desde la pestaña Network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyField {
    Ipv4,
    Gateway,
    Dns,
}

/// Everything clickable inside the system panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemAction {
    /// Pide confirmación en el overlay; no ejecuta nada por sí sola.
    Power(PowerAction),
    /// La vuelta del overlay: el usuario confirmó, ahora sí se ejecuta.
    ConfirmedPower(PowerAction),
    RunUpdate,
    SelectTab(SystemTab),
    Copy(CopyField),
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl SystemTab {
    pub const ALL: [Self; 4] = [Self::System, Self::Network, Self::Power, Self::Updates];

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Network => "Network",
            Self::Power => "Power",
            Self::Updates => "Updates",
        }
    }

    fn code(self) -> u16 {
        match self {
            Self::System => 0,
            Self::Network => 1,
            Self::Power => 2,
            Self::Updates => 3,
        }
    }

    fn from_code(code: u16) -> Option<Self> {
        match code {
            0 => Some(Self::System),
            1 => Some(Self::Network),
            2 => Some(Self::Power),
            3 => Some(Self::Updates),
            _ => None,
        }
    }
}

impl CopyField {
    fn code(self) -> u16 {
        match self {
            Self::Ipv4 => 0,
            Self::Gateway => 1,
            Self::Dns => 2,
        }
    }

    fn from_code(code: u16) -> Option<Self> {
        match code {
            0 => Some(Self::Ipv4),
            1 => Some(Self::Gateway),
            2 => Some(Self::Dns),
            _ => None,
        }
    }
}

impl SystemAction {
    pub fn interaction(self) -> Interaction {
        Interaction::Action(self.component_action())
    }

    pub fn component_action(self) -> ComponentAction {
        ComponentAction::new(TAG, self.code())
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

    /// Acciones que disparan un proceso externo (las pestañas, copias y
    /// el pedido de confirmación se resuelven en el pill).
    pub fn execute(self) -> Result<()> {
        match self {
            Self::ConfirmedPower(action) => action.execute(),
            Self::RunUpdate => launch_update(),
            Self::Power(_) | Self::SelectTab(_) | Self::Copy(_) => Ok(()),
        }
    }

    fn code(self) -> u16 {
        match self {
            Self::Power(action) => power_code(action),
            Self::ConfirmedPower(action) => CONFIRMED_POWER_CODE_BASE + power_code(action),
            Self::RunUpdate => RUN_UPDATE_CODE,
            Self::SelectTab(tab) => TAB_CODE_BASE + tab.code(),
            Self::Copy(field) => COPY_CODE_BASE + field.code(),
        }
    }

    fn from_code(code: u16) -> Option<Self> {
        match code {
            RUN_UPDATE_CODE => Some(Self::RunUpdate),
            code if code >= CONFIRMED_POWER_CODE_BASE => power_from_code(code - CONFIRMED_POWER_CODE_BASE).map(Self::ConfirmedPower),
            code if code >= COPY_CODE_BASE => CopyField::from_code(code - COPY_CODE_BASE).map(Self::Copy),
            code if code >= TAB_CODE_BASE => SystemTab::from_code(code - TAB_CODE_BASE).map(Self::SelectTab),
            code => power_from_code(code).map(Self::Power),
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn power_code(action: PowerAction) -> u16 {
    match action {
        PowerAction::Suspend => 0,
        PowerAction::Hibernate => 1,
        PowerAction::Reboot => 2,
        PowerAction::Shutdown => 3,
    }
}

fn power_from_code(code: u16) -> Option<PowerAction> {
    match code {
        0 => Some(PowerAction::Suspend),
        1 => Some(PowerAction::Hibernate),
        2 => Some(PowerAction::Reboot),
        3 => Some(PowerAction::Shutdown),
        _ => None,
    }
}
