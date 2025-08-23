use iced::widget::{button, column, container, row, text, text_input};
use iced::{Center, Element, Fill, Task};
use msnp11_sdk::{Client, MsnpList};
use std::sync::Arc;

#[derive(Clone)]
pub enum Message {
    EmailChanged(String),
    DisplayNameChanged(String),
    OkPressed,
    CancelPressed,
}

pub enum Action {
    OkPressed(Task<crate::Message>),
    CancelPressed,
}

pub struct AddContact {
    email: String,
    display_name: String,
    client: Arc<Client>,
}

impl AddContact {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            email: String::new(),
            display_name: String::new(),
            client,
        }
    }

    pub fn view(&self) -> Element<Message> {
        container(
            column![
                column![
                    text("Type your contact's e-mail address:").size(14),
                    text_input("Contact e-mail", &self.email)
                        .on_input(Message::EmailChanged)
                        .size(14),
                ]
                .spacing(5),
                column![
                    text("Type your contact's display name:").size(14),
                    text_input("Contact display name", &self.display_name)
                        .on_input(Message::DisplayNameChanged)
                        .size(14),
                ]
                .spacing(5),
                row![
                    button("Ok").on_press(Message::OkPressed),
                    button("Cancel").on_press(Message::CancelPressed)
                ]
                .spacing(5),
            ]
            .spacing(20)
            .align_x(Center),
        )
        .center_x(Fill)
        .padding(20)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Option<Action> {
        let mut action = None;
        match message {
            Message::EmailChanged(email) => self.email = email,
            Message::DisplayNameChanged(display_name) => self.display_name = display_name,
            Message::OkPressed => {
                let client = self.client.clone();
                let email = self.email.clone();
                let display_name = self.display_name.clone();

                action = Some(Action::OkPressed(Task::perform(
                    async move {
                        if !display_name.is_empty() {
                            client
                                .add_contact(&email, &display_name, MsnpList::ForwardList)
                                .await
                        } else {
                            client
                                .add_contact(&email, &email, MsnpList::ForwardList)
                                .await
                        }
                    },
                    crate::Message::EventResult,
                )));
            }

            Message::CancelPressed => action = Some(Action::CancelPressed),
        }

        action
    }
}
