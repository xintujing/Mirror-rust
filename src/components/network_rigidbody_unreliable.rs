use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait};
use crate::core::batcher::{Batch, UnBatch};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct NetworkRigidbodyUnreliable {
    pub network_behaviour: NetworkBehaviour,
}

impl NetworkRigidbodyUnreliable {
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkRigidbodyUnreliable";
    pub fn new(component_index: u8) -> Self {
        NetworkRigidbodyUnreliable {
            network_behaviour: NetworkBehaviour::new(component_index),
        }
    }
}

impl NetworkBehaviourTrait for NetworkRigidbodyUnreliable {
    fn deserialize_objects_all(&self, un_batch: UnBatch, initial_state: bool) {
        todo!()
    }

    fn serialize(&self, initial_state: bool) -> Batch {
        todo!()
    }

    fn deserialize(&self, un_batch: &mut UnBatch, initial_state: bool) {
        todo!()
    }

    fn get_network_behaviour(&self) -> &NetworkBehaviour {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}