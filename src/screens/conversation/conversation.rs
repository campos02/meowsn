use crate::contact_repository::ContactRepository;
use crate::models::contact::Contact;
use crate::models::message;
use crate::models::switchboard_and_participants::SwitchboardAndParticipants;
use crate::screens::conversation::bordered_container::bordered_container;
use crate::screens::conversation::toggle_button::toggle_button;
use crate::sqlite::Sqlite;
use iced::font::{Family, Style, Weight};
use iced::widget::{
    button, column, container, horizontal_space, rich_text, row, scrollable, span, svg, text,
    text_editor, vertical_space,
};
use iced::{Center, Element, Fill, Font, Task, Theme, widget};
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
    UserDisplayNameUpdated(Arc<String>),
    UserDisplayPictureUpdated(Cow<'static, [u8]>),
    MsnpEvent(Box<Event>),
    NewSwitchboard(Arc<String>, Arc<Switchboard>),
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
    user_display_name: Arc<String>,
    switchboards: HashMap<Arc<String>, Arc<Switchboard>>,
    contact_repository: ContactRepository,
    participants: HashMap<Arc<String>, Contact>,
    last_participant: Option<Contact>,
    messages: Vec<message::Message>,
    new_message: text_editor::Content,
    user_display_picture: Option<Cow<'static, [u8]>>,
    message_buffer: Vec<message::Message>,
    sqlite: Sqlite,
    focused: bool,
    participant_typing: Option<Arc<String>>,
    user_typing: bool,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}

impl Conversation {
    pub fn new(
        contact_repository: ContactRepository,
        session_id: Arc<String>,
        switchboard: SwitchboardAndParticipants,
        user_email: Arc<String>,
        user_display_name: Arc<String>,
        sqlite: Sqlite,
    ) -> Self {
        let user_display_picture = if let Ok(user) = sqlite.select_user(&user_email)
            && let Some(picture) = user.display_picture
        {
            Some(Cow::Owned(picture))
        } else {
            None
        };

        let mut messages = Vec::new();
        if switchboard.participants.len() > 1
            && let Ok(message_history) = sqlite.select_messages_by_session_id(&session_id)
        {
            messages = message_history;
        }

        let mut participants = HashMap::with_capacity(switchboard.participants.len());
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

            if switchboard.participants.len() == 1
                && let Ok(message_history) = sqlite.select_messages(&user_email, participant)
            {
                messages = message_history;
            }
        }

        let mut switchboards = HashMap::new();
        switchboards.insert(session_id, switchboard.switchboard);

