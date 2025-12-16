use iced::widget::{button, text};
use iced::{Element, Length, Padding, alignment::{Horizontal, Vertical}};

use crate::theme::{palette, styles};

pub fn view<'a, Message>( ) -> Element<'a, Message> where Message: Clone + 'a {
    let visual_correction = Padding::new(0.0).right(4.0);

    button(
        text("\u{f0f3}")
        .size(palette::typography::F_SIZE_ICON)
        .color(palette::colors::C_TEXT)
        .font(palette::typography::FONT)
        .width(Length::Shrink)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
    )
    .style(styles::transparent_btn)
    .padding(visual_correction)
    .into()
}