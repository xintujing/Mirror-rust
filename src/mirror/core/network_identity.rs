use crate::log_error;
use crate::mirror::core::backend_data::BackendDataStatic;
use crate::mirror::core::network_behaviour::{
    GameObject, NetworkBehaviourFactory, NetworkBehaviourTrait, SyncDirection, SyncMode,
};
use crate::mirror::core::network_connection::NetworkConnectionTrait;
use crate::mirror::core::network_connection_to_client::NetworkConnectionToClient;
use crate::mirror::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::mirror::core::network_reader_pool::NetworkReaderPool;
use crate::mirror::core::network_server::{NetworkServer, NetworkServerStatic};
use crate::mirror::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::mirror::core::network_writer_pool::NetworkWriterPool;
use crate::mirror::core::remote_calls::{RemoteCallType, RemoteProcedureCalls};
use atomic::Atomic;
use dashmap::mapref::one::RefMut;
use dashmap::try_result::TryResult;
use dashmap::DashMap;
use lazy_static::lazy_static;
use std::default::Default;
use std::sync::atomic::Ordering;

lazy_static! {
    static ref NEXT_NETWORK_ID: Atomic<u32> = Atomic::new(1);
}

#[derive(Debug, PartialEq, Eq)]
pub enum Visibility {
    Default,
    ForceHidden,
    ForceShown,
}

#[derive(Debug)]
pub enum OwnedType {
    Client,
    Server,
}

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
    pub valid_parent: bool,
}

