use iced_layershell::actions::LayershellCustomActions;

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