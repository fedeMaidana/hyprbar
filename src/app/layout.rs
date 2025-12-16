use iced::widget::row;
use iced::{Alignment, Element};
use crate::theme::palette;
use crate::{ui::pill::pill, components};
use super::state::Bar;
use super::messages::Message;

pub fn view_left(state: &Bar) -> Element<'_, Message> {
    row![
        pill(components::arch::view()),
        pill(components::date::view(&state.date_str)),
        pill(components::time::view(&state.time_str)),
        pill(components::weather::view()),
    ]
    .spacing(palette::dim::MODULE_SPACING)
    .align_y(Alignment::Center)
    .into()
}

pub fn view_center(_state: &Bar) -> Element<'_, Message> {
    row![
        pill(components::notifications::view()),
        pill(components::workspaces::view()),
        pill(components::control_center::view()),
    ]
    .spacing(palette::dim::MODULE_SPACING)
    .align_y(Alignment::Center)
    .into()
}

pub fn view_right(_state: &Bar) -> Element<'_, Message> {
    pill(components::profile::view())
}