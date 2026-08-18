// ─── < Constants > ────────────────────────────────────────────────────

/// Debajo de esta distancia al target, la transición se da por asentada.
const SETTLE_EPSILON: f32 = 0.01;

// ─── < Structs > ────────────────────────────────────────────────────

/// Valor animado con suavizado exponencial: se acerca al target una
/// fracción fija por unidad de tiempo, así el movimiento es independiente
/// del framerate y desacelera solo (ease-out natural).
#[derive(Debug, Clone, Copy)]
pub struct Transition {
    value: f32,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Transition {
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    /// Salta directo al valor, sin animar.
    pub fn set(&mut self, value: f32) {
        self.value = value;
    }

    /// Avanza hacia `target` según `dt` (segundos) y `speed` (1/s).
    /// Devuelve `true` mientras siga en movimiento: el llamador debe
    /// pedir otro frame.
    pub fn advance(&mut self, target: f32, dt: f32, speed: f32) -> bool {
        let delta = target - self.value;

        if delta.abs() <= SETTLE_EPSILON {
            self.value = target;
            return false;
        }

        let step = 1.0 - (-dt * speed).exp();
        self.value += delta * step;

        if (target - self.value).abs() <= SETTLE_EPSILON {
            self.value = target;
            return false;
        }

        true
    }
}
