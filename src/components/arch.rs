use iced::widget::{container, text};
use iced::{Element, Length, Padding, alignment::{Horizontal, Vertical}};

use crate::theme::palette;

pub fn view<'a, Message: 'a>() -> Element<'a, Message> {
    let visual_correction = Padding::new(0.0).right(4.0);

    container(
        text("\u{f303}")
            .size(palette::typography::F_SIZE_ICON)
            .color(palette::colors::C_TEXT)
            .font(palette::typography::FONT)
            .width(Length::Shrink)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
    )
    .padding(visual_correction)
    .into()
}