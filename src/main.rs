mod status;

use crate::status::Status;
use dark_light::Mode;
use iced::Color;
use iced::border::radius;
use iced::widget::{button, checkbox, column, container, image, pick_list, row, text, text_input};
use iced::window::{Position, Settings, icon};
use iced::{Border, Center, Element, Fill, Size, Theme};

struct State {
    email: String,
    password: String,
    status: Option<Status>,
    remember_me: bool,
    remember_my_password: bool,
}

impl State {
    fn new() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            status: Some(Status::Online),
            remember_me: false,
            remember_my_password: false,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
enum Message {
    EmailChanged(String),
    PasswordChanged(String),
    StatusSelected(Status),
    RememberMeToggled(bool),
    RememberMyPasswordToggled(bool),
}

fn view(state: &State) -> Element<Message> {
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
                    text_input("E-mail address", &state.email)
                        .size(16)
                        .on_input(Message::EmailChanged),
                ]
                .spacing(5),
                column![
                    text("Password:"),
                    text_input("Password", &state.password)
                        .size(16)
                        .on_input(Message::PasswordChanged)
                        .secure(true),
                ]
                .spacing(5)
            ]
            .spacing(10),
            row![
                text("Status: "),
                pick_list(Status::ALL, state.status.as_ref(), Message::StatusSelected)
            ]
            .spacing(3)
            .align_y(Center),
            column![
                row![
                    checkbox("Remember Me", state.remember_me)
                        .on_toggle(Message::RememberMeToggled),
                    button("(Forget Me)").style(|theme: &Theme, status| {
                        match status {
                            button::Status::Hovered | button::Status::Pressed => {
                                button::primary(theme, status)
                            }

                            button::Status::Active | button::Status::Disabled => {
                                button::secondary(theme, status).with_background(Color::TRANSPARENT)
                            }
                        }
                    })
                ]
                .spacing(15)
                .align_y(Center),
                checkbox("Remember My Password", state.remember_my_password)
                    .on_toggle(Message::RememberMyPasswordToggled),
            ],
            button("Sign In"),
        ]
        .spacing(20)
        .align_x(Center),
    )
    .padding(50)
    .center_x(Fill)
    .into()
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::EmailChanged(email) => state.email = email,
        Message::PasswordChanged(password) => state.password = password,
        Message::StatusSelected(status) => state.status = Some(status),
        Message::RememberMeToggled(remember_me) => state.remember_me = remember_me,
        Message::RememberMyPasswordToggled(remember_my_password) => {
            if remember_my_password {
                state.remember_me = true;
            }

            state.remember_my_password = remember_my_password
        }
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

    iced::application("icedm", update, view)
        .window(window_settings)
        .theme(
            |_| match dark_light::detect().unwrap_or(Mode::Unspecified) {
                Mode::Dark => Theme::CatppuccinMocha,
                _ => Theme::CatppuccinLatte,
            },
        )
        .run()
}
