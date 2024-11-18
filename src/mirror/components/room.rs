use crate::mirror::core::network_connection_to_client::NetworkConnectionToClient;

pub struct Room {
    pub id: [u8; 3],
    pub r#type: u8,
    pub name: &'static str,
    pub connects: Vec<NetworkConnectionToClient>,
    pub scene_id: u64,
}