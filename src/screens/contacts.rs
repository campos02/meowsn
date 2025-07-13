use crate::client_wrapper::ClientWrapper;
use crate::enums::contact_list_status::ContactListStatus;
use crate::models::contact::Contact;
use crate::sqlite::Sqlite;
use iced::border::radius;
use iced::widget::{button, column, container, pick_list, row, svg, text, text_input};
use iced::{Background, Border, Center, Color, Element, Fill, Task, Theme, widget};
use msnp11_sdk::{Client, Event, MsnpList, MsnpStatus, PersonalMessage};
use std::sync::Arc;

pub enum Action {
    PersonalSettings {
        client: Option<ClientWrapper>,
        display_name: Option<String>,
    },

    Conversation(Contact),
    SignOut(Task<crate::Message>),
    StatusSelected(Task<crate::Message>),
    PersonalMessageSubmit(Task<crate::Message>),
    ContactUpdated(Contact),
}

#[derive(Debug, Clone)]
pub enum Message {
    PersonalMessageChanged(String),
    PersonalMessageSubmit,
    StatusSelected(ContactListStatus),
    Conversation(Contact),
    MsnpEvent(Event),
}

pub struct Contacts {
    email: Arc<String>,
    display_name: String,
    personal_message: String,
    status: Option<ContactListStatus>,
    contacts: Vec<Contact>,
    client: Arc<Client>,
    sqlite: Sqlite,
}

impl Contacts {
    pub fn new(
        email: Arc<String>,
        personal_message: String,
        client: Arc<Client>,
        sqlite: Sqlite,
    ) -> Self {
        Self {
            email,
            display_name: String::new(),
            personal_message,
            status: Some(ContactListStatus::Online),
            contacts: Vec::new(),
            client,
            sqlite,
        }
    }

