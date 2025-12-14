use iced::widget::{row, text};
use iced::Element;
use crate::theme;

pub fn view<'a, Message>() -> Element<'a, Message>
where Message: 'a
{
    row![
        text("1")
            .size(16)
            .color(theme::TEXT_MAIN)
            .size(theme::FONT_SIZE),
        text("2")
            .size(16)
            .color(theme::TEXT_MAIN)
            .size(theme::FONT_SIZE),
        text("3")
            .size(16)
            .color(theme::TEXT_MAIN)
            .size(theme::FONT_SIZE)
    ]
    .spacing(20)
    .into()
}