// ─── < Imports > ────────────────────────────────────────────────────

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::Weekday;
use serde::{Deserialize, Serialize};

// ─── < Constants > ────────────────────────────────────────────────────

/// Duración por defecto del temporizador (el preset de 5 min).
const DEFAULT_TIMER_MINUTES: u64 = 5;

// ─── < Enums > ────────────────────────────────────────────────────

/// Qué días se repite una alarma.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Repeat {
    #[default]
    Daily,
    Weekdays,
    Weekend,
}

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    pub hour: u8,
    pub minute: u8,
    #[serde(default)]
    pub repeat: Repeat,
    #[serde(default = "enabled_default")]
    pub enabled: bool,
    /// Nombre opcional; el picker no puede escribirlo, pero el JSON sí.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct StopwatchData {
    pub started_at: Option<Instant>,
    pub accumulated: Duration,
    pub laps: Vec<Duration>,
}

#[derive(Debug, Clone)]
pub struct TimerData {
    pub duration: Duration,
    pub started_at: Option<Instant>,
    pub remaining_at_pause: Duration,
    pub finished: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ClockData {
    pub alarms: Vec<Alarm>,
    pub stopwatch: StopwatchData,
    pub timer: TimerData,
}

/// Alarma a medio editar en el picker; `index` None = nueva.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlarmEditor {
    pub index: Option<usize>,
    pub hour: u8,
    pub minute: u8,
    pub repeat: Repeat,
}

/// Estado compartido entre el pill, el panel y el vigilante.
#[derive(Clone, Default)]
pub struct ClockStore {
    inner: Arc<Mutex<ClockData>>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Repeat {
    pub const ALL: [Self; 3] = [Self::Daily, Self::Weekdays, Self::Weekend];

    pub fn label(self) -> &'static str {
        match self {
            Self::Daily => "Todos los días",
            Self::Weekdays => "L a V",
            Self::Weekend => "Finde",
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::Daily => 0,
            Self::Weekdays => 1,
            Self::Weekend => 2,
        }
    }

    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Daily),
            1 => Some(Self::Weekdays),
            2 => Some(Self::Weekend),
            _ => None,
        }
    }

    pub fn matches(self, weekday: Weekday) -> bool {
        let workday = weekday.num_days_from_monday() < 5;

        match self {
            Self::Daily => true,
            Self::Weekdays => workday,
            Self::Weekend => !workday,
        }
    }
}

impl Alarm {
    pub fn time_text(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }

    /// "Despertar · L a V" con nombre, "L a V" sin él.
    pub fn subtitle(&self) -> String {
        match &self.label {
            Some(label) => format!("{label} · {}", self.repeat.label()),
            None => self.repeat.label().to_string(),
        }
    }
}

impl StopwatchData {
    pub fn running(&self) -> bool {
        self.started_at.is_some()
    }

    pub fn elapsed(&self) -> Duration {
        self.accumulated + self.started_at.map(|at| at.elapsed()).unwrap_or(Duration::ZERO)
    }

    pub fn toggle(&mut self) {
        match self.started_at.take() {
            Some(at) => self.accumulated += at.elapsed(),
            None => self.started_at = Some(Instant::now()),
        }
    }

    pub fn lap(&mut self) {
        if self.running() {
            self.laps.push(self.elapsed());
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl TimerData {
    pub fn running(&self) -> bool {
        self.started_at.is_some()
    }

    pub fn remaining(&self) -> Duration {
        let spent = self.started_at.map(|at| at.elapsed()).unwrap_or(Duration::ZERO);

        self.remaining_at_pause.saturating_sub(spent)
    }

    /// 1.0 recién arrancado, 0.0 vencido; alimenta el anillo.
    pub fn fraction_remaining(&self) -> f32 {
        let total = self.duration.as_secs_f32();

        if total <= 0.0 {
            return 0.0;
        }

        (self.remaining().as_secs_f32() / total).clamp(0.0, 1.0)
    }

    pub fn toggle(&mut self) {
        if self.running() {
            self.remaining_at_pause = self.remaining();
            self.started_at = None;

            return;
        }

        // Arrancar vencido o terminado recarga la duración completa.
        if self.finished || self.remaining_at_pause == Duration::ZERO {
            self.remaining_at_pause = self.duration;
            self.finished = false;
        }

        self.started_at = Some(Instant::now());
    }

    pub fn reset(&mut self) {
        self.started_at = None;
        self.remaining_at_pause = self.duration;
        self.finished = false;
    }

    pub fn set_minutes(&mut self, minutes: u32) {
        self.duration = Duration::from_secs(u64::from(minutes) * 60);
        self.reset();
    }
}

impl Default for TimerData {
    fn default() -> Self {
        let duration = Duration::from_secs(DEFAULT_TIMER_MINUTES * 60);

        Self {
            duration,
            started_at: None,
            remaining_at_pause: duration,
            finished: false,
        }
    }
}

impl AlarmEditor {
    /// Editor vacío para una alarma nueva.
    pub fn blank() -> Self {
        Self {
            index: None,
            hour: 7,
            minute: 0,
            repeat: Repeat::Daily,
        }
    }

    pub fn for_alarm(index: usize, alarm: &Alarm) -> Self {
        Self {
            index: Some(index),
            hour: alarm.hour,
            minute: alarm.minute,
            repeat: alarm.repeat,
        }
    }
}

impl ClockStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> ClockData {
        self.lock().clone()
    }

    /// Mutación en bloque bajo un solo lock; devuelve lo que el cierre
    /// quiera reportar (p. ej. si el temporizador acaba de vencer).
    pub fn update<R>(&self, mutate: impl FnOnce(&mut ClockData) -> R) -> R {
        mutate(&mut self.lock())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, ClockData> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("estado del reloj envenenado por un panic previo; se recupera");
                poisoned.into_inner()
            }
        }
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn enabled_default() -> bool {
    true
}
