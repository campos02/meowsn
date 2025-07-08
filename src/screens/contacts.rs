use crate::contact_list_status::ContactListStatus;
use iced::border::radius;
use iced::widget::{button, column, container, image, pick_list, row, text, text_input};
use iced::{Background, Border, Center, Color, Element, Fill, Theme};
use std::sync::Arc;

pub enum Action {
    PersonalSettings,
    SignOut,
    FocusNext,
    Conversation(Arc<String>),
}

#[derive(Debug, Clone)]
pub enum Message {
    PersonalMessageChanged(String),
    PersonalMessageSubmit,
    StatusSelected(ContactListStatus),
    Conversation(Arc<String>),
}

pub struct Contacts {
    personal_message: String,
    status: Option<ContactListStatus>,
}

impl Contacts {
    pub fn new() -> Self {
        Self {
            personal_message: String::new(),
            status: Some(ContactListStatus::Online),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let contacts = ["test 1", "test 2", "test 3"].iter();

        container(
            column![
                row![
                    container(image("assets/default_display_picture.png").width(70))
                        .style(|theme: &Theme| container::Style {
                            border: Border {
                                color: theme.palette().text,
                                width: 1.0,
                                radius: radius(10.0),
                            },
                            ..Default::default()
                        })
                        .padding(3),
                    column![
                        row![
                            text(" testing@example.com").size(14),
                            pick_list(
                                ContactListStatus::ALL,
                                self.status.as_ref(),
                                Message::StatusSelected
                            )
                            .text_size(14)
                            .style(|theme: &Theme, status| {
                                match status {
                                    pick_list::Status::Active => {
                                        let mut list = pick_list::default(theme, status);
                                        list.background = Background::Color(Color::TRANSPARENT);
                                        list.border.width = 0.0;
                                        list
                                    }

                                    _ => {
                                        let mut list = pick_list::default(theme, status);
                                        list.border.color =
                                            theme.extended_palette().secondary.strong.color;
                                        list.background = Background::Color(Color::TRANSPARENT);
                                        list
                                    }
                                }
                            })
                        ]
                        .align_y(Center)
                        .spacing(20),
                        text_input("<Type a personal message>", &self.personal_message)
                            .size(14)
                            .on_input(Message::PersonalMessageChanged)
                            .on_submit(Message::PersonalMessageSubmit)
                            .style(|theme: &Theme, status| {
                                match status {
                                    text_input::Status::Active | text_input::Status::Disabled => {
                                        let mut list = text_input::default(theme, status);
                                        list.border.width = 0.0;
                                        list
                                    }

                                    _ => {
                                        let mut list = text_input::default(theme, status);
                                        list.border.color =
                                            theme.extended_palette().secondary.strong.color;
                                        list
                                    }
                                }
                            }),
                    ]
                    .spacing(5)
                ]
                .spacing(10),
                button("Add a contact")
                    .style(|theme: &Theme, status| {
                        match status {
                            button::Status::Hovered | button::Status::Pressed => {
                                button::primary(theme, status)
                            }

                            button::Status::Active | button::Status::Disabled => {
                                button::secondary(theme, status).with_background(Color::TRANSPARENT)
                            }
                        }
                    })
                    .on_press(Message::PersonalMessageSubmit),
                column(contacts.map(|contact| {
                    button(
                        row![
                            row![
                                container(image("assets/default_display_picture.png").width(40))
                                    .style(|theme: &Theme| container::Style {
                                        border: Border {
                                            color: theme.palette().text,
                                            width: 1.0,
                                            radius: radius(6.0),
                                        },
                                        ..Default::default()
                                    })
                                    .padding(3),
                                text(*contact)
                            ]
                            .spacing(10)
                            .width(Fill)
                            .align_y(Center),
                            row![
                                button(text("Block").size(15))
                                    .on_press(Message::PersonalMessageSubmit),
                                button(text("Delete").size(15))
                                    .on_press(Message::PersonalMessageSubmit)
                            ]
                            .spacing(5)
                        ]
                        .align_y(Center)
                        .spacing(120),
                    )
                    .on_press(Message::Conversation(Arc::new(contact.to_string())))
                    .style(|theme: &Theme, status| match status {
                        button::Status::Hovered | button::Status::Pressed => {
                            button::secondary(theme, status)
                        }

                        button::Status::Active | button::Status::Disabled => {
                            button::secondary(theme, status).with_background(Color::TRANSPARENT)
                        }
                    })
                    .width(Fill)
                    .into()
                }))
                .spacing(10)
            ]
            .spacing(10),
        )
        .padding(15)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Option<Action> {
        let mut action: Option<Action> = None;
        match message {
            Message::PersonalMessageChanged(personal_message) => {
                self.personal_message = personal_message;
            }

            Message::PersonalMessageSubmit => action = Some(Action::FocusNext),
            Message::StatusSelected(status) => match status {
                ContactListStatus::PersonalSettings => action = Some(Action::PersonalSettings),
                ContactListStatus::SignOut => action = Some(Action::SignOut),
                _ => self.status = Some(status),
            },

            Message::Conversation(contact) => action = Some(Action::Conversation(contact)),
        }

        action
    }
}

impl Default for Contacts {
    fn default() -> Self {
        Self::new()
    }
}
