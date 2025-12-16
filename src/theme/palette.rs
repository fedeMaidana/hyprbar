use iced::{Color, Font, Padding, Vector};

// ─── DESIGN TOKENS ───────────────────────────────────────────────────

pub mod colors {
    use super::Color;

    pub const C_TEXT: Color = Color::WHITE;
    pub const BAR_BG: Color = Color::TRANSPARENT;

    pub const GLASS_BG: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.5);
    pub const GLASS_BORDER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.10);
    pub const GLASS_SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.90);
}

pub mod dim {
    use super::{Padding, Vector};

    pub const BAR_HEIGHT: u32 = 30;
    pub const MODULE_SPACING: f32 = 15.0;
    pub const PILL_RADIUS: f32 = 12.0;

    pub const PILL_PADDING: Padding = Padding {
        top: 7.0, right: 12.0, bottom: 3.0, left: 12.0,
    };

    pub const SHADOW_OFFSET: Vector = Vector::new(0.0, 2.0);
    pub const SHADOW_BLUR: f32 = 4.0;
}

pub mod typography {
    use super::Font;

    pub const FONT: Font = Font::with_name("FiraCode Nerd Font");
    pub const F_SIZE_MAIN: f32 = 10.0;
    pub const F_SIZE_ICON: f32 = 14.0;
}