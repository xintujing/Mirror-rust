use crate::batcher::{Batch, UnBatch};
use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct NetworkAnimator {
    pub network_behaviour: NetworkBehaviour,
}

impl NetworkBehaviourTrait for NetworkAnimator {
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

impl NetworkAnimator {
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkAnimator";
}