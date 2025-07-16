use crate::client_wrapper::ClientWrapper;
use crate::enums::contact_list_status::ContactListStatus;
use crate::models::contact::Contact;
use crate::sqlite::Sqlite;
use iced::border::radius;
use iced::widget::{button, column, container, pick_list, row, svg, text, text_input};
use iced::{Background, Border, Center, Color, Element, Fill, Task, Theme, widget};
use image::imageops::FilterType;
use msnp11_sdk::{Client, Event, MsnpList, MsnpStatus, PersonalMessage};
use rfd::FileDialog;
use std::borrow::Cow;
use std::io::Cursor;
use std::sync::Arc;

pub enum Action {
    SignOut(Task<crate::Message>),
    RunTask(Task<crate::Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    PersonalMessageChanged(String),
    PersonalMessageSubmit,
    StatusSelected(ContactListStatus),
    Conversation(Contact),
    MsnpEvent(Event),
    BlockContact(Arc<String>),
    UnblockContact(Arc<String>),
    RemoveContact(Arc<String>),
}

pub struct Contacts {
    email: Arc<String>,
    display_picture: Option<Cow<'static, [u8]>>,
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
        if let Some(user) = sqlite.select_user(&email) {
            if let Some(picture) = user.display_picture {
                return Self {
                    email,
                    display_picture: Some(Cow::Owned(picture)),
                    display_name: String::new(),
                    personal_message,
                    status: Some(ContactListStatus::Online),
                    contacts: Vec::new(),
                    client,
                    sqlite,
                };
            }
        }