impl NetworkIdentity {
    pub fn new_with_asset_id(asset_id: u32) -> Self {
        let mut network_identity = Self::new();
        network_identity.asset_id = asset_id;
        network_identity.awake();
        network_identity
    }
    pub fn new_with_scene_id(scene_id: u64) -> Self {
        let mut network_identity = Self::new();
        network_identity.scene_id = scene_id;
        if let Some(temp) =
            BackendDataStatic::get_backend_data().get_network_identity_data_by_scene_id(scene_id)
        {
            network_identity.valid_parent = temp.valid_parent;
        }
        network_identity.awake();
        network_identity
    }
    fn new() -> Self {
        Self {
            scene_id: 0,
            asset_id: 0,
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
            valid_parent: false,
        }
    }
    pub fn net_id(&self) -> u32 {
        self.net_id
    }
    pub fn set_net_id(&mut self, net_id: u32) {
        // 设置 net_id
        self.net_id = net_id;
        // 设置所有的 component 的 net_id
        for component in self.network_behaviours.iter_mut() {
            component.set_net_id(self.net_id);
        }
        // 如果 conn_to_client 不为0，设置 connection_to_client 的 net_id
        if self.conn_to_client == 0 {
            return;
        }
        // 设置 connection_to_client 的 net_id
        match NetworkServerStatic::network_connections().try_get_mut(&self.conn_to_client) {
            TryResult::Present(mut conn) => {
                conn.set_net_id(self.net_id);
            }
            TryResult::Absent => {
                log_error!(
                    "Failed to set net id on connection to client because connection is absent."
                );
            }
            TryResult::Locked => {
                log_error!(
                    "Failed to set net id on connection to client because connection is locked."
                );
            }
        }
    }
    pub fn is_null(&self) -> bool {
        self.net_id == 0
            && self.asset_id == 0
            && self.game_object.is_null()
            && self.network_behaviours.len() == 0
            && self.scene_id == 0
    }
    pub fn connection_to_client(&self) -> u64 {
        self.conn_to_client
    }
    pub fn set_connection_to_client(&mut self, conn_id: u64) {
        // 设置 conn_id
        self.conn_to_client = conn_id;
        // 设置所有的component的conn_id
        for component in self.network_behaviours.iter_mut() {
            component.set_connection_to_client(self.conn_to_client);
        }
        // 如果 conn_to_client 不为0，设置 connection_to_client 的 net_id
        if self.conn_to_client == 0 {
            return;
        }
        // 添加到conn的owned_objects
        match NetworkServerStatic::network_connections().try_get_mut(&self.conn_to_client) {
            TryResult::Present(mut conn) => {
                conn.add_owned_object(self.net_id);
            }
            TryResult::Absent => {
                log_error!("Failed to set connection to client because connection is absent.");
            }
            TryResult::Locked => {
                log_error!("Failed to set connection to client because connection is locked.");
            }
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

    pub fn handle_remote_call(
        &mut self,
        component_index: u8,
        function_hash: u16,
        remote_call_type: RemoteCallType,
        reader: &mut NetworkReader,
        conn_id: u64,
    ) {
        if component_index as usize >= self.network_behaviours.len() {
            log_error!("Component index out of bounds: ", component_index);
            return;
        }

        // 调用 invoke
        if !RemoteProcedureCalls::invoke(
            function_hash,
            remote_call_type,
            self,
            component_index,
            reader,
            conn_id,
        ) {
            log_error!(
                "Failed to invoke remote call for function hash: ",
                function_hash
            );
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
        if self.asset_id != 0 {
            for component in BackendDataStatic::get_backend_data()
                .get_network_identity_data_network_behaviour_components_by_asset_id(self.asset_id)
            {
                if let Some(network_behaviour) = NetworkBehaviourFactory::create_network_behaviour(
                    self.game_object.clone(),
                    &component,
                ) {
                    self.network_behaviours.push(network_behaviour);
                }
            }
        }
        if self.scene_id != 0 {
            for component in BackendDataStatic::get_backend_data()
                .get_network_identity_data_network_behaviour_components_by_scene_id(self.scene_id)
            {
                if let Some(network_behaviour) = NetworkBehaviourFactory::create_network_behaviour(
                    self.game_object.clone(),
                    &component,
                ) {
                    self.network_behaviours.push(network_behaviour);
                }
            }
        }
        self.validate_components();
    }
    pub fn awake(&mut self) {
        self.initialize_network_behaviours();
        if self.has_spawned {
            log_error!("NetworkIdentity has already spawned.");
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
            NetworkServer::destroy(&mut NetworkConnectionToClient::default(), self);
        }
    }
    pub fn validate_components(&self) {
        if self.network_behaviours.len() > 64 {
            log_error!("NetworkIdentity has too many components. Max is 64.");
        }
    }
    pub fn on_start_server(&mut self) {
        self.network_behaviours
            .iter_mut()
            .for_each(|component| component.on_start_server());
    }
    pub fn on_stop_server(&mut self) {
        self.network_behaviours
            .iter_mut()
            .for_each(|component| component.on_stop_server());
    }
    fn server_dirty_masks(&mut self, initial_state: bool) -> (u64, u64) {
        let mut owner_mask: u64 = 0;
        let mut observers_mask: u64 = 0;
        for (i, component) in self.network_behaviours.iter_mut().enumerate() {
            let nth_bit = 1 << i;
            let dirty = component.is_dirty();

            if initial_state
                || (*component.sync_direction() == SyncDirection::ServerToClient) && dirty
            {
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
    pub fn serialize_server(
        &mut self,
        initial_state: bool,
        owner_writer: &mut NetworkWriter,
        observers_writer: &mut NetworkWriter,
    ) {
        self.validate_components();
        let (owner_mask, observers_mask) = self.server_dirty_masks(initial_state);

        if owner_mask != 0 {
            owner_writer.compress_var_ulong(owner_mask);
        }
        if observers_mask != 0 {
            observers_writer.compress_var_ulong(observers_mask);
        }

        if (owner_mask | observers_mask) != 0 {
            for (i, component) in self.network_behaviours.iter_mut().enumerate() {
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

        let mask = reader.decompress_var_ulong();

        for (i, component) in self.network_behaviours.iter_mut().enumerate() {
            if Self::is_dirty(mask, i as u8) {
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
    pub fn get_server_serialization_at_tick(
        &mut self,
        tick: u32,
    ) -> &mut NetworkIdentitySerialization {
        if self.last_serialization.tick != tick {
            self.last_serialization.reset_writers();
            NetworkWriterPool::get_return(|owner_writer| {
                NetworkWriterPool::get_return(|observers_writer| {
                    self.serialize_server(false, owner_writer, observers_writer);
                    self.last_serialization
                        .owner_writer
                        .write_array_segment_all(owner_writer.to_array_segment());
                    self.last_serialization
                        .observers_writer
                        .write_array_segment_all(observers_writer.to_array_segment());
                });
            });
            self.last_serialization.tick = tick;
        }
        &mut self.last_serialization
    }
    pub fn clear_observers(&mut self) {
        for conn_id in self.observers.to_vec().iter() {
            match NetworkServerStatic::network_connections().try_get_mut(conn_id) {
                TryResult::Present(mut conn) => {
                    conn.remove_from_observing(self, true);
                }
                TryResult::Absent => {
                    log_error!(format!(
                        "Failed to clear observers because connection {} is absent.",
                        conn_id
                    ));
                }
                TryResult::Locked => {
                    log_error!(format!(
                        "Failed to clear observers because connection {} is locked.",
                        conn_id
                    ));
                }
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

        // 添加到观察者
        match NetworkServerStatic::network_connections().try_get_mut(&conn_id) {
            TryResult::Present(mut conn) => {
                conn.add_to_observing(self);
            }
            TryResult::Absent => {
                log_error!("Failed to add observer because connection is absent.");
            }
            TryResult::Locked => {
                log_error!("Failed to add observer because connection is locked.");
            }
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

    pub fn set_active(&mut self, active: bool) {
        self.game_object.set_active(active);
    }

    pub fn get_component<T>(&mut self) -> Option<&T>
    where
        T: NetworkBehaviourTrait,
    {
        for component in self.network_behaviours.iter_mut() {
            if let Some(component) = component.as_any_mut().downcast_ref::<T>() {
                return Some(component);
            }
        }
        None
    }
}
