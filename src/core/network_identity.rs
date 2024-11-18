use crate::core::backend_data::BackendDataStatic;
use crate::core::network_behaviour::{NetworkBehaviourFactory, NetworkBehaviourTrait, SyncDirection, SyncMode};
use crate::core::network_connection::NetworkConnectionTrait;
use crate::core::network_manager::{GameObject, Transform};
use crate::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::core::network_reader_pool::NetworkReaderPool;
use crate::core::network_server::NetworkServerStatic;
use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::core::network_writer_pool::NetworkWriterPool;
use crate::core::remote_calls::{RemoteCallType, RemoteProcedureCalls};
use atomic::Atomic;
use dashmap::mapref::one::RefMut;
use dashmap::DashMap;
use lazy_static::lazy_static;
use std::default::Default;
use std::sync::atomic::Ordering;
use tklog::error;

lazy_static! {
    static ref NEXT_NETWORK_ID: Atomic<u32> = Atomic::new(1);
}

#[derive(Debug, PartialEq, Eq)]
pub enum Visibility { Default, ForceHidden, ForceShown }

#[derive(Debug)]
pub enum OwnedType { Client, Server }

#[derive(Debug)]
pub struct NetworkIdentitySerialization {
    pub tick: u32,
    pub owner_writer: NetworkWriter,
    pub observers_writer: NetworkWriter,
}

impl NetworkIdentitySerialization {
    pub fn new(tick: u32) -> Self {
        NetworkIdentitySerialization {
            tick,
            owner_writer: NetworkWriter::new(),
            observers_writer: NetworkWriter::new(),
        }
    }
    pub fn reset_writers(&mut self) {
        self.owner_writer.reset();
        self.observers_writer.reset();
    }
}

#[derive(Debug)]
pub struct NetworkIdentity {
    conn_to_client: u64,
    net_id: u32,
    had_authority: bool,
    game_object: GameObject,
    pub observers: Vec<u64>,
    pub scene_id: u64,
    pub asset_id: u32,
    pub server_only: bool,
    pub owned_type: OwnedType,
    pub is_owned: bool,
    pub is_init: bool,
    pub destroy_called: bool,
    pub visibility: Visibility,
    pub last_serialization: NetworkIdentitySerialization,
    pub scene_ids: DashMap<u64, u32>,
    pub has_spawned: bool,
    pub spawned_from_instantiate: bool,
    pub network_behaviours: Vec<Box<dyn NetworkBehaviourTrait>>,
}

