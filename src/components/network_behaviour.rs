use crate::components::network_common_behaviour::NetworkCommonBehaviour;
use crate::components::network_transform::network_transform_reliable::NetworkTransformReliable;
use crate::components::network_transform::network_transform_unreliable::NetworkTransformUnreliable;
use crate::core::backend_data::{NetworkBehaviourComponent, NetworkBehaviourSetting};
use crate::core::messages::RpcMessage;
use crate::core::network_connection::NetworkConnectionTrait;
use crate::core::network_identity::NetworkIdentity;
use crate::core::network_manager::GameObject;
use crate::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::core::network_server::NetworkServerStatic;
use crate::core::network_time::NetworkTime;
use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::core::sync_object::SyncObject;
use crate::core::transport::TransportChannel;
use byteorder::ReadBytesExt;
use dashmap::DashMap;
use lazy_static::lazy_static;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Once;
use tklog::{debug, error, warn};

type NetworkBehaviourFactoryType = Box<dyn Fn(GameObject, &NetworkBehaviourComponent) -> Box<dyn NetworkBehaviourTrait> + Send + Sync>;

lazy_static! {
    static ref NETWORK_BEHAVIOURS_FAACTORIES: DashMap<String, NetworkBehaviourFactoryType> = DashMap::new();
}
pub struct NetworkBehaviourFactory;
impl NetworkBehaviourFactory {
    fn add_network_behaviour_factory(name: String, factory: NetworkBehaviourFactoryType) {
        NETWORK_BEHAVIOURS_FAACTORIES.insert(name, factory);
    }
    pub fn create_network_behaviour(name: &str, game_object: GameObject, component: &NetworkBehaviourComponent) -> Option<Box<dyn NetworkBehaviourTrait>> {
        if let Some(factory) = NETWORK_BEHAVIOURS_FAACTORIES.get(name) {
            Some(factory(game_object, component))
        } else {
            error!(format!("NetworkBehaviourFactory::create - factory not found for {}", name));
            None
        }
    }
    pub fn register_network_behaviour_factory() {
        // NetworkTransformUnreliable
        Self::add_network_behaviour_factory(NetworkTransformUnreliable::COMPONENT_TAG.to_string(), Box::new(|game_object: GameObject, component: &NetworkBehaviourComponent| Box::new(NetworkTransformUnreliable::new(game_object, component))));
        // NetworkTransformReliable
        Self::add_network_behaviour_factory(NetworkTransformReliable::COMPONENT_TAG.to_string(), Box::new(|game_object: GameObject, component: &NetworkBehaviourComponent| Box::new(NetworkTransformReliable::new(game_object, component))));
        // QuickStart.PlayerScript
        Self::add_network_behaviour_factory("QuickStart.PlayerScript".to_string(), Box::new(|game_object: GameObject, component: &NetworkBehaviourComponent| Box::new(NetworkCommonBehaviour::new(game_object, component))));
    }
}


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
    pub index: u8,
    // syncVarDirtyBits
    pub sync_var_dirty_bits: u64,
    // syncObjectDirtyBits
    pub sync_object_dirty_bits: u64,
    pub net_id: u32,
    pub connection_to_client: u64,
    pub observers: Vec<u64>,
    pub game_object: GameObject,
    pub sync_objects: Vec<Box<dyn SyncObject>>,
    pub sync_var_hook_guard: u64,
}

