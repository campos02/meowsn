use crate::icedm_window::Window;
use crate::screens::screen::Screen;
use crate::screens::{contacts, conversation, personal_settings, sign_in};
use crate::window_type::WindowType;
use dark_light::Mode;
use iced::widget::horizontal_space;
use iced::window::{Position, Settings, icon};
use iced::{Element, Size, Subscription, Task, Theme, window};
use std::collections::BTreeMap;
use std::sync::Arc;

mod contact_list_status;
mod icedm_window;
mod screens;
mod sign_in_status;
mod window_type;

#[derive(Debug)]
pub enum Message {
    WindowOpened(window::Id, WindowType),
    WindowClosed(window::Id),
    SignIn(window::Id, sign_in::Message),
    Contacts(window::Id, contacts::Message),
    PersonalSettings(window::Id, personal_settings::Message),
    Conversation(window::Id, conversation::Message),
    OpenPersonalSettings,
    OpenConversation(Arc<String>),
}

struct IcedM {
    windows: BTreeMap<window::Id, Window>,
}

impl IcedM {
    fn new() -> (Self, Task<Message>) {
        let (_id, open) = window::open(IcedM::window_settings(Size::new(450.0, 600.0)));
        (
            Self {
                windows: BTreeMap::new(),
            },
            open.map(move |id| Message::WindowOpened(id, WindowType::MainWindow)),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(id, window_type) => {
                let screen = match window_type {
                    WindowType::MainWindow => Screen::SignIn(sign_in::SignIn::default()),
                    WindowType::PersonalSettings => {
                        Screen::PersonalSettings(personal_settings::PersonalSettings::default())
                    }

                    WindowType::Conversation(contact) => {
                        Screen::Conversation(conversation::Conversation::new(contact))
                    }
                };

                let window = Window::new(screen);
                self.windows.insert(id, window);
                Task::none()
            }

            Message::WindowClosed(id) => {
                if let Some(window) = self.windows.remove(&id) {
                    match window.get_screen() {
                        Screen::SignIn(..) | Screen::Contacts(..) => iced::exit(),
                        _ => Task::none(),
                    }
                } else {
                    Task::none()
                }
            }

            Message::Contacts(id, ..)
            | Message::PersonalSettings(id, ..)
            | Message::SignIn(id, ..)
            | Message::Conversation(id, ..) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    return window.update(message);
                }

                Task::none()
            }

            Message::OpenPersonalSettings => {
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
                open.map(move |id| Message::WindowOpened(id, WindowType::PersonalSettings))
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
        window::close_events().map(Message::WindowClosed)
    }

    fn window_settings(size: Size) -> Settings {
        Settings {
            size,
            min_size: Some(size),
            position: Position::Centered,
            icon: if let Ok(icon) = icon::from_file("assets/icedm.png") {
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
