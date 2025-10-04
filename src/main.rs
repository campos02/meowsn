#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]

use crate::contact_repository::ContactRepository;
use crate::meowsn_window::Window;
use crate::screens::screen::Screen;
use crate::screens::{add_contact, contacts, conversation, dialog, personal_settings, sign_in};
use crate::sqlite::Sqlite;
use dark_light::Mode;
use helpers::notify_new_version::notify_new_version;
use iced::futures::channel::mpsc::Sender;
use iced::futures::executor::block_on;
use iced::widget::space;
use iced::window::{Position, UserAttention, icon};
use iced::{Element, Pixels, Size, Subscription, Task, Theme, keyboard, widget, window};
use models::switchboard_and_participants::SwitchboardAndParticipants;
use msnp_listener::Input;
use msnp11_sdk::{Client, MsnpStatus, SdkError, Switchboard};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;

mod contact_repository;
mod enums;
mod helpers;
mod keyboard_listener;
mod meowsn_window;
mod models;
mod msnp_listener;
mod screens;
mod settings;
mod sqlite;
mod svg;
mod widgets;

pub enum Message {
    WindowEvent((window::Id, window::Event)),
    WindowOpened {
        id: window::Id,
        screen: Screen,
        minimized: bool,
    },

    SignIn(window::Id, sign_in::sign_in::Message),
    SignedIn {
        id: window::Id,
        email: Arc<String>,
        result: Result<(String, MsnpStatus, Arc<Client>), SdkError>,
    },

    Contacts(window::Id, contacts::contacts::Message),
    PersonalSettings(window::Id, personal_settings::Message),
    Conversation(window::Id, conversation::conversation::Message),
    Dialog(window::Id, dialog::Message),
    AddContact(window::Id, add_contact::Message),
    ContactFocused(Arc<String>),
    OpenPersonalSettings {
        client: Option<Arc<Client>>,
        display_name: Option<String>,
    },

    OpenConversation {
        contact_repository: ContactRepository,
        email: Arc<String>,
        display_name: Arc<String>,
        contact_email: Arc<String>,
        client: Arc<Client>,
    },

    CreateConversation {
        contact_repository: ContactRepository,
        result: Result<Arc<Switchboard>, SdkError>,
        contact_email: Arc<String>,
        email: Arc<String>,
        display_name: Arc<String>,
    },

    CreateConversationWithSwitchboard {
        contact_repository: ContactRepository,
        email: Arc<String>,
        display_name: Arc<String>,
        session_id: Arc<String>,
        switchboard: SwitchboardAndParticipants,
        minimized: bool,
    },

    OpenDialog(String),
    OpenAddContact(Arc<Client>),
    MsnpEvent(msnp_listener::Event),
    DisplayNameResult(String, Result<(), SdkError>),
    EventResult(Result<msnp11_sdk::Event, SdkError>),
    ContactUpdated(Arc<String>),
    UserDisplayPictureUpdated(Option<Cow<'static, [u8]>>),
    UserDisplayNameUpdated(Arc<String>),
    FocusNext,
    FocusPrevious,
}

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("Message").finish()
    }
}

struct MeowSN {
    windows: BTreeMap<window::Id, Window>,
    main_window_id: window::Id,
    modal_id: Option<window::Id>,
    msnp_subscription_sender: Option<Sender<Input>>,
    sqlite: Sqlite,
}

impl MeowSN {
    fn new() -> (Self, Task<Message>) {
        let sqlite = Sqlite::new().expect("Could not create database");
        let (id, open) = window::open(MeowSN::window_settings(
            Size::new(360.0, 620.0),
            Size::new(360.0, 400.0),
        ));

        (
            Self {
                windows: BTreeMap::new(),
                main_window_id: id,
                modal_id: None,
                msnp_subscription_sender: None,
                sqlite: sqlite.clone(),
            },
            Task::batch([
                open.map(move |id| Message::WindowOpened {
                    id,
                    screen: Screen::SignIn(sign_in::sign_in::SignIn::new(sqlite.clone())),
                    minimized: false,
                }),
                Task::future(notify_new_version()).discard(),
            ]),
        )
    }

