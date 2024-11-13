use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait, SyncDirection, SyncMode};
use crate::components::SyncVar;
use crate::core::backend_data::NetworkBehaviourSetting;
use crate::core::network_manager::GameObject;
use crate::core::network_reader::NetworkReader;
use crate::core::network_writer::NetworkWriter;
use dashmap::DashMap;
use std::any::Any;
use std::fmt::Debug;

#[derive(Debug)]
pub struct NetworkCommonBehaviour {
    network_behaviour: NetworkBehaviour,
    pub sync_vars: DashMap<u8, SyncVar>,
}

impl NetworkCommonBehaviour {
    #[allow(dead_code)]
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkCommon";
    pub fn new(network_behaviour_setting: NetworkBehaviourSetting, component_index: u8, sync_vars: DashMap<u8, SyncVar>) -> Self {
        NetworkCommonBehaviour {
            network_behaviour: NetworkBehaviour::new(network_behaviour_setting, component_index),
            sync_vars,
        }
    }
}

impl NetworkBehaviourTrait for NetworkCommonBehaviour {
    fn sync_interval(&self) -> f64 {
        self.network_behaviour.sync_interval()
    }

    fn set_sync_interval(&mut self, value: f64) {
        self.network_behaviour.set_sync_interval(value)
    }

    fn last_sync_time(&self) -> f64 {
        self.network_behaviour.last_sync_time()
    }

    fn set_last_sync_time(&mut self, value: f64) {
        self.network_behaviour.set_last_sync_time(value)
    }

    fn sync_direction(&mut self) -> &SyncDirection {
        self.network_behaviour.sync_direction()
    }

    fn set_sync_direction(&mut self, value: SyncDirection) {
        self.network_behaviour.set_sync_direction(value)
    }

    fn sync_mode(&mut self) -> &SyncMode {
        self.network_behaviour.sync_mode()
    }

    fn set_sync_mode(&mut self, value: SyncMode) {
        self.network_behaviour.set_sync_mode(value)
    }

    fn component_index(&self) -> u8 {
        self.network_behaviour.component_index()
    }

    fn set_component_index(&mut self, value: u8) {
        self.network_behaviour.set_component_index(value)
    }

    fn sync_var_dirty_bits(&self) -> u64 {
        self.network_behaviour.sync_var_dirty_bits()
    }

    fn set_sync_var_dirty_bits(&mut self, value: u64) {
        self.network_behaviour.set_sync_var_dirty_bits(value)
    }

    fn sync_object_dirty_bits(&self) -> u64 {
        self.network_behaviour.sync_object_dirty_bits()
    }

    fn set_sync_object_dirty_bits(&mut self, value: u64) {
        self.network_behaviour.set_sync_object_dirty_bits(value)
    }

    fn net_id(&self) -> u32 {
        self.network_behaviour.net_id()
    }

    fn set_net_id(&mut self, value: u32) {
        self.network_behaviour.set_net_id(value)
    }

    fn connection_to_client(&self) -> u64 {
        self.network_behaviour.connection_to_client()
    }

    fn set_connection_to_client(&mut self, value: u64) {
        self.network_behaviour.set_connection_to_client(value)
    }

    fn observers(&self) -> &Vec<u64> {
        self.network_behaviour.observers()
    }

    fn set_observers(&mut self, value: Vec<u64>) {
        self.network_behaviour.set_observers(value)
    }

    fn game_object(&self) -> &GameObject {
        self.network_behaviour.game_object()
    }

    fn set_game_object(&mut self, value: GameObject) {
        self.network_behaviour.set_game_object(value)
    }

    fn is_dirty(&self) -> bool {
        self.network_behaviour.is_dirty()
    }

    fn deserialize_objects_all(&self, un_batch: NetworkReader, initial_state: bool) {}

    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        for i in 0..self.sync_vars.len() as u8 {
            if let Some(sync_var) = self.sync_vars.get(&i) {
                writer.write_array_segment_all(sync_var.data.as_slice());
            }
        }
    }

    fn deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool {
        todo!()
    }

    fn on_start_server(&mut self) {}

    fn on_stop_server(&mut self) {}

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}