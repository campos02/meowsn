use crate::models::user::User;
use crate::sign_in_status::SignInStatus;
use iced::Color;
use iced::border::radius;
use iced::widget::{
    button, checkbox, column, combo_box, container, image, pick_list, row, text, text_input,
};
use iced::{Border, Center, Element, Fill, Theme};
use keyring::Entry;
use msnp11_sdk::sdk_error::SdkError;
use msnp11_sdk::{MsnpStatus, PersonalMessage};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2_sqlite::rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;
use r2d2_sqlite::rusqlite::params;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone)]
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
    Dialog(Arc<String>),
}

pub struct SignIn {
    email: Option<String>,
    emails: combo_box::State<String>,
    password: String,
    status: Option<SignInStatus>,
    remember_me: bool,
    remember_my_password: bool,
    signing_in: bool,
    pool: Pool<SqliteConnectionManager>,
}

impl SignIn {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        let mut email = None;
        let mut emails = Vec::new();
        let mut password = String::new();
        let mut remember_me = false;
        let mut remember_my_password = false;

        if let Ok(conn) = pool.get() {
            if let Ok(mut stmt) = conn.prepare("SELECT email FROM users") {
                let users = stmt.query_map([], |row| {
                    Ok(User {
                        email: row.get(0)?,
                        personal_message: None,
                    })
                });

                if let Ok(users) = users {
                    for user in users {
                        if let Ok(user) = user {
                            emails.push(user.email);
                        }
                    }
                }
            }
        }

        if let Some(last_email) = emails.last() {
            email = Some(last_email.clone());
            remember_me = true;

            if let Ok(entry) = Entry::new("icedm", last_email) {
                if let Ok(passwd) = entry.get_password() {
                    password = passwd;
                    remember_my_password = true;
                }
            }
        }

        Self {
            email,
            emails: combo_box::State::new(emails),
            password,
            status: Some(SignInStatus::Online),
            remember_me,
            remember_my_password,
            signing_in: false,
            pool,
        }
    }

    pub fn view(&self) -> Element<Message> {
        container(
            column![
                container(image("assets/default_display_picture.png").width(120))
                    .style(|theme: &Theme| container::Style {
                        border: Border {
                            color: theme.palette().text,
                            width: 1.0,
                            radius: radius(10.0)
                        },
                        ..Default::default()
                    })
                    .padding(3),
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
                        button(text("(Forget Me)").size(14))
                            .style(|theme: &Theme, status| {
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
            Message::EmailInput(email) => self.email = Some(email),
            Message::EmailSelected(email) => {
                if let Ok(entry) = Entry::new("icedm", &email) {
                    if let Ok(passwd) = entry.get_password() {
                        self.password = passwd;
                        self.remember_my_password = true;
                    }
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
                if self.email.as_ref().is_none_or(|email| email.is_empty())
                    || self.password.is_empty()
                {
                    action = Some(Action::Dialog(Arc::new(
                        "Please type your e-mail address and password in their corresponding forms."
                            .to_string(),
                    )))
                } else {
                    self.signing_in = true;
                    if self.remember_me {
                        if let Ok(conn) = self.pool.get() {
                            if let Ok(mut stmt) =
                                conn.prepare("SELECT email FROM users WHERE email = ?1")
                            {
                                if let Ok(rows) = stmt.query(params![self.email]) {
                                    if let Ok(count) = rows.count() {
                                        if count == 0 {
                                            let _ = conn.execute(
                                                "INSERT INTO users (email) VALUES (?1)",
                                                params![self.email],
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        if self.remember_my_password {
                            if let Some(ref email) = self.email {
                                if let Ok(entry) = Entry::new("icedm", email) {
                                    let _ = entry.set_password(&*self.password);
                                }
                            }
                        }
                    }

                    action = Some(Action::SignIn);
                }
            }

            Message::ForgetMe => {
                if let Ok(conn) = self.pool.get() {
                    let _ = conn.execute("DELETE FROM users WHERE email = ?1", params![self.email]);
                }

                if let Some(ref email) = self.email {
                    if let Ok(entry) = Entry::new("icedm", email) {
                        let _ = entry.delete_credential();
                    }
                }

                self.email = Some(String::new());
                self.password = String::new();
                self.remember_me = false;
                self.remember_my_password = false;
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
            Arc::new(self.email.clone().unwrap_or(String::new())),
            Arc::new(self.password.clone()),
            self.status.clone(),
        )
    }
}

pub struct Client {
    pub personal_message: String,
    pub inner: Arc<msnp11_sdk::Client>,
}

impl Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").finish()
    }
}

pub async fn sign_in(
    email: Arc<String>,
    password: Arc<String>,
    status: Option<SignInStatus>,
    pool: Pool<SqliteConnectionManager>,
) -> Result<Client, SdkError> {
    let mut client =
        msnp11_sdk::Client::new("r2m.camposs.net".to_string(), "1863".to_string()).await?;

    if let msnp11_sdk::Event::RedirectedTo { server, port } = client
        .login(
            (*email).clone(),
            (*password).clone(),
            "https://r2m.camposs.net/rdr/pprdr.asp".to_string(),
        )
        .await?
    {
        client = msnp11_sdk::Client::new(server, port).await?;
        client
            .login(
                (*email).clone(),
                (*password).clone(),
                "https://r2m.camposs.net/rdr/pprdr.asp".to_string(),
            )
            .await?;
    }

    let status = match status {
        Some(status) => match status {
            SignInStatus::Busy => MsnpStatus::Busy,
            SignInStatus::Away => MsnpStatus::Away,
            SignInStatus::AppearOffline => MsnpStatus::AppearOffline,
            _ => MsnpStatus::Online,
        },
        None => MsnpStatus::Online,
    };

    client.set_presence(status).await?;

    let mut psm = String::new();
    if let Ok(conn) = pool.get() {
        if let Ok(mut stmt) = conn.prepare("SELECT personal_message FROM users") {
            let users = stmt.query_map([], |row| {
                Ok(User {
                    email: String::new(),
                    personal_message: row.get(0)?,
                })
            });

            if let Ok(users) = users {
                if let Some(Ok(user)) = users.last() {
                    if let Some(personal_message) = user.personal_message {
                        psm = personal_message;
                    }
                }
            }
        }
    }

    let personal_message = PersonalMessage {
        psm,
        current_media: "".to_string(),
    };

    client.set_personal_message(&personal_message).await?;
    Ok(Client {
        personal_message: personal_message.psm,
        inner: Arc::new(client),
    })
}
