use crate::client_wrapper::ClientWrapper;
use crate::models::contact::Contact;

#[derive(Debug)]
pub enum WindowType {
    MainWindow,
    PersonalSettings {
        client: Option<ClientWrapper>,
        display_name: Option<String>,
    },

    Conversation(Contact),
    Dialog(String),
}
