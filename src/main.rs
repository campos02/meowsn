use crate::screens::{contacts, sign_in};
use dark_light::Mode;
use iced::window::{Position, Settings, icon};
use iced::{Element, Size, Theme};

mod screens;
mod status;

enum Screen {
    SignIn(sign_in::SignIn),
    Contacts(contacts::Contacts),
}

#[derive(Debug)]
enum Message {
    SignIn(sign_in::Message),
    Contacts(contacts::Message),
}

struct IcedM {
    screen: Screen,
}

impl IcedM {
    fn new() -> Self {
        Self {
            screen: Screen::SignIn(sign_in::SignIn::default()),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SignIn(message) => {
                if let Screen::SignIn(sign_in) = &mut self.screen {
                    if let Some(action) = sign_in.update(message) {
                        match action {
                            sign_in::Action::SignIn => {
                                self.screen = Screen::Contacts(contacts::Contacts::new());
                            }

                            sign_in::Action::PersonalSettings => {
                                self.screen = Screen::Contacts(contacts::Contacts::new());
                            }
                        }
                    }
                }
            }

            Message::Contacts(message) => {
                if let Screen::Contacts(contacts) = &mut self.screen {
                    contacts.update(message);
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::SignIn(sign_in) => sign_in.view().map(Message::SignIn),
            Screen::Contacts(contacts) => contacts.view().map(Message::Contacts),
        }
    }
}

impl Default for IcedM {
    fn default() -> Self {
        Self::new()
    }
}

pub fn main() -> iced::Result {
    let mut window_settings = Settings::default();
    window_settings.size = Size::new(350.0, 700.0);
    window_settings.min_size = Some(window_settings.size);
    window_settings.position = Position::Centered;

    if let Ok(icon) = icon::from_file("assets/icedm.png") {
        window_settings.icon = Some(icon);
    }

    iced::application("icedm", IcedM::update, IcedM::view)
        .window(window_settings)
        .theme(
            |_| match dark_light::detect().unwrap_or(Mode::Unspecified) {
                Mode::Dark => Theme::CatppuccinMocha,
                _ => Theme::CatppuccinLatte,
            },
        )
        .run()
}
