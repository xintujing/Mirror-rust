use crate::components::network_behaviour_base::{NetworkBehaviourBase, NetworkBehaviourTrait};
use crate::components::SyncVar;
use crate::core::backend_data::NetworkBehaviourSetting;
use crate::core::network_reader::NetworkReader;
use crate::core::network_writer::NetworkWriter;
use dashmap::DashMap;
use std::fmt::Debug;

#[derive(Debug)]
pub struct NetworkCommonBehaviour {
    pub network_behaviour: NetworkBehaviourBase,
    pub sync_vars: DashMap<u8, SyncVar>,
}

impl NetworkCommonBehaviour {
    #[allow(dead_code)]
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkCommon";
    pub fn new(network_behaviour_setting: NetworkBehaviourSetting, component_index: u8, sync_vars: DashMap<u8, SyncVar>) -> Self {
        NetworkCommonBehaviour {
            network_behaviour: NetworkBehaviourBase::new(network_behaviour_setting, component_index),
            sync_vars,
        }
    }
}

impl NetworkBehaviourTrait for NetworkCommonBehaviour {
    fn get_network_behaviour_base(&mut self) -> &mut NetworkBehaviourBase {
        self.network_behaviour.get_network_behaviour_base()
    }

    fn deserialize_objects_all(&self, un_batch: NetworkReader, initial_state: bool) {}

    fn serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        for i in 0..self.sync_vars.len() as u8 {
            if let Some(sync_var) = self.sync_vars.get(&i) {
                writer.write_array_segment_all(sync_var.data.as_ref());
            }
        }
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