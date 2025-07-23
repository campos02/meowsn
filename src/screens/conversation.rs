use crate::models::contact::Contact;
use crate::models::message;
use crate::sqlite::Sqlite;
use iced::border::radius;
use iced::font::{Style, Weight};
use iced::widget::{
    column, container, horizontal_space, rich_text, row, span, svg, text, text_editor,
    vertical_space,
};
use iced::{Border, Element, Fill, Font, Task, Theme, widget};
use msnp11_sdk::{Event, PlainText, Switchboard};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub enum Message {
    Edit(text_editor::Action),
    ContactUpdated(Contact),
    UserDisplayPictureUpdated(Cow<'static, [u8]>),
    MsnpEvent(Event),
}

pub struct Conversation {
    user_email: Arc<String>,
    switchboard: Arc<Switchboard>,
    session_id: String,
    contacts: HashMap<Arc<String>, Contact>,
    messages: Vec<message::Message>,
    new_message: text_editor::Content,
    user_display_picture: Option<Cow<'static, [u8]>>,
}

impl Conversation {
    pub fn new(
        switchboard: Arc<Switchboard>,
        user_email: Arc<String>,
        contacts: HashMap<Arc<String>, Contact>,
        sqlite: Sqlite,
    ) -> Self {
        let mut user_display_picture = None;
        if let Ok(user) = sqlite.select_user(&user_email) {
            if let Some(picture) = user.display_picture {
                user_display_picture = Some(Cow::Owned(picture))
            }
        }

        let session_id = switchboard
            .get_session_id()
            .unwrap_or_default()
            .unwrap_or_default();

        Self {
            user_email,
            switchboard,
            session_id,
            contacts,
            messages: Vec::new(),
            new_message: text_editor::Content::new(),
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
                        if self.contacts.len() == 1 {
                            let display_name = &self
                                .contacts
                                .iter()
                                .next()
                                .expect("Could not get next contact")
                                .1
                                .display_name;

                            text(&**display_name).font(Font {
                                weight: Weight::Bold,
                                ..Font::default()
                            })
                        } else {
                            text("")
                        },
                        " ",
                        if self.contacts.len() == 1 {
                            let email = &self
                                .contacts
                                .iter()
                                .next()
                                .expect("Could not get next contact")
                                .1
                                .email;

                            text(format!("<{email}>"))
                        } else {
                            text("")
                        },
                    ]
                    .width(Fill),
                    column(self.messages.iter().map(|message| {
                        column![
                            if !message.is_nudge {
                                row![
                                    text(&*message.sender).font(Font {
                                        weight: Weight::Bold,
                                        ..Font::default()
                                    }),
                                    " said:"
                                ]
                            } else {
                                row![column![
                                    text("⸺"),
                                    text(format!("{} sent you a nudge!", &*message.sender)),
                                    text("⸺")
                                ]]
                            },
                            if !message.is_nudge {
                                container(rich_text([span(message.text.replace("\r\n", "\n"))
                                    .underline(message.underline)
                                    .strikethrough(message.strikethrough)
                                    .font(Font {
                                        weight: if message.bold {
                                            Weight::Bold
                                        } else {
                                            Weight::Normal
                                        },
                                        style: if message.italic {
                                            Style::Italic
                                        } else {
                                            Style::Normal
                                        },
                                        ..Font::default()
                                    })]))
                                .padding(10)
                            } else {
                                container(horizontal_space().height(7))
                            }
                        ]
                        .into()
                    }))
                    .height(Fill),
                    text_editor(&self.new_message)
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

    pub fn update(&mut self, message: Message) -> Task<crate::Message> {
        match message {
            Message::Edit(edit_action) => {
                if let text_editor::Action::Edit(text_editor::Edit::Enter) = edit_action {
                    self.messages.push(message::Message {
                        sender: self.user_email.clone(),
                        receiver: None,
                        is_nudge: false,
                        text: Arc::new(self.new_message.text().replace("\n", "\r\n")),
                        bold: false,
                        italic: false,
                        underline: false,
                        strikethrough: false,
                        color: Arc::new("0".to_string()),
                    });

                    let message = PlainText {
                        bold: false,
                        italic: false,
                        underline: false,
                        strikethrough: false,
                        color: "0".to_string(),
                        text: self.new_message.text().replace("\n", "\r\n"),
                    };

                    let switchboard = self.switchboard.clone();
                    self.new_message = text_editor::Content::new();

                    return Task::perform(
                        async move { switchboard.send_text_message(&message).await },
                        crate::Message::EmptyResultFuture,
                    );
                } else {
                    self.new_message.perform(edit_action);
                }
            }

            Message::ContactUpdated(contact) => {
                let old_contact = self.contacts.get_mut(&contact.email);
                if let Some(old_contact) = old_contact {
                    *old_contact = contact;
                }
            }

            Message::UserDisplayPictureUpdated(picture) => {
                self.user_display_picture = Some(picture);
            }

            Message::MsnpEvent(event) => match event {
                Event::TextMessage { email, message } => {
                    self.messages.push(message::Message {
                        sender: Arc::new(email),
                        receiver: Some(self.user_email.clone()),
                        is_nudge: false,
                        text: Arc::new(message.text),
                        bold: message.bold,
                        italic: message.italic,
                        underline: message.underline,
                        strikethrough: message.strikethrough,
                        color: Arc::new(message.color),
                    });
                }

                Event::Nudge { email } => {
                    let sender = Arc::new(email);
                    self.messages.push(message::Message {
                        sender: sender.clone(),
                        receiver: Some(self.user_email.clone()),
                        is_nudge: true,
                        text: Arc::new(format!("{sender} sent you a nudge!")),
                        bold: false,
                        italic: false,
                        underline: false,
                        strikethrough: false,
                        color: Arc::new(String::from("0")),
                    });
                }

                _ => (),
            },
        }

        Task::none()
    }

    pub fn get_contacts(&self) -> &HashMap<Arc<String>, Contact> {
        &self.contacts
    }

    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }
}
