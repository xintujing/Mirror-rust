use crate::mirror::components::network_transform::network_transform_base::Transform;
use crate::mirror::core::network_manager::NetworkManager;

pub struct NetworkStartPosition;
impl NetworkStartPosition {

    pub fn awake(){
        NetworkManager::register_start_position(Transform::default());
    }
}