use crate::mirror::components::network_transform::network_transform_base::Transform;
use crate::mirror::core::network_manager::NetworkManagerStatic;
use nalgebra::Vector3;
use rand::Rng;

pub struct NetworkStartPosition;
impl NetworkStartPosition {
    pub fn awake() {
        let mut start = Transform::default();
        // 在生成 五个随机位置
        for _ in 0..5 {
            let x = rand::rng().random_range(-8.0..8.0);
            let y = 0.5;
            let z = rand::rng().random_range(-8.0..8.0);
            let v3 = Vector3::new(x, y, z);
            start.position = v3;
            start.local_position = v3;
            NetworkManagerStatic::add_start_position(start);
        }
    }
}