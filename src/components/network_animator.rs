use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait};
use crate::core::network_reader::NetworkReader;
use crate::core::network_writer::NetworkWriter;
use std::any::Any;

#[derive(Debug)]
pub struct NetworkAnimator {
    pub network_behaviour: NetworkBehaviour,
}

impl NetworkBehaviourTrait for NetworkAnimator {
    fn get_network_behaviour_base(&mut self) -> &mut NetworkBehaviour {
        self.network_behaviour.get_network_behaviour_base()
    }

    fn deserialize_objects_all(&self, un_batch: NetworkReader, initial_state: bool) {
        todo!()
    }

    fn serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        todo!()
    }

    fn deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool {
        todo!()
    }

    fn on_start_server(&mut self) {
        todo!()
    }

    fn on_stop_server(&mut self) {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}


impl NetworkAnimator {
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkAnimator";
}