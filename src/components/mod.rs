use bytes::Bytes;

pub mod network_behaviour_base;
pub mod network_common_behaviour;
pub mod network_animator;
pub mod network_rigidbody_unreliable;
pub mod room;
pub mod network_transform;

#[derive(Debug, Clone)]
pub struct SyncVar {
    pub r#type: String,
    pub data: Bytes,
    pub is_dirty: bool,
    pub dirty_bit: u32,
}

impl SyncVar {
    pub fn new(r#type: String, data: Bytes, dirty_bit: u32) -> Self {
        SyncVar {
            r#type,
            data,
            is_dirty: false,
            dirty_bit,
        }
    }
}