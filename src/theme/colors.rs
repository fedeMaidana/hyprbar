// ─── < Imports > ────────────────────────────────────────────────────

use vello::peniko::Color;

use super::hyprcolor;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub pill_bg: Color,
    pub pill_hover_bg: Color,
    pub pill_border: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent: Color,

    pub panel_bg: Color,
    pub panel_border: Color,
    pub panel_divider: Color,
    pub panel_raised: Color,

    pub control_bg: Color,
    pub control_hover_bg: Color,

    pub slot_active_bg: Color,
    pub slot_active_text: Color,
    pub slot_inactive_bg: Color,
    pub slot_empty_bg: Color,
    pub slot_hover_bg: Color,

    pub danger_bg: Color,
    pub danger_text: Color,

    pub meter_warning: Color,
    pub meter_critical: Color,
    /// Estados buenos: batería sana, uptime, "up to date".
    pub positive: Color,

    pub clock_day: Color,
    pub clock_night: Color,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Palette {
    pub fn dark() -> Self {
        Self {
            pill_bg: Color::from_rgba8(0x00, 0x00, 0x00, 0x80),
            pill_hover_bg: Color::from_rgba8(0x2e, 0x2e, 0x36, 0xa8),
            pill_border: Color::from_rgba8(0xff, 0xff, 0xff, 0x0f),
            text_primary: Color::from_rgba8(0xf5, 0xf5, 0xf7, 0xff),
            text_secondary: Color::from_rgba8(0xa0, 0xa0, 0xa8, 0xff),
            accent: Color::from_rgba8(0x9a, 0x8c, 0xff, 0xff),

            panel_bg: Color::from_rgba8(0x15, 0x15, 0x1b, 0xf0),
            panel_border: Color::from_rgba8(0xff, 0xff, 0xff, 0x17),
            panel_divider: Color::from_rgba8(0xff, 0xff, 0xff, 0x0e),
            panel_raised: Color::from_rgba8(0xff, 0xff, 0xff, 0x12),

            control_bg: Color::from_rgba8(0xff, 0xff, 0xff, 0x14),
            control_hover_bg: Color::from_rgba8(0x9a, 0x8c, 0xff, 0x38),

            slot_active_bg: Color::from_rgba8(0xe8, 0xe4, 0xff, 0xff),
            slot_active_text: Color::from_rgba8(0x2a, 0x25, 0x4a, 0xff),
            slot_inactive_bg: Color::from_rgba8(0x40, 0x40, 0x48, 0xa0),
            slot_empty_bg: Color::from_rgba8(0x30, 0x30, 0x38, 0x60),
            slot_hover_bg: Color::from_rgba8(0x58, 0x58, 0x64, 0xc0),

            danger_bg: Color::from_rgba8(0x8c, 0x3a, 0x3a, 0xff),
            danger_text: Color::from_rgba8(0xff, 0xe0, 0xe0, 0xff),

            meter_warning: Color::from_rgba8(0xd8, 0xa2, 0x3a, 0xff),
            meter_critical: Color::from_rgba8(0xd8, 0x5a, 0x5a, 0xff),
            positive: Color::from_rgba8(0x5d, 0xd3, 0x9e, 0xff),

            clock_day: Color::from_rgba8(0xef, 0xb3, 0x5a, 0xff),
            clock_night: Color::from_rgba8(0x7e, 0x90, 0xd1, 0xff),
        }
    }

    pub fn light() -> Self {
        Self {
            pill_bg: Color::from_rgba8(0xff, 0xff, 0xff, 0xb8),
            pill_hover_bg: Color::from_rgba8(0xff, 0xff, 0xff, 0xe0),
            pill_border: Color::from_rgba8(0x00, 0x00, 0x00, 0x14),
            text_primary: Color::from_rgba8(0x1c, 0x1c, 0x22, 0xff),
            text_secondary: Color::from_rgba8(0x55, 0x55, 0x5f, 0xff),
            accent: Color::from_rgba8(0x6a, 0x5a, 0xe0, 0xff),

            panel_bg: Color::from_rgba8(0xf7, 0xf7, 0xfa, 0xf7),
            panel_border: Color::from_rgba8(0x00, 0x00, 0x00, 0x1a),
            panel_divider: Color::from_rgba8(0x00, 0x00, 0x00, 0x12),
            panel_raised: Color::from_rgba8(0x00, 0x00, 0x00, 0x0d),

            control_bg: Color::from_rgba8(0x00, 0x00, 0x00, 0x0f),
            control_hover_bg: Color::from_rgba8(0x6a, 0x5a, 0xe0, 0x38),

            slot_active_bg: Color::from_rgba8(0x6a, 0x5a, 0xe0, 0xff),
            slot_active_text: Color::from_rgba8(0xff, 0xff, 0xff, 0xff),
            slot_inactive_bg: Color::from_rgba8(0x00, 0x00, 0x00, 0x26),
            slot_empty_bg: Color::from_rgba8(0x00, 0x00, 0x00, 0x14),
            slot_hover_bg: Color::from_rgba8(0x00, 0x00, 0x00, 0x3d),

            danger_bg: Color::from_rgba8(0xd6, 0x45, 0x45, 0xff),
            danger_text: Color::from_rgba8(0xff, 0xff, 0xff, 0xff),

            meter_warning: Color::from_rgba8(0xb0, 0x7a, 0x18, 0xff),
            meter_critical: Color::from_rgba8(0xc2, 0x3a, 0x3a, 0xff),
            positive: Color::from_rgba8(0x1f, 0x9d, 0x66, 0xff),

            clock_day: Color::from_rgba8(0xb5, 0x7a, 0x10, 0xff),
            clock_night: Color::from_rgba8(0x4a, 0x5e, 0xae, 0xff),
        }
    }

    /// Tiñe la paleta estática con el accent de hyprcolor, si existe.
    /// El único llamador es `Theme::of` más el watcher de la paleta:
    /// los constructores de arriba son puros a propósito.
    pub fn refresh_from_hyprcolor(&mut self) {
        let Some(hyprcolor) = hyprcolor::load() else {
            return;
        };

        self.accent = hyprcolor.accent;
        self.slot_active_bg = hyprcolor.accent;
        self.slot_active_text = contrast_text_for(hyprcolor.accent);
        self.control_hover_bg = hyprcolor.accent.with_alpha(ACCENT_HOVER_ALPHA);
    }
}

// ─── < Constants > ────────────────────────────────────────────────────

/// Transparencia del accent cuando pinta fondos de hover.
const ACCENT_HOVER_ALPHA: f32 = 0.22;

/// Por encima de esta luminancia percibida el fondo se considera claro
/// y lleva texto oscuro.
const CONTRAST_LUMINANCE_THRESHOLD: f32 = 0.6;

// ─── < Public Functions > ────────────────────────────────────────────────────

/// Elige texto oscuro o claro según la luminancia percibida del fondo
/// (pesos Rec. 601 sobre los componentes sRGB; alcanza para decidir
/// entre dos textos).
#[doc(hidden)]
pub fn contrast_text_for(background: Color) -> Color {
    let [r, g, b, _] = background.components;
    let luminance = 0.299 * r + 0.587 * g + 0.114 * b;

    if luminance > CONTRAST_LUMINANCE_THRESHOLD {
        // Texto casi negro para fondos claros.
        Color::from_rgba8(0x20, 0x20, 0x28, 0xff)
    } else {
        // El mismo blanco tiza que text_primary del modo oscuro.
        Color::from_rgba8(0xf5, 0xf5, 0xf7, 0xff)
    }
}
