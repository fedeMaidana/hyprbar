use iced::widget::container;
use iced::{Element, Length, alignment::Vertical};
use crate::theme::{palette, styles};

pub fn pill<'a, Message: 'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    let dynamic_height = (palette::dim::BAR_HEIGHT as f32) - 6.0;

    container(content)
        .padding(palette::dim::PILL_PADDING)
        .style(styles::glass_pill)
        .height(Length::Fixed(dynamic_height))
        .align_y(Vertical::Center)
        .into()
}