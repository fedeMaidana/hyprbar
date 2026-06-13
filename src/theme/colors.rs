// ─── < Imports > ────────────────────────────────────────────────────

use vello::peniko::Color;

use super::hyprcolor;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub pill_bg: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent: Color,
    pub shadow: Color,

    pub slot_active_bg: Color,
    pub slot_active_text: Color,
    pub slot_inactive_bg: Color,
    pub slot_empty_bg: Color,
    pub slot_hover_bg: Color,

    pub danger_bg: Color,
    pub danger_text: Color,

    pub meter_warning: Color,
    pub meter_critical: Color,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Palette {
    pub fn dark() -> Self {
        let mut palette = Self {
            pill_bg: Color::from_rgba8(0x00, 0x00, 0x00, 0x80),
            text_primary: Color::from_rgba8(0xf5, 0xf5, 0xf7, 0xff),
            text_secondary: Color::from_rgba8(0xa0, 0xa0, 0xa8, 0xff),
            accent: Color::from_rgba8(0x9a, 0x8c, 0xff, 0xff),
            shadow: Color::from_rgba8(0x00, 0x00, 0x00, 0x40),

            slot_active_bg: Color::from_rgba8(0xe8, 0xe4, 0xff, 0xff),
            slot_active_text: Color::from_rgba8(0x2a, 0x25, 0x4a, 0xff),
            slot_inactive_bg: Color::from_rgba8(0x40, 0x40, 0x48, 0xa0),
            slot_empty_bg: Color::from_rgba8(0x30, 0x30, 0x38, 0x60),
            slot_hover_bg: Color::from_rgba8(0x58, 0x58, 0x64, 0xc0),

            danger_bg: Color::from_rgba8(0x8c, 0x3a, 0x3a, 0xff),
            danger_text: Color::from_rgba8(0xff, 0xe0, 0xe0, 0xff),

            meter_warning: Color::from_rgba8(0xd8, 0xa2, 0x3a, 0xff),
            meter_critical: Color::from_rgba8(0xd8, 0x5a, 0x5a, 0xff),
        };

        palette.refresh_from_hyprcolor();

        palette
    }

    pub fn refresh_from_hyprcolor(&mut self) {
        let Some(hyprcolor) = hyprcolor::load() else {
            return;
        };

        self.accent = hyprcolor.accent;
        self.slot_active_bg = hyprcolor.accent;
        self.slot_active_text = hyprcolor.foreground;
    }
}
