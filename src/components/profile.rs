use iced::widget::{text};
use iced::{Element};
use crate::theme::palette;

pub fn view<'a, Message>() -> Element<'a, Message> where Message: 'a {
    text("User")
        .size(palette::typography::F_SIZE_MAIN)
        .color(palette::colors::C_TEXT)
        .into()
}