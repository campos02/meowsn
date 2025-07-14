use crate::client_wrapper::ClientWrapper;
use crate::icedm_window::Window;
use crate::models::contact::Contact;
use crate::msnp_listener::Input;
use crate::screens::screen::Screen;
use crate::screens::{contacts, conversation, dialog, personal_settings, sign_in};
use crate::sqlite::Sqlite;
use dark_light::Mode;
use enums::window_type::WindowType;
use iced::futures::channel::mpsc::Sender;
use iced::widget::horizontal_space;
use iced::window::{Position, Settings, icon};
use iced::{Element, Size, Subscription, Task, Theme, keyboard, widget, window};
use msnp11_sdk::sdk_error::SdkError;
use std::collections::BTreeMap;
use std::sync::Arc;

mod client_wrapper;
mod enums;
mod icedm_window;
mod keyboard_listener;
mod models;
mod msnp_listener;
mod screens;
mod settings;
mod sign_in_async;
mod sqlite;

#[derive(Debug)]
pub enum Message {
    WindowEvent((window::Id, window::Event)),
    WindowOpened(window::Id, WindowType),
    SignIn(window::Id, sign_in::Message),
    SignedIn(window::Id, Arc<String>, Result<ClientWrapper, SdkError>),
    Contacts(window::Id, contacts::Message),
    PersonalSettings(window::Id, personal_settings::Message),
    Conversation(window::Id, conversation::Message),
    Dialog(window::Id, dialog::Message),
    OpenPersonalSettings {
        client: Option<ClientWrapper>,
        display_name: Option<String>,
    },

    OpenConversation(Contact),
    OpenDialog(String),
    MsnpEvent(msnp_listener::Event),
    EmptyResultFuture(Result<(), SdkError>),
    ContactUpdated(Contact),
    FocusNext,
    FocusPrevious,
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

    fn update(&mut self, message: Message) -> Task<Message> {
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

                    WindowType::Conversation(contact) => {
                        Screen::Conversation(conversation::Conversation::new(contact))
                    }

                    WindowType::Dialog(message) => Screen::Dialog(dialog::Dialog::new(message)),
                };

                let window = Window::new(screen, self.sqlite.clone());
                self.windows.insert(id, window);
                Task::none()
            }

            Message::WindowEvent((id, event)) => match event {
                window::Event::Closed => {
                    if let Some(window) = self.windows.remove(&id) {
                        match window.get_screen() {
                            Screen::SignIn(..) | Screen::Contacts(..) => iced::exit(),
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
            | Message::Dialog(id, ..) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    return window.update(message);
                }

                Task::none()
            }

            Message::SignedIn(id, .., ref result) => {
                if let Some(sender) = self.msnp_subscription_sender.as_mut() {
                    if let Ok(client) = result {
                        if let Err(error) =
                            sender.start_send(Input::NewClient(client.inner.clone()))
                        {
                            return Task::done(Message::OpenDialog(error.to_string()));
                        }
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

                if let Some(window) = self
                    .windows
                    .iter()
                    .find(|window| matches!(window.1.get_screen(), Screen::PersonalSettings(..)))
                {
                    return window::gain_focus(*window.0);
                }

                let (_id, open) = window::open(IcedM::window_settings(Size::new(500.0, 500.0)));
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

            Message::OpenConversation(contact) => {
                if self.windows.keys().last().is_none() {
                    return Task::none();
                };

                let (_id, open) = window::open(IcedM::window_settings(Size::new(1000.0, 600.0)));
                open.map(move |id| {
                    Message::WindowOpened(id, WindowType::Conversation(contact.clone()))
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

            Message::MsnpEvent(event) => match event {
                msnp_listener::Event::Ready(sender) => {
                    self.msnp_subscription_sender = Some(sender.clone());
                    Task::none()
                }

                msnp_listener::Event::NsEvent(event) => {
                    if let Some(window) = self
                        .windows
                        .iter_mut()
                        .find(|window| matches!(window.1.get_screen(), Screen::Contacts(..)))
                    {
                        return window.1.update(Message::Contacts(
                            *window.0,
                            contacts::Message::MsnpEvent(event),
                        ));
                    }

                    Task::none()
                }

                msnp_listener::Event::SbEvent { .. } => Task::none(),
            },

            Message::ContactUpdated(contact) => {
                if let Some(window) = self.windows.iter_mut().find(|window| {
                    if let Screen::Conversation(conversation) = window.1.get_screen() {
                        return conversation.get_contact().email == contact.email;
                    } else {
                        false
                    }
                }) {
                    return window.1.update(Message::Conversation(
                        *window.0,
                        conversation::Message::ContactUpdated(contact),
                    ));
                }

                Task::none()
            }

            Message::FocusNext => widget::focus_next(),
            Message::FocusPrevious => widget::focus_previous(),
            _ => Task::none(),
        }
    }

    fn view(&self, window_id: window::Id) -> Element<Message> {
        if let Some(window) = self.windows.get(&window_id) {
            window.view(window_id).into()
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
            icon: if let Ok(icon) = icon::from_file("assets/icedm.ico") {
                Some(icon)
            } else {
                None
            },
            ..Settings::default()
        }
    }
}

pub fn main() -> iced::Result {
    iced::daemon("icedm", IcedM::update, IcedM::view)
        .subscription(IcedM::subscription)
        .theme(
            |_, _| match dark_light::detect().unwrap_or(Mode::Unspecified) {
                Mode::Dark => Theme::CatppuccinMocha,
                _ => Theme::CatppuccinLatte,
            },
        )
        .run_with(IcedM::new)
}
