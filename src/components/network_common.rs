use crate::batcher::{Batch, UnBatch};
use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait};
use crate::components::SyncVar;
use dashmap::DashMap;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct NetworkCommon {
    pub network_behaviour: NetworkBehaviour,
    pub sync_vars: DashMap<String, SyncVar>,
}

impl NetworkCommon {
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkCommon";
    pub fn new(component_index: u8, sync_vars: DashMap<String, SyncVar>) -> Self {
        NetworkCommon {
            network_behaviour: NetworkBehaviour::new(component_index),
            sync_vars,
        }
    }
}

impl NetworkBehaviourTrait for NetworkCommon {
    fn deserialize_objects_all(&self, un_batch: UnBatch) {}

    fn serialize(&self) -> Batch {
        Batch::new()
    }

    fn deserialize(&self, un_batch: &mut UnBatch) {}

    fn get_network_behaviour(&self) -> &NetworkBehaviour {
        &self.network_behaviour
    }
}