use crate::settings::Settings;
use crate::settings;
use iced::widget::{button, checkbox, column, container, text, text_input, vertical_space};
use iced::{Center, Element, Fill, Task, Theme};
use msnp11_sdk::Client;
use std::sync::Arc;
use crate::msnp_listener;

#[allow(dead_code)]
pub enum Action {
    SavePressed(Task<crate::Message>),
    RunTask(Task<crate::Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    DisplayNameChanged(String),
    ServerChanged(String),
    NexusUrlChanged(String),
    CheckForUpdatesToggled(bool),
    Save,
}

pub struct PersonalSettings {
    client: Option<Arc<Client>>,
    display_name: String,
    server: String,
    nexus_url: String,
    check_for_updates: bool,
}

impl PersonalSettings {
    pub fn new(client: Option<Arc<Client>>, display_name: Option<String>) -> Self {
        let settings = settings::get_settings().unwrap_or_default();
        Self {
            client,
            display_name: display_name.unwrap_or_default(),
            server: settings.server,
            nexus_url: settings.nexus_url,
            check_for_updates: settings.check_for_updates,
        }
    }

    pub fn view(&self) -> Element<Message> {
        container(
            column![
                column![
                    text("Display name:").size(14),
                    if self.client.is_some() {
                        text_input("Display name", &self.display_name)
                            .size(14)
                            .on_input(Message::DisplayNameChanged)
                    } else {
                        text_input("Display name", &self.display_name).size(14)
                    }
                ]
                .spacing(5),
                column![
                    text("Server:").size(14),
                    text_input("Server", &self.server)
                        .size(14)
                        .on_input(Message::ServerChanged)
                ]
                .spacing(5),
                column![
                    text("Nexus URL:").size(14),
                    text_input("Nexus URL", &self.nexus_url)
                        .size(14)
                        .on_input(Message::NexusUrlChanged)
                ]
                .spacing(5),
                container(
                    checkbox("Check for updates on startup", self.check_for_updates)
                        .on_toggle(Message::CheckForUpdatesToggled)
                )
                .width(Fill),
                vertical_space().height(5),
                button("Save").on_press(Message::Save),
                vertical_space().height(Fill),
                text("icedm v0.5.0").style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().secondary.weak.color),
                }),
            ]
            .spacing(15)
            .align_x(Center),
        )
        .padding(50)
        .center_x(Fill)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Option<Action> {
        let mut action = None;
        match message {
            Message::DisplayNameChanged(display_name) => self.display_name = display_name,
            Message::ServerChanged(server) => self.server = server,
            Message::NexusUrlChanged(nexus_url) => self.nexus_url = nexus_url,
            Message::CheckForUpdatesToggled(check_for_updates) => {
                self.check_for_updates = check_for_updates
            }

            Message::Save => {
                self.display_name = self.display_name.trim().to_string();
                self.server = self.server.trim().to_string();
                self.nexus_url = self.nexus_url.trim().to_string();

                let settings = Settings {
                    server: self.server.clone(),
                    nexus_url: self.nexus_url.clone(),
                    check_for_updates: self.check_for_updates,
                };

                let _ = settings::save_settings(&settings);

                if let Some(client) = self.client.clone() {
                    let display_name = self.display_name.clone();
                    let new_display_name = display_name.clone();

                    action = Some(Action::SavePressed(Task::batch([
                        Task::perform(
                            async move { client.set_display_name(&display_name).await },
                            crate::Message::UnitResult,
                        ),
                        Task::done(crate::Message::MsnpEvent(
                            msnp_listener::Event::NotificationServer(
                                msnp11_sdk::Event::DisplayName(new_display_name),
                            ),
                        )),
                    ])));
                } else {
                    action = Some(Action::SavePressed(Task::none()))
                }
            }
        }

        action
    }
}
