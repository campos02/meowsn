use msnp11_sdk::Client;
use std::fmt::Debug;
use std::sync::Arc;

pub struct ClientWrapper {
    pub personal_message: String,
    pub inner: Arc<Client>,
}

impl Debug for ClientWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").finish()
    }
}
