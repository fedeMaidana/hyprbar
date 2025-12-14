use iced::widget::{button, container};
use iced::{Background, Border, Color, Font, Shadow, Vector, Padding};


pub const FONT: Font = Font::with_name("FiraCode Nerd Font");

pub const FONT_SIZE: f32 = 10.0;
pub const ICON_SIZE: f32 = 14.0;

pub const PILL_PADDING: Padding = Padding {
    top: 7.0,
    right: 12.0,
    bottom: 3.0,
    left: 12.0,
};
pub const PILL_RADIUS: f32 = 12.0;

pub const BAR_BG: Color = Color::TRANSPARENT;
pub const TEXT_MAIN: Color = Color::WHITE;

pub const GLASS_BG: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.5);
pub const GLASS_BORDER_COLOR: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.10);
pub const GLASS_SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.90);

// --- ESTILOS ---

pub fn style_pill(_theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT_MAIN),
        background: Some(Background::Color(GLASS_BG)),
        border: Border {
            radius: PILL_RADIUS.into(),
            width: 0.0,
            color: GLASS_BORDER_COLOR,
        },
        shadow: Shadow {
            color: GLASS_SHADOW,
            offset: Vector::new(0.0, 2.0),
            blur_radius: 4.0
        }
    }
}

pub fn style_bar(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BAR_BG)),
        ..Default::default()
    }
}

pub fn style_transparent_btn(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: TEXT_MAIN,
        ..Default::default()
    }
}