use bytes::Bytes;

pub mod network_behaviour;
pub mod network_transform_reliable;
pub mod network_transform_unreliable;
pub mod network_common;

#[derive(Debug, Clone)]
pub struct SyncVar {
    pub r#type: &'static str,
    pub data: Bytes,
    pub is_dirty: bool,
    pub dirty_bit: u32,
}

impl SyncVar {
    pub fn new() -> Self {
        SyncVar {
            r#type: "",
            data: Bytes::new(),
            is_dirty: false,
            dirty_bit: 0,
        }
    }
}