impl NetworkBehaviour {
    pub fn new(game_object: GameObject, network_behaviour_setting: NetworkBehaviourSetting, component_index: u8) -> Self {
        NetworkBehaviour {
            sync_interval: 0.0,
            last_sync_time: 0.0,
            sync_direction: SyncDirection::from_u8(network_behaviour_setting.sync_direction),
            sync_mode: SyncMode::Observers,
            index: component_index,
            sync_var_dirty_bits: 0,
            sync_object_dirty_bits: 0,
            net_id: 0,
            connection_to_client: 0,
            observers: Default::default(),
            game_object,
            sync_objects: Default::default(),
            sync_var_hook_guard: 0,
        }
    }
    pub fn is_dirty(&self) -> bool {
        self.sync_var_dirty_bits | self.sync_object_dirty_bits != 0u64 &&
            NetworkTime::local_time() - self.last_sync_time > self.sync_interval
    }
    pub fn early_invoke(identity: &mut NetworkIdentity, component_index: u8) -> &mut Box<dyn NetworkBehaviourTrait> {
        // 需要传递给 component 的参数
        let observers = identity.observers.clone();
        // 获取 component
        let component = &mut identity.network_behaviours[component_index as usize];
        // 设置 component 的参数
        component.set_observers(observers);
        // 返回 component
        component
    }
    pub fn late_invoke(identity: &mut NetworkIdentity, component_index: u8) {
        // 获取 component
        let component = &identity.network_behaviours[component_index as usize];
        identity.set_game_object(component.game_object().clone());
    }
    pub fn error_correction(size: usize, safety: u8) -> usize {
        let cleared = size & 0xFFFFFF00;
        cleared | safety as usize
    }
}


