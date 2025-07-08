use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum WindowType {
    MainWindow,
    PersonalSettings,
    Conversation(Arc<String>),
}
