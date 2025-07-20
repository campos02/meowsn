use msnp11_sdk::Switchboard;
use std::sync::Arc;

pub struct SwitchboardAndParticipants {
    pub switchboard: Arc<Switchboard>,
    pub participants: Vec<Arc<String>>,
}
