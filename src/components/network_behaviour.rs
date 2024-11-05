use crate::core::backend_data::NetworkBehaviourSetting;
use crate::core::network_reader::NetworkReader;
use crate::core::network_time::NetworkTime;
use crate::core::network_writer::NetworkWriter;
use std::any::Any;
use std::fmt::Debug;

#[derive(Debug, PartialOrd, PartialEq)]
pub enum SyncDirection {
    ServerToClient,
    ClientToServer,
}

impl SyncDirection {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => SyncDirection::ServerToClient,
            1 => SyncDirection::ClientToServer,
            _ => panic!("Invalid SyncDirection value"),
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub enum SyncMode {
    Observers,
    Owners,
}

#[derive(Debug)]
pub struct NetworkBehaviour {
    pub sync_interval: f64,
    pub last_sync_time: f64,
    pub sync_direction: SyncDirection,
    pub sync_mode: SyncMode,
    // ComponentIndex
    pub component_index: u8,
    // syncVarDirtyBits
    pub sync_var_dirty_bits: u64,
    // syncObjectDirtyBits
    pub sync_object_dirty_bits: u64,
}

impl NetworkBehaviour {
    pub fn new(network_behaviour_setting: NetworkBehaviourSetting, component_index: u8) -> Self {
        NetworkBehaviour {
            sync_interval: 0.0,
            last_sync_time: 0.0,
            sync_direction: SyncDirection::from_u8(network_behaviour_setting.sync_direction),
            sync_mode: SyncMode::Observers,
            component_index,
            sync_var_dirty_bits: 0,
            sync_object_dirty_bits: 0,
        }
    }
    pub fn is_dirty(&self) -> bool {
        self.sync_var_dirty_bits | self.sync_object_dirty_bits != 0u64 &&
            NetworkTime::local_time() - self.last_sync_time > self.sync_interval
    }
}


pub trait NetworkBehaviourTrait: Any + Send + Sync + Debug {
    fn get_network_behaviour_base(&mut self) -> &mut NetworkBehaviour;
    // DeserializeObjectsAll
    fn deserialize_objects_all(&self, un_batch: NetworkReader, initial_state: bool);
    // Serialize
    fn serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool);
    // Deserialize
    fn deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool;
    // SetDirty
    fn set_dirty(&mut self) {
        self.set_sync_var_dirty_bit(u64::MAX);
    }
    // SetSyncVarDirtyBit
    fn set_sync_var_dirty_bit(&mut self, dirty_bit: u64) {
        self.get_network_behaviour_base().sync_var_dirty_bits |= dirty_bit;
    }
    // SyncDirection
    fn sync_direction(&mut self) -> &SyncDirection {
        &self.get_network_behaviour_base().sync_direction
    }
    fn on_start_server(&mut self);
    fn on_stop_server(&mut self);
    fn clear_all_dirty_bits(&mut self) {
        self.get_network_behaviour_base().sync_var_dirty_bits = 0;
        self.get_network_behaviour_base().sync_object_dirty_bits = 0;

        // TODO syncObjects
    }
    fn as_any(&self) -> &dyn Any
    where
        Self: Sized,
    {
        self
    }
}

impl NetworkBehaviourTrait for NetworkBehaviour {
    fn get_network_behaviour_base(&mut self) -> &mut NetworkBehaviour {
        self
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