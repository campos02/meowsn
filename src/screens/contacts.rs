use iced::border::radius;
use iced::widget::{container, image};
use iced::{Border, Element, Fill, Theme};

#[derive(Debug, Clone)]
pub enum Message {
    EmailChanged(String),
}

pub struct Contacts {
    email: String,
}

impl Contacts {
    pub fn new() -> Self {
        Self {
            email: String::new(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        container(
            container(image("assets/default_display_picture.png").width(130)).style(
                |theme: &Theme| container::Style {
                    border: Border {
                        color: theme.palette().text,
                        width: 1.0,
                        radius: radius(10.0),
                    },
                    ..Default::default()
                },
            ),
        )
        .padding(50)
        .center_x(Fill)
        .into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::EmailChanged(email) => self.email = email,
        }
    }
}

impl Default for Contacts {
    fn default() -> Self {
        Self::new()
    }
}
