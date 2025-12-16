use iced::{widget::text, alignment::Vertical};
use iced::Element;
use crate::theme::palette;

pub fn view<'a, Message>(date_str: &'a str) -> Element<'a, Message> {
    text(date_str)
        .size(palette::typography::F_SIZE_MAIN)
        .color(palette::colors::C_TEXT)
        .font(palette::typography::FONT)
        .line_height(text::LineHeight::Relative(1.0))
        .align_y(Vertical::Center)
        .into()
}