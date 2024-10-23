use crate::core::backend_data::NetworkBehaviourSetting;
use crate::core::batcher::{Batch, UnBatch};
use std::any::Any;

pub trait NetworkBehaviourTrait {
    // DeserializeObjectsAll
    fn deserialize_objects_all(&self, un_batch: UnBatch, initial_state: bool);
    // Serialize
    fn serialize(&self, initial_state: bool) -> Batch;
    // Deserialize
    fn deserialize(&self, un_batch: &mut UnBatch, initial_state: bool);
    fn get_network_behaviour(&self) -> &NetworkBehaviour;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Clone)]
pub struct NetworkBehaviour {
    pub sync_direction: u8,
    // ComponentIndex
    pub component_index: u8,
    // syncVarDirtyBits
    pub sync_var_dirty_bits: u64,
    // syncObjectDirtyBits
    pub sync_object_dirty_bits: u64,
}

impl NetworkBehaviour {
    pub fn new(network_behaviour_setting: NetworkBehaviourSetting, component_index: u8) -> Self {
        NetworkBehaviour {
            sync_direction: network_behaviour_setting.sync_direction,
            component_index,
            sync_var_dirty_bits: 0,
            sync_object_dirty_bits: 0,
        }
    }
}