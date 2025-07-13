use crate::client_wrapper::ClientWrapper;
use crate::settings::Settings;
use crate::{msnp_listener, settings};
use iced::widget::{button, column, container, text, text_input};
use iced::{Center, Element, Fill, Task};
use msnp11_sdk::Client;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    DisplayNameChanged(String),
    ServerChanged(String),
    NexusUrlChanged(String),
    Save,
}

pub struct PersonalSettings {
    client: Option<Arc<Client>>,
    display_name: String,
    server: String,
    nexus_url: String,
}

impl PersonalSettings {
    pub fn new(client: Option<ClientWrapper>, display_name: Option<String>) -> Self {
        let settings = settings::get_settings().unwrap_or_default();
        Self {
            client: client.map(|client| client.inner),
            display_name: display_name.unwrap_or_default(),
            server: settings.server,
            nexus_url: settings.nexus_url,
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
                button("Save").on_press(Message::Save),
            ]
            .spacing(15)
            .align_x(Center),
        )
        .padding(50)
        .center_x(Fill)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<crate::Message> {
        match message {
            Message::DisplayNameChanged(display_name) => self.display_name = display_name,
            Message::ServerChanged(server) => self.server = server,
            Message::NexusUrlChanged(nexus_url) => self.nexus_url = nexus_url,
            Message::Save => {
                self.display_name = self.display_name.trim().to_string();
                self.server = self.server.trim().to_string();
                self.nexus_url = self.nexus_url.trim().to_string();

                let settings = Settings {
                    server: self.server.clone(),
                    nexus_url: self.nexus_url.clone(),
                };

                let _ = settings::save_settings(&settings);

                if let Some(client) = self.client.clone() {
                    let display_name = self.display_name.clone();
                    let new_display_name = display_name.clone();

                    return Task::batch([
                        Task::perform(
                            async move { client.set_display_name(&display_name).await },
                            crate::Message::EmptyResultFuture,
                        ),
                        Task::done(crate::Message::MsnpEvent(msnp_listener::Event::NsEvent(
                            msnp11_sdk::Event::DisplayName(new_display_name),
                        ))),
                    ]);
                }
            }
        }

        Task::none()
    }
}
