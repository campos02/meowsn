#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]

use crate::contact_repository::ContactRepository;
use crate::icedm_window::Window;
use crate::msnp_listener::Input;
use crate::screens::screen::Screen;
use crate::screens::{add_contact, contacts, conversation, dialog, personal_settings, sign_in};
use crate::sqlite::Sqlite;
use dark_light::Mode;
use enums::window_type::WindowType;
use iced::futures::channel::mpsc::Sender;
use iced::widget::horizontal_space;
use iced::window::{Position, Settings, icon};
use iced::{Element, Size, Subscription, Task, Theme, keyboard, widget, window};
use models::switchboard_and_participants::SwitchboardAndParticipants;
use msnp11_sdk::sdk_error::SdkError;
use msnp11_sdk::{Client, MsnpStatus, Switchboard};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;

mod contact_repository;
mod enums;
mod icedm_window;
mod keyboard_listener;
mod models;
mod msnp_listener;
mod pick_display_picture;
mod screens;
mod settings;
mod sign_in_async;
mod sqlite;

#[derive(Clone)]
pub enum Message {
    WindowEvent((window::Id, window::Event)),
    WindowOpened(window::Id, WindowType),
    SignIn(window::Id, sign_in::Message),
    SignedIn {
        id: window::Id,
        email: Arc<String>,
        result: Result<(String, MsnpStatus, Arc<Client>), SdkError>,
    },

    Contacts(window::Id, contacts::Message),
    PersonalSettings(window::Id, personal_settings::Message),
    Conversation(window::Id, conversation::Message),
    Dialog(window::Id, dialog::Message),
    AddContact(window::Id, add_contact::Message),
    OpenPersonalSettings {
        client: Option<Arc<Client>>,
        display_name: Option<String>,
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
        switchboard: SwitchboardAndParticipants,
    },

    OpenConversation {
        contact_repository: ContactRepository,
        email: Arc<String>,
        display_name: Arc<String>,
        contact_email: Arc<String>,
        client: Arc<Client>,
    },

    OpenDialog(String),
    OpenAddContact(Arc<Client>),
    MsnpEvent(msnp_listener::Event),
    EmptyResultFuture(Result<(), SdkError>),
    Empty(()),
    EventFuture(Result<msnp11_sdk::Event, SdkError>),
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

struct IcedM {
    windows: BTreeMap<window::Id, Window>,
    modal_id: Option<window::Id>,
    msnp_subscription_sender: Option<Sender<Input>>,
    sqlite: Sqlite,
}

impl IcedM {
    fn new() -> (Self, Task<Message>) {
        let sqlite = Sqlite::new().expect("Could not create database");
        let (_id, open) = window::open(IcedM::window_settings(Size::new(450.0, 600.0)));
        (
            Self {
                windows: BTreeMap::new(),
                modal_id: None,
                msnp_subscription_sender: None,
                sqlite,
            },
            open.map(move |id| Message::WindowOpened(id, WindowType::MainWindow)),
        )
    }

