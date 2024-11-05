pub mod network_behaviour;
pub mod network_common_behaviour;
pub mod network_animator;
pub mod network_rigidbody_unreliable;
pub mod room;
pub mod network_transform;

#[derive(Debug, Clone)]
pub struct SyncVar {
    pub r#type: String,
    pub data: Vec<u8>,
    pub is_dirty: bool,
    pub dirty_bit: u32,
}

impl SyncVar {
    pub fn new(r#type: String, data: Vec<u8>, dirty_bit: u32) -> Self {
        SyncVar {
            r#type,
            data,
            is_dirty: false,
            dirty_bit,
        }
    }
}