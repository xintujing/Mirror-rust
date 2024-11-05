use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait};
use crate::components::network_transform::network_transform_base::NetworkTransformBase;
use crate::components::network_transform::transform_sync_data::SyncData;
use crate::core::backend_data::{NetworkBehaviourSetting, NetworkTransformBaseSetting, NetworkTransformUnreliableSetting};
use crate::core::network_reader::NetworkReader;
use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use nalgebra::{Quaternion, Vector3};

#[derive(Debug)]
pub struct NetworkTransformUnreliable {
    pub network_transform_base: NetworkTransformBase,
    // network_transform_unreliable_setting: NetworkTransformUnreliableSetting
    pub buffer_reset_multiplier: f32,
    pub changed_detection: bool,
    pub position_sensitivity: f32,
    pub rotation_sensitivity: f32,
    pub scale_sensitivity: f32,

    pub network_behaviour: NetworkBehaviour,

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
            network_behaviour: NetworkBehaviour::new(network_behaviour_setting, component_index),
            sync_data: SyncData::new(8, position, quaternion, scale),
        }
    }
}
#[allow(dead_code)]
impl NetworkBehaviourTrait for NetworkTransformUnreliable {
    fn get_network_behaviour_base(&mut self) -> &mut NetworkBehaviour {
        self.network_behaviour.get_network_behaviour_base()
    }

    fn deserialize_objects_all(&self, un_batch: NetworkReader, initial_state: bool) {
        todo!()
    }

    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        if initial_state {
            if self.network_transform_base.sync_position {
                writer.write_vector3(self.sync_data.position);
            }
            if self.network_transform_base.sync_rotation {
                writer.write_quaternion(self.sync_data.quat_rotation);
            }
            if self.network_transform_base.sync_scale {
                writer.write_vector3(self.sync_data.scale);
            }
        }
    }

    fn deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool {
        todo!()
    }

    fn on_start_server(&mut self) {
        // TODO
    }

    fn on_stop_server(&mut self) {
        todo!()
    }
}