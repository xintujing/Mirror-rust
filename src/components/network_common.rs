use crate::batcher::{Batch, UnBatch};
use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait};
use crate::components::SyncVar;
use dashmap::DashMap;
use std::any::Any;
use std::fmt::Debug;
use tklog::debug;

#[derive(Debug, Clone)]
pub struct NetworkCommon {
    pub network_behaviour: NetworkBehaviour,
    pub sync_vars: DashMap<u8, SyncVar>,
}

impl NetworkCommon {
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkCommon";
    pub fn new(component_index: u8, sync_vars: DashMap<u8, SyncVar>) -> Self {
        NetworkCommon {
            network_behaviour: NetworkBehaviour::new(component_index),
            sync_vars,
        }
    }
}

impl NetworkBehaviourTrait for NetworkCommon {
    fn deserialize_objects_all(&self, un_batch: UnBatch) {}

    fn serialize(&self) -> Batch {
        let mut batch = Batch::new();
        for i in 0..self.sync_vars.len() as u8 {
            if let Some(sync_var) = self.sync_vars.get(&(i + 1)) {
                debug!(format!("sync_var: {:?}", sync_var.data.to_vec()));
                batch.write(sync_var.data.as_ref());
            }
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