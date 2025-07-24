use std::sync::Arc;

#[derive(Clone)]
pub struct Message {
    pub sender: Arc<String>,
    pub receiver: Option<Arc<String>>,
    pub is_nudge: bool,
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub color: String,
}
