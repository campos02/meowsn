use crate::contact_repository::ContactRepository;
use crate::models::switchboard_and_participants::SwitchboardAndParticipants;
use msnp11_sdk::Client;
use std::sync::Arc;

#[derive(Clone)]
pub enum WindowType {
    MainWindow,
    PersonalSettings {
        client: Option<Arc<Client>>,
        display_name: Option<String>,
    },

    Conversation {
        contact_repository: ContactRepository,
        switchboard: SwitchboardAndParticipants,
        email: Arc<String>,
    },

    Dialog(String),
    AddContact(Arc<Client>),
}
