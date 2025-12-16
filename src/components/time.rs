use iced::widget::text;
use iced::Element;
use crate::theme::palette;

pub fn view<'a, Message>(time_str: &'a str) -> Element<'a, Message> {
    text(time_str)
        .size(palette::typography::F_SIZE_MAIN)
        .color(palette::colors::C_TEXT)
        .into()
}