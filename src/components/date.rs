use iced::{widget::text, alignment::Vertical};
use iced::Element;
use crate::theme;

pub fn view<'a, Message>(date_str: &'a str) -> Element<'a, Message> {
    text(date_str)
        .size(theme::FONT_SIZE)
        .color(theme::TEXT_MAIN)
        .font(theme::FONT)
        .line_height(text::LineHeight::Relative(1.0))
        .align_y(Vertical::Center)
        .into()
}