use crate::models::contact::Contact;
use crate::screens::sign_in::Client;
use std::sync::Arc;

#[derive(Debug)]
pub enum WindowType {
    MainWindow,
    PersonalSettings {
        client: Option<Client>,
        display_name: Option<String>,
    },

    Conversation(Contact),
    Dialog(Arc<String>),
}
