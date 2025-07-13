use crate::screens::screen::Screen;
use crate::screens::{contacts, sign_in};
use crate::sqlite::Sqlite;
use crate::{Message, sign_in_async};
use iced::{Element, Task, window};

pub struct Window {
    screen: Screen,
    sqlite: Sqlite,
}

impl Window {
    pub fn new(screen: Screen, sqlite: Sqlite) -> Self {
        Self { screen, sqlite }
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
                                    sign_in_async::sign_in_async(
                                        email.clone(),
                                        password,
                                        status,
                                        self.sqlite.clone(),
                                    ),
                                    move |result| Message::SignedIn(id, email.clone(), result),
                                )
                            }

                            sign_in::Action::PersonalSettings => {
                                Task::done(Message::OpenPersonalSettings {
                                    client: None,
                                    display_name: None,
                                })
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
                            contacts::Action::PersonalSettings {
                                client,
                                display_name,
                            } => Task::done(Message::OpenPersonalSettings {
                                client,
                                display_name,
                            }),

                            contacts::Action::SignOut(task) => {
                                self.screen =
                                    Screen::SignIn(sign_in::SignIn::new(self.sqlite.clone()));
                                task
                            }

                            contacts::Action::PersonalMessageSubmit(task)
                            | contacts::Action::StatusSelected(task) => task,

                            contacts::Action::Conversation(contact) => {
                                Task::done(Message::OpenConversation(contact))
                            }

                            contacts::Action::ContactUpdated(contact) => {
                                Task::done(Message::ContactUpdated(contact))
                            }
                        };
                    }
                }

                Task::none()
            }

            Message::PersonalSettings(.., message) => {
                if let Screen::PersonalSettings(personal_settings) = &mut self.screen {
                    return personal_settings.update(message);
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

            Message::SignedIn(.., email, result) => {
                match result {
                    Ok(client) => {
                        self.screen = Screen::Contacts(contacts::Contacts::new(
                            email,
                            client.personal_message,
                            client.inner,
                            self.sqlite.clone(),
                        ));
                    }

                    Err(error) => {
                        if let Screen::SignIn(sign_in) = &mut self.screen {
                            sign_in.update(sign_in::Message::SignInFailed);
                        }

                        return Task::done(Message::OpenDialog(error.to_string()));
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