impl NetworkIdentity {
    pub fn new(asset_id: u32) -> Self {
        let mut network_identity = NetworkIdentity {
            scene_id: 0,
            asset_id,
            net_id: 0,
            had_authority: false,
            game_object: GameObject::default(),
            server_only: false,
            owned_type: OwnedType::Client,
            is_owned: false,
            observers: Default::default(),
            conn_to_client: 0,
            is_init: false,
            destroy_called: false,
            visibility: Visibility::Default,
            last_serialization: NetworkIdentitySerialization::new(0),
            scene_ids: Default::default(),
            has_spawned: false,
            spawned_from_instantiate: false,
            network_behaviours: Default::default(),
        };
        network_identity.awake();
        network_identity
    }
    pub fn net_id(&self) -> u32 {
        self.net_id
    }
    pub fn set_net_id(&mut self, net_id: u32) {
        self.net_id = net_id;
        for component in self.network_behaviours.iter_mut() {
            component.set_net_id(self.net_id);
        }
        if let Some(mut conn) = NetworkServerStatic::get_static_network_connections().get_mut(&self.conn_to_client) {
            conn.set_net_id(self.net_id);
        }
    }
    pub fn is_null(&self) -> bool {
        self.net_id == 0 && self.asset_id == 0 && self.game_object.is_null() && self.network_behaviours.len() == 0 && self.scene_id == 0
    }
    pub fn connection_to_client(&self) -> u64 {
        self.conn_to_client
    }
    pub fn set_connection_to_client(&mut self, conn_id: u64) {
        self.conn_to_client = conn_id;
        for component in self.network_behaviours.iter_mut() {
            component.set_connection_to_client(self.conn_to_client);
        }
        if let Some(mut conn) = NetworkServerStatic::get_static_network_connections().get_mut(&self.conn_to_client) {
            conn.owned().push(self.net_id);
        }
    }
    pub fn game_object(&self) -> &GameObject {
        &self.game_object
    }
    pub fn set_game_object(&mut self, game_object: GameObject) {
        self.game_object = game_object;
        for component in self.network_behaviours.iter_mut() {
            component.set_game_object(self.game_object.clone());
        }
    }
    pub fn set_transform(&mut self, transform: Transform) {
        self.game_object.transform = transform;
        for component in self.network_behaviours.iter_mut() {
            component.set_game_object(self.game_object.clone());
        }
    }
    pub fn handle_remote_call(&mut self, component_index: u8, function_hash: u16, remote_call_type: RemoteCallType, reader: &mut NetworkReader, conn_id: u64) {
        if component_index as usize >= self.network_behaviours.len() {
            error!("Component index out of bounds: ", component_index);
            return;
        }

        // 调用 invoke
        if !RemoteProcedureCalls::invoke(function_hash, remote_call_type, self, component_index, reader, conn_id) {
            error!("Failed to invoke remote call for function hash: ", function_hash);
        }
    }
    pub fn reset_statics() {
        Self::reset_server_statics();
    }
    pub fn reset_server_statics() {
        Self::set_static_next_network_id(1);
    }
    pub fn get_scene_identity(&self, scene_id: u64) -> Option<RefMut<u64, u32>> {
        if let Some(scene_identity) = self.scene_ids.get_mut(&scene_id) {
            return Some(scene_identity);
        }
        None
    }
    pub fn initialize_network_behaviours(&mut self) {
        for component in BackendDataStatic::get_backend_data().get_network_identity_data_network_behaviour_components_by_asset_id(self.asset_id) {
            if let Some(network_behaviour) = NetworkBehaviourFactory::create_network_behaviour(component.sub_class.as_str(), self.game_object.clone(), component) {
                self.network_behaviours.push(network_behaviour);
            }
        }
        self.validate_components();
    }
    pub fn awake(&mut self)
    {
        self.initialize_network_behaviours();
        if self.has_spawned {
            error!("NetworkIdentity has already spawned.");
            self.spawned_from_instantiate = true;
        }
        self.has_spawned = true;
    }
    pub fn on_validate(&mut self) {
        self.has_spawned = false;
    }
    pub fn on_destroy(&mut self) {
        if self.spawned_from_instantiate {
            return;
        }

        if !self.destroy_called {
            // TODO NetworkServer.Destroy(game_object);
        }
    }
    pub fn validate_components(&self) {
        if self.network_behaviours.len() == 0 {
            error!("NetworkIdentity has no components.");
        } else if self.network_behaviours.len() > 64 {
            error!("NetworkIdentity has too many components. Max is 64.");
        }
    }
    pub fn on_start_server(&mut self) {
        self.network_behaviours.iter_mut().for_each(|component| {
            component.on_start_server()
        });
    }
    pub fn on_stop_server(&mut self) {
        self.network_behaviours.iter_mut().for_each(|component| {
            component.on_stop_server()
        });
    }
    fn server_dirty_masks(&mut self, initial_state: bool) -> (u64, u64) {
        let mut owner_mask: u64 = 0;
        let mut observers_mask: u64 = 0;
        for i in 0..self.network_behaviours.len() {
            let component = &mut self.network_behaviours[i];
            let nth_bit = 1 << i;
            let dirty = component.is_dirty();

            if initial_state || (*component.sync_direction() == SyncDirection::ServerToClient) && dirty {
                owner_mask |= nth_bit;
            }

            if *component.sync_mode() == SyncMode::Observers {
                if initial_state || dirty {
                    observers_mask |= nth_bit;
                }
            }
        }
        (owner_mask, observers_mask)
    }
    fn is_dirty(mask: u64, index: u8) -> bool {
        (mask & (1 << index)) != 0
    }
    pub fn serialize_server(&mut self, initial_state: bool, owner_writer: &mut NetworkWriter, observers_writer: &mut NetworkWriter) {
        self.validate_components();
        let (owner_mask, observers_mask) = self.server_dirty_masks(initial_state);

        if owner_mask != 0 {
            owner_writer.compress_var_uint(owner_mask);
        }
        if observers_mask != 0 {
            observers_writer.compress_var_uint(observers_mask);
        }

        if (owner_mask | observers_mask) != 0 {
            for i in 0..self.network_behaviours.len() {
                let component = &mut self.network_behaviours[i];

                let owner_dirty = Self::is_dirty(owner_mask, i as u8);
                let observers_dirty = Self::is_dirty(observers_mask, i as u8);

                if owner_dirty || observers_dirty {
                    NetworkWriterPool::get_return(|temp| {
                        // Serialize the component
                        component.serialize(temp, initial_state);

                        let segment = temp.to_bytes();

                        if owner_dirty {
                            owner_writer.write_array_segment_all(&segment);
                        }
                        if observers_dirty {
                            observers_writer.write_array_segment_all(&segment);
                        }
                    });
                    if !initial_state {
                        component.clear_all_dirty_bits();
                    }
                }
            }
        }
    }
    pub fn deserialize_server(&mut self, reader: &mut NetworkReader) -> bool {
        self.validate_components();

        let mask = reader.decompress_var_uint();

        for i in 0..self.network_behaviours.len() {
            if Self::is_dirty(mask, i as u8) {
                let component = &mut self.network_behaviours[i];
                if *component.sync_direction() == SyncDirection::ClientToServer {
                    NetworkReaderPool::get_return(|reader| {
                        if !component.deserialize(reader, false) {
                            return;
                        }
                        component.set_dirty();
                    });
                }
            }
        }
        true
    }
    pub fn get_server_serialization_at_tick(&mut self, tick: u32) -> &mut NetworkIdentitySerialization {
        if self.last_serialization.tick != tick {
            self.last_serialization.reset_writers();
            NetworkWriterPool::get_return(|owner_writer| {
                NetworkWriterPool::get_return(|observers_writer| {
                    self.serialize_server(false, owner_writer, observers_writer);
                    self.last_serialization.owner_writer.write_array_segment_all(owner_writer.to_array_segment());
                    self.last_serialization.observers_writer.write_array_segment_all(observers_writer.to_array_segment());
                });
            });
            self.last_serialization.tick = tick;
        }
        &mut self.last_serialization
    }
    pub fn clear_observers(&mut self) {
        for i in 0..self.observers.len() {
            let conn_id = self.observers[i];
            if let Some(mut conn) = NetworkServerStatic::get_static_network_connections().get_mut(&conn_id) {
                conn.remove_from_observing(self, true);
            }
        }
        self.observers.clear();
    }

