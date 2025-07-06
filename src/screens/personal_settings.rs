use iced::widget::{column, container, text_input};
use iced::{Element, Fill};

#[derive(Debug, Clone)]
pub enum Message {
    ServerChanged(String),
    NexusUrlChanged(String),
    Save,
}

pub struct PersonalSettings {
    server: String,
    nexus_url: String,
}

impl PersonalSettings {
    pub fn new() -> Self {
        Self {
            server: String::new(),
            nexus_url: String::new(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        container(
            column![
                text_input("Server", &self.server)
                    .size(16)
                    .on_input(Message::ServerChanged),
                text_input("Nexus URL", &self.nexus_url)
                    .size(16)
                    .on_input(Message::NexusUrlChanged),
            ]
            .spacing(5),
        )
        .padding(50)
        .center_x(Fill)
        .into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ServerChanged(server) => self.server = server,
            Message::NexusUrlChanged(nexus_url) => self.nexus_url = nexus_url,
            Message::Save => {}
        }
    }
}

impl Default for PersonalSettings {
    fn default() -> Self {
        Self::new()
    }
}
