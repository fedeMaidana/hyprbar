use vello::peniko::Color;

use super::hyprcolor;

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
}

impl Palette {
    pub fn dark() -> Self {
        let hyprcolor = hyprcolor::load();

        Self {
            pill_bg: Color::from_rgba8(0x00, 0x00, 0x00, 0x80),
            text_primary: Color::from_rgba8(0xf5, 0xf5, 0xf7, 0xff),
            text_secondary: Color::from_rgba8(0xa0, 0xa0, 0xa8, 0xff),
            accent: hyprcolor
                .map(|palette| palette.accent)
                .unwrap_or_else(|| Color::from_rgba8(0x9a, 0x8c, 0xff, 0xff)),
            shadow: Color::from_rgba8(0x00, 0x00, 0x00, 0x40),

            slot_active_bg: hyprcolor
                .map(|palette| palette.accent)
                .unwrap_or_else(|| Color::from_rgba8(0xe8, 0xe4, 0xff, 0xff)),
            slot_active_text: hyprcolor
                .map(|palette| palette.foreground)
                .unwrap_or_else(|| Color::from_rgba8(0x2a, 0x25, 0x4a, 0xff)),

            slot_inactive_bg: Color::from_rgba8(0x40, 0x40, 0x48, 0xa0),
            slot_empty_bg: Color::from_rgba8(0x30, 0x30, 0x38, 0x60),
        }
    }
}