        Self {
            user_email,
            user_display_name,
            switchboards,
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

    pub fn view(&self) -> Element<'_, Message> {
        container(
            row![
                column![
                    row![
                        "To: ",
                        if self.participants.len() == 1 {
                            text(if let Some(contact) = &self.participants.values().next() {
                                &contact.display_name
                            } else {
                                ""
                            })
                            .font(Font {
                                weight: Weight::Bold,
                                ..Font::default()
                            })
                        } else if self.participants.len() > 1 {
                            text(format!("{} participants", self.participants.len()))
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
                            text(format!(
                                "<{}>",
                                if let Some(email) = &self.participants.keys().next() {
                                    &email
                                } else {
                                    ""
                                }
                            ))
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
                                    text(if message.sender == self.user_email {
                                        &self.user_display_name
                                    } else if let Some(participant) =
                                        self.participants.get(&message.sender)
                                    {
                                        &*participant.display_name
                                    } else if let Some(participant) = &self.last_participant
                                        && participant.email == message.sender
                                    {
                                        &*participant.display_name
                                    } else {
                                        &*message.sender
                                    })
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
                        toggle_button(
                            text("B").align_x(Center).font(Font {
                                family: Family::Serif,
                                ..Font::default()
                            }),
                            self.bold
                        )
                        .width(30)
                        .on_press(Message::BoldPressed),
                        toggle_button(
                            text("I").align_x(Center).font(Font {
                                family: Family::Serif,
                                ..Font::default()
                            }),
                            self.italic
                        )
                        .width(30)
                        .on_press(Message::ItalicPressed),
                        toggle_button(
                            text("U").align_x(Center).font(Font {
                                family: Family::Serif,
                                ..Font::default()
                            }),
                            self.underline
                        )
                        .width(30)
                        .on_press(Message::UnderlinePressed),
                        toggle_button(
                            text("S").align_x(Center).font(Font {
                                family: Family::Serif,
                                ..Font::default()
                            }),
                            self.strikethrough
                        )
                        .width(30)
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
                        && let Some(contact) = &self.participants.values().next()
                        && let Some(picture) = contact.display_picture.clone()
                    {
                        bordered_container(
                            widget::image(widget::image::Handle::from_bytes(Box::from(picture))),
                            10.0,
                        )
                        .width(100)
                    } else if self.participants.len() > 1 {
                        container(
                            column(self.participants.values().map(|participant| {
                                row![
                                    if let Some(picture) = participant.display_picture.clone() {
                                        bordered_container(
                                            widget::image(widget::image::Handle::from_bytes(
                                                Box::from(picture),
                                            )),
                                            5.0,
                                        )
                                        .width(40)
                                    } else {
                                        bordered_container(
                                            svg(crate::svg::default_display_picture()),
                                            5.0,
                                        )
                                        .width(40)
                                    },
                                    text(&*participant.display_name).size(14)
                                ]
                                .spacing(5)
                                .align_y(Center)
                                .width(110)
                                .into()
                            }))
                            .spacing(5),
                        )
                    } else if let Some(last_participant) = &self.last_participant
                        && let Some(picture) = &last_participant.display_picture
                    {
                        bordered_container(
                            widget::image(widget::image::Handle::from_bytes(Box::from(
                                picture.clone(),
                            ))),
                            10.0,
                        )
                        .width(100)
                    } else {
                        bordered_container(svg(crate::svg::default_display_picture()), 10.0)
                            .width(100)
                    },
                    vertical_space().height(Fill),
                    if let Some(picture) = self.user_display_picture.clone() {
                        bordered_container(
                            widget::image(widget::image::Handle::from_bytes(Box::from(picture))),
                            10.0,
                        )
                        .width(100)
                    } else {
                        bordered_container(svg(crate::svg::default_display_picture()), 10.0)
                            .width(100)
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
                let Some(switchboard) = self.switchboards.values().next().cloned() else {
                    return action;
                };

                if let text_editor::Action::Edit(text_editor::Edit::Enter) = edit_action
                    && !self.new_message.text().trim().is_empty()
                {
                    let message = message::Message {
                        sender: self.user_email.clone(),
                        receiver: if self.participants.len() == 1 {
                            self.participants
                                .values()
                                .next()
                                .map(|participant| participant.email.clone())
                        } else if self.participants.is_empty() {
                            self.last_participant
                                .as_ref()
                                .map(|participant| participant.email.clone())
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
                            crate::Message::UnitResult,
                        )));
                    } else {
                        self.message_buffer.push(message);

                        if let Some(last_participant) = self.last_participant.clone() {
                            action = Some(Action::RunTask(Task::perform(
                                async move { switchboard.invite(&last_participant.email).await },
                                crate::Message::UnitResult,
                            )));
                        }
                    }
                } else {
                    self.new_message.perform(edit_action);
                    let email = self.user_email.clone();

                    if !self.user_typing {
                        self.user_typing = true;
                        action = Some(Action::UserTypingTimeout(Task::perform(
                            async move { switchboard.send_typing_user(&email).await },
                            crate::Message::UnitResult,
                        )));
                    }
                };
            }

            Message::SendNudge => {
                let Some(switchboard) = self.switchboards.values().next().cloned() else {
                    return action;
                };

                let message = message::Message {
                    sender: self.user_email.clone(),
                    receiver: if self.participants.len() == 1 {
                        self.participants
                            .values()
                            .next()
                            .map(|participant| participant.email.clone())
                    } else if self.participants.is_empty() {
                        self.last_participant
                            .as_ref()
                            .map(|participant| participant.email.clone())
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

                self.new_message = text_editor::Content::new();
                if !self.participants.is_empty() {
                    let _ = self.sqlite.insert_message(&message);
                    self.messages.push(message);

                    action = Some(Action::RunTask(Task::perform(
                        async move { switchboard.send_nudge().await },
                        crate::Message::UnitResult,
                    )));
                } else {
                    self.message_buffer.push(message);
                    if let Some(last_participant) = self.last_participant.clone() {
                        action = Some(Action::RunTask(Task::perform(
                            async move { switchboard.invite(&last_participant.email).await },
                            crate::Message::UnitResult,
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

            Message::UserDisplayNameUpdated(display_name) => {
                self.user_display_name = display_name;
            }

            Message::NewSwitchboard(session_id, switchboard) => {
                self.switchboards.insert(session_id, switchboard);
            }

            Message::MsnpEvent(event) => match *event {
                Event::TypingNotification { email } => {
                    if self.participant_typing.is_none() {
                        self.participant_typing =
                            if let Some(participant) = self.participants.get(&email) {
                                Some(participant.display_name.clone())
                            } else {
                                Some(Arc::new(email))
                            };

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
                            .summary(
                                format!(
                                    "{} said:",
                                    if let Some(participant) =
                                        self.participants.get(&message.sender)
                                    {
                                        &participant.display_name
                                    } else if let Some(participant) = &self.last_participant
                                        && participant.email == message.sender
                                    {
                                        &*participant.display_name
                                    } else {
                                        &message.sender
                                    }
                                )
                                .as_str(),
                            )
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
                        text: format!(
                            "{} just sent you a nudge!",
                            if let Some(participant) = self.participants.get(&sender) {
                                &participant.display_name
                            } else if let Some(participant) = &self.last_participant
                                && participant.email == sender
                            {
                                &*participant.display_name
                            } else {
                                &sender
                            }
                        ),
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
                    if self.participants.len() == 1 && !self.message_buffer.is_empty() {
                        let Some(switchboard) = self.switchboards.values().next().cloned() else {
                            return action;
                        };

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
                            crate::Message::Unit,
                        )));
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
                        let Some(switchboard) = self.switchboards.values().next().cloned() else {
                            return action;
                        };

                        let email = participant.email.clone();
                        tasks.push(Task::perform(
                            async move {
                                switchboard
                                    .request_contact_display_picture(&email, &msn_object)
                                    .await
                            },
                            crate::Message::UnitResult,
                        ));
                    }
                }

                if self.participants.len() == 1
                    && let Some(participant) = self.participants.values().next()
                {
                    tasks.push(Task::done(crate::Message::ContactFocused(
                        participant.email.clone(),
                    )));
                } else if self.participants.is_empty()
                    && let Some(participant) = &self.last_participant
                {
                    tasks.push(Task::done(crate::Message::ContactFocused(
                        participant.email.clone(),
                    )));
                }

                action = Some(Action::RunTask(Task::batch(tasks)));
            }

            Message::Unfocused => self.focused = false,
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

    pub fn contains_switchboard(&self, session_id: &Arc<String>) -> bool {
        self.switchboards.contains_key(session_id)
    }

    pub fn get_title(&self) -> String {
        if !self.participants.is_empty() {
            let mut title = "".to_string();
            for participant in self.participants.values() {
                title.push_str(&participant.display_name);
                title.push_str(", ");
            }

            title.pop();
            title.pop();
            title.push_str(" - Conversation");
            title
        } else if let Some(last_participant) = &self.last_participant {
            let mut title = (*last_participant.display_name).clone();
            title.push_str(" - Conversation");
            title
        } else {
            "Conversation".to_string()
        }
    }

    pub fn leave_switchboards_task(&self) -> Task<crate::Message> {
        let mut tasks = Vec::with_capacity(self.switchboards.len());
        for switchboard in self.switchboards.values() {
            let switchboard = switchboard.clone();
            tasks.push(Task::perform(
                async move { switchboard.disconnect().await },
                crate::Message::UnitResult,
            ));
        }

        Task::batch(tasks)
    }
}
