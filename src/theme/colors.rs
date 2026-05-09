//! Paleta de colores. Usar [`Color`] en componentes, nunca literales.

use vello::peniko::Color;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub pill_bg: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent: Color,
    pub shadow: Color,

    /// Fondo de un slot de workspace activo (resaltado claro).
    pub slot_active_bg: Color,
    /// Texto sobre slot activo (oscuro para contraste).
    pub slot_active_text: Color,
    /// Fondo de un slot existente pero no activo.
    pub slot_inactive_bg: Color,
    /// Fondo de un slot que no existe aún en Hyprland (placeholder).
    pub slot_empty_bg: Color,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            pill_bg: Color::from_rgba8(0x00, 0x00, 0x00, 0x80),
            text_primary: Color::from_rgba8(0xf5, 0xf5, 0xf7, 0xff),
            text_secondary: Color::from_rgba8(0xa0, 0xa0, 0xa8, 0xff),
            accent: Color::from_rgba8(0x9a, 0x8c, 0xff, 0xff),
            shadow: Color::from_rgba8(0x00, 0x00, 0x00, 0x40),

            // Slot activo: lavanda muy claro casi blanco
            slot_active_bg: Color::from_rgba8(0xe8, 0xe4, 0xff, 0xff),
            slot_active_text: Color::from_rgba8(0x2a, 0x25, 0x4a, 0xff),
            // Slot existente: gris medio translúcido
            slot_inactive_bg: Color::from_rgba8(0x40, 0x40, 0x48, 0xa0),
            // Slot vacío: gris muy sutil
            slot_empty_bg: Color::from_rgba8(0x30, 0x30, 0x38, 0x60),
        }
    }
}
