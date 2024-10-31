use crate::components::network_behaviour_base::{NetworkBehaviourBase, NetworkBehaviourTrait};
use crate::components::network_transform::network_transform_base::NetworkTransformBase;
use crate::components::network_transform::transform_sync_data::SyncData;
use crate::core::backend_data::{NetworkBehaviourSetting, NetworkTransformBaseSetting, NetworkTransformUnreliableSetting};
use crate::core::network_reader::NetworkReader;
use crate::core::network_writer::NetworkWriter;
use nalgebra::{Quaternion, Vector3};

pub struct NetworkTransformUnreliable {
    pub network_transform_base: NetworkTransformBase,
    // network_transform_unreliable_setting: NetworkTransformUnreliableSetting
    pub buffer_reset_multiplier: f32,
    pub changed_detection: bool,
    pub position_sensitivity: f32,
    pub rotation_sensitivity: f32,
    pub scale_sensitivity: f32,

    pub network_behaviour: NetworkBehaviourBase,

    pub sync_data: SyncData,
}

impl NetworkTransformUnreliable {
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkTransformUnreliable";
    pub fn new(network_transform_base_setting: NetworkTransformBaseSetting, network_transform_unreliable_setting: NetworkTransformUnreliableSetting, network_behaviour_setting: NetworkBehaviourSetting, component_index: u8, position: Vector3<f32>, quaternion: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        NetworkTransformUnreliable {
            network_transform_base: NetworkTransformBase::new(network_transform_base_setting),
            buffer_reset_multiplier: network_transform_unreliable_setting.buffer_reset_multiplier,
            changed_detection: network_transform_unreliable_setting.changed_detection,
            position_sensitivity: network_transform_unreliable_setting.position_sensitivity,
            rotation_sensitivity: network_transform_unreliable_setting.rotation_sensitivity,
            scale_sensitivity: network_transform_unreliable_setting.scale_sensitivity,
            network_behaviour: NetworkBehaviourBase::new(network_behaviour_setting, component_index),
            sync_data: SyncData::new(8, position, quaternion, scale),
        }
    }
}
#[allow(dead_code)]
impl NetworkBehaviourTrait for NetworkTransformUnreliable {
    fn get_network_behaviour_base(&mut self) -> &mut NetworkBehaviourBase {
        self.network_behaviour.get_network_behaviour_base()
    }

    fn deserialize_objects_all(&self, un_batch: NetworkReader, initial_state: bool) {
        todo!()
    }

    fn serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
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

    // fn serialize(&mut self, initial_state: bool) -> Batch {
    //     let mut batch = Batch::new();
    //     if initial_state {
    //         if self.network_transform_base.sync_position {
    //             batch.write_vector3_f32_le(self.sync_data.position);
    //         }
    //         if self.network_transform_base.sync_rotation {
    //             batch.write_quaternion_f32_le(self.sync_data.quat_rotation);
    //         }
    //         if self.network_transform_base.sync_scale {
    //             batch.write_vector3_f32_le(self.sync_data.scale);
    //         }
    //     }
    //     batch
    // }

    // fn deserialize(&mut self, un_batch: &mut UnBatch, initial_state: bool) {
    //     if initial_state {
    //         if self.network_transform_base.sync_position {
    //             if let Ok(position) = un_batch.read_vector3_f32_le() {
    //                 self.sync_data.position = position;
    //             }
    //         }
    //         if self.network_transform_base.sync_rotation {
    //             if let Ok(quat_rotation) = un_batch.read_quaternion_f32_le() {
    //                 self.sync_data.quat_rotation = quat_rotation;
    //             }
    //         }
    //         if self.network_transform_base.sync_scale {
    //             if let Ok(scale) = un_batch.read_vector3_f32_le() {
    //                 self.sync_data.scale = scale;
    //             }
    //         }
    //     }
    // }

}