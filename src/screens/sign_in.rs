use crate::status::Status;
use iced::Color;
use iced::border::radius;
use iced::widget::{button, checkbox, column, container, image, pick_list, row, text, text_input};
use iced::{Border, Center, Element, Fill, Theme};

#[derive(Debug, Clone)]
pub enum Message {
    EmailChanged(String),
    PasswordChanged(String),
    StatusSelected(Status),
    RememberMeToggled(bool),
    RememberMyPasswordToggled(bool),
    SignIn,
}

#[derive(Debug, Clone)]
pub enum Action {
    SignIn,
    PersonalSettings,
}

pub struct SignIn {
    email: String,
    password: String,
    status: Option<Status>,
    remember_me: bool,
    remember_my_password: bool,
}

impl SignIn {
    pub fn new() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            status: Some(Status::Online),
            remember_me: false,
            remember_my_password: false,
        }
    }

    pub fn view(&self) -> Element<Message> {
        container(
            column![
                container(image("assets/default_display_picture.png").width(130)).style(
                    |theme: &Theme| container::Style {
                        border: Border {
                            color: theme.palette().text,
                            width: 1.0,
                            radius: radius(10.0)
                        },
                        ..Default::default()
                    }
                ),
                column![
                    column![
                        text("E-mail address:"),
                        text_input("E-mail address", &self.email)
                            .size(16)
                            .on_input(Message::EmailChanged),
                    ]
                    .spacing(5),
                    column![
                        text("Password:"),
                        text_input("Password", &self.password)
                            .size(16)
                            .on_input(Message::PasswordChanged)
                            .secure(true),
                    ]
                    .spacing(5)
                ]
                .spacing(10),
                row![
                    text("Status: "),
                    pick_list(Status::ALL, self.status.as_ref(), Message::StatusSelected)
                ]
                .spacing(3)
                .align_y(Center),
                column![
                    row![
                        checkbox("Remember Me", self.remember_me)
                            .on_toggle(Message::RememberMeToggled),
                        button("(Forget Me)").style(|theme: &Theme, status| {
                            match status {
                                button::Status::Hovered | button::Status::Pressed => {
                                    button::primary(theme, status)
                                }

                                button::Status::Active | button::Status::Disabled => {
                                    button::secondary(theme, status)
                                        .with_background(Color::TRANSPARENT)
                                }
                            }
                        })
                    ]
                    .spacing(15)
                    .align_y(Center),
                    checkbox("Remember My Password", self.remember_my_password)
                        .on_toggle(Message::RememberMyPasswordToggled),
                ],
                button("Sign In").on_press(Message::SignIn),
            ]
            .spacing(20)
            .align_x(Center),
        )
        .padding(50)
        .center_x(Fill)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Option<Action> {
        let mut action: Option<Action> = None;

        match message {
            Message::EmailChanged(email) => self.email = email,
            Message::PasswordChanged(password) => self.password = password,
            Message::StatusSelected(status) => {
                if let Status::PersonalSettings = status {
                    action = Some(Action::PersonalSettings);
                } else {
                    self.status = Some(status);
                }
            }
            Message::RememberMeToggled(remember_me) => self.remember_me = remember_me,
            Message::RememberMyPasswordToggled(remember_my_password) => {
                if remember_my_password {
                    self.remember_me = true;
                }

                self.remember_my_password = remember_my_password;
            }
            Message::SignIn => action = Some(Action::SignIn),
        }

        action
    }
}

impl Default for SignIn {
    fn default() -> Self {
        Self::new()
    }
}
