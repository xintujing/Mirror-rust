use crate::mirror::components::network_animator::NetworkAnimator;
use crate::mirror::components::network_common_behaviour::NetworkCommonBehaviour;
use crate::mirror::components::network_rigidbody::network_rigidbody_reliable::NetworkRigidbodyReliable;
use crate::mirror::components::network_rigidbody::network_rigidbody_unreliable::NetworkRigidbodyUnreliable;
use crate::mirror::components::network_room_player::NetworkRoomPlayer;
use crate::mirror::components::network_transform::network_transform_base::Transform;
use crate::mirror::components::network_transform::network_transform_reliable::NetworkTransformReliable;
use crate::mirror::components::network_transform::network_transform_unreliable::NetworkTransformUnreliable;
use crate::mirror::core::backend_data::{
    BackendDataStatic, NetworkBehaviourComponent, NetworkBehaviourSetting,
};
use crate::mirror::core::messages::{EntityStateMessage, RpcMessage};
use crate::mirror::core::network_connection::NetworkConnectionTrait;
use crate::mirror::core::network_identity::NetworkIdentity;
use crate::mirror::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::mirror::core::network_server::NetworkServerStatic;
use crate::mirror::core::network_time::NetworkTime;
use crate::mirror::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::mirror::core::sync_object::SyncObject;
use crate::mirror::core::transport::TransportChannel;
use crate::{log_error, log_warn};
use dashmap::try_result::TryResult;
use dashmap::DashMap;
use lazy_static::lazy_static;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Once;

type NetworkBehaviourFactoryType =
fn(GameObject, &NetworkBehaviourComponent) -> Box<dyn NetworkBehaviourTrait>;

lazy_static! {
    static ref NETWORK_BEHAVIOURS_FACTORIES: DashMap<String, NetworkBehaviourFactoryType> =
        DashMap::new();
}
pub struct NetworkBehaviourFactory;
impl NetworkBehaviourFactory {
    pub fn add_network_behaviour_factory(name: String, factory: NetworkBehaviourFactoryType) {
        NETWORK_BEHAVIOURS_FACTORIES.insert(name, factory);
    }
    pub fn create_network_behaviour(
        game_object: GameObject,
        component: &NetworkBehaviourComponent,
    ) -> Option<Box<dyn NetworkBehaviourTrait>> {
        // 根据 类名 从 NETWORK_BEHAVIOURS_FACTORIES 中获取对应的工厂方法
        match NETWORK_BEHAVIOURS_FACTORIES.get(&component.sub_class) {
            // 如果存在则调用工厂方法创建 NetworkBehaviour
            Some(factory) => Some(factory(game_object, component)),
            // 如果不存在则创建 NetworkCommonBehaviour
            None => Some(Box::new(NetworkCommonBehaviour::new(
                game_object,
                component,
            ))),
        }
    }
    pub fn register_network_behaviour_factory() {
        // NetworkTransformUnreliable
        Self::add_network_behaviour_factory(
            NetworkTransformUnreliable::COMPONENT_TAG.to_string(),
            |game_object: GameObject, component: &NetworkBehaviourComponent| {
                Box::new(NetworkTransformUnreliable::new(game_object, component))
            },
        );
        // NetworkTransformReliable
        Self::add_network_behaviour_factory(
            NetworkTransformReliable::COMPONENT_TAG.to_string(),
            |game_object: GameObject, component: &NetworkBehaviourComponent| {
                Box::new(NetworkTransformReliable::new(game_object, component))
            },
        );
        // NetworkRigidbodyUnreliable
        Self::add_network_behaviour_factory(
            NetworkRigidbodyUnreliable::COMPONENT_TAG.to_string(),
            |game_object: GameObject, component: &NetworkBehaviourComponent| {
                Box::new(NetworkTransformUnreliable::new(game_object, component))
            },
        );
        // NetworkRigidbodyReliable
        Self::add_network_behaviour_factory(
            NetworkRigidbodyReliable::COMPONENT_TAG.to_string(),
            |game_object: GameObject, component: &NetworkBehaviourComponent| {
                Box::new(NetworkTransformReliable::new(game_object, component))
            },
        );
        // NetworkAnimator
        Self::add_network_behaviour_factory(
            NetworkAnimator::COMPONENT_TAG.to_string(),
            |game_object: GameObject, component: &NetworkBehaviourComponent| {
                Box::new(NetworkAnimator::new(game_object, component))
            },
        );
        // Mirror.NetworkRoomPlayer
        Self::add_network_behaviour_factory(
            NetworkRoomPlayer::COMPONENT_TAG.to_string(),
            |game_object: GameObject, component: &NetworkBehaviourComponent| {
                Box::new(NetworkRoomPlayer::new(game_object, component))
            },
        );
    }
}

