use crate::batcher::{Batch, UnBatch};
use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait};
use crate::sync_data::SyncData;
use nalgebra::{Quaternion, Vector3};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct NetworkTransformUnreliable {
    pub network_behaviour: NetworkBehaviour,

    pub sync_position: bool,
    pub sync_rotation: bool,
    pub sync_scale: bool,

    pub only_sync_on_change: bool,
    pub compress_rotation: bool,

    pub sync_data: SyncData,
}

impl NetworkTransformUnreliable {
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkTransformUnreliable";
    pub fn new(component_index: u8, sync_position: bool, sync_rotation: bool, sync_scale: bool, position: Vector3<f32>, quaternion: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        NetworkTransformUnreliable {
            network_behaviour: NetworkBehaviour::new(component_index),
            sync_position,
            sync_rotation,
            sync_scale,
            only_sync_on_change: true,
            compress_rotation: true,
            sync_data: SyncData::new(0, position, quaternion, scale),
        }
    }
}
impl NetworkBehaviourTrait for NetworkTransformUnreliable {
    fn deserialize_objects_all(&self, un_batch: UnBatch) {}

    fn serialize(&self) -> Batch {
        let mut batch = Batch::new();
        if self.sync_position {
            batch.write_vector3_f32_le(self.sync_data.position);
        }
        if self.sync_rotation {
            batch.write_quaternion_f32_le(self.sync_data.quat_rotation);
        }
        if self.sync_scale {
            batch.write_vector3_f32_le(self.sync_data.scale);
        }
        batch
    }

    fn deserialize(&self, un_batch: &mut UnBatch) {}

    fn get_network_behaviour(&self) -> &NetworkBehaviour {
        &self.network_behaviour
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}