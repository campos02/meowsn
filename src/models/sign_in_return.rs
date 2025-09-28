use msnp11_sdk::{Client, MsnpStatus};
use std::sync::Arc;

pub struct SignInReturn {
    pub email: Arc<String>,
    pub status: MsnpStatus,
    pub personal_message: String,
    pub display_picture: Option<Arc<[u8]>>,
    pub client: Arc<Client>,
}
