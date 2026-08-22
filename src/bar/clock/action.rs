// ─── < Imports > ────────────────────────────────────────────────────

use crate::components::{ComponentAction, ComponentTag, Interaction};

use super::state::Repeat;

// ─── < Constants > ────────────────────────────────────────────────────

const TAG: ComponentTag = ComponentTag::new("clock");

const TAB_CODE_BASE: u16 = 0;
const TOGGLE_ALARM_CODE: u16 = 10;
const EDIT_ALARM_CODE: u16 = 11;
const DELETE_ALARM_CODE: u16 = 12;
const NEW_ALARM_CODE: u16 = 13;
const EDITOR_SAVE_CODE: u16 = 14;
const EDITOR_CANCEL_CODE: u16 = 15;
const EDITOR_HOUR_UP_CODE: u16 = 16;
const EDITOR_HOUR_DOWN_CODE: u16 = 17;
const EDITOR_MINUTE_UP_CODE: u16 = 18;
const EDITOR_MINUTE_DOWN_CODE: u16 = 19;
const EDITOR_REPEAT_CODE: u16 = 20;
const STOPWATCH_TOGGLE_CODE: u16 = 30;
const STOPWATCH_LAP_CODE: u16 = 31;
const STOPWATCH_RESET_CODE: u16 = 32;
const TIMER_TOGGLE_CODE: u16 = 40;
const TIMER_RESET_CODE: u16 = 41;
const TIMER_PRESET_CODE: u16 = 42;

// ─── < Enums > ────────────────────────────────────────────────────

/// Pestañas del panel del reloj.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClockTab {
    #[default]
    Clock,
    Alarms,
    Stopwatch,
    Timer,
}

/// Everything clickable inside the clock panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockAction {
    SelectTab(ClockTab),
    ToggleAlarm(usize),
    EditAlarm(usize),
    DeleteAlarm,
    NewAlarm,
    EditorSave,
    EditorCancel,
    EditorHourUp,
    EditorHourDown,
    EditorMinuteUp,
    EditorMinuteDown,
    EditorRepeat(Repeat),
    StopwatchToggle,
    StopwatchLap,
    StopwatchReset,
    TimerToggle,
    TimerReset,
    /// Minutos del preset elegido (1, 5, 10, 25).
    TimerPreset(u32),
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl ClockTab {
    pub const ALL: [Self; 4] = [Self::Clock, Self::Alarms, Self::Stopwatch, Self::Timer];

    pub fn label(self) -> &'static str {
        match self {
            Self::Clock => "Reloj",
            Self::Alarms => "Alarmas",
            Self::Stopwatch => "Cronómetro",
            Self::Timer => "Temporizador",
        }
    }

    fn code(self) -> u16 {
        match self {
            Self::Clock => 0,
            Self::Alarms => 1,
            Self::Stopwatch => 2,
            Self::Timer => 3,
        }
    }

    fn from_code(code: u16) -> Option<Self> {
        match code {
            0 => Some(Self::Clock),
            1 => Some(Self::Alarms),
            2 => Some(Self::Stopwatch),
            3 => Some(Self::Timer),
            _ => None,
        }
    }
}

impl ClockAction {
    pub fn interaction(self) -> Interaction {
        let (code, value) = self.encode();

        Interaction::Action(ComponentAction::new(TAG, code).with_value(value))
    }

    pub fn from_interaction(interaction: Interaction) -> Option<Self> {
        let Interaction::Action(action) = interaction else {
            return None;
        };

        if action.owner() != TAG {
            return None;
        }

        Self::decode(action.id(), action.value())
    }

    /// El id lleva la acción y `value` el payload (índice, repetición
    /// o minutos), así el envoltorio opaco alcanza para todo el panel.
    fn encode(self) -> (u16, i32) {
        match self {
            Self::SelectTab(tab) => (TAB_CODE_BASE + tab.code(), 0),
            Self::ToggleAlarm(index) => (TOGGLE_ALARM_CODE, index as i32),
            Self::EditAlarm(index) => (EDIT_ALARM_CODE, index as i32),
            Self::DeleteAlarm => (DELETE_ALARM_CODE, 0),
            Self::NewAlarm => (NEW_ALARM_CODE, 0),
            Self::EditorSave => (EDITOR_SAVE_CODE, 0),
            Self::EditorCancel => (EDITOR_CANCEL_CODE, 0),
            Self::EditorHourUp => (EDITOR_HOUR_UP_CODE, 0),
            Self::EditorHourDown => (EDITOR_HOUR_DOWN_CODE, 0),
            Self::EditorMinuteUp => (EDITOR_MINUTE_UP_CODE, 0),
            Self::EditorMinuteDown => (EDITOR_MINUTE_DOWN_CODE, 0),
            Self::EditorRepeat(repeat) => (EDITOR_REPEAT_CODE, repeat.code() as i32),
            Self::StopwatchToggle => (STOPWATCH_TOGGLE_CODE, 0),
            Self::StopwatchLap => (STOPWATCH_LAP_CODE, 0),
            Self::StopwatchReset => (STOPWATCH_RESET_CODE, 0),
            Self::TimerToggle => (TIMER_TOGGLE_CODE, 0),
            Self::TimerReset => (TIMER_RESET_CODE, 0),
            Self::TimerPreset(minutes) => (TIMER_PRESET_CODE, minutes as i32),
        }
    }

    fn decode(code: u16, value: i32) -> Option<Self> {
        match code {
            TOGGLE_ALARM_CODE => Some(Self::ToggleAlarm(value.max(0) as usize)),
            EDIT_ALARM_CODE => Some(Self::EditAlarm(value.max(0) as usize)),
            DELETE_ALARM_CODE => Some(Self::DeleteAlarm),
            NEW_ALARM_CODE => Some(Self::NewAlarm),
            EDITOR_SAVE_CODE => Some(Self::EditorSave),
            EDITOR_CANCEL_CODE => Some(Self::EditorCancel),
            EDITOR_HOUR_UP_CODE => Some(Self::EditorHourUp),
            EDITOR_HOUR_DOWN_CODE => Some(Self::EditorHourDown),
            EDITOR_MINUTE_UP_CODE => Some(Self::EditorMinuteUp),
            EDITOR_MINUTE_DOWN_CODE => Some(Self::EditorMinuteDown),
            EDITOR_REPEAT_CODE => Repeat::from_code(value.max(0) as u8).map(Self::EditorRepeat),
            STOPWATCH_TOGGLE_CODE => Some(Self::StopwatchToggle),
            STOPWATCH_LAP_CODE => Some(Self::StopwatchLap),
            STOPWATCH_RESET_CODE => Some(Self::StopwatchReset),
            TIMER_TOGGLE_CODE => Some(Self::TimerToggle),
            TIMER_RESET_CODE => Some(Self::TimerReset),
            TIMER_PRESET_CODE => Some(Self::TimerPreset(value.max(0) as u32)),
            code => ClockTab::from_code(code - TAB_CODE_BASE).map(Self::SelectTab),
        }
    }
}
