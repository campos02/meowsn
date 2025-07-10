use crate::Message;
use crate::screens::screen::Screen;
use crate::screens::{contacts, sign_in};
use iced::{Element, Task, widget, window};
use std::sync::Arc;

pub struct Window {
    screen: Screen,
}

impl Window {
    pub fn new(screen: Screen) -> Self {
        Self { screen }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SignIn(id, message) => {
                if let Screen::SignIn(sign_in) = &mut self.screen {
                    if let Some(action) = sign_in.update(message) {
                        return match action {
                            sign_in::Action::SignIn => {
                                let (email, password, status) = sign_in.get_sign_in_info();
                                Task::perform(
                                    sign_in::sign_in(email, password, status),
                                    move |result| Message::SignedIn(id, result),
                                )
                            }

                            sign_in::Action::PersonalSettings => {
                                Task::done(Message::OpenPersonalSettings)
                            }

                            sign_in::Action::Dialog(message) => {
                                Task::done(Message::OpenDialog(message))
                            }
                        };
                    }
                }

                Task::none()
            }

            Message::Contacts(.., message) => {
                if let Screen::Contacts(contacts) = &mut self.screen {
                    if let Some(action) = contacts.update(message) {
                        return match action {
                            contacts::Action::PersonalSettings => {
                                Task::done(Message::OpenPersonalSettings)
                            }

                            contacts::Action::SignOut(client) => {
                                self.screen = Screen::SignIn(sign_in::SignIn::default());
                                Task::perform(async move { client.disconnect().await }, |result| {
                                    Message::EmptyResultFuture(result)
                                })
                            }

                            contacts::Action::FocusNext => widget::focus_next(),
                            contacts::Action::Conversation(contact) => {
                                Task::done(Message::OpenConversation(contact))
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

            Message::SignedIn(.., result) => {
                match result {
                    Ok(client) => {
                        self.screen = Screen::Contacts(contacts::Contacts::new(client.inner))
                    }

                    Err(error) => {
                        if let Screen::SignIn(sign_in) = &mut self.screen {
                            sign_in.update(sign_in::Message::SignInFailed);
                        }

                        return Task::done(Message::OpenDialog(Arc::new(error.to_string())));
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
