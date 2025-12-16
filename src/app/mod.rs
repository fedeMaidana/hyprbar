pub mod state;
pub mod messages;
pub mod layout;

pub use self::state::Bar;

use iced::{Alignment, Element, Length, Padding, Task, Theme, Color};
use iced::widget::{container, row, Space};
use iced_layershell::{Application, Appearance};

use crate::theme::{palette, styles};

use self::messages::Message;
use self::layout::{view_left, view_center, view_right};

impl Application for Bar {
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Task<Message>) {
        Bar::new()
    }

    fn namespace(&self) -> String { "hyprbar".into() }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        // Delegamos la lógica al State
        self.update(message)
    }

    fn view(&self) -> Element<'_, Message> {
        let content = row![
            view_left(self),
            Space::with_width(Length::Fill),
            view_center(self),
            Space::with_width(Length::Fill),
            view_right(self),
        ]
        .padding(Padding::from([0, 10]))
        .align_y(Alignment::Center)
        .width(Length::Fill);

        container(content)
            .style(styles::bar)
            .into()
    }

    fn style(&self, _: &Self::Theme) -> Appearance {
        Appearance {
            background_color: Color::TRANSPARENT,
            text_color: palette::colors::C_TEXT
        }
    }
}