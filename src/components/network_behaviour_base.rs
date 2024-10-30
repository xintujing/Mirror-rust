use crate::core::backend_data::NetworkBehaviourSetting;
use crate::core::batcher::UnBatch;
use crate::core::network_reader::NetworkReader;
use crate::core::network_time::NetworkTime;
use crate::core::network_writer::NetworkWriter;
use std::any::Any;

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

pub trait NetworkBehaviourTrait: Any + Send + Sync {
    fn get_network_behaviour_base(&mut self) -> &mut NetworkBehaviourBase;
    // DeserializeObjectsAll
    fn deserialize_objects_all(&self, un_batch: UnBatch, initial_state: bool);
    // Serialize
    fn serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool);
    // Deserialize
    fn deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool);
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct NetworkBehaviourBase {
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

impl NetworkBehaviourBase {
    pub fn new(network_behaviour_setting: NetworkBehaviourSetting, component_index: u8) -> Self {
        NetworkBehaviourBase {
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

impl NetworkBehaviourTrait for NetworkBehaviourBase {
    fn get_network_behaviour_base(&mut self) -> &mut NetworkBehaviourBase {
        self
    }

    fn deserialize_objects_all(&self, un_batch: UnBatch, initial_state: bool) {
        todo!()
    }

    fn serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        todo!()
    }

    fn deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) {
        todo!()
    }


    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}