use iced::widget::{button, text};
use iced::{Element, Length, Padding, alignment::{Horizontal, Vertical}};

use crate::theme;

pub fn view<'a, Message>( ) -> Element<'a, Message> where Message: Clone + 'a {
    let visual_correction = Padding::new(0.0).right(4.0);

    button(
        text("\u{f0f3}")
        .size(theme::ICON_SIZE)
        .color(theme::TEXT_MAIN)
        .font(theme::FONT)
        .width(Length::Shrink)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
    )
    .style(theme::style_transparent_btn)
    .padding(visual_correction)
    .into()
}