use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait};
use crate::core::backend_data::NetworkBehaviourSetting;
use crate::core::network_reader::NetworkReader;
use crate::core::network_writer::NetworkWriter;

#[derive(Debug)]
pub struct NetworkRigidbodyUnreliable {
    pub network_behaviour: NetworkBehaviour,
}

impl NetworkRigidbodyUnreliable {
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkRigidbodyUnreliable";
    pub fn new(network_behaviour_setting: NetworkBehaviourSetting, component_index: u8) -> Self {
        NetworkRigidbodyUnreliable {
            network_behaviour: NetworkBehaviour::new(network_behaviour_setting, component_index),
        }
    }
}

impl NetworkBehaviourTrait for NetworkRigidbodyUnreliable {
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
}