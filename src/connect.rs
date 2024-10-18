use crate::backend_data::BackendData;
use crate::network_identity::NetworkIdentity;
use std::sync::Arc;

#[derive(Debug)]
pub struct Connect {
    pub connect_id: u64,
    pub is_ready: bool,
    pub is_authenticated: bool,
    pub authentication_data: Vec<u8>,
    pub address: &'static str,
    pub identity: NetworkIdentity,
    pub owned_identity: Vec<NetworkIdentity>,
    pub ob_connects_id: Vec<u64>,
    pub last_message_time: f64,
    pub last_ping_time: f64,
    pub rtt: f64,
    pub backend_data: Arc<BackendData>,
    // snapshots: SortedList<f64, TimeSnapshot>,
}

impl Connect {
    pub fn new(backend_data: Arc<BackendData>, scene_id: u64, asset_id: u32) -> Self {
        Connect {
            connect_id: 0,
            is_ready: false,
            is_authenticated: false,
            authentication_data: Vec::new(),
            address: "",
            identity: NetworkIdentity::new(Arc::clone(&backend_data), scene_id, asset_id),
            owned_identity: Vec::new(),
            ob_connects_id: Vec::new(),
            last_message_time: 0.0,
            last_ping_time: 0.0,
            rtt: 0.0,
            backend_data,
        }
    }
}