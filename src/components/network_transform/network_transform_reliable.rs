use crate::components::network_behaviour::{NetworkBehaviourTrait, SyncDirection, SyncMode};
use crate::components::network_transform::network_transform_base::{CoordinateSpace, NetworkTransformBase, NetworkTransformBaseTrait};
use crate::core::backend_data::{NetworkBehaviourSetting, NetworkTransformBaseSetting, NetworkTransformReliableSetting};
use crate::core::network_manager::GameObject;
use crate::core::network_reader::NetworkReader;
use crate::core::network_writer::NetworkWriter;
use nalgebra::{Quaternion, Vector3};
use std::any::Any;
use std::fmt::Debug;

#[derive(Debug)]
pub struct NetworkTransformReliable {
    network_transform_base: NetworkTransformBase,

    // NetworkTransformReliableSetting start
    pub only_sync_on_change_correction_multiplier: f32,
    pub rotation_sensitivity: f32,
    pub position_precision: f32,
    pub scale_precision: f32,
    pub compress_rotation: bool,
    // NetworkTransformReliableSetting end

    pub last_serialized_position: Vector3<i64>,
    pub last_deserialized_position: Vector3<i64>,
    pub last_serialized_scale: Vector3<i64>,
    pub last_deserialized_scale: Vector3<i64>,
}

impl NetworkTransformReliable {
    #[allow(dead_code)]
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkTransformReliable";
    #[allow(dead_code)]
    pub fn new(network_transform_base_setting: NetworkTransformBaseSetting, network_transform_reliable_setting: NetworkTransformReliableSetting, network_behaviour_setting: NetworkBehaviourSetting, component_index: u8, position: Vector3<f32>, quaternion: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        NetworkTransformReliable {
            network_transform_base: NetworkTransformBase::new(network_transform_base_setting, network_behaviour_setting, component_index),
            only_sync_on_change_correction_multiplier: network_transform_reliable_setting.only_sync_on_change_correction_multiplier,
            rotation_sensitivity: network_transform_reliable_setting.rotation_sensitivity,
            position_precision: network_transform_reliable_setting.position_precision,
            scale_precision: network_transform_reliable_setting.scale_precision,
            compress_rotation: true,
            last_serialized_position: Default::default(),
            last_deserialized_position: Default::default(),
            last_serialized_scale: Default::default(),
            last_deserialized_scale: Default::default(),
        }
    }
}


impl NetworkBehaviourTrait for NetworkTransformReliable {
    fn sync_interval(&self) -> f64 {
        self.network_transform_base.network_behaviour.sync_interval()
    }

    fn set_sync_interval(&mut self, value: f64) {
        self.network_transform_base.network_behaviour.set_sync_interval(value)
    }

    fn last_sync_time(&self) -> f64 {
        self.network_transform_base.network_behaviour.last_sync_time()
    }

    fn set_last_sync_time(&mut self, value: f64) {
        self.network_transform_base.network_behaviour.set_last_sync_time(value)
    }

    fn sync_direction(&mut self) -> &SyncDirection {
        self.network_transform_base.network_behaviour.sync_direction()
    }

    fn set_sync_direction(&mut self, value: SyncDirection) {
        self.network_transform_base.network_behaviour.set_sync_direction(value)
    }

    fn sync_mode(&mut self) -> &SyncMode {
        self.network_transform_base.network_behaviour.sync_mode()
    }

    fn set_sync_mode(&mut self, value: SyncMode) {
        self.network_transform_base.network_behaviour.set_sync_mode(value)
    }

    fn component_index(&self) -> u8 {
        self.network_transform_base.network_behaviour.component_index()
    }

    fn set_component_index(&mut self, value: u8) {
        self.network_transform_base.network_behaviour.set_component_index(value)
    }

    fn sync_var_dirty_bits(&self) -> u64 {
        self.network_transform_base.network_behaviour.sync_var_dirty_bits()
    }

    fn set_sync_var_dirty_bits(&mut self, value: u64) {
        self.network_transform_base.network_behaviour.set_sync_var_dirty_bits(value)
    }

