use crate::client_wrapper::ClientWrapper;
use crate::models::contact::Contact;
use std::sync::Arc;

#[derive(Debug)]
pub enum WindowType {
    MainWindow,
    PersonalSettings {
        client: Option<ClientWrapper>,
        display_name: Option<String>,
    },

    Conversation {
        user_email: Arc<String>,
        contact: Contact,
    },

    Dialog(String),
}
