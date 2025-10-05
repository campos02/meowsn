use std::sync::Arc;

#[derive(Clone)]
pub struct DisplayPicture {
    pub data: Arc<[u8]>,
    pub hash: Arc<String>,
}
