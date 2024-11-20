use crate::mirror::core::backend_data::{BackendDataStatic, NetworkBehaviourComponent, SyncVarData};
use crate::mirror::core::network_behaviour::{GameObject, NetworkBehaviour, NetworkBehaviourTrait, SyncDirection, SyncMode};
use crate::mirror::core::network_writer::NetworkWriter;
use crate::mirror::core::sync_object::SyncObject;
use dashmap::DashMap;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Once;
use tklog::debug;

#[derive(Debug)]
pub struct NetworkCommonBehaviour {
    network_behaviour: NetworkBehaviour,
    sync_vars: DashMap<u8, SyncVarData>,
}

impl NetworkCommonBehaviour {
    #[allow(dead_code)]
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkCommon";
}

impl NetworkBehaviourTrait for NetworkCommonBehaviour {
    fn new(game_object: GameObject, network_behaviour_component: &NetworkBehaviourComponent) -> Self
    where
        Self: Sized,
    {
        Self::call_register_delegate();
        let sync_vars = DashMap::new();
        for (i, sync_var) in BackendDataStatic::get_backend_data().get_sync_var_data_s_by_sub_class(network_behaviour_component.sub_class.as_ref()).iter().enumerate() {
            sync_vars.insert(i as u8, (*sync_var).clone());
        }
        Self {
            network_behaviour: NetworkBehaviour::new(game_object, network_behaviour_component.network_behaviour_setting.clone(), network_behaviour_component.index),
            sync_vars,
        }
    }

    fn register_delegate()
    where
        Self: Sized,
    {
        debug!("Registering delegate for NetworkCommonBehaviour");
    }

    fn get_once() -> &'static Once
    where
        Self: Sized,
    {
        static ONCE: Once = Once::new();
        &ONCE
    }

    fn sync_interval(&self) -> f64 {
        self.network_behaviour.sync_interval
    }

    fn set_sync_interval(&mut self, value: f64) {
        self.network_behaviour.sync_interval = value
    }

    fn last_sync_time(&self) -> f64 {
        self.network_behaviour.last_sync_time
    }

    fn set_last_sync_time(&mut self, value: f64) {
        self.network_behaviour.last_sync_time = value
    }

    fn sync_direction(&mut self) -> &SyncDirection {
        &self.network_behaviour.sync_direction
    }

    fn set_sync_direction(&mut self, value: SyncDirection) {
        self.network_behaviour.sync_direction = value
    }

    fn sync_mode(&mut self) -> &SyncMode {
        &self.network_behaviour.sync_mode
    }

    fn set_sync_mode(&mut self, value: SyncMode) {
        self.network_behaviour.sync_mode = value
    }

    fn index(&self) -> u8 {
        self.network_behaviour.index
    }

    fn set_index(&mut self, value: u8) {
        self.network_behaviour.index = value
    }

    fn sync_var_dirty_bits(&self) -> u64 {
        self.network_behaviour.sync_var_dirty_bits
    }

    fn __set_sync_var_dirty_bits(&mut self, value: u64) {
        self.network_behaviour.sync_var_dirty_bits = value
    }

    fn update_sync_var(&mut self, index: u8, value: Vec<u8>) {
        if let Some(mut sync_var) = self.sync_vars.get_mut(&index) {
            sync_var.value = value;
        }
        self.set_sync_var_dirty_bits(1 << index);
    }

    fn sync_object_dirty_bits(&self) -> u64 {
        self.network_behaviour.sync_object_dirty_bits
    }

    fn __set_sync_object_dirty_bits(&mut self, value: u64) {
        self.network_behaviour.sync_object_dirty_bits = value
    }

    fn net_id(&self) -> u32 {
        self.network_behaviour.net_id
    }

    fn set_net_id(&mut self, value: u32) {
        self.network_behaviour.net_id = value
    }

    fn connection_to_client(&self) -> u64 {
        self.network_behaviour.connection_to_client
    }

    fn set_connection_to_client(&mut self, value: u64) {
        self.network_behaviour.connection_to_client = value
    }

    fn observers(&self) -> &Vec<u64> {
        &self.network_behaviour.observers
    }

    fn set_observers(&mut self, value: Vec<u64>) {
        self.network_behaviour.observers = value
    }

    fn game_object(&self) -> &GameObject {
        &self.network_behaviour.game_object
    }

    fn set_game_object(&mut self, value: GameObject) {
        self.network_behaviour.game_object = value
    }

    fn sync_objects(&mut self) -> &mut Vec<Box<dyn SyncObject>> {
        &mut self.network_behaviour.sync_objects
    }

    fn set_sync_objects(&mut self, value: Vec<Box<dyn SyncObject>>) {
        self.network_behaviour.sync_objects = value
    }

    fn sync_var_hook_guard(&self) -> u64 {
        self.network_behaviour.sync_var_hook_guard
    }

    fn __set_sync_var_hook_guard(&mut self, value: u64) {
        self.network_behaviour.sync_var_hook_guard = value
    }

    fn is_dirty(&self) -> bool {
        self.network_behaviour.is_dirty()
    }

    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        if initial_state {
            for i in 0..self.sync_vars.len() as u8 {
                if let Some(sync_var) = self.sync_vars.get(&i) {
                    writer.write_array_segment_all(sync_var.value.as_slice());
                }
            }
        } else {
            for i in 0..self.sync_vars.len() as u8 {
                if self.sync_var_dirty_bits() & (1 << i) != 0 {
                    if let Some(sync_var) = self.sync_vars.get(&i) {
                        writer.write_array_segment_all(sync_var.value.as_slice());
                    }
                }
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}