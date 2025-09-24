use crate::contact_repository::ContactRepository;
use crate::enums::contact_list_status::ContactListStatus;
use crate::helpers::pick_display_picture::pick_display_picture;
use crate::models::contact::Contact;
use crate::models::switchboard_and_participants::SwitchboardAndParticipants;
use crate::msnp_listener::Input;
use crate::screens::contacts::bordered_container::bordered_container;
use crate::screens::contacts::contact_map::contact_map;
use crate::screens::contacts::transparent_button::transparent_button;
use crate::sqlite::Sqlite;
use iced::futures::channel::mpsc::Sender;
use iced::futures::executor::block_on;
use iced::widget::{column, container, pick_list, row, scrollable, svg, text, text_input};
use iced::{Background, Center, Color, Element, Fill, Padding, Task, Theme, widget};
use msnp11_sdk::{Client, Event, MsnpList, MsnpStatus, PersonalMessage, SdkError};
use rfd::AsyncFileDialog;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

pub enum Action {
    SignOut(Task<crate::Message>),
    RunTask(Task<crate::Message>),
    NewMessage,
    SetPersonalMessage(Arc<Client>, PersonalMessage),
    SetPresence(Arc<Client>, ContactListStatus),
    BlockContact(Arc<Client>, Arc<String>),
    UnblockContact(Arc<Client>, Arc<String>),
    RemoveContact {
        client: Arc<Client>,
        contact: Arc<String>,
        guid: Arc<String>,
    },
}

#[derive(Clone)]
pub enum Message {
    PersonalMessageChanged(String),
    PersonalMessageSubmit,
    PersonalMessageResult(Result<(), SdkError>),
    StatusSelected(ContactListStatus),
    StatusResult(ContactListStatus, Result<(), SdkError>),
    Conversation(Contact),
    NotificationServerEvent(Event),
    SwitchboardEvent(Arc<String>, Event),
    BlockContact(Arc<String>),
    BlockResult(Arc<String>, Result<(), SdkError>),
    UnblockContact(Arc<String>),
    UnblockResult(Arc<String>, Result<(), SdkError>),
    RemoveContact {
        contact: Arc<String>,
        guid: Arc<String>,
    },

    RemoveResult(Arc<String>, Result<(), SdkError>),
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
                            .width(250)
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
                .height(Fill)
                .width(Fill)
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

                action = Some(Action::SetPersonalMessage(client, personal_message));
            }

            Message::PersonalMessageResult(result) => {
                if let Err(error) = result {
                    action = Some(Action::RunTask(Task::done(crate::Message::OpenDialog(
                        error.to_string(),
                    ))))
                } else {
                    let _ = self
                        .sqlite
                        .update_personal_message(&self.email, &self.personal_message);
                }
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
                    action = Some(Action::SignOut(
                        Task::future(async move { client.disconnect().await }).discard(),
                    ));
                }

                _ => {
                    let client = self.client.clone();
                    action = Some(Action::SetPresence(client, status));
                }
            },

            Message::StatusResult(status, result) => {
                if let Err(error) = result {
                    action = Some(Action::RunTask(Task::done(crate::Message::OpenDialog(
                        error.to_string(),
                    ))))
                } else {
                    self.status = Some(status);
                }
            }

            Message::BlockContact(contact) => {
                let client = self.client.clone();
                action = Some(Action::BlockContact(client, contact));
            }

            Message::BlockResult(contact, result) => {
                if let Err(error) = result {
                    action = Some(Action::RunTask(Task::done(crate::Message::OpenDialog(
                        error.to_string(),
                    ))))
                } else {
                    let contact = if let Some(contact) = self.online_contacts.get_mut(&contact) {
                        Some(contact)
                    } else {
                        self.offline_contacts.get_mut(&contact)
                    };

                    if let Some(contact) = contact {
                        contact.lists.push(MsnpList::BlockList);
                        contact.lists.retain(|list| list != &MsnpList::AllowList);
                    }
                }
            }

            Message::UnblockContact(contact) => {
                let client = self.client.clone();
                action = Some(Action::UnblockContact(client, contact));
            }

            Message::UnblockResult(contact, result) => {
                if let Err(error) = result {
                    action = Some(Action::RunTask(Task::done(crate::Message::OpenDialog(
                        error.to_string(),
                    ))))
                } else {
                    let contact = if let Some(contact) = self.online_contacts.get_mut(&contact) {
                        Some(contact)
                    } else {
                        self.offline_contacts.get_mut(&contact)
                    };

                    if let Some(contact) = contact {
                        contact.lists.retain(|list| list != &MsnpList::BlockList);
                        contact.lists.push(MsnpList::AllowList);
                    }
                }
            }

            Message::RemoveContact { contact, guid } => {
                let client = self.client.clone();
                action = Some(Action::RemoveContact {
                    client,
                    contact,
                    guid,
                });
            }

            Message::RemoveResult(contact, result) => {
                if let Err(error) = result {
                    action = Some(Action::RunTask(Task::done(crate::Message::OpenDialog(
                        error.to_string(),
                    ))))
                } else {
                    self.online_contacts.remove(&contact);
                    self.offline_contacts.remove(&contact);
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
                    contact.opening_conversation = false;
                }
            }

            Message::Conversation(contact) => {
                if contact.status.is_some()
                    && self.status != Some(ContactListStatus::AppearOffline)
                    && !contact.opening_conversation
                {
                    if let Some(contact) = self.online_contacts.get_mut(&contact.email) {
                        contact.opening_conversation = true;
                    }

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
                        ..Contact::default()
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
                    if let Some(mut switchboard) = self.orphan_switchboards.remove(&*session_id) {
                        switchboard.participants.push(Arc::new(email));

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
