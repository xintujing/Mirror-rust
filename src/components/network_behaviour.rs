use crate::batcher::{Batch, UnBatch};
use std::fmt;

pub trait NetworkBehaviourTrait: fmt::Debug {
    // DeserializeObjectsAll
    fn deserialize_objects_all(&self, un_batch: UnBatch);
    // Serialize
    fn serialize(&self) -> Batch;
    // Deserialize
    fn deserialize(&self, un_batch: &mut UnBatch);
    fn get_network_behaviour(&self) -> &NetworkBehaviour;
}

#[derive(Debug, Clone)]
pub struct NetworkBehaviour {
    // ComponentIndex
    pub component_index: u8,
    // syncVarDirtyBits
    pub sync_var_dirty_bits: u64,
    // syncObjectDirtyBits
    pub sync_object_dirty_bits: u64,
}

impl NetworkBehaviour {
    pub fn new(component_index: u8) -> Self {
        NetworkBehaviour {
            component_index,
            sync_var_dirty_bits: 0,
            sync_object_dirty_bits: 0,
        }
    }
}