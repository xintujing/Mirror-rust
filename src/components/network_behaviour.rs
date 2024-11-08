use crate::core::backend_data::NetworkBehaviourSetting;
use crate::core::network_reader::NetworkReader;
use crate::core::network_time::NetworkTime;
use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
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
    sync_interval: f64,
    last_sync_time: f64,
    sync_direction: SyncDirection,
    sync_mode: SyncMode,
    // ComponentIndex
    component_index: u8,
    // syncVarDirtyBits
    sync_var_dirty_bits: u64,
    // syncObjectDirtyBits
    sync_object_dirty_bits: u64,
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
    // 字段 get  set start
    fn sync_interval(&self) -> f64;
    fn set_sync_interval(&mut self, value: f64);
    fn last_sync_time(&self) -> f64;
    fn set_last_sync_time(&mut self, value: f64);
    fn sync_direction(&mut self) -> &SyncDirection;
    fn set_sync_direction(&mut self, value: SyncDirection);
    fn sync_mode(&mut self) -> &SyncMode;
    fn set_sync_mode(&mut self, value: SyncMode);
    fn component_index(&self) -> u8;
    fn set_component_index(&mut self, value: u8);
    fn sync_var_dirty_bits(&self) -> u64;
    fn set_sync_var_dirty_bits(&mut self, value: u64);
    fn sync_object_dirty_bits(&self) -> u64;
    fn set_sync_object_dirty_bits(&mut self, value: u64);
    // 字段 get  set end
    fn is_dirty(&self) -> bool;
    // DeserializeObjectsAll
    fn deserialize_objects_all(&self, un_batch: NetworkReader, initial_state: bool);
    // Serialize
    fn serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        let header_position = writer.get_position();
        writer.write_byte(0);
        let content_position = writer.get_position();
        self.on_serialize(writer, initial_state);
        let end_position = writer.get_position();
        writer.set_position(header_position);
        let size = (end_position - content_position) as u8;
        let safety = size & 0xFF;
        writer.write_byte(safety);
        writer.set_position(end_position);
    }
    // void OnSerialize(NetworkWriter writer, bool initialState)
    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool);
    // Deserialize
    fn deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool;
    // SetDirty
    fn set_dirty(&mut self) {
        self.set_sync_var_dirty_bit(u64::MAX);
    }
    // SetSyncVarDirtyBit
    fn set_sync_var_dirty_bit(&mut self, dirty_bit: u64) {
        self.set_sync_var_dirty_bits(self.sync_var_dirty_bits() | dirty_bit);
    }
    fn on_start_server(&mut self);
    fn on_stop_server(&mut self);
    fn clear_all_dirty_bits(&mut self) {
        self.set_sync_var_dirty_bits(0);
        self.set_sync_object_dirty_bits(0);

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
    fn sync_interval(&self) -> f64 {
        self.sync_interval
    }

    fn set_sync_interval(&mut self, value: f64) {
        self.sync_interval = value;
    }

    fn last_sync_time(&self) -> f64 {
        self.last_sync_time
    }

    fn set_last_sync_time(&mut self, value: f64) {
        self.last_sync_time = value;
    }

    fn sync_direction(&mut self) -> &SyncDirection {
        &self.sync_direction
    }

    fn set_sync_direction(&mut self, value: SyncDirection) {
        self.sync_direction = value;
    }

    fn sync_mode(&mut self) -> &SyncMode {
        &self.sync_mode
    }

    fn set_sync_mode(&mut self, value: SyncMode) {
        self.sync_mode = value;
    }

    fn component_index(&self) -> u8 {
        self.component_index
    }

    fn set_component_index(&mut self, value: u8) {
        self.component_index = value;
    }

    fn sync_var_dirty_bits(&self) -> u64 {
        self.sync_var_dirty_bits
    }

    fn set_sync_var_dirty_bits(&mut self, value: u64) {
        self.sync_var_dirty_bits = value;
    }

    fn sync_object_dirty_bits(&self) -> u64 {
        self.sync_object_dirty_bits
    }

    fn set_sync_object_dirty_bits(&mut self, value: u64) {
        self.sync_object_dirty_bits = value;
    }

    fn is_dirty(&self) -> bool {
        self.sync_var_dirty_bits | self.sync_object_dirty_bits != 0u64 &&
            NetworkTime::local_time() - self.last_sync_time > self.sync_interval
    }

    fn deserialize_objects_all(&self, un_batch: NetworkReader, initial_state: bool) {
        todo!()
    }

    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        // SerializeSyncObjects(writer, initialState);
        // SerializeSyncVars(writer, initialState);
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