use iced::widget::{row, text};
use iced::Element;
use crate::theme::palette;

pub fn view<'a, Message>() -> Element<'a, Message>
where Message: 'a
{
    row![
        text("1")
            .size(16)
            .color(palette::colors::C_TEXT)
            .size(palette::typography::F_SIZE_MAIN),
        text("2")
            .size(16)
            .color(palette::colors::C_TEXT)
            .size(palette::typography::F_SIZE_MAIN),
        text("3")
            .size(16)
            .color(palette::colors::C_TEXT)
            .size(palette::typography::F_SIZE_MAIN)
    ]
    .spacing(20)
    .into()
}