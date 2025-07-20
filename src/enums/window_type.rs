use crate::models::contact::Contact;
use msnp11_sdk::{Client, Switchboard};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub enum WindowType {
    MainWindow,
    PersonalSettings {
        client: Option<Arc<Client>>,
        display_name: Option<String>,
    },

    Conversation {
        switchboard: Arc<Switchboard>,
        email: Arc<String>,
        contacts: HashMap<Arc<String>, Contact>,
    },

    Dialog(String),
    AddContact(Arc<Client>),
}
