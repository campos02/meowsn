use crate::contact_repository::ContactRepository;
use crate::models::contact::Contact;
use crate::models::message;
use crate::models::switchboard_and_participants::SwitchboardAndParticipants;
use crate::sqlite::Sqlite;
use iced::border::radius;
use iced::font::{Family, Style, Weight};
use iced::widget::{
    button, column, container, horizontal_space, rich_text, row, scrollable, span, svg, text,
    text_editor, vertical_space,
};
use iced::{Border, Center, Color, Element, Fill, Font, Task, Theme, widget};
use msnp11_sdk::{Event, PlainText, Switchboard};
use notify_rust::Notification;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

pub enum Action {
    ParticipantTypingTimeout,
    UserTypingTimeout(Task<crate::Message>),
    RunTask(Task<crate::Message>),
}

#[derive(Clone)]
pub enum Message {
    Edit(text_editor::Action),
    ContactUpdated(Arc<String>),
    UserDisplayPictureUpdated(Cow<'static, [u8]>),
    MsnpEvent(Box<Event>),
    Focused,
    Unfocused,
    ParticipantTypingTimeout,
    UserTypingTimeout,
    BoldPressed,
    ItalicPressed,
    UnderlinePressed,
    StrikethroughPressed,
    SendNudge,
}

pub struct Conversation {
    user_email: Arc<String>,
    switchboard: Arc<Switchboard>,
    session_id: String,
    contact_repository: ContactRepository,
    participants: HashMap<Arc<String>, Contact>,
    last_participant: Option<Contact>,
    messages: Vec<message::Message>,
    new_message: text_editor::Content,
    user_display_picture: Option<Cow<'static, [u8]>>,
    message_buffer: Vec<message::Message>,
    sqlite: Sqlite,
    focused: bool,
    participant_typing: Option<String>,
    user_typing: bool,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}

impl Conversation {
    pub fn new(
        contact_repository: ContactRepository,
        switchboard: SwitchboardAndParticipants,
        user_email: Arc<String>,
        sqlite: Sqlite,
    ) -> Self {
        let user_display_picture = if let Ok(user) = sqlite.select_user(&user_email)
            && let Some(picture) = user.display_picture
        {
            Some(Cow::Owned(picture))
        } else {
            None
        };

        let session_id = switchboard
            .switchboard
            .get_session_id()
            .unwrap_or_default()
            .unwrap_or_default();

        let mut messages = Vec::new();
        if switchboard.participants.len() > 1
            && let Ok(message_history) = sqlite.select_messages_by_session_id(&session_id)
        {
            messages = message_history;
        }

        let mut participants = HashMap::new();
        participants.reserve(switchboard.participants.len());

        for participant in &switchboard.participants {
            participants.insert(
                participant.clone(),
                contact_repository
                    .get_contact(participant)
                    .unwrap_or(Contact {
                        email: participant.clone(),
                        display_name: participant.clone(),
                        ..Contact::default()
                    }),
            );

            if switchboard.participants.len() == 1 {
                if let Ok(message_history) = sqlite.select_messages(&user_email, participant) {
                    messages = message_history;
                }
            }
        }

        Self {
            user_email,
            switchboard: switchboard.switchboard,
            session_id,
            contact_repository,
            participants,
            last_participant: None,
            messages,
            new_message: text_editor::Content::new(),
            user_display_picture,
            message_buffer: Vec::new(),
            sqlite,
            focused: true,
            participant_typing: None,
            user_typing: false,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let default_picture = include_bytes!("../../assets/default_display_picture.svg");
        let picture_border = |theme: &Theme| container::Style {
            border: Border {
                color: theme.palette().text,
                width: 1.0,
                radius: radius(10.0),
            },
            ..Default::default()
        };

        container(
            row![
                column![
                    row![
                        "To: ",
                        if self.participants.len() == 1 {
                            let display_name = &self
                                .participants
                                .iter()
                                .next()
                                .expect("Could not get next contact")
                                .1
                                .display_name;

                            text(&**display_name).font(Font {
                                weight: Weight::Bold,
                                ..Font::default()
                            })
                        } else if let Some(last_participant) = &self.last_participant {
                            text(&*last_participant.display_name).font(Font {
                                weight: Weight::Bold,
                                ..Font::default()
                            })
                        } else {
                            text("")
                        },
                        " ",
                        if self.participants.len() == 1 {
                            let email = &self
                                .participants
                                .iter()
                                .next()
                                .expect("Could not get next contact")
                                .1
                                .email;

                            text(format!("<{email}>"))
                        } else if let Some(last_participant) = &self.last_participant {
                            text(format!("<{}>", last_participant.email))
                        } else {
                            text("")
                        },
                    ]
                    .width(Fill),
                    scrollable(column(self.messages.iter().map(|message| {
                        let history = |theme: &Theme| text::Style {
                            color: if !message.is_history {
                                Some(theme.palette().text)
                            } else {
                                Some(theme.extended_palette().secondary.weak.color)
                            },
                        };

                        column![
                            if !message.is_nudge {
                                row![
                                    text(&*message.sender)
                                        .font(Font {
                                            weight: Weight::Bold,
                                            ..Font::default()
                                        })
                                        .style(history),
                                    text(" said:").style(history)
                                ]
                            } else {
                                row![text(format!("⸺⸺\n{}\n⸺⸺", message.text)).style(history)]
                            },
                            if !message.is_nudge {
                                container(
                                    rich_text([span(message.text.replace("\r\n", "\n"))
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
                                        })])
                                    .style(history),
                                )
                                .padding(10)
                            } else {
                                container(horizontal_space().height(7))
                            }
                        ]
                        .into()
                    })))
                    .anchor_bottom()
                    .height(Fill),
                    if let Some(participant) = &self.participant_typing {
                        text(format!("{participant} is writing a message...")).size(14)
                    } else {
                        text("").size(14)
                    },
                    row![
                        button(text("B").align_x(Center).font(Font {
                            family: Family::Serif,
                            ..Font::default()
                        }))
                        .width(30)
                        .style(|theme: &Theme, status| {
                            if self.bold {
                                button::primary(theme, status)
                            } else {
                                match status {
                                    button::Status::Hovered | button::Status::Pressed => {
                                        button::primary(theme, status)
                                    }

                                    button::Status::Active | button::Status::Disabled => {
                                        button::secondary(theme, status)
                                            .with_background(Color::TRANSPARENT)
                                    }
                                }
                            }
                        })
                        .on_press(Message::BoldPressed),
                        button(text("I").align_x(Center).font(Font {
                            family: Family::Serif,
                            ..Font::default()
                        }))
                        .width(30)
                        .style(|theme: &Theme, status| {
                            if self.italic {
                                button::primary(theme, status)
                            } else {
                                match status {
                                    button::Status::Hovered | button::Status::Pressed => {
                                        button::primary(theme, status)
                                    }

                                    button::Status::Active | button::Status::Disabled => {
                                        button::secondary(theme, status)
                                            .with_background(Color::TRANSPARENT)
                                    }
                                }
                            }
                        })
                        .on_press(Message::ItalicPressed),
                        button(text("U").align_x(Center).font(Font {
                            family: Family::Serif,
                            ..Font::default()
                        }))
                        .width(30)
                        .style(|theme: &Theme, status| {
                            if self.underline {
                                button::primary(theme, status)
                            } else {
                                match status {
                                    button::Status::Hovered | button::Status::Pressed => {
                                        button::primary(theme, status)
                                    }

                                    button::Status::Active | button::Status::Disabled => {
                                        button::secondary(theme, status)
                                            .with_background(Color::TRANSPARENT)
                                    }
                                }
                            }
                        })
                        .on_press(Message::UnderlinePressed),
                        button(text("S").align_x(Center).font(Font {
                            family: Family::Serif,
                            ..Font::default()
                        }))
                        .width(30)
                        .style(|theme: &Theme, status| {
                            if self.strikethrough {
                                button::primary(theme, status)
                            } else {
                                match status {
                                    button::Status::Hovered | button::Status::Pressed => {
                                        button::primary(theme, status)
                                    }

                                    button::Status::Active | button::Status::Disabled => {
                                        button::secondary(theme, status)
                                            .with_background(Color::TRANSPARENT)
                                    }
                                }
                            }
                        })
                        .on_press(Message::StrikethroughPressed),
                        button("Nudge").on_press(Message::SendNudge)
                    ]
                    .spacing(5),
                    text_editor(&self.new_message)
                        .height(100)
                        .on_action(Message::Edit),
                ]
                .spacing(10),
                column![
                    if self.participants.len() == 1
                        && let Some(picture) = &self
                            .participants
                            .iter()
                            .next()
                            .expect("Could not get next contact")
                            .1
                            .display_picture
                    {
                        container(widget::image(widget::image::Handle::from_bytes(Box::from(
                            picture.clone(),
                        ))))
                        .width(100)
                        .style(picture_border)
                        .padding(3)
                    } else if let Some(last_participant) = &self.last_participant
                        && let Some(display_picture) = &last_participant.display_picture
                    {
                        container(widget::image(widget::image::Handle::from_bytes(Box::from(
                            display_picture.clone(),
                        ))))
                        .width(100)
                        .style(picture_border)
                        .padding(3)
                    } else {
                        container(svg(svg::Handle::from_memory(default_picture)).width(100))
                            .style(picture_border)
                            .padding(3)
                    },
                    vertical_space().height(Fill),
                    if let Some(picture) = self.user_display_picture.clone() {
                        container(widget::image(widget::image::Handle::from_bytes(Box::from(
                            picture,
                        ))))
                        .width(100)
                        .style(picture_border)
                        .padding(3)
                    } else {
                        container(svg(svg::Handle::from_memory(default_picture)))
                            .width(100)
                            .style(picture_border)
                            .padding(3)
                    }
                ]
            ]
            .spacing(10),
        )
        .padding(30)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Option<Action> {
        let mut action = None;
        match message {
            Message::Edit(edit_action) => {
                if let text_editor::Action::Edit(text_editor::Edit::Enter) = edit_action {
                    let message = message::Message {
                        sender: self.user_email.clone(),
                        receiver: if self.participants.len() == 1 {
                            Some(
                                self.participants
                                    .iter()
                                    .next()
                                    .expect("Could not get next contact")
                                    .1
                                    .email
                                    .clone(),
                            )
                        } else if self.participants.is_empty()
                            && let Some(last_participant) = &self.last_participant
                        {
                            Some(last_participant.email.clone())
                        } else {
                            None
                        },
                        is_nudge: false,
                        text: self.new_message.text().replace("\n", "\r\n"),
                        bold: self.bold,
                        italic: self.italic,
                        underline: self.underline,
                        strikethrough: self.strikethrough,
                        session_id: None,
                        color: "0".to_string(),
                        is_history: false,
                    };

                    let switchboard = self.switchboard.clone();
                    self.new_message = text_editor::Content::new();

                    if !self.participants.is_empty() {
                        let plain_text = PlainText {
                            bold: message.bold,
                            italic: message.italic,
                            underline: message.underline,
                            strikethrough: message.strikethrough,
                            color: message.color.clone(),
                            text: message.text.clone(),
                        };

                        let _ = self.sqlite.insert_message(&message);
                        self.messages.push(message);

                        action = Some(Action::RunTask(Task::perform(
                            async move { switchboard.send_text_message(&plain_text).await },
                            crate::Message::EmptyResultFuture,
                        )));
                    } else {
                        self.message_buffer.push(message);

                        if let Some(last_participant) = self.last_participant.clone() {
                            action = Some(Action::RunTask(Task::perform(
                                async move { switchboard.invite(&last_participant.email).await },
                                crate::Message::EmptyResultFuture,
                            )));
                        }
                    }
                } else {
                    self.new_message.perform(edit_action);

                    let switchboard = self.switchboard.clone();
                    let email = self.user_email.clone();

                    if !self.user_typing {
                        self.user_typing = true;
                        action = Some(Action::UserTypingTimeout(Task::perform(
                            async move { switchboard.send_typing_user(&email).await },
                            crate::Message::EmptyResultFuture,
                        )));
                    }
                };
            }

            Message::SendNudge => {
                let message = message::Message {
                    sender: self.user_email.clone(),
                    receiver: if self.participants.len() == 1 {
                        Some(
                            self.participants
                                .iter()
                                .next()
                                .expect("Could not get next contact")
                                .1
                                .email
                                .clone(),
                        )
                    } else if self.participants.is_empty()
                        && let Some(last_participant) = &self.last_participant
                    {
                        Some(last_participant.email.clone())
                    } else {
                        None
                    },
                    is_nudge: true,
                    text: "You just sent a nudge!".to_string(),
                    bold: false,
                    italic: false,
                    underline: false,
                    strikethrough: false,
                    session_id: None,
                    color: "0".to_string(),
                    is_history: false,
                };

                let switchboard = self.switchboard.clone();
                self.new_message = text_editor::Content::new();

                if !self.participants.is_empty() {
                    let _ = self.sqlite.insert_message(&message);
                    self.messages.push(message);

                    action = Some(Action::RunTask(Task::perform(
                        async move { switchboard.send_nudge().await },
                        crate::Message::EmptyResultFuture,
                    )));
                } else {
                    self.message_buffer.push(message);

                    if let Some(last_participant) = self.last_participant.clone() {
                        action = Some(Action::RunTask(Task::perform(
                            async move { switchboard.invite(&last_participant.email).await },
                            crate::Message::EmptyResultFuture,
                        )));
                    }
                }
            }

            Message::ContactUpdated(contact) => {
                if let Some(contact) = self.contact_repository.get_contact(&contact) {
                    self.participants.insert(contact.email.clone(), contact);
                }
            }

            Message::UserDisplayPictureUpdated(picture) => {
                self.user_display_picture = Some(picture);
            }

            Message::MsnpEvent(event) => match *event {
                Event::TypingNotification { email } => {
                    if self.participant_typing.is_none() {
                        self.participant_typing = Some(email);
                        action = Some(Action::ParticipantTypingTimeout);
                    }
                }

                Event::TextMessage { email, message } => {
                    let message = message::Message {
                        sender: Arc::new(email),
                        receiver: Some(self.user_email.clone()),
                        is_nudge: false,
                        text: message.text,
                        bold: message.bold,
                        italic: message.italic,
                        underline: message.underline,
                        strikethrough: message.strikethrough,
                        session_id: None,
                        color: message.color,
                        is_history: false,
                    };

                    let _ = self.sqlite.insert_message(&message);
                    if !self.focused {
                        let _ = Notification::new()
                            .summary(format!("{} said:", message.sender).as_str())
                            .body(&message.text)
                            .show();
                    }

                    self.messages.push(message);
                    self.participant_typing = None;
                }

                Event::Nudge { email } => {
                    let sender = Arc::new(email);
                    let message = message::Message {
                        sender: sender.clone(),
                        receiver: Some(self.user_email.clone()),
                        is_nudge: true,
                        text: format!("{sender} just sent you a nudge!"),
                        bold: false,
                        italic: false,
                        underline: false,
                        strikethrough: false,
                        session_id: None,
                        color: "0".to_string(),
                        is_history: false,
                    };

                    let _ = self.sqlite.insert_message(&message);
                    if !self.focused {
                        let _ = Notification::new()
                            .summary("New message")
                            .body(&message.text)
                            .show();
                    }

                    self.messages.push(message);
                    self.participant_typing = None;
                }

                Event::ParticipantInSwitchboard { email } => {
                    let email = Arc::new(email);
                    self.participants.insert(
                        email.clone(),
                        self.contact_repository
                            .get_contact(&email)
                            .unwrap_or(Contact {
                                email: email.clone(),
                                display_name: email.clone(),
                                ..Contact::default()
                            }),
                    );

                    // If the switchboard had no prior participants
                    if self.participants.len() == 1 {
                        let switchboard = self.switchboard.clone();

                        if !self.message_buffer.is_empty() {
                            let messages = self.message_buffer.clone();
                            self.messages.reserve(messages.len());

                            for message in self.message_buffer.drain(..) {
                                let _ = self.sqlite.insert_message(&message);
                                self.messages.push(message);
                            }

                            action = Some(Action::RunTask(Task::perform(
                                async move {
                                    for message in messages {
                                        let message = PlainText {
                                            bold: message.bold,
                                            italic: message.italic,
                                            underline: message.underline,
                                            strikethrough: message.strikethrough,
                                            color: message.color,
                                            text: message.text,
                                        };

                                        let _ = switchboard.send_text_message(&message).await;
                                    }
                                },
                                crate::Message::Empty,
                            )));
                        }
                    }
                }

                Event::ParticipantLeftSwitchboard { email } => {
                    let email = Arc::new(email);
                    let removed = self.participants.remove(&email);

                    if self.participants.is_empty()
                        && let Some(contact) = removed
                    {
                        self.last_participant = Some(contact);
                    }
                }

                _ => (),
            },

            Message::Focused => {
                self.focused = true;
                let mut tasks = Vec::new();

                for participant in self.participants.values() {
                    if participant.display_picture.is_none()
                        && let Some(status) = &participant.status
                        && let Some(msn_object) = status.msn_object_string.clone()
                    {
                        let switchboard = self.switchboard.clone();
                        let email = participant.email.clone();

                        tasks.push(Task::perform(
                            async move {
                                switchboard
                                    .request_contact_display_picture(&email, &msn_object)
                                    .await
                            },
                            crate::Message::EmptyResultFuture,
                        ));
                    }
                }

                action = Some(Action::RunTask(Task::batch(tasks)));
            }

            Message::Unfocused => {
                self.focused = false;
            }

            Message::ParticipantTypingTimeout => self.participant_typing = None,
            Message::UserTypingTimeout => self.user_typing = false,
            Message::BoldPressed => self.bold = !self.bold,
            Message::ItalicPressed => self.italic = !self.italic,
            Message::UnderlinePressed => self.underline = !self.underline,
            Message::StrikethroughPressed => self.strikethrough = !self.strikethrough,
        }

        action
    }

    pub fn get_participants(&self) -> &HashMap<Arc<String>, Contact> {
        &self.participants
    }

    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }

    pub fn leave_switchboard_task(&self) -> Task<crate::Message> {
        let switchboard = self.switchboard.clone();
        Task::perform(
            async move { switchboard.disconnect().await },
            crate::Message::EmptyResultFuture,
        )
    }
}
