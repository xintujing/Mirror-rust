use crate::batcher::{Batch, UnBatch};
use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait};
use crate::sync_data::SyncData;
use nalgebra::{Quaternion, Vector3};
use std::any::Any;
use std::cell::Cell;

#[derive(Debug, Clone)]
pub struct NetworkTransformUnreliable {
    pub network_behaviour: NetworkBehaviour,

    pub sync_position: bool,
    pub sync_rotation: bool,
    pub sync_scale: bool,

    pub only_sync_on_change: bool,
    pub compress_rotation: bool,

    pub sync_data: Cell<SyncData>,
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
            sync_data: Cell::new(SyncData::new(8, position, quaternion, scale)),
        }
    }
}
impl NetworkBehaviourTrait for NetworkTransformUnreliable {
    fn deserialize_objects_all(&self, un_batch: UnBatch, initial_state: bool) {}

    fn serialize(&self, initial_state: bool) -> Batch {
        let mut batch = Batch::new();
        if initial_state {
            if self.sync_position {
                batch.write_vector3_f32_le(self.sync_data.get().position);
            }
            if self.sync_rotation {
                batch.write_quaternion_f32_le(self.sync_data.get().quat_rotation);
            }
            if self.sync_scale {
                batch.write_vector3_f32_le(self.sync_data.get().scale);
            }
        }
        batch
    }

    fn deserialize(&self, un_batch: &mut UnBatch, initial_state: bool) {
        if initial_state {
            if self.sync_position {
                if let Ok(position) = un_batch.read_vector3_f32_le() {
                    self.sync_data.get().position = position;
                }
            }
            if self.sync_rotation {
                if let Ok(quat_rotation) = un_batch.read_quaternion_f32_le() {
                    self.sync_data.get().quat_rotation = quat_rotation;
                }
            }
            if self.sync_scale {
                if let Ok(scale) = un_batch.read_vector3_f32_le() {
                    self.sync_data.get().scale = scale;
                }
            }
        }
    }

    fn get_network_behaviour(&self) -> &NetworkBehaviour {
        &self.network_behaviour
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}