        Self {
            email,
            display_picture: None,
            display_name: String::new(),
            personal_message,
            status: Some(ContactListStatus::Online),
            contacts: Vec::new(),
            client,
            sqlite,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let default_picture = include_bytes!("../../assets/default_display_picture.svg");

        container(
            column![
                row![
                    if let Some(picture) = self.display_picture.clone() {
                        container(widget::image(widget::image::Handle::from_bytes(Box::from(
                            picture,
                        ))))
                        .width(70)
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
                            .width(70)
                            .style(|theme: &Theme| container::Style {
                                border: Border {
                                    color: theme.palette().text,
                                    width: 1.0,
                                    radius: radius(10.0),
                                },
                                ..Default::default()
                            })
                            .padding(3)
                    },
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
                row![
                    svg(svg::Handle::from_memory(include_bytes!(
                        "../../assets/add_contact.svg"
                    )))
                    .width(30),
                    button("Add a contact")
                        .style(|theme: &Theme, status| {
                            match status {
                                button::Status::Hovered | button::Status::Pressed => {
                                    button::primary(theme, status)
                                }

                                button::Status::Active | button::Status::Disabled => {
                                    button::secondary(theme, status)
                                        .with_background(Color::TRANSPARENT)
                                }
                            }
                        })
                        .on_press(Message::PersonalMessageSubmit),
                ]
                .align_y(Center),
                column(self.contacts.iter().map(|contact| {
                    row![
                        row![
                            svg(if let Some(status) = &contact.status {
                                if contact.lists.contains(&MsnpList::BlockList) {
                                    svg::Handle::from_memory(include_bytes!(
                                        "../../assets/default_display_picture_blocked.svg"
                                    ))
                                } else {
                                    match status.status {
                                        MsnpStatus::Busy | MsnpStatus::OnThePhone => {
                                            svg::Handle::from_memory(include_bytes!(
                                                "../../assets/default_display_picture_busy.svg"
                                            ))
                                        }

                                        MsnpStatus::Away
                                        | MsnpStatus::Idle
                                        | MsnpStatus::BeRightBack
                                        | MsnpStatus::OutToLunch => {
                                            svg::Handle::from_memory(include_bytes!(
                                                "../../assets/default_display_picture_away.svg"
                                            ))
                                        }

                                        _ => svg::Handle::from_memory(default_picture),
                                    }
                                }
                            } else if contact.lists.contains(&MsnpList::BlockList) {
                                svg::Handle::from_memory(include_bytes!(
                                    "../../assets/default_display_picture_offline_blocked.svg"
                                ))
                            } else {
                                svg::Handle::from_memory(include_bytes!(
                                    "../../assets/default_display_picture_offline.svg"
                                ))
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
                                    .on_press(Message::BlockContact(contact.email.clone()))
                            } else {
                                button(text("Unblock").size(15))
                                    .on_press(Message::UnblockContact(contact.email.clone()))
                            },
                            button(text("Delete").size(15))
                                .on_press(Message::RemoveContact(contact.email.clone()))
                        ]
                        .spacing(5)
                    ]
                    .align_y(Center)
                    .spacing(10)
                    .width(Fill)
                    .into()
                }))
                .spacing(10)
                .padding(10)
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
                    .update_personal_message(&self.email, &personal_message.psm);

                action = Some(Action::RunTask(Task::batch([
                    Task::perform(
                        async move { client.set_personal_message(&personal_message).await },
                        crate::Message::EmptyResultFuture,
                    ),
                    widget::focus_next(),
                ])));
            }

            Message::StatusSelected(status) => match status {
                ContactListStatus::ChangeDisplayPicture => {
                    let picture = FileDialog::new()
                        .add_filter("Images", &["png", "jpeg", "jpg"])
                        .set_directory("/")
                        .set_title("Select a display picture")
                        .pick_file();

                    if let Some(picture) = picture {
                        if let Ok(picture) = image::open(&picture) {
                            let mut bytes = Vec::new();
                            if picture
                                .resize_to_fill(200, 200, FilterType::Triangle)
                                .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
                                .is_ok()
                            {
                                let client = self.client.clone();
                                if let Ok(hash) = client.set_display_picture(bytes.clone()) {
                                    if self
                                        .sqlite
                                        .update_user_display_picture(&self.email, &hash)
                                        .is_err()
                                    {
                                        self.sqlite.update_user_with_new_display_picture(
                                            &self.email,
                                            bytes.as_slice(),
                                            &hash,
                                        )
                                    }

                                    let cow = Cow::from(bytes);
                                    action = Some(Action::RunTask(Task::done(
                                        crate::Message::UserDisplayPictureUpdated(cow.clone()),
                                    )));

                                    self.display_picture = Some(cow);
                                }
                            }
                        }
                    }
                }

                ContactListStatus::PersonalSettings => {
                    action = Some(Action::RunTask(Task::done(
                        crate::Message::OpenPersonalSettings {
                            client: Some(ClientWrapper {
                                personal_message: String::new(),
                                inner: self.client.clone(),
                            }),
                            display_name: Some(self.display_name.trim().to_string()),
                        },
                    )))
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
                        ContactListStatus::Busy => MsnpStatus::Busy,
                        ContactListStatus::Away => MsnpStatus::Away,
                        ContactListStatus::AppearOffline => MsnpStatus::AppearOffline,
                        _ => MsnpStatus::Online,
                    };

                    action = Some(Action::RunTask(Task::perform(
                        async move { client.set_presence(presence).await },
                        crate::Message::EmptyResultFuture,
                    )));

                    self.status = Some(status);
                }
            },

            Message::BlockContact(email) => {
                let contact = self
                    .contacts
                    .iter_mut()
                    .find(|contact| *contact.email == *email);

                if let Some(contact) = contact {
                    contact.lists.push(MsnpList::BlockList);
                    contact.lists.retain(|list| list != &MsnpList::AllowList);

                    let client = self.client.clone();
                    let email = contact.email.clone();
                    action = Some(Action::RunTask(Task::perform(
                        async move { client.block_contact(&email).await },
                        crate::Message::EmptyResultFuture,
                    )));
                }
            }

            Message::UnblockContact(email) => {
                let contact = self
                    .contacts
                    .iter_mut()
                    .find(|contact| *contact.email == *email);

                if let Some(contact) = contact {
                    contact.lists.retain(|list| list != &MsnpList::BlockList);
                    contact.lists.push(MsnpList::AllowList);

                    let client = self.client.clone();
                    let email = contact.email.clone();
                    action = Some(Action::RunTask(Task::perform(
                        async move { client.unblock_contact(&email).await },
                        crate::Message::EmptyResultFuture,
                    )));
                }
            }

            Message::RemoveContact(email) => {
                let contact = self
                    .contacts
                    .iter_mut()
                    .position(|contact| *contact.email == *email);

                if let Some(contact) = contact {
                    let email = self.contacts[contact].email.clone();
                    self.contacts.remove(contact);

                    let client = self.client.clone();
                    action = Some(Action::RunTask(Task::perform(
                        async move { client.remove_contact_from_forward_list(&email).await },
                        crate::Message::EmptyResultFuture,
                    )));
                }
            }

            Message::Conversation(contact) => {
                action = Some(Action::RunTask(Task::done(
                    crate::Message::OpenConversation(self.email.clone(), contact.clone()),
                )))
            }

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
                        lists,
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
                        action = Some(Action::RunTask(Task::done(crate::Message::ContactUpdated(
                            contact.clone(),
                        ))));
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
                        action = Some(Action::RunTask(Task::done(crate::Message::ContactUpdated(
                            contact.clone(),
                        ))));
                    }
                }

                Event::ContactOffline { email } => {
                    let contact = self
                        .contacts
                        .iter_mut()
                        .find(|contact| *contact.email == email);

                    if let Some(contact) = contact {
                        contact.status = None;
                        action = Some(Action::RunTask(Task::done(crate::Message::ContactUpdated(
                            contact.clone(),
                        ))));
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