    fn sync_object_dirty_bits(&self) -> u64 {
        self.network_transform_base.network_behaviour.sync_object_dirty_bits()
    }

    fn set_sync_object_dirty_bits(&mut self, value: u64) {
        self.network_transform_base.network_behaviour.set_sync_object_dirty_bits(value)
    }

    fn net_id(&self) -> u32 {
        self.network_transform_base.network_behaviour.net_id()
    }

    fn set_net_id(&mut self, value: u32) {
        self.network_transform_base.network_behaviour.set_net_id(value)
    }

    fn connection_to_client(&self) -> u64 {
        self.network_transform_base.network_behaviour.connection_to_client()
    }

    fn set_connection_to_client(&mut self, value: u64) {
        self.network_transform_base.network_behaviour.set_connection_to_client(value)
    }

    fn observers(&self) -> &Vec<u64> {
        self.network_transform_base.network_behaviour.observers()
    }

    fn set_observers(&mut self, value: Vec<u64>) {
        self.network_transform_base.network_behaviour.set_observers(value)
    }

    fn game_object(&self) -> &GameObject {
        self.network_transform_base.network_behaviour.game_object()
    }

    fn set_game_object(&mut self, value: GameObject) {
        self.network_transform_base.network_behaviour.set_game_object(value)
    }


    fn is_dirty(&self) -> bool {
        self.network_transform_base.network_behaviour.is_dirty()
    }

