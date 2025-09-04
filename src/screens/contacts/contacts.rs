use crate::contact_repository::ContactRepository;
use crate::enums::contact_list_status::ContactListStatus;
use crate::helpers::pick_display_picture::pick_display_picture;
use crate::models::contact::Contact;
use crate::models::message;
use crate::models::switchboard_and_participants::SwitchboardAndParticipants;
use crate::msnp_listener::Input;
use crate::screens::contacts::bordered_container::bordered_container;
use crate::screens::contacts::contact_map::contact_map;
use crate::screens::contacts::transparent_button::transparent_button;
use crate::sqlite::Sqlite;
use iced::futures::channel::mpsc::Sender;
use iced::futures::executor::block_on;
use iced::widget::{column, container, pick_list, row, scrollable, svg, text, text_input};
use iced::{Background, Center, Color, Element, Padding, Task, Theme, widget};
use msnp11_sdk::{Client, Event, MsnpList, MsnpStatus, PersonalMessage};
use notify_rust::Notification;
use rfd::AsyncFileDialog;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

pub enum Action {
    SignOut(Task<crate::Message>),
    RunTask(Task<crate::Message>),
    NewMessage,
}

#[derive(Clone)]
pub enum Message {
    PersonalMessageChanged(String),
    PersonalMessageSubmit,
    StatusSelected(ContactListStatus),
    Conversation(Contact),
    NotificationServerEvent(Event),
    SwitchboardEvent(Arc<String>, Event),
    RemoveSwitchboard(Arc<String>),
    BlockContact(Arc<String>),
    UnblockContact(Arc<String>),
    RemoveContact(Arc<String>),
    ContactFocused(Arc<String>),
    AddContact,
    UserDisplayPictureUpdated(Cow<'static, [u8]>),
}

pub struct Contacts {
    email: Arc<String>,
    display_picture: Option<Cow<'static, [u8]>>,
    display_name: Arc<String>,
    personal_message: String,
    status: Option<ContactListStatus>,
    contact_repository: ContactRepository,
    online_contacts: HashMap<Arc<String>, Contact>,
    offline_contacts: HashMap<Arc<String>, Contact>,
    client: Arc<Client>,
    orphan_switchboards: HashMap<Arc<String>, SwitchboardAndParticipants>,
    sqlite: Sqlite,
    msnp_subscription_sender: Option<Sender<Input>>,
}

impl Contacts {
    pub fn new(
        email: Arc<String>,
        personal_message: String,
        initial_status: MsnpStatus,
        client: Arc<Client>,
        sqlite: Sqlite,
        msnp_subscription_sender: Option<Sender<Input>>,
    ) -> Self {
        let initial_status = match initial_status {
            MsnpStatus::Busy => ContactListStatus::Busy,
            MsnpStatus::Away => ContactListStatus::Away,
            MsnpStatus::AppearOffline => ContactListStatus::AppearOffline,
            _ => ContactListStatus::Online,
        };

        Self {
            email: email.clone(),
            display_picture: if let Ok(user) = sqlite.select_user(&email) {
                user.display_picture.map(Cow::Owned)
            } else {
                None
            },
            display_name: Arc::new(String::new()),
            personal_message,
            status: Some(initial_status),
            contact_repository: ContactRepository::new(),
            online_contacts: HashMap::new(),
            offline_contacts: HashMap::new(),
            client,
            orphan_switchboards: HashMap::new(),
            sqlite,
            msnp_subscription_sender,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(
            column![
                row![
                    if let Some(picture) = self.display_picture.clone() {
                        bordered_container(widget::image(widget::image::Handle::from_bytes(
                            Box::from(picture),
                        )))
                    } else {
                        bordered_container(svg(crate::svg::default_display_picture()))
                    },
                    column![
                        row![
                            text(format!(" {}", self.display_name)).size(14),
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
                    svg(crate::svg::add_contact()).width(30),
                    transparent_button("Add a contact").on_press(Message::AddContact),
                ]
                .align_y(Center),
                scrollable(column![
                    if !self.online_contacts.is_empty() {
                        column(
                            self.online_contacts
                                .values()
                                .map(|contact| contact_map(contact)),
                        )
                        .spacing(10)
                        .padding(Padding {
                            top: 10.0,
                            right: 10.0,
                            bottom: 0.0,
                            left: 10.0,
                        })
                    } else {
                        column![].height(0).padding(0)
                    },
                    column(
                        self.offline_contacts
                            .values()
                            .map(|contact| contact_map(contact))
                    )
                    .spacing(10)
                    .padding(10)
                ])
            ]
            .spacing(10),
        )
        .padding(15)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Option<Action> {
        let mut action = None;
        match message {
            Message::UserDisplayPictureUpdated(picture) => {
                self.display_picture = Some(picture);
            }

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
                        crate::Message::UnitResult,
                    ),
                    widget::focus_next(),
                ])));
            }

