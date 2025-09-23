use crate::enums::sign_in_status::SignInStatus;
use crate::screens::sign_in::bordered_container::bordered_container;
use crate::sqlite::Sqlite;
use iced::widget;
use iced::widget::{
    button, checkbox, column, combo_box, container, pick_list, row, svg, text, text_input,
};
use iced::{Center, Element, Fill};
use keyring::Entry;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone)]
pub enum Message {
    EmailInput(String),
    EmailSelected(String),
    PasswordChanged(String),
    StatusSelected(SignInStatus),
    ForgetMe,
    RememberMeToggled(bool),
    RememberMyPasswordToggled(bool),
    SignIn,
    SignInFailed,
}

pub enum Action {
    SignIn,
    PersonalSettings,
    Dialog(String),
}

pub struct SignIn {
    email: Option<String>,
    display_picture: Option<Cow<'static, [u8]>>,
    emails: combo_box::State<String>,
    password: String,
    status: Option<SignInStatus>,
    remember_me: bool,
    remember_my_password: bool,
    signing_in: bool,
    sqlite: Sqlite,
}

impl SignIn {
    pub fn new(sqlite: Sqlite) -> Self {
        let mut email = None;
        let mut password = String::new();
        let mut remember_me = false;
        let mut remember_my_password = false;
        let mut display_picture = None;

        let emails = sqlite.select_user_emails().unwrap_or_default();
        if let Some(last_email) = emails.first() {
            email = Some(last_email.to_owned());
            remember_me = true;

            if let Ok(entry) = Entry::new("meowsn", last_email)
                && let Ok(passwd) = entry.get_password()
            {
                password = passwd;
                remember_my_password = true;
            }
        }

        if let Some(ref email) = email
            && let Ok(user) = sqlite.select_user(email)
            && let Some(picture) = user.display_picture
        {
            display_picture = Some(Cow::Owned(picture))
        }

        Self {
            email,
            display_picture,
            emails: combo_box::State::new(emails),
            password,
            status: Some(SignInStatus::Online),
            remember_me,
            remember_my_password,
            signing_in: false,
            sqlite,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(
            column![
                if let Some(picture) = self.display_picture.clone() {
                    bordered_container(widget::image(widget::image::Handle::from_bytes(Box::from(
                        picture,
                    ))))
                } else {
                    bordered_container(svg(crate::svg::default_display_picture()))
                },
                column![
                    column![
                        text("E-mail address:").size(14),
                        combo_box(
                            &self.emails,
                            "E-mail address",
                            self.email.as_ref(),
                            Message::EmailSelected,
                        )
                        .size(14.0)
                        .on_input(Message::EmailInput),
                    ]
                    .spacing(5),
                    column![
                        text("Password:").size(14),
                        text_input("Password", &self.password)
                            .size(14)
                            .on_input(Message::PasswordChanged)
                            .secure(true),
                    ]
                    .spacing(5)
                ]
                .spacing(10),
                row![
                    text("Status: ").size(14),
                    pick_list(
                        SignInStatus::ALL,
                        self.status.as_ref(),
                        Message::StatusSelected
                    )
                    .text_size(14)
                ]
                .spacing(3)
                .align_y(Center),
                column![
                    row![
                        checkbox("Remember Me", self.remember_me)
                            .on_toggle(Message::RememberMeToggled)
                            .size(12),
                        button("(Forget Me)")
                            .style(button::text)
                            .on_press(Message::ForgetMe)
                    ]
                    .spacing(15)
                    .align_y(Center),
                    checkbox("Remember My Password", self.remember_my_password)
                        .on_toggle(Message::RememberMyPasswordToggled)
                        .size(12),
                ],
                if self.signing_in {
                    button("Sign In")
                } else {
                    button("Sign In").on_press(Message::SignIn)
                },
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
            Message::EmailInput(email) => {
                self.email = Some(email);
                self.display_picture = None;
            }

            Message::EmailSelected(email) => {
                if let Ok(entry) = Entry::new("meowsn", &email)
                    && let Ok(passwd) = entry.get_password()
                {
                    self.password = passwd;
                    self.remember_my_password = true;
                }

                if let Ok(user) = self.sqlite.select_user(&email)
                    && let Some(picture) = user.display_picture
                {
                    self.display_picture = Some(Cow::Owned(picture))
                }

                self.remember_me = true;
                self.email = Some(email);
            }

            Message::PasswordChanged(password) => self.password = password,
            Message::StatusSelected(status) => {
                if let SignInStatus::PersonalSettings = status {
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

            Message::SignIn => {
                if let Some(ref mut email) = self.email {
                    *email = email.trim().to_string();
                }

                if self.email.as_ref().is_none_or(|email| email.is_empty())
                    || self.password.is_empty()
                {
                    action = Some(Action::Dialog("Please type your e-mail address and password in their corresponding forms."
                        .to_string()))
                } else {
                    self.signing_in = true;
                    if self.remember_me {
                        if let Some(ref email) = self.email {
                            let _ = self.sqlite.insert_user_if_not_in_db(email);
                        }

                        if self.remember_my_password
                            && let Some(ref email) = self.email
                            && let Ok(entry) = Entry::new("meowsn", email)
                        {
                            let _ = entry.set_password(&self.password);
                        }
                    }

                    action = Some(Action::SignIn);
                }
            }

            Message::ForgetMe => {
                if let Some(ref email) = self.email {
                    let _ = self.sqlite.delete_user(email);
                    if let Ok(entry) = Entry::new("meowsn", email) {
                        let _ = entry.delete_credential();
                    }
                }

                self.email = Some(String::new());
                self.emails =
                    combo_box::State::new(self.sqlite.select_user_emails().unwrap_or_default());

                self.password = String::new();
                self.remember_me = false;
                self.remember_my_password = false;
                self.display_picture = None;
                self.status = Some(SignInStatus::Online);
            }

            Message::SignInFailed => {
                self.signing_in = false;
            }
        }

        action
    }

    pub fn get_sign_in_info(&self) -> (Arc<String>, Arc<String>, Option<SignInStatus>) {
        (
            Arc::new(self.email.clone().unwrap_or_default()),
            Arc::new(self.password.clone()),
            self.status.clone(),
        )
    }
}
