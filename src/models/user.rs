use std::sync::Arc;

pub struct User {
    pub personal_message: Option<String>,
    pub display_picture: Option<Arc<[u8]>>,
}
