use crate::contact_repository::ContactRepository;
use crate::enums::contact_list_status::ContactListStatus;
use crate::models::contact::Contact;
use crate::models::message;
use crate::models::switchboard_and_participants::SwitchboardAndParticipants;
use crate::msnp_listener::Input;
use crate::pick_display_picture::pick_display_picture;
use crate::sqlite::Sqlite;
use iced::border::radius;
use iced::font::Weight;
use iced::futures::channel::mpsc::Sender;
use iced::widget::{button, column, container, pick_list, row, scrollable, svg, text, text_input};
use iced::{Background, Border, Center, Color, Element, Fill, Font, Padding, Task, Theme, widget};
use iced_aw::ContextMenu;
use msnp11_sdk::{Client, Event, MsnpList, MsnpStatus, PersonalMessage};
use notify_rust::Notification;
use rfd::AsyncFileDialog;
use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;
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
    orphan_switchboards: HashMap<Rc<String>, SwitchboardAndParticipants>,
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

        if let Ok(user) = sqlite.select_user(&email) {
            if let Some(picture) = user.display_picture {
                return Self {
                    email,
                    display_picture: Some(Cow::Owned(picture)),
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
                };
            }
        }

        Self {
            email,
            display_picture: None,
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
                            ..container::Style::default()
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
                                ..container::Style::default()
                            })
                            .padding(3)
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
                scrollable(column![
                    column(
                        self.online_contacts
                            .values()
                            .map(|contact| Self::contact_map(
                                contact,
                                Cow::Borrowed(default_picture)
                            ))
                    )
                    .spacing(10)
                    .padding(Padding {
                        top: 10.0,
                        right: 10.0,
                        bottom: 0.0,
                        left: 10.0,
                    }),
                    column(
                        self.offline_contacts
                            .values()
                            .map(|contact| Self::contact_map(
                                contact,
                                Cow::Borrowed(default_picture)
                            ))
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

    fn contact_map<'a>(
        contact: &'a Contact,
        default_picture: Cow<'static, [u8]>,
    ) -> Element<'a, Message> {
        ContextMenu::new(
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
                                | MsnpStatus::OutToLunch => svg::Handle::from_memory(
                                    include_bytes!("../../assets/default_display_picture_away.svg"),
                                ),

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
                            button::secondary(theme, status).with_background(Color::TRANSPARENT)
                        }
                    })
                    .width(Fill)
                ]
                .align_y(Center)
            ]
            .align_y(Center)
            .spacing(10)
            .width(Fill),
            || {
                let menu_button = |theme: &Theme, status| match status {
                    button::Status::Hovered | button::Status::Pressed => {
                        button::primary(theme, status)
                    }

                    button::Status::Active | button::Status::Disabled => {
                        button::secondary(theme, status).with_background(Color::TRANSPARENT)
                    }
                };

                container(column![
                    if !contact.lists.contains(&MsnpList::BlockList) {
                        button(text("Block").size(15))
                            .on_press(Message::BlockContact(contact.email.clone()))
                            .style(menu_button)
                            .width(Fill)
                    } else {
                        button(text("Unblock").size(15))
                            .on_press(Message::UnblockContact(contact.email.clone()))
                            .style(menu_button)
                            .width(Fill)
                    },
                    button(text("Delete").size(15))
                        .on_press(Message::RemoveContact(contact.email.clone()))
                        .style(menu_button)
                        .width(Fill)
                ])
                .style(|theme: &Theme| container::Style {
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: radius(2.0),
                    },
                    background: Some(Background::Color(
                        theme.extended_palette().secondary.base.color,
                    )),
                    ..container::Style::default()
                })
                .width(150)
                .into()
            },
        )
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
                        crate::Message::EmptyResultFuture,
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
                        crate::Message::EmptyResultFuture,
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
                        crate::Message::EmptyResultFuture,
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
                if contact.status.is_some() && self.status != Some(ContactListStatus::AppearOffline)
                {
                    let mut open_session_id = None;
                    if let Some((session_id, switchboard)) = self
                        .orphan_switchboards
                        .iter()
                        .find(|(_, switchboard)| switchboard.participants.contains(&contact.email))
                    {
                        open_session_id = Some(session_id.clone());
                        action = Some(Action::RunTask(Task::done(
                            crate::Message::CreateConversationWithSwitchboard {
                                contact_repository: self.contact_repository.clone(),
                                email: self.email.clone(),
                                display_name: self.display_name.clone(),
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

                    if let Some(open_session_id) = open_session_id {
                        self.orphan_switchboards.remove(&open_session_id);
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

                        self.contact_repository.update_contacts(&[contact.clone()]);
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

                        self.contact_repository.update_contacts(&[contact.clone()]);
                    }

                    if let Some(contact) = contact.cloned() {
                        self.online_contacts.remove(&email);
                        self.offline_contacts.insert(contact.email.clone(), contact);
                    }
                }

                Event::SessionAnswered(switchboard) => {
                    if let Ok(Some(session_id)) = switchboard.get_session_id() {
                        self.orphan_switchboards.insert(
                            Rc::new(session_id),
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

                Event::Nudge { email } => {
                    let sender = Arc::new(email);
                    let mut contact = if let Some(contact) = self.online_contacts.get_mut(&sender) {
                        Some(contact)
                    } else {
                        self.offline_contacts.get_mut(&sender)
                    };

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

                    if let Some(contact) = &mut contact {
                        contact.new_messages = true;
                    }

                    if let Some((_, switchboard)) = self
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

                    if let Some(contact) = &mut contact {
                        contact.new_messages = true;
                    }

                    if let Some((_, switchboard)) = self
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

                        self.contact_repository.update_contacts(&[contact.clone()]);
                    }
                }

                _ => (),
            },
        }

        action
    }
}