    fn update(&mut self, mut message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened {
                id,
                mut screen,
                minimized,
            } => {
                let task = if minimized {
                    if let Screen::Conversation(conversation) = &mut screen
                        && let Some(conversation::conversation::Action::RunTask(task)) =
                            conversation.update(conversation::conversation::Message::WindowOpened)
                    {
                        task
                    } else {
                        Task::none()
                    }
                } else if let Screen::Conversation(conversation) = &mut screen
                    && let Some(conversation::conversation::Action::RunTask(task)) =
                        conversation.update(conversation::conversation::Message::Focused)
                {
                    task
                } else {
                    Task::none()
                };

                let window = Window::new(
                    id,
                    screen,
                    self.sqlite.clone(),
                    self.msnp_subscription_sender.clone(),
                );

                self.windows.insert(id, window);
                task
            }

            Message::WindowEvent((id, event)) => match event {
                window::Event::Closed => {
                    if let Some(window) = self.windows.remove(&id) {
                        match window.get_screen() {
                            Screen::SignIn(..) | Screen::Contacts(..) => iced::exit(),
                            Screen::Conversation(conversation) => {
                                conversation.leave_switchboards_task()
                            }

                            Screen::Dialog(..) => {
                                self.modal_id = None;
                                Task::none()
                            }

                            _ => Task::none(),
                        }
                    } else {
                        Task::none()
                    }
                }

                window::Event::Focused => {
                    if let Some(modal) = self.modal_id {
                        window::gain_focus(modal)
                    } else if let Some(window) = self.windows.get_mut(&id) {
                        window.update(Message::Conversation(
                            id,
                            conversation::conversation::Message::Focused,
                        ))
                    } else {
                        Task::none()
                    }
                }

                window::Event::Unfocused => {
                    if let Some(window) = self.windows.get_mut(&id) {
                        window.update(Message::Conversation(
                            id,
                            conversation::conversation::Message::Unfocused,
                        ))
                    } else {
                        Task::none()
                    }
                }

                _ => Task::none(),
            },

            Message::Contacts(id, ..)
            | Message::PersonalSettings(id, ..)
            | Message::SignIn(id, ..)
            | Message::Conversation(id, ..)
            | Message::Dialog(id, ..)
            | Message::AddContact(id, ..) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    return window.update(message);
                }

