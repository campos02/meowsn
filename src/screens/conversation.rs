use crate::models::contact::Contact;
use iced::border::radius;
use iced::widget::{
    column, container, rich_text, row, span, svg, text, text_editor, vertical_space,
};
use iced::{Border, Element, Fill, Font, Theme, font};

#[derive(Debug, Clone)]
pub enum Message {
    Edit(text_editor::Action),
    ContactUpdated(Contact),
}

pub struct Conversation {
    contact: Contact,
    message: text_editor::Content,
}

impl Conversation {
    pub fn new(contact: Contact) -> Self {
        Self {
            contact,
            message: text_editor::Content::new(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let default_picture = include_bytes!("../../assets/default_display_picture.svg");

        container(
            row![
                column![
                    row![
                        "To: ",
                        text(&*self.contact.display_name).font(Font {
                            weight: font::Weight::Bold,
                            ..Font::default()
                        }),
                        " ",
                        text(format!("<{}>", &*self.contact.email))
                    ]
                    .width(Fill),
                    column![
                        row![text(&*self.contact.display_name), " said:"],
                        container(rich_text([span("Test Message")]).height(Fill)).padding(10)
                    ],
                    text_editor(&self.message)
                        .height(105)
                        .on_action(Message::Edit),
                ]
                .spacing(20),
                column![
                    container(svg(svg::Handle::from_memory(default_picture)).width(100))
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
                    container(svg(svg::Handle::from_memory(default_picture)).width(100))
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

            Message::ContactUpdated(contact) => {
                self.contact = contact;
            }
        }
    }

    pub fn get_contact(&self) -> &Contact {
        &self.contact
    }
}
