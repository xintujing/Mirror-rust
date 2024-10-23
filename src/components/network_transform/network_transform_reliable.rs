use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait};
use crate::components::network_transform::network_transform_base::NetworkTransformBase;
use crate::components::network_transform::transform_sync_data::SyncData;
use crate::core::backend_data::{NetworkBehaviourSetting, NetworkTransformBaseSetting, NetworkTransformReliableSetting};
use crate::core::batcher::{Batch, UnBatch};
use crate::core::tools::compress::{scale_to_long0, Compress, Decompress};
use nalgebra::{Quaternion, Vector3};
use std::any::Any;
use std::cell::Cell;

pub struct NetworkTransformReliable {
    pub network_transform_base: NetworkTransformBase,

    // NetworkTransformReliableSetting start
    pub only_sync_on_change_correction_multiplier: f32,
    pub rotation_sensitivity: f32,
    pub position_precision: f32,
    pub scale_precision: f32,
    pub compress_rotation: bool,
    // NetworkTransformReliableSetting end

    pub network_behaviour: NetworkBehaviour,

    pub last_serialized_position: Cell<Vector3<i64>>,
    pub last_deserialized_position: Cell<Vector3<i64>>,
    pub last_serialized_scale: Cell<Vector3<i64>>,
    pub last_deserialized_scale: Cell<Vector3<i64>>,

    pub sync_data: Cell<SyncData>,
}

impl NetworkTransformReliable {
    #[allow(dead_code)]
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkTransformReliable";
    #[allow(dead_code)]
    pub fn new(network_transform_base_setting: NetworkTransformBaseSetting, network_transform_reliable_setting: NetworkTransformReliableSetting, network_behaviour_setting: NetworkBehaviourSetting, component_index: u8, position: Vector3<f32>, quaternion: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        NetworkTransformReliable {
            network_transform_base: NetworkTransformBase::new(network_transform_base_setting),
            only_sync_on_change_correction_multiplier: network_transform_reliable_setting.only_sync_on_change_correction_multiplier,
            rotation_sensitivity: network_transform_reliable_setting.rotation_sensitivity,
            position_precision: network_transform_reliable_setting.position_precision,
            scale_precision: network_transform_reliable_setting.scale_precision,
            compress_rotation: true,
            network_behaviour: NetworkBehaviour::new(network_behaviour_setting, component_index),
            last_serialized_position: Default::default(),
            last_deserialized_position: Default::default(),
            last_serialized_scale: Default::default(),
            last_deserialized_scale: Default::default(),
            sync_data: Cell::new(SyncData::new(0, position, quaternion, scale)),
        }
    }
}
impl NetworkBehaviourTrait for NetworkTransformReliable {
    fn deserialize_objects_all(&self, un_batch: UnBatch, initial_state: bool) {}

    fn serialize(&self, initial_state: bool) -> Batch {
        let mut batch = Batch::new();
        if initial_state {
            if self.network_transform_base.sync_position {
                batch.write_vector3_f32_le(self.sync_data.get().position);
            }
            if self.network_transform_base.sync_rotation {
                if self.compress_rotation {
                    batch.write_u32_le(self.sync_data.get().quat_rotation.compress());
                } else {
                    batch.write_quaternion_f32_le(self.sync_data.get().quat_rotation);
                }
            }
            if self.network_transform_base.sync_scale {
                batch.write_vector3_f32_le(self.sync_data.get().scale);
            }
        } else {
            if self.network_transform_base.sync_position {
                let (_, v3) = scale_to_long0(self.sync_data.get().position, self.position_precision);
                batch.compress_var_i64_le(v3.x - self.last_serialized_position.get().x);
                batch.compress_var_i64_le(v3.y - self.last_serialized_position.get().y);
                batch.compress_var_i64_le(v3.z - self.last_serialized_position.get().z);
                self.last_serialized_position.set(v3);
            }
            if self.network_transform_base.sync_rotation {
                if self.compress_rotation {
                    batch.write_u32_le(self.sync_data.get().quat_rotation.compress());
                } else {
                    batch.write_quaternion_f32_le(self.sync_data.get().quat_rotation);
                }
            }
            if self.network_transform_base.sync_scale {
                let (_, v3) = scale_to_long0(self.sync_data.get().scale, self.scale_precision);
                batch.compress_var_i64_le(v3.x - self.last_serialized_scale.get().x);
                batch.compress_var_i64_le(v3.y - self.last_serialized_scale.get().y);
                batch.compress_var_i64_le(v3.z - self.last_serialized_scale.get().z);
                self.last_serialized_scale.set(v3);
            }
        }
        batch
    }

    fn deserialize(&self, un_batch: &mut UnBatch, initial_state: bool) {
        if initial_state {
            if self.network_transform_base.sync_position {
                if let Ok(position) = un_batch.read_vector3_f32_le() {
                    self.sync_data.get().position = position;
                }
            }
            if self.network_transform_base.sync_rotation {
                if self.compress_rotation {
                    if let Ok(compressed) = un_batch.read_u32_le() {
                        self.sync_data.get().quat_rotation = Quaternion::decompress(compressed);
                    }
                } else {
                    if let Ok(quat_rotation) = un_batch.read_quaternion_f32_le() {
                        self.sync_data.get().quat_rotation = quat_rotation;
                    }
                }
            }
            if self.network_transform_base.sync_scale {
                if let Ok(scale) = un_batch.read_vector3_f32_le() {
                    self.sync_data.get().scale = scale;
                }
            }
        } else {
            if self.network_transform_base.sync_position {
                let mut x = self.sync_data.get().position.x as i64;
                let mut y = self.sync_data.get().position.y as i64;
                let mut z = self.sync_data.get().position.z as i64;
                if let Ok(x_) = un_batch.decompress_var_i64_le() {
                    x = self.last_deserialized_position.get().x + x_;
                }
                if let Ok(y_) = un_batch.decompress_var_i64_le() {
                    y += self.last_deserialized_position.get().y;
                }
                if let Ok(z_) = un_batch.decompress_var_i64_le() {
                    z += self.last_deserialized_position.get().z;
                }
                self.last_deserialized_position.set(Vector3::new(x, y, z));
                self.sync_data.get().position = Vector3::new(x as f32, y as f32, z as f32);
            }
            if self.network_transform_base.sync_rotation {
                if self.compress_rotation {
                    if let Ok(compressed) = un_batch.read_u32_le() {
                        self.sync_data.get().quat_rotation = Quaternion::decompress(compressed);
                    }
                } else {
                    if let Ok(quat_rotation) = un_batch.read_quaternion_f32_le() {
                        self.sync_data.get().quat_rotation = quat_rotation;
                    }
                }
            }
            if self.network_transform_base.sync_scale {
                let mut x = self.sync_data.get().scale.x as i64;
                let mut y = self.sync_data.get().scale.y as i64;
                let mut z = self.sync_data.get().scale.z as i64;
                if let Ok(x_) = un_batch.decompress_var_i64_le() {
                    x = self.last_deserialized_scale.get().x + x_;
                }
                if let Ok(y_) = un_batch.decompress_var_i64_le() {
                    y += self.last_deserialized_scale.get().y;
                }
                if let Ok(z_) = un_batch.decompress_var_i64_le() {
                    z += self.last_deserialized_scale.get().z;
                }
                self.last_deserialized_scale.set(Vector3::new(x, y, z));
                self.sync_data.get().scale = Vector3::new(x as f32, y as f32, z as f32);
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