    fn update(&mut self, mut message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(id, window_type) => {
                let screen = match window_type {
                    WindowType::MainWindow => {
                        Screen::SignIn(sign_in::SignIn::new(self.sqlite.clone()))
                    }

                    WindowType::PersonalSettings {
                        client,
                        display_name,
                    } => Screen::PersonalSettings(personal_settings::PersonalSettings::new(
                        client,
                        display_name,
                    )),

                    WindowType::Conversation {
                        contact_repository,
                        switchboard,
                        email,
                        display_name,
                    } => Screen::Conversation(conversation::Conversation::new(
                        contact_repository,
                        switchboard,
                        email,
                        display_name,
                        self.sqlite.clone(),
                    )),

                    WindowType::Dialog(message) => Screen::Dialog(dialog::Dialog::new(message)),
                    WindowType::AddContact(client) => {
                        Screen::AddContact(add_contact::AddContact::new(client))
                    }
                };

                let window = Window::new(
                    id,
                    screen,
                    self.sqlite.clone(),
                    self.msnp_subscription_sender.clone(),
                );

                self.windows.insert(id, window);
                Task::none()
            }

            Message::WindowEvent((id, event)) => match event {
                window::Event::Closed => {
                    if let Some(window) = self.windows.remove(&id) {
                        match window.get_screen() {
                            Screen::SignIn(..) | Screen::Contacts(..) => iced::exit(),
                            Screen::Conversation(conversation) => {
                                conversation.leave_switchboard_task()
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
                        window.update(Message::Conversation(id, conversation::Message::Focused))
                    } else {
                        Task::none()
                    }
                }

                window::Event::Unfocused => {
                    if let Some(window) = self.windows.get_mut(&id) {
                        window.update(Message::Conversation(id, conversation::Message::Unfocused))
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
                {
                    if let Err(error) = sender.start_send(Input::NewClient(client.clone())) {
                        return Task::done(Message::OpenDialog(error.to_string()));
                    }
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

                let (_, open) = window::open(IcedM::window_settings(Size::new(500.0, 500.0)));
                open.map(move |id| {
                    Message::WindowOpened(
                        id,
                        WindowType::PersonalSettings {
                            client: client.take(),
                            display_name: display_name.take(),
                        },
                    )
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

                        let (_id, open) =
                            window::open(IcedM::window_settings(Size::new(1000.0, 600.0)));

                        open.map(move |id| {
                            Message::WindowOpened(
                                id,
                                WindowType::Conversation {
                                    contact_repository: std::mem::take(&mut contact_repository),
                                    switchboard: SwitchboardAndParticipants {
                                        switchboard: switchboard.clone(),
                                        participants: vec![std::mem::take(&mut contact)],
                                    },
                                    email: std::mem::take(&mut email),
                                    display_name: std::mem::take(&mut display_name),
                                },
                            )
                        })
                    }

                    Err(error) => Task::done(Message::OpenDialog(error.to_string())),
                }
            }

            Message::CreateConversationWithSwitchboard {
                mut contact_repository,
                mut email,
                mut display_name,
                switchboard,
            } => {
                if self.windows.keys().last().is_none() {
                    return Task::none();
                };

                let (_, open) = window::open(IcedM::window_settings(Size::new(1000.0, 600.0)));
                open.map(move |id| {
                    Message::WindowOpened(
                        id,
                        WindowType::Conversation {
                            contact_repository: std::mem::take(&mut contact_repository),
                            switchboard: switchboard.clone(),
                            email: std::mem::take(&mut email),
                            display_name: std::mem::take(&mut display_name),
                        },
                    )
                })
            }

            Message::OpenDialog(mut message) => {
                if self.windows.keys().last().is_none() {
                    return Task::none();
                };

                if let Some(id) = self.modal_id {
                    return window::gain_focus(id);
                }

                let (id, open) = window::open(IcedM::window_settings(Size::new(400.0, 150.0)));
                self.modal_id = Some(id);

                open.map(move |id| {
                    Message::WindowOpened(id, WindowType::Dialog(std::mem::take(&mut message)))
                })
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

                let (_, open) = window::open(IcedM::window_settings(Size::new(400.0, 220.0)));
                open.map(move |id| {
                    Message::WindowOpened(id, WindowType::AddContact(client.clone()))
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
                            let mut tasks = Vec::new();
                            for (id, window) in self.windows.iter_mut() {
                                // Close windows that aren't the main one and open dialog with disconnection message
                                tasks.push(
                                    if !matches!(
                                        window.get_screen(),
                                        Screen::Contacts(..) | Screen::SignIn(..)
                                    ) {
                                        window::close::<Message>(*id)
                                    } else {
                                        // Using Disconnected as a default to replace the event, since there's only supposed to be one
                                        // window with this screen type
                                        window.update(Message::Contacts(
                                            *id,
                                            contacts::Message::NotificationServerEvent(
                                                std::mem::replace(
                                                    event,
                                                    msnp11_sdk::Event::Disconnected,
                                                ),
                                            ),
                                        ))
                                    },
                                );
                            }

                            Task::batch(tasks)
                        }

                        _ => {
                            if let Some((id, window)) =
                                self.windows.iter_mut().find(|(_, window)| {
                                    matches!(window.get_screen(), Screen::Contacts(..))
                                })
                            {
                                return window.update(Message::Contacts(
                                    *id,
                                    // Using Authenticated as a default event
                                    contacts::Message::NotificationServerEvent(std::mem::replace(
                                        event,
                                        msnp11_sdk::Event::Authenticated,
                                    )),
                                ));
                            }

                            Task::none()
                        }
                    }
                }

                msnp_listener::Event::Switchboard { .. } => {
                    if self.windows.keys().last().is_none() {
                        return Task::none();
                    };

                    let mut tasks = Vec::new();
                    for (_, window) in self.windows.iter_mut() {
                        if matches!(
                            window.get_screen(),
                            Screen::Conversation(..) | Screen::Contacts(..)
                        ) {
                            tasks.push(window.update(message.clone()));
                        }
                    }

                    Task::batch(tasks)
                }
            },

            Message::ContactUpdated(mut contact) => {
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
                            conversation::Message::ContactUpdated(std::mem::take(&mut contact)),
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
                                    conversation::Message::UserDisplayPictureUpdated(
                                        picture.clone(),
                                    ),
                                )));
                            }

                            Screen::Contacts(..) => {
                                tasks.push(window.update(Message::Contacts(
                                    *id,
                                    contacts::Message::UserDisplayPictureUpdated(picture.clone()),
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
                            conversation::Message::UserDisplayNameUpdated(display_name.clone()),
                        )));
                    }
                }

                Task::batch(tasks)
            }

