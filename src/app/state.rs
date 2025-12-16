use chrono::{Local, Timelike, DateTime};
use iced::Task;
use super::messages::Message;

pub struct Bar {
    pub time_str: String,
    pub date_str: String,
    pub last_tick: DateTime<Local>,
}

impl Bar {
    pub fn new() -> (Self, Task<Message>) {
        let now = Local::now();
        (
            Self {
                time_str: now.format("%H:%M").to_string(),
                date_str: now.format("%d/%m").to_string(),
                last_tick: now,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let now = Local::now();
                if now.minute() != self.last_tick.minute() {
                    self.time_str = now.format("%H:%M").to_string();
                    self.date_str = now.format("%d/%m").to_string();
                    self.last_tick = now;
                }
            }
        }
        Task::none()
    }
}