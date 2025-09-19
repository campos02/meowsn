use msnp11_sdk::{MsnpList, Presence};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct Contact {
    pub email: Arc<String>,
    pub display_name: Arc<String>,
    pub guid: Arc<String>,
    pub lists: Vec<MsnpList>,
    pub status: Option<Arc<Presence>>,
    pub personal_message: Option<Arc<String>>,
    pub display_picture: Option<Cow<'static, [u8]>>,
    pub opening_conversation: bool,
}
