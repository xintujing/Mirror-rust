use crate::mirror::core::network_behaviour::Transform;
use crate::mirror::core::network_manager::NetworkManager;

pub struct NetworkStartPosition;
impl NetworkStartPosition {

    pub fn awake(){
        NetworkManager::register_start_position(Transform::default());
    }
}