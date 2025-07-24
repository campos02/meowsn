use crate::contact_repository::ContactRepository;
use crate::enums::contact_list_status::ContactListStatus;
use crate::models::contact::Contact;
use crate::msnp_listener::Input;
use crate::sqlite::Sqlite;
use crate::switchboard_and_participants::SwitchboardAndParticipants;
use iced::border::radius;
use iced::font::Weight;
use iced::futures::channel::mpsc::Sender;
use iced::widget::{button, column, container, pick_list, row, svg, text, text_input};
use iced::{Background, Border, Center, Color, Element, Fill, Font, Task, Theme, widget};
use image::imageops::FilterType;
use msnp11_sdk::{Client, Event, MsnpList, MsnpStatus, PersonalMessage};
use rfd::FileDialog;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

pub enum Action {
    SignOut(Task<crate::Message>),
    RunTask(Task<crate::Message>),
}

#[derive(Clone)]
pub enum Message {
    PersonalMessageChanged(String),
    PersonalMessageSubmit,
    StatusSelected(ContactListStatus),
    Conversation(Contact),
    NotificationServerEvent(Event),
    SwitchboardEvent(Arc<String>, Event),
    BlockContact(Arc<String>),
    UnblockContact(Arc<String>),
    RemoveContact(Arc<String>),
    AddContact,
}

pub struct Contacts {
    email: Arc<String>,
    display_picture: Option<Cow<'static, [u8]>>,
    display_name: String,
    personal_message: String,
    status: Option<ContactListStatus>,
    contact_repository: ContactRepository,
    contacts: Vec<Contact>,
    client: Arc<Client>,
    orphan_switchboards: HashMap<String, SwitchboardAndParticipants>,
    sqlite: Sqlite,
    msnp_subscription_sender: Option<Sender<Input>>,
}

impl Contacts {
    pub fn new(
        email: Arc<String>,
        personal_message: String,
        client: Arc<Client>,
        sqlite: Sqlite,
        msnp_subscription_sender: Option<Sender<Input>>,
    ) -> Self {
        if let Ok(user) = sqlite.select_user(&email) {
            if let Some(picture) = user.display_picture {
                return Self {
                    email,
                    display_picture: Some(Cow::Owned(picture)),
                    display_name: String::new(),
                    personal_message,
                    status: Some(ContactListStatus::Online),
                    contact_repository: ContactRepository::new(),
                    contacts: Vec::new(),
                    client,
                    orphan_switchboards: HashMap::new(),
                    sqlite,
                    msnp_subscription_sender,
                };
            }
        }

        Self {
            email,
            display_picture: None,
            display_name: String::new(),
            personal_message,
            status: Some(ContactListStatus::Online),
            contact_repository: ContactRepository::new(),
            contacts: Vec::new(),
            client,
            orphan_switchboards: HashMap::new(),
            sqlite,
            msnp_subscription_sender,
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
                        .on_press(Message::AddContact),
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
                            button(if contact.new_messages {
                                text(&*contact.display_name).font(Font {
                                    weight: Weight::Bold,
                                    ..Font::default()
                                })
                            } else {
                                text(&*contact.display_name)
                            })
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

                let _ = self
                    .sqlite
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
                    let picture_path = FileDialog::new()
                        .add_filter("Images", &["png", "jpeg", "jpg"])
                        .set_directory("/")
                        .set_title("Select a display picture")
                        .pick_file();

                    if let Some(picture_path) = picture_path {
                        if let Ok(picture) = image::open(&picture_path) {
                            let mut bytes = Vec::new();
                            if picture
                                .resize_to_fill(200, 200, FilterType::Triangle)
                                .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
                                .is_ok()
                            {
                                let client = self.client.clone();
                                if let Ok(hash) = client.set_display_picture(bytes.clone()) {
                                    let _ =
                                        self.sqlite.insert_display_picture(bytes.as_slice(), &hash);

                                    let _ =
                                        self.sqlite.update_user_display_picture(&self.email, &hash);

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
                            client: Some(self.client.clone()),
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
                    let guid = self.contacts[contact].guid.clone();
                    self.contacts.remove(contact);

                    let client = self.client.clone();
                    action = Some(Action::RunTask(Task::perform(
                        async move { client.remove_contact_from_forward_list(&guid).await },
                        crate::Message::EmptyResultFuture,
                    )));
                }
            }

            Message::AddContact => {
                action = Some(Action::RunTask(Task::done(crate::Message::OpenAddContact(
                    self.client.clone(),
                ))));
            }

            Message::Conversation(contact) => {
                let mut session_id = String::new();
                if let Some(switchboard) = self
                    .orphan_switchboards
                    .iter()
                    .find(|switchboard| switchboard.1.participants.contains(&contact.email))
                {
                    session_id = switchboard.0.clone();
                }

                if session_id.is_empty() {
                    action = Some(Action::RunTask(Task::done(
                        crate::Message::OpenConversation {
                            contact_repository: self.contact_repository.clone(),
                            email: self.email.clone(),
                            contact_email: contact.email,
                            client: self.client.clone(),
                        },
                    )));
                } else if let Some(switchboard) = self.orphan_switchboards.remove(&session_id) {
                    action = Some(Action::RunTask(Task::done(
                        crate::Message::CreateConversationWithSwitchboard {
                            contact_repository: self.contact_repository.clone(),
                            email: self.email.clone(),
                            switchboard,
                        },
                    )));
                }
            }

            Message::NotificationServerEvent(event) => match event {
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
                    let contact = Contact {
                        email: Arc::new(email),
                        display_name: Arc::new(display_name),
                        guid: Arc::new(guid),
                        lists,
                        status: None,
                        personal_message: None,
                        display_picture: None,
                        new_messages: false,
                    };

                    self.contacts.push(contact.clone());
                    self.contact_repository.add_contacts(&[contact]);
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
                            contact.email.clone(),
                        ))));

                        self.contact_repository.update_contacts(&[contact.clone()]);
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
                            contact.email.clone(),
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
                            contact.email.clone(),
                        ))));

                        self.contact_repository.update_contacts(&[contact.clone()]);
                    }

                    self.contacts
                        .sort_unstable_by_key(|contact| contact.status.is_none());
                }

                Event::SessionAnswered(switchboard) => {
                    if let Ok(Some(session_id)) = switchboard.get_session_id() {
                        self.orphan_switchboards.insert(
                            session_id,
                            SwitchboardAndParticipants {
                                switchboard: switchboard.clone(),
                                participants: Vec::new(),
                            },
                        );

                        if let Some(ref mut sender) = self.msnp_subscription_sender {
                            let _ = sender.start_send(Input::NewSwitchboard(switchboard.clone()));
                        }
                    }
                }

                _ => (),
            },

            Message::SwitchboardEvent(session_id, event) => match event {
                Event::ParticipantInSwitchboard { email } => {
                    if let Some(switchboard) = self.orphan_switchboards.get_mut(&*session_id) {
                        switchboard.participants.push(Arc::new(email));
                    }
                }

                Event::ParticipantLeftSwitchboard { email } => {
                    if let Some(switchboard) = self.orphan_switchboards.get_mut(&*session_id) {
                        switchboard
                            .participants
                            .retain(|participant| **participant != email);
                    }
                }

                Event::Nudge { email } | Event::TextMessage { email, .. } => {
                    let contact = self
                        .contacts
                        .iter_mut()
                        .find(|contact| *contact.email == email);

                    if let Some(contact) = contact {
                        contact.new_messages = true;
                    }
                }

                _ => (),
            },
        }

        action
    }
}