            Message::StatusSelected(status) => match status {
                ContactListStatus::ChangeDisplayPicture => {
                    let picture = AsyncFileDialog::new()
                        .add_filter("Images", &["png", "jpeg", "jpg"])
                        .set_directory("/")
                        .set_title("Select a display picture")
                        .pick_file();

                    action = Some(Action::RunTask(Task::perform(
                        pick_display_picture(
                            picture,
                            self.email.clone(),
                            self.client.clone(),
                            self.sqlite.clone(),
                        ),
                        |result| crate::Message::UserDisplayPictureUpdated(result.ok()),
                    )));
                }

                ContactListStatus::PersonalSettings => {
                    action = Some(Action::RunTask(Task::done(
                        crate::Message::OpenPersonalSettings {
                            client: Some(self.client.clone()),
                            display_name: Some(self.display_name.trim().to_string()),
                        },
                    )));
                }

                ContactListStatus::SignOut => {
                    let client = self.client.clone();
                    action = Some(Action::SignOut(Task::perform(
                        async move { client.disconnect().await },
                        crate::Message::UnitResult,
                    )));
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
                        crate::Message::UnitResult,
                    )));

                    self.status = Some(status);
                }
            },

            Message::RemoveSwitchboard(session_id) => {
                self.orphan_switchboards.remove(&session_id);
            }

            Message::BlockContact(email) => {
                let contact = if let Some(contact) = self.online_contacts.get_mut(&email) {
                    Some(contact)
                } else {
                    self.offline_contacts.get_mut(&email)
                };

                if let Some(contact) = contact {
                    contact.lists.push(MsnpList::BlockList);
                    contact.lists.retain(|list| list != &MsnpList::AllowList);

                    let client = self.client.clone();
                    let email = contact.email.clone();

                    action = Some(Action::RunTask(Task::perform(
                        async move { client.block_contact(&email).await },
                        crate::Message::UnitResult,
                    )));
                }
            }

            Message::UnblockContact(email) => {
                let contact = if let Some(contact) = self.online_contacts.get_mut(&email) {
                    Some(contact)
                } else {
                    self.offline_contacts.get_mut(&email)
                };

                if let Some(contact) = contact {
                    contact.lists.retain(|list| list != &MsnpList::BlockList);
                    contact.lists.push(MsnpList::AllowList);

                    let client = self.client.clone();
                    let email = contact.email.clone();

                    action = Some(Action::RunTask(Task::perform(
                        async move { client.unblock_contact(&email).await },
                        crate::Message::UnitResult,
                    )));
                }
            }

            Message::RemoveContact(email) => {
                let contact = if let Some(contact) = self.online_contacts.get_mut(&email) {
                    Some(contact)
                } else {
                    self.offline_contacts.get_mut(&email)
                };

                if let Some(contact) = contact {
                    let guid = contact.guid.clone();
                    self.online_contacts.remove(&email);
                    self.offline_contacts.remove(&email);

                    let client = self.client.clone();
                    action = Some(Action::RunTask(Task::perform(
                        async move { client.remove_contact_from_forward_list(&guid).await },
                        crate::Message::UnitResult,
                    )));
                }
            }

            Message::AddContact => {
                action = Some(Action::RunTask(Task::done(crate::Message::OpenAddContact(
                    self.client.clone(),
                ))));
            }

            Message::ContactFocused(email) => {
                let contact = if let Some(contact) = self.online_contacts.get_mut(&email) {
                    Some(contact)
                } else {
                    self.offline_contacts.get_mut(&email)
                };

                if let Some(contact) = contact {
                    contact.new_messages = false;
                }
            }

            Message::Conversation(contact) => {
                if contact.status.is_some() && self.status != Some(ContactListStatus::AppearOffline)
                {
                    if let Some((session_id, switchboard)) = self
                        .orphan_switchboards
                        .iter()
                        .find(|(_, switchboard)| switchboard.participants.contains(&contact.email))
                    {
                        action = Some(Action::RunTask(Task::done(
                            crate::Message::CreateConversationWithSwitchboard {
                                contact_repository: self.contact_repository.clone(),
                                email: self.email.clone(),
                                display_name: self.display_name.clone(),
                                session_id: session_id.clone(),
                                switchboard: switchboard.clone(),
                                minimized: false,
                            },
                        )));
                    } else {
                        action = Some(Action::RunTask(Task::done(
                            crate::Message::OpenConversation {
                                contact_repository: self.contact_repository.clone(),
                                email: self.email.clone(),
                                display_name: self.display_name.clone(),
                                contact_email: contact.email.clone(),
                                client: self.client.clone(),
                            },
                        )));
                    }

                    let contact =
                        if let Some(contact) = self.online_contacts.get_mut(&contact.email) {
                            Some(contact)
                        } else {
                            self.offline_contacts.get_mut(&contact.email)
                        };

                    if let Some(contact) = contact {
                        contact.new_messages = false;
                    }
                }
            }

            Message::NotificationServerEvent(event) => match event {
                Event::DisplayName(display_name) => {
                    self.display_name = Arc::new(display_name);
                    action = Some(Action::RunTask(Task::done(
                        crate::Message::UserDisplayNameUpdated(self.display_name.clone()),
                    )));
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

                    self.offline_contacts
                        .insert(contact.email.clone(), contact.clone());

                    self.contact_repository.add_contacts(&[contact]);
                }

                Event::PresenceUpdate {
                    email,
                    display_name,
                    presence,
                } => {
                    let mut contact = if let Some(contact) = self.online_contacts.get_mut(&email) {
                        Some(contact)
                    } else {
                        self.offline_contacts.get_mut(&email)
                    };

                    let mut previous_status = None;
                    if let Some(contact) = &mut contact {
                        if let Some(msn_object) = &presence.msn_object
                            && msn_object.object_type == 3
                        {
                            contact.display_picture = self
                                .sqlite
                                .select_display_picture(&msn_object.sha1d)
                                .ok()
                                .map(Cow::Owned);
                        }

                        if let Some(status) = &contact.status {
                            previous_status = Some(status.status.clone());
                        }

                        contact.display_name = Arc::new(display_name);
                        contact.status = Some(Arc::new(presence));

                        action = Some(Action::RunTask(Task::done(crate::Message::ContactUpdated(
                            contact.email.clone(),
                        ))));

                        self.contact_repository
                            .update_contacts(std::slice::from_ref(contact));
                    }

                    if let Some(contact) = contact.cloned()
                        && previous_status.is_none()
                    {
                        self.offline_contacts.remove(&email);
                        self.online_contacts.insert(contact.email.clone(), contact);
                    }
                }

                Event::PersonalMessageUpdate {
                    email,
                    personal_message,
                } => {
                    let contact = if let Some(contact) = self.online_contacts.get_mut(&email) {
                        Some(contact)
                    } else {
                        self.offline_contacts.get_mut(&email)
                    };

                    if let Some(contact) = contact {
                        contact.personal_message = Some(Arc::new(personal_message.psm));
                        action = Some(Action::RunTask(Task::done(crate::Message::ContactUpdated(
                            contact.email.clone(),
                        ))));
                    }
                }

                Event::ContactOffline { email } => {
                    let mut contact = if let Some(contact) = self.online_contacts.get_mut(&email) {
                        Some(contact)
                    } else {
                        self.offline_contacts.get_mut(&email)
                    };

                    if let Some(contact) = &mut contact {
                        contact.status = None;
                        action = Some(Action::RunTask(Task::done(crate::Message::ContactUpdated(
                            contact.email.clone(),
                        ))));

                        self.contact_repository
                            .update_contacts(std::slice::from_ref(contact));
                    }

                    if let Some(contact) = contact.cloned() {
                        self.online_contacts.remove(&email);
                        self.offline_contacts.insert(contact.email.clone(), contact);
                    }
                }

                Event::SessionAnswered(switchboard) => {
                    if let Ok(session_id) = block_on(switchboard.get_session_id()) {
                        self.orphan_switchboards.insert(
                            Arc::new(session_id),
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
                        action = Some(Action::RunTask(Task::done(
                            crate::Message::AddSwitchboardToConversation {
                                session_id: session_id.clone(),
                                switchboard: switchboard.clone(),
                            },
                        )));
                    }
                }

                Event::ParticipantLeftSwitchboard { email } => {
                    if let Some(switchboard) = self.orphan_switchboards.get_mut(&*session_id) {
                        switchboard
                            .participants
                            .retain(|participant| **participant != email);

                        action = Some(Action::RunTask(Task::done(
                            crate::Message::AddSwitchboardToConversation {
                                session_id: session_id.clone(),
                                switchboard: switchboard.clone(),
                            },
                        )));
                    }
                }

                Event::Nudge { email } => {
                    let sender = Arc::new(email);
                    let mut contact = if let Some(contact) = self.online_contacts.get_mut(&sender) {
                        Some(contact)
                    } else {
                        self.offline_contacts.get_mut(&sender)
                    };

                    if self.orphan_switchboards.contains_key(&*session_id) {
                        let message = message::Message {
                            sender: sender.clone(),
                            receiver: Some(self.email.clone()),
                            is_nudge: true,
                            text: format!(
                                "{} just sent you a nudge!",
                                if let Some(ref contact) = contact {
                                    &contact.display_name
                                } else {
                                    &sender
                                }
                            ),
                            bold: false,
                            italic: false,
                            underline: false,
                            strikethrough: false,
                            session_id: Some(session_id.clone()),
                            color: "0".to_string(),
                            is_history: true,
                        };

                        let _ = self.sqlite.insert_message(&message);
                        let _ = Notification::new()
                            .summary("New message")
                            .body(&message.text)
                            .show();

                        action = Some(Action::NewMessage);
                    }

                    if let Some(contact) = &mut contact {
                        contact.new_messages = true;
                    }

                    if let Some((session_id, switchboard)) = self
                        .orphan_switchboards
                        .extract_if(|sess_id, switchboard| {
                            **sess_id == *session_id
                                && (switchboard.participants.len() > 1 || contact.is_none())
                        })
                        .next()
                    {
                        action = Some(Action::RunTask(Task::done(
                            crate::Message::CreateConversationWithSwitchboard {
                                contact_repository: self.contact_repository.clone(),
                                email: self.email.clone(),
                                display_name: self.display_name.clone(),
                                session_id,
                                switchboard: switchboard.clone(),
                                minimized: true,
                            },
                        )));
                    }
                }

                Event::TextMessage { email, message } => {
                    let sender = Arc::new(email);
                    let mut contact = if let Some(contact) = self.online_contacts.get_mut(&sender) {
                        Some(contact)
                    } else {
                        self.offline_contacts.get_mut(&sender)
                    };

                    if self.orphan_switchboards.contains_key(&*session_id) {
                        let message = message::Message {
                            sender: sender.clone(),
                            receiver: Some(self.email.clone()),
                            is_nudge: false,
                            text: message.text,
                            bold: message.bold,
                            italic: message.italic,
                            underline: message.underline,
                            strikethrough: message.strikethrough,
                            session_id: Some(session_id.clone()),
                            color: message.color,
                            is_history: true,
                        };

                        let _ = self.sqlite.insert_message(&message);
                        let _ = Notification::new()
                            .summary(&format!(
                                "{} said:",
                                if let Some(ref contact) = contact {
                                    &contact.display_name
                                } else {
                                    &sender
                                }
                            ))
                            .body(&message.text)
                            .show();

                        action = Some(Action::NewMessage);
                    }

                    if let Some(contact) = &mut contact {
                        contact.new_messages = true;
                    }

                    if let Some((session_id, switchboard)) = self
                        .orphan_switchboards
                        .extract_if(|sess_id, switchboard| {
                            **sess_id == *session_id
                                && (switchboard.participants.len() > 1 || contact.is_none())
                        })
                        .next()
                    {
                        action = Some(Action::RunTask(Task::done(
                            crate::Message::CreateConversationWithSwitchboard {
                                contact_repository: self.contact_repository.clone(),
                                email: self.email.clone(),
                                display_name: self.display_name.clone(),
                                session_id,
                                switchboard: switchboard.clone(),
                                minimized: true,
                            },
                        )));
                    }
                }

                Event::DisplayPicture { email, data } => {
                    let contact = if let Some(contact) = self.online_contacts.get_mut(&email) {
                        Some(contact)
                    } else {
                        self.offline_contacts.get_mut(&email)
                    };

                    if let Some(contact) = contact {
                        if let Some(status) = &contact.status
                            && let Some(msn_object) = &status.msn_object
                        {
                            let _ = self.sqlite.insert_display_picture(&data, &msn_object.sha1d);
                        }

                        contact.display_picture = Some(Cow::Owned(data));
                        action = Some(Action::RunTask(Task::done(crate::Message::ContactUpdated(
                            contact.email.clone(),
                        ))));

                        self.contact_repository
                            .update_contacts(std::slice::from_ref(contact));
                    }
                }

                _ => (),
            },
        }

        action
    }
}
