use crate::models::contact::Contact;
use crate::sqlite::Sqlite;
use iced::border::radius;
use iced::widget::{
    column, container, rich_text, row, span, svg, text, text_editor, vertical_space,
};
use iced::{Border, Element, Fill, Font, Theme, font, widget};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    Edit(text_editor::Action),
    ContactUpdated(Contact),
    UserDisplayPictureUpdated(Cow<'static, [u8]>),
}

pub struct Conversation {
    contact: Contact,
    message: text_editor::Content,
    user_display_picture: Option<Cow<'static, [u8]>>,
}

impl Conversation {
    pub fn new(user_email: Arc<String>, contact: Contact, sqlite: Sqlite) -> Self {
        let mut user_display_picture = None;

        if let Some(user) = sqlite.select_user(&user_email) {
            if let Some(picture) = user.display_picture {
                user_display_picture = Some(Cow::Owned(picture))
            }
        }

        Self {
            contact,
            message: text_editor::Content::new(),
            user_display_picture,
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
                    if let Some(picture) = self.user_display_picture.clone() {
                        container(widget::image(widget::image::Handle::from_bytes(Box::from(
                            picture,
                        ))))
                        .width(105)
                        .style(|theme: &Theme| container::Style {
                            border: Border {
                                color: theme.palette().text,
                                width: 1.0,
                                radius: radius(10.0),
                            },
                            ..Default::default()
                        })
                        .padding(3)
                    } else {
                        container(svg(svg::Handle::from_memory(default_picture)))
                            .width(100)
                            .style(|theme: &Theme| container::Style {
                                border: Border {
                                    color: theme.palette().text,
                                    width: 1.0,
                                    radius: radius(10.0),
                                },
                                ..Default::default()
                            })
                            .padding(3)
                    }
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

            Message::UserDisplayPictureUpdated(picture) => {
                self.user_display_picture = Some(picture);
            }
        }
    }

    pub fn get_contact(&self) -> &Contact {
        &self.contact
    }
}