    fn deserialize_objects_all(&self, un_batch: NetworkReader, initial_state: bool) {}

    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        todo!()
    }

    fn deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool {
        todo!()
    }

    fn on_start_server(&mut self) {
        todo!()
    }

    fn on_stop_server(&mut self) {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    // fn serialize(&mut self, initial_state: bool) -> Batch {
    //     let mut batch = Batch::new();
    //     if initial_state {
    //         if self.network_transform_base.sync_position {
    //             batch.write_vector3_f32_le(self.sync_data.position);
    //         }
    //         if self.network_transform_base.sync_rotation {
    //             if self.compress_rotation {
    //                 batch.write_u32_le(self.sync_data.quat_rotation.compress());
    //             } else {
    //                 batch.write_quaternion_f32_le(self.sync_data.quat_rotation);
    //             }
    //         }
    //         if self.network_transform_base.sync_scale {
    //             batch.write_vector3_f32_le(self.sync_data.scale);
    //         }
    //     } else {
    //         if self.network_transform_base.sync_position {
    //             let (_, v3) = scale_to_long_0(self.sync_data.position, self.position_precision);
    //             batch.compress_var_i64_le(v3.x - self.last_serialized_position.x);
    //             batch.compress_var_i64_le(v3.y - self.last_serialized_position.y);
    //             batch.compress_var_i64_le(v3.z - self.last_serialized_position.z);
    //             self.last_serialized_position = v3;
    //         }
    //         if self.network_transform_base.sync_rotation {
    //             if self.compress_rotation {
    //                 batch.write_u32_le(self.sync_data.quat_rotation.compress());
    //             } else {
    //                 batch.write_quaternion_f32_le(self.sync_data.quat_rotation);
    //             }
    //         }
    //         if self.network_transform_base.sync_scale {
    //             let (_, v3) = scale_to_long_0(self.sync_data.scale, self.scale_precision);
    //             batch.compress_var_i64_le(v3.x - self.last_serialized_scale.x);
    //             batch.compress_var_i64_le(v3.y - self.last_serialized_scale.y);
    //             batch.compress_var_i64_le(v3.z - self.last_serialized_scale.z);
    //             self.last_serialized_scale = v3;
    //         }
    //     }
    //     batch
    // }
    //
    // fn deserialize(&mut self, un_batch: &mut UnBatch, initial_state: bool) {
    //     if initial_state {
    //         if self.network_transform_base.sync_position {
    //             if let Ok(position) = un_batch.read_vector3_f32_le() {
    //                 self.sync_data.position = position;
    //             }
    //         }
    //         if self.network_transform_base.sync_rotation {
    //             if self.compress_rotation {
    //                 if let Ok(compressed) = un_batch.read_u32_le() {
    //                     self.sync_data.quat_rotation = Quaternion::decompress(compressed);
    //                 }
    //             } else {
    //                 if let Ok(quat_rotation) = un_batch.read_quaternion_f32_le() {
    //                     self.sync_data.quat_rotation = quat_rotation;
    //                 }
    //             }
    //         }
    //         if self.network_transform_base.sync_scale {
    //             if let Ok(scale) = un_batch.read_vector3_f32_le() {
    //                 self.sync_data.scale = scale;
    //             }
    //         }
    //     } else {
    //         if self.network_transform_base.sync_position {
    //             let mut x = self.sync_data.position.x as i64;
    //             let mut y = self.sync_data.position.y as i64;
    //             let mut z = self.sync_data.position.z as i64;
    //             if let Ok(x_) = un_batch.decompress_var_i64_le() {
    //                 x = self.last_deserialized_position.x + x_;
    //             }
    //             if let Ok(y_) = un_batch.decompress_var_i64_le() {
    //                 y += self.last_deserialized_position.y;
    //             }
    //             if let Ok(z_) = un_batch.decompress_var_i64_le() {
    //                 z += self.last_deserialized_position.z;
    //             }
    //             self.last_deserialized_position = Vector3::new(x, y, z);
    //             self.sync_data.position = Vector3::new(x as f32, y as f32, z as f32);
    //         }
    //         if self.network_transform_base.sync_rotation {
    //             if self.compress_rotation {
    //                 if let Ok(compressed) = un_batch.read_u32_le() {
    //                     self.sync_data.quat_rotation = Quaternion::decompress(compressed);
    //                 }
    //             } else {
    //                 if let Ok(quat_rotation) = un_batch.read_quaternion_f32_le() {
    //                     self.sync_data.quat_rotation = quat_rotation;
    //                 }
    //             }
    //         }
    //         if self.network_transform_base.sync_scale {
    //             let mut x = self.sync_data.scale.x as i64;
    //             let mut y = self.sync_data.scale.y as i64;
    //             let mut z = self.sync_data.scale.z as i64;
    //             if let Ok(x_) = un_batch.decompress_var_i64_le() {
    //                 x = self.last_deserialized_scale.x + x_;
    //             }
    //             if let Ok(y_) = un_batch.decompress_var_i64_le() {
    //                 y += self.last_deserialized_scale.y;
    //             }
    //             if let Ok(z_) = un_batch.decompress_var_i64_le() {
    //                 z += self.last_deserialized_scale.z;
    //             }
    //             self.last_deserialized_scale = Vector3::new(x, y, z);
    //             self.sync_data.scale = Vector3::new(x as f32, y as f32, z as f32);
    //         }
    //     }
    // }
}

impl NetworkTransformBaseTrait for NetworkTransformReliable {
    fn coordinate_space(&self) -> &CoordinateSpace {
        &self.network_transform_base.coordinate_space
    }

    fn set_coordinate_space(&mut self, value: CoordinateSpace) {
        self.network_transform_base.coordinate_space = value;
    }

    fn get_game_object(&self) -> &GameObject {
        self.network_transform_base.network_behaviour.game_object()
    }

    fn set_game_object(&mut self, value: GameObject) {
        self.network_transform_base.network_behaviour.set_game_object(value)
    }

    fn sync_position(&self) -> bool {
        self.network_transform_base.sync_position
    }

    fn sync_rotation(&self) -> bool {
        self.network_transform_base.sync_rotation
    }

    fn interpolate_position(&self) -> bool {
        self.network_transform_base.interpolate_position
    }

    fn interpolate_rotation(&self) -> bool {
        self.network_transform_base.interpolate_rotation
    }

    fn interpolate_scale(&self) -> bool {
        self.network_transform_base.interpolate_scale
    }

    fn sync_scale(&self) -> bool {
        self.network_transform_base.sync_scale
    }

}