pub mod network_behaviour;
pub mod network_transform;
pub mod network_common;

#[derive(Debug, Clone)]
pub struct SyncVar {
    pub r#type: &'static str,
    pub data: Vec<u8>,
    pub is_dirty: bool,
    pub dirty_bit: u32,
}

impl SyncVar {
    pub fn new() -> Self {
        SyncVar {
            r#type: "",
            data: Vec::new(),
            is_dirty: false,
            dirty_bit: 0,
        }
    }
}