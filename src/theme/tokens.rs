//! Design tokens para spacing, radio, dimensiones.

#[derive(Debug, Clone, Copy)]
pub struct Tokens {
    pub bar_height: f32,
    pub bar_margin_top: f32,
    pub bar_margin_x: f32,

    /// Altura fija de TODAS las pills. Mantenerla así garantiza que se vean
    /// alineadas. Cada pill ajusta solo su ancho según contenido.
    pub pill_height: f32,
    pub pill_gap: f32,
    pub pill_padding_x: f32,
    pub pill_padding_y: f32,
    pub pill_radius: f32,

    pub shadow_offset_y: f32,
}

impl Default for Tokens {
    fn default() -> Self {
        Self {
            bar_height: 36.0,
            bar_margin_top: 4.0,
            bar_margin_x: 10.0,

            pill_height: 26.0,
            pill_gap: 5.0,
            pill_padding_x: 9.0,
            pill_padding_y: 6.0,
            pill_radius: 12.0,

            shadow_offset_y: 1.0,
        }
    }
}
