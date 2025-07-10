use crate::models::contact::Contact;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum WindowType {
    MainWindow,
    PersonalSettings,
    Conversation(Contact),
    Dialog(Arc<String>),
}
