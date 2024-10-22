use crate::backend_data::BackendData;
use crate::network_identity::NetworkIdentity;
use crate::tools::utils::get_sec_timestamp_f64;
use std::sync::Arc;

#[derive(Debug)]
pub struct NetworkConnection {
    pub connection_id: u64,
    pub is_ready: bool,
    pub is_authenticated: bool,
    pub authentication_data: Vec<u8>,
    pub address: &'static str,
    pub identity: NetworkIdentity,
    pub owned_identities: Vec<NetworkIdentity>,
    pub ob_connects_id: Vec<u64>,
    pub last_message_time: f64,
    pub remote_time_stamp: f64,

    pub last_ping_time: f64,
    pub rtt: f64,
    pub backend_data: Arc<BackendData>,
    // snapshots: SortedList<f64, TimeSnapshot>,
}

impl NetworkConnection {
    pub fn new(backend_data: Arc<BackendData>, scene_id: u64, asset_id: u32) -> Self {
        let ts = get_sec_timestamp_f64();
        NetworkConnection {
            connection_id: 0,
            is_ready: false,
            is_authenticated: false,
            authentication_data: Vec::new(),
            address: "",
            identity: NetworkIdentity::new(Arc::clone(&backend_data), scene_id, asset_id),
            owned_identities: Vec::new(),
            ob_connects_id: Vec::new(),
            last_message_time: ts,
            remote_time_stamp: ts,
            last_ping_time: ts,
            rtt: 0.0,
            backend_data,
        }
    }
}