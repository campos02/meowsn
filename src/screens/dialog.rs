use iced::widget::{button, column, container, text};
use iced::{Center, Element, Fill};

#[derive(Debug, Clone)]
pub enum Message {
    OkPressed,
}

pub enum Action {
    OkPressed,
}

pub struct Dialog {
    message: String,
}

impl Dialog {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn view(&self) -> Element<Message> {
        container(
            column![
                text(&*self.message).height(Fill),
                button("Ok").on_press(Message::OkPressed)
            ]
            .spacing(20)
            .align_x(Center),
        )
        .center_x(Fill)
        .padding(20)
        .into()
    }

    pub fn update(&mut self, _message: Message) -> Option<Action> {
        Some(Action::OkPressed)
    }
}