// GameObject
#[derive(Debug, Clone)]
pub struct GameObject {
    pub scene_name: String,
    pub prefab: String,
    pub transform: Transform,
    pub active: bool,
}

// GameObject 的默认实现
impl GameObject {
    pub fn new_with_prefab(prefab: String) -> Self {
        Self {
            scene_name: "".to_string(),
            prefab,
            transform: Transform::default(),
            active: false,
        }
    }
    pub fn new_with_scene_name(scene_name: String) -> Self {
        Self {
            scene_name,
            prefab: "".to_string(),
            transform: Transform::default(),
            active: false,
        }
    }
    pub fn default() -> Self {
        Self {
            scene_name: "".to_string(),
            prefab: "".to_string(),
            transform: Transform::default(),
            active: false,
        }
    }
    pub fn is_has_component(&self) -> bool {
        if self.prefab == "" {
            return false;
        }
        if let None =
            BackendDataStatic::get_backend_data().get_asset_id_by_asset_name(self.prefab.as_str())
        {
            return false;
        }
        true
    }
    pub fn get_identity_by_prefab(&mut self) -> Option<NetworkIdentity> {
        // 如果 prefab 不为空
        if let Some(asset_id) =
            BackendDataStatic::get_backend_data().get_asset_id_by_asset_name(self.prefab.as_str())
        {
            let mut identity = NetworkIdentity::new_with_asset_id(asset_id);
            identity.set_game_object(self.clone());
            return Some(identity);
        };
        None
    }
    pub fn get_identity_by_scene_name(&mut self) -> Option<NetworkIdentity> {
        // 如果 scene_name 不为空
        if let Some(scene_id) = BackendDataStatic::get_backend_data()
            .get_scene_id_by_scene_name(self.scene_name.as_str())
        {
            let mut identity = NetworkIdentity::new_with_scene_id(scene_id);
            identity.set_game_object(self.clone());
            return Some(identity);
        };
        None
    }
    pub fn is_null(&self) -> bool {
        self.scene_name == "" && self.prefab == ""
    }
    pub fn set_active(&mut self, value: bool) {
        self.active = value;
    }
}
// GameObject 的 PartialEq 实现
impl PartialEq for GameObject {
    fn eq(&self, other: &Self) -> bool {
        self.scene_name == other.scene_name && self.prefab == other.prefab
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
            _ => SyncDirection::ServerToClient,
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
    pub sub_class: String,
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
    pub run_start: bool,
}

impl NetworkBehaviour {
    pub fn new(
        game_object: GameObject,
        network_behaviour_setting: NetworkBehaviourSetting,
        component_index: u8,
        sub_class: String,
    ) -> Self {
        NetworkBehaviour {
            sync_interval: 0.0,
            last_sync_time: 0.0,
            sync_direction: SyncDirection::from_u8(network_behaviour_setting.sync_direction),
            sync_mode: SyncMode::Observers,
            index: component_index,
            sub_class,
            sync_var_dirty_bits: u64::MAX,
            sync_object_dirty_bits: u64::MAX,
            net_id: 0,
            connection_to_client: 0,
            observers: Default::default(),
            game_object,
            sync_objects: Default::default(),
            sync_var_hook_guard: 0,
            run_start: true,
        }
    }
    pub fn is_dirty(&self) -> bool {
        self.sync_var_dirty_bits | self.sync_object_dirty_bits != 0u64
            && NetworkTime::local_time() - self.last_sync_time > self.sync_interval
    }
    // pub fn early_invoke(
    //     identity: &mut NetworkIdentity,
    //     component_index: u8,
    // ) -> &mut Box<dyn NetworkBehaviourTrait> {
    //     // 需要传递给 component 的参数
    //     let observers = identity.observers.clone();
    //     // 获取 component
    //     let component = &mut identity.network_behaviours[component_index as usize];
    //     // 设置 component 的参数
    //     component.set_observers(observers);
    //     // 返回 component
    //     component
    // }
    // pub fn late_invoke(identity: &mut NetworkIdentity, component_index: u8) {
    //     // 获取 component
    //     let component = &identity.network_behaviours[component_index as usize];
    //     identity.set_game_object(component.game_object().clone());
    // }
    pub fn error_correction(size: usize, safety: u8) -> usize {
        let cleared = size & 0xFFFFFF00;
        cleared | safety as usize
    }
    pub fn sync_var_equal<T>(a: &T, b: &T) -> bool
    where
        T: PartialEq,
    {
        a == b
    }
}

pub trait NetworkBehaviourTrait: Any + Send + Sync + Debug {
    fn new(
        game_object: GameObject,
        network_behaviour_component: &NetworkBehaviourComponent,
    ) -> Self
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
    fn sub_class(&self) -> String;
    fn set_sub_class(&mut self, value: String);
    fn sync_var_dirty_bits(&self) -> u64;
    // SetSyncVarDirtyBit
    fn set_sync_var_dirty_bits(&mut self, dirty_bit: u64) {
        self.__set_sync_var_dirty_bits(self.sync_var_dirty_bits() | dirty_bit);
    }
    fn __set_sync_var_dirty_bits(&mut self, value: u64);
    fn sync_object_dirty_bits(&self) -> u64;
    fn set_sync_object_dirty_bits(&mut self, value: u64) {
        self.__set_sync_object_dirty_bits(self.sync_object_dirty_bits() | value);
    }
    fn __set_sync_object_dirty_bits(&mut self, value: u64);
    fn net_id(&self) -> u32;
    fn set_net_id(&mut self, value: u32);
    fn connection_to_client(&self) -> u64;
    fn set_connection_to_client(&mut self, value: u64);
    fn observers(&self) -> &Vec<u64>;
    fn add_observer(&mut self, conn_id: u64);
    fn remove_observer(&mut self, value: u64);
    fn game_object(&self) -> &GameObject;
    fn set_game_object(&mut self, value: GameObject);
    fn sync_objects(&mut self) -> &mut Vec<Box<dyn SyncObject>>;
    fn set_sync_objects(&mut self, value: Vec<Box<dyn SyncObject>>);
    fn add_sync_object(&mut self, value: Box<dyn SyncObject>);
    fn has_sync_objects(&mut self) -> bool {
        self.sync_objects().len() > 0
    }
    fn sync_var_hook_guard(&self) -> u64;
    fn get_sync_var_hook_guard(&self, dirty_bit: u64) -> bool {
        (dirty_bit & self.sync_var_hook_guard()) != 0
    }
    // SetSyncVarHookGuard(ulong dirtyBit, bool value)
    fn set_sync_var_hook_guard(&mut self, dirty_bit: u64, value: bool) {
        if value {
            self.__set_sync_var_hook_guard(self.sync_var_hook_guard() | dirty_bit);
        } else {
            self.__set_sync_var_hook_guard(self.sync_var_hook_guard() & !dirty_bit);
        }
    }
    fn __set_sync_var_hook_guard(&mut self, value: u64);
    fn set_sync_var_with_guard(&mut self, dirty_bit: u64) {
        self.set_sync_var_dirty_bits(dirty_bit);
        if self.get_sync_var_hook_guard(dirty_bit) {
            return;
        }
        self.set_sync_var_hook_guard(dirty_bit, true);
        self.set_sync_var_hook_guard(dirty_bit, false);
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
        self.deserialize_sync_objects(reader, initial_state)
            && self.deserialize_sync_vars(reader, initial_state)
    }
    // Deserialize
    fn deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool {
        let mut result: bool;

        let safety = reader.read_byte();
        let chunk_start = reader.get_position();

        result = self.on_deserialize(reader, initial_state);

        let size = reader.get_position() - chunk_start;
        let size_hash = size as u8 & 0xFF;
        if size_hash != safety {
            log_warn!(format!(
                "Deserialize failed. Size mismatch. Expected: {}, Received: {}",
                size_hash, safety
            ));
            let corrected_size = NetworkBehaviour::error_correction(size, safety);
            reader.set_position(chunk_start + corrected_size);
            result = false;
        }
        result
    }
    // DeserializeSyncObjects
    fn deserialize_sync_objects(
        &mut self,
        reader: &mut NetworkReader,
        initial_state: bool,
    ) -> bool {
        if initial_state {
            self.deserialize_objects_all(reader)
        } else {
            self.deserialize_sync_object_delta(reader)
        }
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
        self.set_sync_var_dirty_bits(u64::MAX);
    }
    fn clear_all_dirty_bits(&mut self) {
        self.set_last_sync_time(NetworkTime::local_time());
        self.__set_sync_var_dirty_bits(0);
        self.__set_sync_object_dirty_bits(0);
        for sync_object in self.sync_objects().iter_mut() {
            sync_object.clear_changes();
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn send_rpc_internal(
        &self,
        function_full_name: &str,
        function_hash_code: i32,
        writer: &NetworkWriter,
        channel: TransportChannel,
        include_owner: bool,
    ) {
        if !NetworkServerStatic::active() {
            log_error!(format!(
                "RPC Function {} called without an active server.",
                function_full_name
            ));
            return;
        }
        let mut rpc = RpcMessage::new(
            self.net_id(),
            self.index(),
            function_hash_code as u16,
            writer.to_bytes(),
        );
        self.observers().iter().for_each(
            |observer| match NetworkServerStatic::network_connections().try_get_mut(observer) {
                TryResult::Present(mut conn_to_client) => {
                    let is_owner = conn_to_client.connection_id() == self.connection_to_client();
                    if (!is_owner || include_owner) && conn_to_client.is_ready() {
                        conn_to_client.send_network_message(&mut rpc, channel);
                    }
                }
                TryResult::Absent => {
                    log_error!(format!("Failed because connection {} is absent.", observer));
                }
                TryResult::Locked => {
                    log_error!(format!("Failed because connection {} is locked.", observer));
                }
            },
        );
    }
    fn send_entity_internal(
        &self,
        writer: &NetworkWriter,
        channel: TransportChannel,
        include_owner: bool,
    ) {
        if !NetworkServerStatic::active() {
            log_error!("EntityStateMessage called without an active server.");
            return;
        }
        let mut entity_message = EntityStateMessage::new(self.net_id(), writer.to_bytes());
        for observer in self.observers().iter() {
            match NetworkServerStatic::network_connections().try_get_mut(observer) {
                TryResult::Present(mut conn_to_client) => {
                    let is_owner = conn_to_client.connection_id() == self.connection_to_client();
                    if (!is_owner || include_owner) && conn_to_client.is_ready() {
                        conn_to_client.send_network_message(&mut entity_message, channel);
                    }
                }
                TryResult::Absent => {
                    log_error!(format!("Failed because connection {} is absent.", observer));
                }
                TryResult::Locked => {
                    log_error!(format!("Failed because connection {} is locked.", observer));
                }
            }
        }
    }
    fn on_start_server(&mut self) {}
    fn on_stop_server(&mut self) {}
    fn start(&mut self) {}
    fn update(&mut self) {
        self.start()
    }
    fn late_update(&mut self) {}
    // SerializeSyncVars
    fn serialize_sync_vars(&mut self, writer: &mut NetworkWriter, initial_state: bool);
    // DeserializeSyncVars
    fn deserialize_sync_vars(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool;
}
