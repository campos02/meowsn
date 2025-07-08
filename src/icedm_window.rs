use crate::Message;
use crate::screens::screen::Screen;
use crate::screens::{contacts, sign_in};
use iced::{Element, Task, widget, window};

pub struct Window {
    screen: Screen,
}

impl Window {
    pub fn new(screen: Screen) -> Self {
        Self { screen }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SignIn(.., message) => {
                if let Screen::SignIn(sign_in) = &mut self.screen {
                    if let Some(action) = sign_in.update(message) {
                        match action {
                            sign_in::Action::SignIn => {
                                self.screen = Screen::Contacts(contacts::Contacts::new());
                            }

                            sign_in::Action::PersonalSettings => {
                                return Task::done(Message::OpenPersonalSettings);
                            }

                            sign_in::Action::Dialog(message) => {
                                return Task::done(Message::OpenDialog(message));
                            }
                        }
                    }
                }

                Task::none()
            }

            Message::Contacts(.., message) => {
                if let Screen::Contacts(contacts) = &mut self.screen {
                    if let Some(action) = contacts.update(message) {
                        match action {
                            contacts::Action::PersonalSettings => {
                                return Task::done(Message::OpenPersonalSettings);
                            }

                            contacts::Action::SignOut => {
                                self.screen = Screen::SignIn(sign_in::SignIn::default());
                            }

                            contacts::Action::FocusNext => return widget::focus_next(),
                            contacts::Action::Conversation(contact) => {
                                return Task::done(Message::OpenConversation(contact));
                            }
                        };
                    }
                }

                Task::none()
            }

            Message::PersonalSettings(.., message) => {
                if let Screen::PersonalSettings(personal_settings) = &mut self.screen {
                    personal_settings.update(message);
                }

                Task::none()
            }

            Message::Conversation(.., message) => {
                if let Screen::Conversation(conversation) = &mut self.screen {
                    conversation.update(message);
                }

                Task::none()
            }

            Message::Dialog(id, message) => {
                if let Screen::Dialog(dialog) = &mut self.screen {
                    if let Some(_action) = dialog.update(message) {
                        return window::close::<Message>(id);
                    }
                }

                Task::none()
            }

            _ => Task::none(),
        }
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

            Screen::Conversation(conversation) => conversation
                .view()
                .map(move |message| Message::Conversation(id, message)),

            Screen::Dialog(dialog) => dialog
                .view()
                .map(move |message| Message::Dialog(id, message)),
        }
    }

    pub fn get_screen(&self) -> &Screen {
        &self.screen
    }
}
