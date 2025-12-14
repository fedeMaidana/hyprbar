use iced::widget::{text};
use iced::{Element};
use crate::theme;

pub fn view<'a, Message>() -> Element<'a, Message> where Message: 'a {
    text("User")
        .size(theme::FONT_SIZE)
        .color(theme::TEXT_MAIN)
        .into()
}