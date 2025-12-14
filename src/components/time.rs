use iced::widget::text;
use iced::Element;
use crate::theme;

// CORRECCIÓN: time_str: &'a str
pub fn view<'a, Message>(time_str: &'a str) -> Element<'a, Message> {
    text(time_str)
        .size(theme::FONT_SIZE)
        .color(theme::TEXT_MAIN)
        .into()
}