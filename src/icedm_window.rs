use crate::helpers::sign_in_async;
use crate::msnp_listener::Input;
use crate::screens::screen::Screen;
use crate::screens::{add_contact, contacts, conversation, personal_settings, sign_in};
use crate::sqlite::Sqlite;
use crate::{Message, msnp_listener};
use iced::futures::channel::mpsc::Sender;
use iced::{Element, Task, window};
use std::time::Duration;

pub struct Window {
    id: window::Id,
    screen: Screen,
    sqlite: Sqlite,
    msnp_subscription_sender: Option<Sender<Input>>,
}

impl Window {
    pub fn new(
        id: window::Id,
        screen: Screen,
        sqlite: Sqlite,
        msnp_subscription_sender: Option<Sender<Input>>,
    ) -> Self {
        Self {
            id,
            screen,
            sqlite,
            msnp_subscription_sender,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SignIn(id, message) => {
                if let Screen::SignIn(sign_in) = &mut self.screen
                    && let Some(action) = sign_in.update(message)
                {
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
                                move |result| Message::SignedIn {
                                    id,
                                    email: email.clone(),
                                    result,
                                },
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

                Task::none()
            }

            Message::Contacts(.., message) => {
                if let contacts::Message::NotificationServerEvent(ref event) = message {
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

                if let Screen::Contacts(contacts) = &mut self.screen
                    && let Some(action) = contacts.update(message)
                {
                    return match action {
                        contacts::Action::SignOut(task) => {
                            self.screen = Screen::SignIn(sign_in::SignIn::new(self.sqlite.clone()));
                            task
                        }

                        contacts::Action::RunTask(task) => task,
                    };
                }

                Task::none()
            }

            Message::PersonalSettings(id, message) => {
                if let Screen::PersonalSettings(personal_settings) = &mut self.screen
                    && let Some(action) = personal_settings.update(message)
                {
                    return match action {
                        personal_settings::Action::RunTask(task) => task,
                        personal_settings::Action::SavePressed(task) => {
                            Task::batch([task, window::close::<Message>(id)])
                        }
                    };
                }

                Task::none()
            }

            Message::Conversation(id, message) => {
                if let Screen::Conversation(conversation) = &mut self.screen
                    && let Some(action) = conversation.update(message)
                {
                    return match action {
                        conversation::Action::ParticipantTypingTimeout => {
                            Task::perform(tokio::time::sleep(Duration::from_secs(5)), move |_| {
                                Message::Conversation(
                                    id,
                                    conversation::Message::ParticipantTypingTimeout,
                                )
                            })
                        }

                        conversation::Action::UserTypingTimeout(task) => {
                            let id = self.id;
                            Task::batch([
                                task,
                                Task::perform(
                                    tokio::time::sleep(Duration::from_secs(5)),
                                    move |_| {
                                        Message::Conversation(
                                            id,
                                            conversation::Message::UserTypingTimeout,
                                        )
                                    },
                                ),
                            ])
                        }

                        conversation::Action::RunTask(task) => task,
                    };
                }

                Task::none()
            }

            Message::Dialog(id, message) => {
                if let Screen::Dialog(dialog) = &mut self.screen
                    && dialog.update(message).is_some()
                {
                    return window::close::<Message>(id);
                }

                Task::none()
            }

            Message::AddContact(id, message) => {
                if let Screen::AddContact(add_contact) = &mut self.screen
                    && let Some(action) = add_contact.update(message)
                {
                    return match action {
                        add_contact::Action::OkPressed(task) => {
                            Task::batch([task, window::close::<Message>(id)])
                        }

                        add_contact::Action::CancelPressed => window::close::<Message>(id),
                    };
                }

                Task::none()
            }

            Message::SignedIn {
                id: _,
                email,
                result,
            } => {
                match result {
                    Ok((personal_message, initial_status, client)) => {
                        self.screen = Screen::Contacts(contacts::Contacts::new(
                            email,
                            personal_message,
                            initial_status,
                            client,
                            self.sqlite.clone(),
                            self.msnp_subscription_sender.clone(),
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

            Message::MsnpEvent(event) => {
                if let msnp_listener::Event::Switchboard { session_id, event } = event {
                    match &mut self.screen {
                        Screen::Conversation(conversation) => {
                            if *conversation.get_session_id() == *session_id
                                && let Some(action) = conversation
                                    .update(conversation::Message::MsnpEvent(Box::from(event)))
                            {
                                return match action {
                                    conversation::Action::ParticipantTypingTimeout => {
                                        let id = self.id;
                                        Task::perform(
                                            tokio::time::sleep(Duration::from_secs(5)),
                                            move |_| {
                                                Message::Conversation(
                                                    id,
                                                    conversation::Message::ParticipantTypingTimeout,
                                                )
                                            },
                                        )
                                    }

                                    conversation::Action::UserTypingTimeout(task) => {
                                        let id = self.id;
                                        Task::batch([
                                            task,
                                            Task::perform(
                                                tokio::time::sleep(Duration::from_secs(5)),
                                                move |_| {
                                                    Message::Conversation(
                                                        id,
                                                        conversation::Message::UserTypingTimeout,
                                                    )
                                                },
                                            ),
                                        ])
                                    }

                                    conversation::Action::RunTask(task) => task,
                                };
                            }
                        }

                        Screen::Contacts(contacts) => {
                            if let Some(action) = contacts
                                .update(contacts::Message::SwitchboardEvent(session_id, event))
                            {
                                return match action {
                                    contacts::Action::SignOut(task) => {
                                        self.screen = Screen::SignIn(sign_in::SignIn::new(
                                            self.sqlite.clone(),
                                        ));
                                        task
                                    }

                                    contacts::Action::RunTask(task) => task,
                                };
                            }
                        }

                        _ => (),
                    }
                }

                Task::none()
            }

            _ => Task::none(),
        }
    }

    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
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
