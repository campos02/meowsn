use msnp11_sdk::{Client, MsnpStatus};
use std::borrow::Cow;
use std::sync::Arc;

pub struct SignInReturn {
    pub status: MsnpStatus,
    pub personal_message: String,
    pub display_picture: Option<Cow<'static, [u8]>>,
    pub client: Arc<Client>,
}
