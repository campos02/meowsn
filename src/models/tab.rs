use std::sync::Arc;

pub struct Tab {
    pub msn_tab: msnp11_sdk::Tab,
    pub image: Arc<[u8]>,
}
