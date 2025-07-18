use crate::screens::screen::Screen;
use crate::screens::{add_contact, contacts, sign_in};
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
                if let contacts::Message::MsnpEvent(ref event) = message {
                    match event {
                        msnp11_sdk::Event::Disconnected => {
                            self.screen = Screen::SignIn(sign_in::SignIn::new(self.sqlite.clone()));
                            return Task::done(Message::OpenDialog(
                                "Lost connection to the server".to_string(),
                            ));
                        }

                        msnp11_sdk::Event::LoggedInAnotherDevice => {
                            self.screen = Screen::SignIn(sign_in::SignIn::new(self.sqlite.clone()));
                            return Task::done(Message::OpenDialog(
                                "Disconnected as you have signed in on another computer"
                                    .to_string(),
                            ));
                        }

                        _ => (),
                    }
                }

                if let Screen::Contacts(contacts) = &mut self.screen {
                    if let Some(action) = contacts.update(message) {
                        return match action {
                            contacts::Action::SignOut(task) => {
                                self.screen =
                                    Screen::SignIn(sign_in::SignIn::new(self.sqlite.clone()));
                                task
                            }

                            contacts::Action::RunTask(task) => task,
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

            Message::AddContact(id, message) => {
                if let Screen::AddContact(add_contact) = &mut self.screen {
                    if let Some(action) = add_contact.update(message) {
                        return match action {
                            add_contact::Action::OkPressed(task) => {
                                Task::batch([task, window::close::<Message>(id)])
                            }

                            add_contact::Action::CancelPressed => window::close::<Message>(id),
                        };
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

            Screen::AddContact(client) => client
                .view()
                .map(move |message| Message::AddContact(id, message)),
        }
    }

    pub fn get_screen(&self) -> &Screen {
        &self.screen
    }
}