                Task::none()
            }

            Message::SignedIn {
                id,
                email: _,
                ref result,
            } => {
                if let Some(sender) = self.msnp_subscription_sender.as_mut()
                    && let Ok((_, _, client)) = result
                    && let Err(error) = sender.start_send(Input::NewClient(client.clone()))
                {
                    return Task::done(Message::OpenDialog(error.to_string()));
                }

                if let Some(window) = self.windows.get_mut(&id) {
                    return window.update(message);
                }

                Task::none()
            }

            Message::OpenPersonalSettings {
                mut client,
                mut display_name,
            } => {
                if self.windows.keys().last().is_none() {
                    return Task::none();
                };

                if let Some((id, _)) = self
                    .windows
                    .iter()
                    .find(|(_, window)| matches!(window.get_screen(), Screen::PersonalSettings(..)))
                {
                    return window::gain_focus(*id);
                }

                let (_, open) = window::open(MeowSN::window_settings(
                    Size::new(400.0, 500.0),
                    Size::new(400.0, 500.0),
                ));

                open.map(move |id| Message::WindowOpened {
                    id,
                    screen: Screen::PersonalSettings(personal_settings::PersonalSettings::new(
                        client.take(),
                        display_name.take(),
                    )),
                    minimized: false,
                })
            }

            Message::OpenConversation {
                contact_repository,
                email,
                display_name,
                contact_email,
                client,
            } => {
                if self.windows.keys().last().is_none() {
                    return Task::none();
                };

                if let Some((id, _)) = self.windows.iter_mut().find(|(_, window)| {
                    let Screen::Conversation(conversation) = window.get_screen() else {
                        return false;
                    };

                    conversation.get_participants().contains_key(&contact_email)
                }) {
                    window::gain_focus(*id)
                } else {
                    let contact_email = contact_email.clone();
                    let contact = contact_email.clone();

                    Task::perform(
                        async move { client.create_session(&contact_email).await },
                        move |result| Message::CreateConversation {
                            contact_repository: contact_repository.clone(),
                            result: result.map(Arc::new),
                            contact_email: contact.clone(),
                            email: email.clone(),
                            display_name: display_name.clone(),
                        },
                    )
                }
            }

            Message::CreateConversation {
                mut contact_repository,
                result,
                contact_email: mut contact,
                mut email,
                mut display_name,
            } => {
                if self.windows.keys().last().is_none() {
                    return Task::none();
                };

                match result {
                    Ok(switchboard) => {
                        if let Some(ref mut sender) = self.msnp_subscription_sender {
                            let _ = sender.start_send(Input::NewSwitchboard(switchboard.clone()));
                        }

                        let sqlite = self.sqlite.clone();
                        let (_, open) = window::open(MeowSN::window_settings(
                            Size::new(1100.0, 650.0),
                            Size::new(800.0, 400.0),
                        ));

                        open.map(move |id| Message::WindowOpened {
                            id,
                            screen: Screen::Conversation(
                                conversation::conversation::Conversation::new(
                                    std::mem::take(&mut contact_repository),
                                    Arc::new(
                                        block_on(switchboard.get_session_id()).unwrap_or_default(),
                                    ),
                                    SwitchboardAndParticipants {
                                        switchboard: switchboard.clone(),
                                        participants: vec![std::mem::take(&mut contact)],
                                    },
                                    std::mem::take(&mut email),
                                    std::mem::take(&mut display_name),
                                    sqlite.clone(),
                                ),
                            ),
                            minimized: false,
                        })
                    }

                    Err(error) => Task::done(Message::OpenDialog(error.to_string())),
                }
            }

            Message::CreateConversationWithSwitchboard {
                mut contact_repository,
                mut email,
                mut display_name,
                session_id,
                switchboard,
                minimized,
            } => {
                if self.windows.keys().last().is_none() {
                    return Task::none();
                };

                if let Some((id, window)) = self.windows.iter_mut().find(|(_, window)| {
                    let Screen::Conversation(conversation) = window.get_screen() else {
                        return false;
                    };

                    conversation.get_participants().len() == 1
                        && switchboard.participants.iter().all(|participant| {
                            conversation.get_participants().contains_key(participant)
                        })
                }) {
                    window.update(Message::Conversation(
                        *id,
                        conversation::conversation::Message::NewSwitchboard(
                            session_id.clone(),
                            switchboard.switchboard,
                        ),
                    ))
                } else {
                    let mut settings =
                        MeowSN::window_settings(Size::new(1000.0, 600.0), Size::new(800.0, 400.0));

                    settings.visible = false;
                    let (id, open) = window::open(settings);

                    let mut session_id = session_id.clone();
                    let sqlite = self.sqlite.clone();

                    open.map(move |id| Message::WindowOpened {
                        id,
                        screen: Screen::Conversation(
                            conversation::conversation::Conversation::new(
                                std::mem::take(&mut contact_repository),
                                std::mem::take(&mut session_id),
                                switchboard.clone(),
                                std::mem::take(&mut email),
                                std::mem::take(&mut display_name),
                                sqlite.clone(),
                            ),
                        ),
                        minimized,
                    })
                    .chain(window::minimize(id, minimized))
                }
            }

            Message::OpenDialog(mut message) => {
                if self.windows.keys().last().is_none() {
                    return Task::none();
                };

                if let Some(id) = self.modal_id {
                    return window::gain_focus(id);
                }

                let (id, open) = window::open(MeowSN::window_settings(
                    Size::new(400.0, 150.0),
                    Size::new(400.0, 150.0),
                ));

                self.modal_id = Some(id);
                Task::batch([
                    open.map(move |id| Message::WindowOpened {
                        id,
                        screen: Screen::Dialog(dialog::Dialog::new(std::mem::take(&mut message))),
                        minimized: false,
                    }),
                    window::request_user_attention(
                        self.main_window_id,
                        Some(UserAttention::Critical),
                    ),
                ])
            }

            Message::OpenAddContact(client) => {
                if self.windows.keys().last().is_none() {
                    return Task::none();
                };

                if let Some((id, _)) = self
                    .windows
                    .iter()
                    .find(|(_, window)| matches!(window.get_screen(), Screen::AddContact(..)))
                {
                    return window::gain_focus(*id);
                }

                let (_, open) = window::open(MeowSN::window_settings(
                    Size::new(450.0, 220.0),
                    Size::new(450.0, 220.0),
                ));

                open.map(move |id| Message::WindowOpened {
                    id,
                    screen: Screen::AddContact(add_contact::AddContact::new(client.clone())),
                    minimized: false,
                })
            }

            Message::MsnpEvent(ref mut event) => match event {
                msnp_listener::Event::Ready(sender) => {
                    self.msnp_subscription_sender = Some(sender.clone());
                    Task::none()
                }

                msnp_listener::Event::NotificationServer(event) => {
                    if self.windows.keys().last().is_none() {
                        return Task::none();
                    };

                    match event {
                        msnp11_sdk::Event::Disconnected
                        | msnp11_sdk::Event::LoggedInAnotherDevice => {
                            let mut tasks = Vec::with_capacity(self.windows.len());
                            for (id, _) in self.windows.iter_mut() {
                                if *id != self.main_window_id {
                                    tasks.push(window::close::<Message>(*id));
                                }
                            }

                            if let Some(window) = self.windows.get_mut(&self.main_window_id) {
                                // Using Disconnected as a default
                                tasks.push(window.update(Message::Contacts(
                                    self.main_window_id,
                                    contacts::contacts::Message::NotificationServerEvent(
                                        std::mem::replace(event, msnp11_sdk::Event::Disconnected),
                                    ),
                                )))
                            }

                            Task::batch(tasks)
                        }

                        _ => {
                            if let Some(window) = self.windows.get_mut(&self.main_window_id) {
                                return window.update(Message::Contacts(
                                    self.main_window_id,
                                    // Using Authenticated as a default event
                                    contacts::contacts::Message::NotificationServerEvent(
                                        std::mem::replace(event, msnp11_sdk::Event::Authenticated),
                                    ),
                                ));
                            }

                            Task::none()
                        }
                    }
                }

                msnp_listener::Event::Switchboard { session_id, event } => {
                    if self.windows.keys().last().is_none() {
                        return Task::none();
                    };

                    let mut tasks = Vec::new();
                    for (_, window) in self.windows.iter_mut() {
                        if matches!(
                            window.get_screen(),
                            Screen::Conversation(..) | Screen::Contacts(..)
                        ) {
                            tasks.push(window.update(Message::MsnpEvent(
                                msnp_listener::Event::Switchboard {
                                    session_id: session_id.clone(),
                                    event: event.clone(),
                                },
                            )));
                        }
                    }

                    Task::batch(tasks)
                }
            },

            Message::DisplayNameResult(display_name, result) => {
                if let Err(error) = result {
                    Task::done(Message::OpenDialog(error.to_string()))
                } else {
                    Task::done(Message::MsnpEvent(
                        msnp_listener::Event::NotificationServer(msnp11_sdk::Event::DisplayName(
                            display_name,
                        )),
                    ))
                }
            }

            Message::ContactUpdated(contact) => {
                if self.windows.keys().last().is_none() {
                    return Task::none();
                };

                let mut tasks = Vec::new();
                for (id, window) in self.windows.iter_mut() {
                    if let Screen::Conversation(conversation) = window.get_screen()
                        && conversation.get_participants().contains_key(&contact)
                    {
                        tasks.push(window.update(Message::Conversation(
                            *id,
                            conversation::conversation::Message::ContactUpdated(contact.clone()),
                        )));
                    }
                }

                Task::batch(tasks)
            }

            Message::UserDisplayPictureUpdated(picture) => {
                if self.windows.keys().last().is_none() {
                    return Task::none();
                };

                let mut tasks = Vec::new();
                if let Some(picture) = picture {
                    for (id, window) in self.windows.iter_mut() {
                        match window.get_screen() {
                            Screen::Conversation(..) => {
                                tasks.push(window.update(Message::Conversation(
                                    *id,
                                    conversation::conversation::Message::UserDisplayPictureUpdated(
                                        picture.clone(),
                                    ),
                                )));
                            }

                            Screen::Contacts(..) => {
                                tasks.push(window.update(Message::Contacts(
                                    *id,
                                    contacts::contacts::Message::UserDisplayPictureUpdated(
                                        picture.clone(),
                                    ),
                                )));
                            }

                            _ => (),
                        }
                    }
                }

                Task::batch(tasks)
            }

            Message::UserDisplayNameUpdated(display_name) => {
                if self.windows.keys().last().is_none() {
                    return Task::none();
                };

                let mut tasks = Vec::new();
                for (id, window) in self.windows.iter_mut() {
                    if matches!(window.get_screen(), Screen::Conversation(..)) {
                        tasks.push(window.update(Message::Conversation(
                            *id,
                            conversation::conversation::Message::UserDisplayNameUpdated(
                                display_name.clone(),
                            ),
                        )));
                    }
                }

                Task::batch(tasks)
            }

            Message::FocusNext => widget::operation::focus_next(),
            Message::FocusPrevious => widget::operation::focus_previous(),
            Message::EventResult(result) => {
                match result {
                    Ok(event) => {
                        if self.windows.keys().last().is_none() {
                            return Task::none();
                        };

                        if let Some(window) = self.windows.get_mut(&self.main_window_id) {
                            return window.update(Message::Contacts(
                                self.main_window_id,
                                contacts::contacts::Message::NotificationServerEvent(event),
                            ));
                        }
                    }

                    Err(error) => {
                        return Task::done(Message::OpenDialog(error.to_string()));
                    }
                }

                Task::none()
            }

            Message::ContactFocused(email) => {
                if let Some(window) = self.windows.get_mut(&self.main_window_id) {
                    return window.update(Message::Contacts(
                        self.main_window_id,
                        contacts::contacts::Message::ContactFocused(email),
                    ));
                }

                Task::none()
            }
        }
    }

    fn title(&self, window_id: window::Id) -> String {
        if let Some(window) = self.windows.get(&window_id) {
            match window.get_screen() {
                Screen::Conversation(conversation) => conversation.get_title(),
                Screen::AddContact(..) => "Add new contact".to_string(),
                Screen::PersonalSettings(..) => "Personal settings".to_string(),
                _ => "meowsn".to_string(),
            }
        } else {
            "meowsn".to_string()
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if let Some(window) = self.windows.get(&window_id) {
            window.view(window_id)
        } else {
            space().into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::events().map(Message::WindowEvent),
            keyboard::on_key_press(keyboard_listener::listen),
            Subscription::run(msnp_listener::listen).map(Message::MsnpEvent),
        ])
    }

    fn window_settings(size: Size, min_size: Size) -> window::Settings {
        window::Settings {
            size,
            min_size: Some(min_size),
            position: Position::Centered,
            icon: icon::from_file_data(include_bytes!("../assets/meowsn.ico"), None).ok(),
            ..Default::default()
        }
    }

    fn theme(&self, _: window::Id) -> Theme {
        match dark_light::detect().unwrap_or(Mode::Unspecified) {
            Mode::Dark => Theme::CatppuccinMocha,
            _ => Theme::CatppuccinLatte,
        }
    }
}

#[cfg(target_os = "macos")]
pub fn main() -> iced::Result {
    let id = notify_rust::get_bundle_identifier_or_default("meowsn");
    notify_rust::set_application(&id).expect("Could not set application name");

    iced::daemon(MeowSN::new, MeowSN::update, MeowSN::view)
        .subscription(MeowSN::subscription)
        .title(MeowSN::title)
        .theme(MeowSN::theme)
        .settings(iced::Settings {
            default_text_size: Pixels(14.0),
            ..Default::default()
        })
        .run()
}

#[cfg(not(target_os = "macos"))]
pub fn main() -> iced::Result {
    iced::daemon(MeowSN::new, MeowSN::update, MeowSN::view)
        .subscription(MeowSN::subscription)
        .title(MeowSN::title)
        .theme(MeowSN::theme)
        .settings(iced::Settings {
            default_text_size: Pixels(14.0),
            ..Default::default()
        })
        .run()
}
