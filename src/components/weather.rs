use iced::widget::text;
use iced::{Element};
use crate::theme::palette;

pub fn view<'a, Message>() -> Element<'a, Message> {
    text(" +22°C")
        .size(palette::typography::F_SIZE_MAIN)
        .color(palette::colors::C_TEXT)
        .font(palette::typography::FONT)
        .into()
}