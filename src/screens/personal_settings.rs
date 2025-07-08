use iced::widget::{button, column, container, text, text_input};
use iced::{Center, Element, Fill};

#[derive(Debug, Clone)]
pub enum Message {
    DisplayNameChanged(String),
    PersonalMessageChanged(String),
    ServerChanged(String),
    NexusUrlChanged(String),
    Save,
}

pub struct PersonalSettings {
    display_name: String,
    personal_message: String,
    server: String,
    nexus_url: String,
}

impl PersonalSettings {
    pub fn new() -> Self {
        Self {
            display_name: String::new(),
            personal_message: String::new(),
            server: String::new(),
            nexus_url: String::new(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        container(
            column![
                column![
                    text("Display name:").size(14),
                    text_input("Display name", &self.server)
                        .size(14)
                        .on_input(Message::ServerChanged)
                ]
                .spacing(5),
                column![
                    text("Personal message:").size(14),
                    text_input("Personal message", &self.server)
                        .size(14)
                        .on_input(Message::ServerChanged)
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

    pub fn update(&mut self, message: Message) {
        match message {
            Message::DisplayNameChanged(display_name) => self.display_name = display_name,
            Message::PersonalMessageChanged(personal_message) => {
                self.personal_message = personal_message
            }

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
