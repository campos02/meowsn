use msnp11_sdk::Switchboard;
use std::sync::Arc;

#[derive(Clone)]
pub struct SwitchboardAndParticipants {
    pub switchboard: Arc<Switchboard>,
    pub participants: Vec<Arc<String>>,
}
