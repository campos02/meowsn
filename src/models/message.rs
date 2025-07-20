use std::sync::Arc;

pub struct Message {
    pub sender: Arc<String>,
    pub receiver: Option<Arc<String>>,
    pub is_nudge: bool,
    pub text: Arc<String>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub color: Arc<String>,
}