pub trait NetworkBehaviourTrait: Any + Send + Sync + Debug {
    fn new(game_object: GameObject, network_behaviour_component: &NetworkBehaviourComponent) -> Self
    where
        Self: Sized;
    fn register_delegate()
    where
        Self: Sized;
    fn call_register_delegate()
    where
        Self: Sized,
    {
        Self::get_once().call_once(Self::register_delegate);
    }
    fn get_once() -> &'static Once
    where
        Self: Sized;
    // 字段 get  set start
    fn sync_interval(&self) -> f64;
    fn set_sync_interval(&mut self, value: f64);
    fn last_sync_time(&self) -> f64;
    fn set_last_sync_time(&mut self, value: f64);
    fn sync_direction(&mut self) -> &SyncDirection;
    fn set_sync_direction(&mut self, value: SyncDirection);
    fn sync_mode(&mut self) -> &SyncMode;
    fn set_sync_mode(&mut self, value: SyncMode);
    fn index(&self) -> u8;
    fn set_index(&mut self, value: u8);
    fn sync_var_dirty_bits(&self) -> u64;
    fn set_sync_var_dirty_bits(&mut self, value: u64);
    fn sync_object_dirty_bits(&self) -> u64;
    fn set_sync_object_dirty_bits(&mut self, value: u64);
    fn net_id(&self) -> u32;
    fn set_net_id(&mut self, value: u32);
    fn connection_to_client(&self) -> u64;
    fn set_connection_to_client(&mut self, value: u64);
    fn observers(&self) -> &Vec<u64>;
    fn set_observers(&mut self, value: Vec<u64>);
    fn game_object(&self) -> &GameObject;
    fn set_game_object(&mut self, value: GameObject);
    fn sync_objects(&mut self) -> &mut Vec<Box<dyn SyncObject>>;
    fn set_sync_objects(&mut self, value: Vec<Box<dyn SyncObject>>);
    fn has_sync_objects(&mut self) -> bool {
        self.sync_objects().len() > 0
    }
    fn sync_var_hook_guard(&self) -> u64;
    fn set_sync_var_hook_guard(&mut self, value: u64);
    fn get_sync_var_hook_guard(&self, dirty_bit: u64) -> bool {
        (dirty_bit & self.sync_var_hook_guard()) != 0
    }
    // SetSyncVarHookGuard(ulong dirtyBit, bool value)
    fn update_sync_var_hook_guard(&mut self, dirty_bit: u64, value: bool) {
        if value {
            self.set_sync_var_hook_guard(self.sync_var_hook_guard() | dirty_bit);
        } else {
            self.set_sync_var_hook_guard(self.sync_var_hook_guard() & !dirty_bit);
        }
    }
    // 字段 get  set end
    fn is_dirty(&self) -> bool;
    // DeserializeObjectsAll
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
    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        self.serialize_sync_objects(writer, initial_state);
        self.serialize_sync_vars(writer, initial_state);
    }
    // SerializeSyncObjects
    fn serialize_sync_objects(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        if initial_state {
            self.serialize_objects_all(writer);
        } else {
            self.serialize_sync_object_delta(writer);
        }
    }
    // SerializeSyncVars
    // TODO USED BY WEAVER
    fn serialize_sync_vars(&mut self, writer: &mut NetworkWriter, initial_state: bool) {}
    fn serialize_objects_all(&mut self, writer: &mut NetworkWriter) {
        for sync_object in self.sync_objects().iter_mut() {
            sync_object.on_serialize_all(writer);
        }
    }
    fn serialize_sync_object_delta(&mut self, writer: &mut NetworkWriter) {
        writer.write_ulong(self.sync_object_dirty_bits());
        for i in 0..self.sync_objects().len() {
            if self.sync_object_dirty_bits() & (1 << i) != 0 {
                let sync_object = &mut self.sync_objects()[i];
                sync_object.on_serialize_delta(writer);
            }
        }
    }
    // OnDeserialize
    fn on_deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool {
        let mut result = false;
        result = self.deserialize_sync_objects(reader, initial_state);
        result = self.deserialize_sync_vars(reader, initial_state);
        result
    }
    // Deserialize
    fn deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool {
        let mut result = true;

        let safety = reader.read_byte();
        let chunk_start = reader.get_position();

        result = self.on_deserialize(reader, initial_state);

        let size = reader.get_position() - chunk_start;
        let size_hash = size as u8 & 0xFF;
        if size_hash != safety {
            warn!(format!("Deserialize failed. Size mismatch. Expected: {}, Received: {}", size_hash, safety));
            let corrected_size = NetworkBehaviour::error_correction(size, safety);
            reader.set_position(chunk_start + corrected_size);
            result = false;
        }
        result
    }
    // DeserializeSyncObjects
    fn deserialize_sync_objects(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool {
        if initial_state {
            self.deserialize_objects_all(reader)
        } else {
            self.deserialize_sync_object_delta(reader)
        }
    }
    // DeserializeSyncVars
    // TODO USED BY WEAVER
    fn deserialize_sync_vars(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool {
        true
    }
    // deserializeObjectsAll
    fn deserialize_objects_all(&mut self, reader: &mut NetworkReader) -> bool {
        let mut result = true;
        for sync_object in self.sync_objects().iter_mut() {
            let succ = sync_object.on_deserialize_all(reader);
            if !succ {
                result = false;
            }
        }
        result
    }
    // DeserializeSyncObjectDelta
    fn deserialize_sync_object_delta(&mut self, reader: &mut NetworkReader) -> bool {
        let mut result = true;
        let dirty = reader.read_ulong();
        for i in 0..self.sync_objects().len() {
            if dirty & (1 << i) != 0 {
                let sync_object = &mut self.sync_objects()[i];
                let succ = sync_object.on_deserialize_delta(reader);
                if !succ {
                    result = false;
                }
            }
        }
        result
    }
    // SetDirty
    fn set_dirty(&mut self) {
        self.set_sync_var_dirty_bit(u64::MAX);
    }
    // SetSyncVarDirtyBit
    fn set_sync_var_dirty_bit(&mut self, dirty_bit: u64) {
        self.set_sync_var_dirty_bits(self.sync_var_dirty_bits() | dirty_bit);
    }
    fn clear_all_dirty_bits(&mut self) {
        self.set_sync_var_dirty_bits(0);
        self.set_sync_object_dirty_bits(0);

        // TODO sync_objects
    }
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn send_rpc_internal(&self, function_full_name: &'static str, function_hash_code: i32, writer: &NetworkWriter, channel: TransportChannel, include_owner: bool) {
        if !NetworkServerStatic::get_static_active() {
            error!(format!("RPC Function {} called without an active server.", function_full_name));
            return;
        }

        let mut rpc = RpcMessage::new(self.net_id(), self.index(), function_hash_code as u16, writer.to_bytes());
        for observer in self.observers().iter() {
            if let Some(mut conn_to_client) = NetworkServerStatic::get_static_network_connections().get_mut(&observer) {
                let is_owner = conn_to_client.connection_id() == self.connection_to_client();
                if (!is_owner || include_owner) && conn_to_client.is_ready() {
                    conn_to_client.send_network_message(&mut rpc, channel);
                }
            }
        }
    }
    fn on_start_server(&mut self) {}
    fn on_stop_server(&mut self) {}
    fn update(&mut self) {}
    fn late_update(&mut self) {}
}