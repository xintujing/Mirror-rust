use crate::core::network_connection::NetworkConnection;

pub struct Room {
    pub id: [u8; 3],
    pub r#type: u8,
    pub name: &'static str,
    pub connects: Vec<NetworkConnection>,
    pub scene_id: u64,
}