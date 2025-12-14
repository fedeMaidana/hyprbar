use iced::widget::text;
use iced::{Element};
use crate::theme;

// Por ahora es estático, luego le pasaremos temperatura real
pub fn view<'a, Message>() -> Element<'a, Message> {
    text(" +22°C")
        .size(theme::FONT_SIZE)
        .color(theme::TEXT_MAIN)
        .font(theme::FONT)
        .into()
}