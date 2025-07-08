use iced::border::radius;
use iced::widget::{
    column, container, image, rich_text, row, span, text, text_editor, vertical_space,
};
use iced::{Border, Element, Fill, Font, Theme, font};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    Edit(text_editor::Action),
}

pub struct Conversation {
    contact: Arc<String>,
    message: text_editor::Content,
}

impl Conversation {
    pub fn new(contact: Arc<String>) -> Self {
        Self {
            contact,
            message: text_editor::Content::new(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        container(
            row![
                column![
                    row![
                        "To: ",
                        text(&*self.contact).font(Font {
                            weight: font::Weight::Bold,
                            ..Font::default()
                        }),
                        " ",
                        "<testing@example.com>"
                    ]
                    .width(Fill),
                    column![
                        row![text(&*self.contact), " said:"],
                        container(rich_text([span("Test Message")]).height(Fill)).padding(10)
                    ],
                    text_editor(&self.message)
                        .height(105)
                        .on_action(Message::Edit),
                ]
                .spacing(20),
                column![
                    container(image("assets/default_display_picture.png").width(100))
                        .style(|theme: &Theme| container::Style {
                            border: Border {
                                color: theme.palette().text,
                                width: 1.0,
                                radius: radius(10.0),
                            },
                            ..Default::default()
                        })
                        .padding(3),
                    vertical_space().height(Fill),
                    container(image("assets/default_display_picture.png").width(100))
                        .style(|theme: &Theme| container::Style {
                            border: Border {
                                color: theme.palette().text,
                                width: 1.0,
                                radius: radius(10.0),
                            },
                            ..Default::default()
                        })
                        .padding(3)
                ]
            ]
            .spacing(10),
        )
        .padding(30)
        .into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Edit(action) => {
                self.message.perform(action);
            }
        }
    }
}
