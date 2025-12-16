use iced::widget::{button, container};
use iced::{Background, Border, Shadow, Theme};
use crate::theme::palette::{colors, dim};

// ─── WIDGET STYLES ───────────────────────────────────────────────────

pub fn bar(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::BAR_BG)),
        ..Default::default()
    }
}

pub fn glass_pill(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(colors::C_TEXT),
        background: Some(Background::Color(colors::GLASS_BG)),
        border: Border {
            radius: dim::PILL_RADIUS.into(),
            width: 0.0,
            color: colors::GLASS_BORDER,
        },
        shadow: Shadow {
            color: colors::GLASS_SHADOW,
            offset: dim::SHADOW_OFFSET,
            blur_radius: dim::SHADOW_BLUR,
        },
    }
}

pub fn transparent_btn(_: &Theme, _: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: colors::C_TEXT,
        ..Default::default()
    }
}