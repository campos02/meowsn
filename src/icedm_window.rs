use crate::Message;
use crate::screens::screen::Screen;
use crate::screens::{contacts, personal_settings, sign_in};
use crate::window_type::WindowType;
use iced::{Element, window};

pub enum Action {
    PersonalSettings(WindowType),
}

pub struct Window {
    screen: Screen,
}

impl Window {
    pub fn new(screen: Screen) -> Self {
        Self { screen }
    }

    pub fn update(&mut self, message: Message) -> Option<Action> {
        let mut window_action: Option<Action> = None;

        match message {
            Message::SignIn(.., message) => {
                if let Screen::SignIn(sign_in) = &mut self.screen {
                    if let Some(action) = sign_in.update(message) {
                        match action {
                            sign_in::Action::SignIn => {
                                self.screen = Screen::Contacts(contacts::Contacts::new());
                            }

                            sign_in::Action::PersonalSettings => {
                                window_action =
                                    Some(Action::PersonalSettings(WindowType::PersonalSettings));
                            }
                        }
                    }
                }
            }

            Message::Contacts(.., message) => {
                if let Screen::Contacts(contacts) = &mut self.screen {
                    contacts.update(message);
                    self.screen = Screen::Contacts(contacts::Contacts::new());
                }
            }

            Message::PersonalSettings(.., message) => {
                if let Screen::PersonalSettings(personal_settings) = &mut self.screen {
                    personal_settings.update(message);
                    self.screen =
                        Screen::PersonalSettings(personal_settings::PersonalSettings::new());
                }
            }

            _ => (),
        }

        window_action
    }

    pub fn view(&self, id: window::Id) -> Element<Message> {
        match &self.screen {
            Screen::SignIn(sign_in) => sign_in
                .view()
                .map(move |message| Message::SignIn(id, message)),

            Screen::Contacts(contacts) => contacts
                .view()
                .map(move |message| Message::Contacts(id, message)),

            Screen::PersonalSettings(personal_settings) => personal_settings
                .view()
                .map(move |message| Message::PersonalSettings(id, message)),
        }
    }

    pub fn get_screen(&self) -> &Screen {
        &self.screen
    }
}