    pub fn reset_state(&mut self) {
        self.has_spawned = false;
        self.is_owned = false;
        self.notify_authority();

        self.net_id = 0;
        self.conn_to_client = 0;

        self.clear_observers();
    }

    pub fn notify_authority(&mut self) {
        // TODO NotifyAuthority();
        if !self.had_authority && self.is_owned {}
        if self.had_authority && !self.is_owned {}
    }

    // AddObserver(NetworkConnectionToClient conn)
    pub fn add_observer(&mut self, conn_id: u64) {
        // 如果观察者已存在
        if self.observers.contains(&conn_id) {
            return;
        }

        // 如果没有观察者
        if self.observers.len() == 0 {
            self.clear_all_components_dirty_bits()
        }
        // 添加观察者
        self.observers.push(conn_id);

        if let Some(mut conn) = NetworkServerStatic::get_static_network_connections().get_mut(&conn_id) {
            // 添加到观察者
            conn.add_to_observing(self);
        }
    }
    fn clear_all_components_dirty_bits(&mut self) {
        for component in self.network_behaviours.iter_mut() {
            component.clear_all_dirty_bits()
        }
    }
    pub fn remove_observer(&mut self, conn_id: u64) {
        self.observers.retain(|id| *id != conn_id);
    }
    pub fn set_client_owner(&mut self, conn_id: u64) {
        // do nothing if it already has an owner
        if self.conn_to_client != 0 {
            return;
        }
        self.conn_to_client = conn_id;
    }
    pub fn get_static_next_network_id() -> u32 {
        let id = NEXT_NETWORK_ID.load(Ordering::Relaxed);
        NEXT_NETWORK_ID.store(id + 1, Ordering::Relaxed);
        id
    }
    pub fn set_static_next_network_id(id: u32) {
        NEXT_NETWORK_ID.store(id, Ordering::Relaxed);
    }
}