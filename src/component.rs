use crate::tools::generate_id;
use bytes::Bytes;
use dashmap::DashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub net_id: u32,
    pub connection_id: u64,
    pub address: String,
    pub is_authenticated: bool,
    pub is_ready: bool,
    pub last_message_time: f32,
    pub remote_time_stamp: f64,
    pub sync_var_map: DashMap<String, Bytes>,
}

impl Component {
    pub fn new(connection_id: u64, address: String) -> Self {
        Component {
            name: "".to_string(),
            net_id: generate_id(),
            connection_id,
            address,
            is_authenticated: false,
            is_ready: false,
            last_message_time: 0.0,
            remote_time_stamp: 0.0,
            sync_var_map: Default::default(),
        }
    }
}
