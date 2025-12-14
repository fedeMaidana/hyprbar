use iced::widget::{container, row, Space};
use iced::{Alignment, alignment::Vertical, Element, Length, Padding, Task, Theme, Color};
use iced_layershell::actions::LayershellCustomActions;
use iced_layershell::{Application, Appearance};
use chrono::{Datelike, Local, Timelike};

use crate::theme;
use crate::components;

pub struct Bar {
    time_str: String,
    date_str: String,
    last_minute: u8,
    last_day: u8,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
}

impl TryFrom<Message> for LayershellCustomActions {
    type Error = Message;
    fn try_from(msg: Message) -> Result<Self, Self::Error> { Err(msg) }
}
impl From<LayershellCustomActions> for Message {
    fn from(_: LayershellCustomActions) -> Self { Message::Tick }
}

impl Application for Bar {
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Task<Message>) {
        let now = Local::now();
        (
            Self {
                time_str: now.format("%H:%M").to_string(),
                date_str: now.format("%d/%m").to_string(),
                last_minute: now.minute() as u8,
                last_day: now.day() as u8,
            },
            Task::none(),
        )
    }

    fn namespace(&self) -> String { String::from("rusty_bar") }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let now = Local::now();
                let current_minute = now.minute() as u8;
                if current_minute != self.last_minute {
                    self.time_str = now.format("%H:%M").to_string();
                    self.last_minute = current_minute;
                    let current_day = now.day() as u8;
                    if current_day != self.last_day {
                        self.date_str = now.format("%d/%m").to_string();
                        self.last_day = current_day;
                    }
                }
            }
        }
        Task::none()
    }

    fn style(&self, _theme: &Self::Theme) -> Appearance {
        Appearance {
            background_color: Color::TRANSPARENT,
            text_color: Color::WHITE
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // --- IZQUIERDA: CÁPSULAS SEPARADAS ---
        let left_section = row![
            self.pill(components::arch::view()), // Logo en su propia pastilla
            self.pill(components::date::view(&self.date_str)), // Fecha sola
            self.pill(components::time::view(&self.time_str)), // Hora sola
            self.pill(components::weather::view()), // Clima solo
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        // --- CENTRO: Workspaces y Controles ---
        let center_section = row![
            self.pill(components::notifications::view()),
            self.pill(components::workspaces::view()),
            self.pill(components::control_center::view()),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // --- DERECHA ---
        let right_section = self.pill(components::profile::view());

        // --- LAYOUT PRINCIPAL ---
        let bar_content = row![
            left_section,
            Space::with_width(Length::Fill),
            center_section,
            Space::with_width(Length::Fill),
            right_section
        ]
        .padding(Padding::from([0, 10]))
        .align_y(Alignment::Center)
        .width(Length::Fill);

        container(bar_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::style_bar)
            .into()
    }
}

// Métodos auxiliares de UI para la struct Bar
impl Bar {
    fn pill<'a>(&self, content: Element<'a, Message>) -> Element<'a, Message> {
        container(content)
            .padding(theme::PILL_PADDING)
            .style(theme::style_pill)
            .height(Length::Fixed(24.0))
            .align_y(Vertical::Center)
            .into()
    }
}