    pub fn view(&self) -> Element<Message> {
        container(
            column![
                row![
                    container(svg("assets/default_display_picture.svg").width(70))
                        .style(|theme: &Theme| container::Style {
                            border: Border {
                                color: theme.palette().text,
                                width: 1.0,
                                radius: radius(10.0),
                            },
                            ..Default::default()
                        })
                        .padding(3),
                    column![
                        row![
                            text(&self.display_name).size(14),
                            pick_list(
                                ContactListStatus::ALL,
                                self.status.as_ref(),
                                Message::StatusSelected
                            )
                            .text_size(14)
                            .style(|theme: &Theme, status| {
                                match status {
                                    pick_list::Status::Active => {
                                        let mut list = pick_list::default(theme, status);
                                        list.background = Background::Color(Color::TRANSPARENT);
                                        list.border.width = 0.0;
                                        list
                                    }

                                    _ => {
                                        let mut list = pick_list::default(theme, status);
                                        list.border.color =
                                            theme.extended_palette().secondary.strong.color;
                                        list.background = Background::Color(Color::TRANSPARENT);
                                        list
                                    }
                                }
                            })
                        ]
                        .align_y(Center)
                        .spacing(20),
                        text_input("<Type a personal message>", &self.personal_message)
                            .size(14)
                            .on_input(Message::PersonalMessageChanged)
                            .on_submit(Message::PersonalMessageSubmit)
                            .style(|theme: &Theme, status| {
                                match status {
                                    text_input::Status::Active | text_input::Status::Disabled => {
                                        let mut list = text_input::default(theme, status);
                                        list.border.width = 0.0;
                                        list
                                    }

                                    _ => {
                                        let mut list = text_input::default(theme, status);
                                        list.border.color =
                                            theme.extended_palette().secondary.strong.color;
                                        list
                                    }
                                }
                            }),
                    ]
                    .spacing(5)
                ]
                .spacing(10),
                button("Add a contact")
                    .style(|theme: &Theme, status| {
                        match status {
                            button::Status::Hovered | button::Status::Pressed => {
                                button::primary(theme, status)
                            }

                            button::Status::Active | button::Status::Disabled => {
                                button::secondary(theme, status).with_background(Color::TRANSPARENT)
                            }
                        }
                    })
                    .on_press(Message::PersonalMessageSubmit),
                column(self.contacts.iter().map(|contact| {
                    row![
                        row![
                            svg(if let Some(status) = &contact.status {
                                match status.status {
                                    MsnpStatus::Busy | MsnpStatus::OnThePhone => {
                                        "assets/default_display_picture_busy.svg"
                                    }

                                    MsnpStatus::Away
                                    | MsnpStatus::Idle
                                    | MsnpStatus::BeRightBack
                                    | MsnpStatus::OutToLunch => {
                                        "assets/default_display_picture_away.svg"
                                    }

                                    _ => "assets/default_display_picture.svg",
                                }
                            } else {
                                if contact.lists.contains(&MsnpList::BlockList) {
                                    "assets/default_display_picture_offline_blocked.svg"
                                } else {
                                    "assets/default_display_picture_offline.svg"
                                }
                            })
                            .width(30),
                            button(text(&*contact.display_name))
                                .on_press(Message::Conversation(contact.clone()))
                                .style(|theme: &Theme, status| match status {
                                    button::Status::Hovered | button::Status::Pressed => {
                                        button::secondary(theme, status)
                                    }

                                    button::Status::Active | button::Status::Disabled => {
                                        button::secondary(theme, status)
                                            .with_background(Color::TRANSPARENT)
                                    }
                                })
                                .width(Fill)
                        ]
                        .align_y(Center),
                        row![
                            if !contact.lists.contains(&MsnpList::BlockList) {
                                button(text("Block").size(15))
                                    .on_press(Message::PersonalMessageSubmit)
                            } else {
                                button(text("Unblock").size(15))
                                    .on_press(Message::PersonalMessageSubmit)
                            },
                            button(text("Delete").size(15))
                                .on_press(Message::PersonalMessageSubmit)
                        ]
                        .spacing(5)
                    ]
                    .align_y(Center)
                    .spacing(10)
                    .width(Fill)
                    .into()
                }))
                .spacing(10)
            ]
            .spacing(10),
        )
        .padding(15)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Option<Action> {
        let mut action: Option<Action> = None;
        match message {
            Message::PersonalMessageChanged(personal_message) => {
                self.personal_message = personal_message;
            }

            Message::PersonalMessageSubmit => {
                let client = self.client.clone();
                let personal_message = PersonalMessage {
                    psm: self.personal_message.clone(),
                    current_media: "".to_string(),
                };

                self.sqlite
                    .update_personal_message(&*self.email, &personal_message.psm);

                action = Some(Action::PersonalMessageSubmit(Task::batch([
                    Task::perform(
                        async move { client.set_personal_message(&personal_message).await },
                        crate::Message::EmptyResultFuture,
                    ),
                    widget::focus_next(),
                ])));
            }

            Message::StatusSelected(status) => match status {
                ContactListStatus::PersonalSettings => {
                    action = Some(Action::PersonalSettings {
                        client: Some(ClientWrapper {
                            personal_message: String::new(),
                            inner: self.client.clone(),
                        }),
                        display_name: Some(self.display_name.trim().to_string()),
                    })
                }

                ContactListStatus::SignOut => {
                    let client = self.client.clone();
                    action = Some(Action::SignOut(Task::perform(
                        async move { client.disconnect().await },
                        crate::Message::EmptyResultFuture,
                    )))
                }

                _ => {
                    let client = self.client.clone();
                    let presence = match status {
                        ContactListStatus::Online => MsnpStatus::Online,
                        ContactListStatus::Busy => MsnpStatus::Busy,
                        ContactListStatus::Away => MsnpStatus::Away,
                        ContactListStatus::AppearOffline => MsnpStatus::AppearOffline,
                        _ => MsnpStatus::Online,
                    };

                    action = Some(Action::StatusSelected(Task::perform(
                        async move { client.set_presence(presence).await },
                        crate::Message::EmptyResultFuture,
                    )));

                    self.status = Some(status);
                }
            },

            Message::Conversation(contact) => action = Some(Action::Conversation(contact)),
            Message::MsnpEvent(event) => match event {
                Event::DisplayName(display_name) => {
                    self.display_name = display_name;
                    self.display_name.insert(0, ' ');
                }

                Event::ContactInForwardList {
                    email,
                    display_name,
                    guid,
                    lists,
                    ..
                } => {
                    self.contacts.push(Contact {
                        email: Arc::new(email),
                        display_name: Arc::new(display_name),
                        guid: Arc::new(guid),
                        lists: Arc::new(lists),
                        status: None,
                        personal_message: None,
                    });
                }

                Event::PresenceUpdate {
                    email,
                    display_name,
                    presence,
                } => {
                    let contact = self
                        .contacts
                        .iter_mut()
                        .find(|contact| *contact.email == email);

                    if let Some(contact) = contact {
                        contact.display_name = Arc::new(display_name);
                        contact.status = Some(Arc::new(presence));
                        action = Some(Action::ContactUpdated(contact.clone()));
                    }

                    self.contacts
                        .sort_unstable_by_key(|contact| contact.status.is_none());
                }

                Event::PersonalMessageUpdate {
                    email,
                    personal_message,
                } => {
                    let contact = self
                        .contacts
                        .iter_mut()
                        .find(|contact| *contact.email == email);

                    if let Some(contact) = contact {
                        contact.personal_message = Some(Arc::new(personal_message.psm));
                        action = Some(Action::ContactUpdated(contact.clone()));
                    }
                }

                Event::ContactOffline { email } => {
                    let contact = self
                        .contacts
                        .iter_mut()
                        .find(|contact| *contact.email == email);

                    if let Some(contact) = contact {
                        contact.status = None;
                        action = Some(Action::ContactUpdated(contact.clone()));
                    }

                    self.contacts
                        .sort_unstable_by_key(|contact| contact.status.is_none());
                }

                _ => (),
            },
        }

        action
    }
}