            Message::FocusNext => widget::focus_next(),
            Message::FocusPrevious => widget::focus_previous(),
            Message::EventFuture(result) => {
                match result {
                    Ok(event) => {
                        if self.windows.keys().last().is_none() {
                            return Task::none();
                        };

                        if let Some((id, window)) = self
                            .windows
                            .iter_mut()
                            .find(|(_, window)| matches!(window.get_screen(), Screen::Contacts(..)))
                        {
                            return window.update(Message::Contacts(
                                *id,
                                contacts::Message::NotificationServerEvent(event),
                            ));
                        }
                    }

                    Err(error) => {
                        return Task::done(Message::OpenDialog(error.to_string()));
                    }
                }

                Task::none()
            }

            _ => Task::none(),
        }
    }

    fn title(&self, window_id: window::Id) -> String {
        if let Some(window) = self.windows.get(&window_id) {
            match window.get_screen() {
                Screen::Conversation(..) => "Conversation".to_string(),
                Screen::AddContact(..) => "Add new contact".to_string(),
                Screen::PersonalSettings(..) => "Personal settings".to_string(),
                _ => "icedm".to_string(),
            }
        } else {
            "icedm".to_string()
        }
    }

    fn view(&self, window_id: window::Id) -> Element<Message> {
        if let Some(window) = self.windows.get(&window_id) {
            window.view(window_id)
        } else {
            horizontal_space().into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::events().map(Message::WindowEvent),
            keyboard::on_key_press(keyboard_listener::listen),
            Subscription::run(msnp_listener::listen).map(Message::MsnpEvent),
        ])
    }

    fn window_settings(size: Size) -> Settings {
        Settings {
            size,
            min_size: Some(size),
            position: Position::Centered,
            icon: icon::from_file_data(include_bytes!("../assets/icedm.ico"), None).ok(),
            ..Settings::default()
        }
    }
}

pub fn main() -> iced::Result {
    iced::daemon(IcedM::title, IcedM::update, IcedM::view)
        .subscription(IcedM::subscription)
        .theme(
            |_, _| match dark_light::detect().unwrap_or(Mode::Unspecified) {
                Mode::Dark => Theme::CatppuccinMocha,
                _ => Theme::CatppuccinLatte,
            },
        )
        .run_with(IcedM::new)
}
