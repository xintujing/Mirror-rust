use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, PartialEq, Eq)]
enum SyncMode {
    Observers,
    Owner,
}

#[derive(Debug, PartialEq, Eq)]
enum SyncDirection {
    ServerToClient,
    ClientToServer,
}

pub struct NetworkBehaviour {
    pub sync_direction: SyncDirection,
    pub sync_mode: SyncMode,
    pub sync_interval: f32,
    pub is_server: bool,
    pub is_client: bool,
    pub is_local_player: bool,
    pub is_server_only: bool,
    pub is_client_only: bool,
    pub is_owned: bool,
    pub authority: bool,
    pub net_id: u32,
    pub connection_to_server: Option<NetworkConnection>,
    pub connection_to_client: Option<NetworkConnectionToClient>,
    sync_objects: Vec<Box<dyn SyncObject>>,
    sync_var_dirty_bits: AtomicU64,
    sync_object_dirty_bits: AtomicU64,
    sync_var_hook_guard: AtomicU64,
}

impl NetworkBehaviour {
    pub fn new() -> Self {
        NetworkBehaviour {
            sync_direction: SyncDirection::ServerToClient,
            sync_mode: SyncMode::Observers,
            sync_interval: 0.0,
            is_server: false,
            is_client: false,
            is_local_player: false,
            is_server_only: false,
            is_client_only: false,
            is_owned: false,
            authority: false,
            net_id: 0,
            connection_to_server: None,
            connection_to_client: None,
            sync_objects: Vec::new(),
            sync_var_dirty_bits: AtomicU64::new(0),
            sync_object_dirty_bits: AtomicU64::new(0),
            sync_var_hook_guard: AtomicU64::new(0),
        }
    }

    pub fn set_sync_var_dirty_bit(&self, dirty_bit: u64) {
        self.sync_var_dirty_bits.fetch_or(dirty_bit, Ordering::SeqCst);
    }

    pub fn set_dirty(&self) {
        self.set_sync_var_dirty_bit(u64::MAX);
    }

    pub fn is_dirty(&self) -> bool {
        let dirty_bits = self.sync_var_dirty_bits.load(Ordering::Relaxed)
            | self.sync_object_dirty_bits.load(Ordering::Relaxed);
        dirty_bits != 0
            && NetworkTime::local_time() - self.last_sync_time >= self.sync_interval
    }

    pub fn clear_all_dirty_bits(&self) {
        self.last_sync_time = NetworkTime::local_time();
        self.sync_var_dirty_bits.store(0, Ordering::SeqCst);
        self.sync_object_dirty_bits.store(0, Ordering::SeqCst);

        for sync_object in &self.sync_objects {
            sync_object.clear_changes();
        }
    }

    pub fn init_sync_object(&mut self, sync_object: Box<dyn SyncObject>) {
        let index = self.sync_objects.len();
        self.sync_objects.push(sync_object);

        let nth_bit = 1u64 << index;
        let on_dirty = move || {
            self.sync_object_dirty_bits.fetch_or(nth_bit, Ordering::SeqCst);
        };
        self.sync_objects[index].on_dirty = on_dirty;

        let is_writable = || {
            if NetworkServer::active() && NetworkClient::active() {
                self.sync_direction == SyncDirection::ServerToClient || self.is_owned
            } else if NetworkServer::active() {
                self.sync_direction == SyncDirection::ServerToClient
            } else if NetworkClient::active() {
                self.sync_direction == SyncDirection::ClientToServer && self.is_owned
            } else {
                false
            }
        };
        self.sync_objects[index].is_writable = is_writable;

        let is_recording = || {
            if self.is_server && self.is_client {
                self.net_identity.observers.len() > 0
            } else if self.is_server {
                self.net_identity.observers.len() > 0
            } else if self.is_client {
                self.sync_direction == SyncDirection::ClientToServer && self.is_owned
            } else {
                false
            }
        };
        self.sync_objects[index].is_recording = is_recording;
    }

    pub fn send_command_internal(
        &self,
        function_full_name: &str,
        function_hash_code: i32,
        writer: &mut NetworkWriter,
        channel_id: i32,
        requires_authority: bool,
    ) {
        if !NetworkClient::active() {
            println!("Command {} called on {} without an active client.", function_full_name, self.name());
            return;
        }

        if !NetworkClient::ready() {
            if channel_id == Channels::Reliable {
                println!("Command {} called on {} while NetworkClient is not ready.", function_full_name, self.name());
            }
            return;
        }

        if !(
            !requires_authority
                || self.is_local_player
                || self.is_owned
        ) {
            println!("Command {} called on {} without authority.", function_full_name, self.name());
            return;
        }

        if NetworkClient::connection().is_none() {
            println!("Command {} called on {} with no client running.", function_full_name, self.name());
            return;
        }

        if self.net_id == 0 {
            println!("Command {} called on {} with netId=0. Maybe it wasn't spawned yet?", function_full_name, self.name());
            return;
        }

        let message = CommandMessage {
            net_id: self.net_id,
            component_index: self.component_index,
            function_hash: function_hash_code as u16,
            payload: writer.to_array_segment(),
        };

        NetworkClient::connection().unwrap().send(message, channel_id);
    }

    pub fn send_rpc_internal(
        &self,
        function_full_name: &str,
        function_hash_code: i32,
        writer: &mut NetworkWriter,
        channel_id: i32,
        include_owner: bool,
    ) {
        if !NetworkServer::active() {
            println!("RPC Function {} called without an active server.", function_full_name);
            return;
        }

        if !self.is_server {
            println!("ClientRpc {} called on un-spawned object: {}", function_full_name, self.name());
            return;
        }

        let message = RpcMessage {
            net_id: self.net_id,
            component_index: self.component_index,
            function_hash: function_hash_code as u16,
            payload: writer.to_array_segment(),
        };

        for conn in self.net_identity.observers.values() {
            let is_owner = conn == self.net_identity.connection_to_client;
            if (!is_owner || include_owner) && conn.is_ready() {
                conn.send(message, channel_id);
            }
        }
    }

    pub fn send_target_rpc_internal(
        &self,
        conn: Option<&NetworkConnectionToClient>,
        function_full_name: &str,
        function_hash_code: i32,
        writer: &mut NetworkWriter,
        channel_id: i32,
    ) {
        if !NetworkServer::active() {
            println!("TargetRPC {} was called on {} when server not active.", function_full_name, self.name());
            return;
        }

        if !self.is_server {
            println!("TargetRpc {} called on {} but that object has not been spawned or has been unspawned.", function_full_name, self.name());
            return;
        }

        let conn = conn.unwrap_or(self.net_identity.connection_to_client.as_ref());

        if conn.is_none() {
            println!("TargetRPC {} can't be sent because it was given a null connection.", function_full_name);
            return;
        }

        let conn = conn.unwrap();

        let message = RpcMessage {
            net_id: self.net_id,
            component_index: self.component_index,
            function_hash: function_hash_code as u16,
            payload: writer.to_array_segment(),
        };

        conn.send(message, channel_id);
    }

    pub fn generated_sync_var_setter<T>(
        &self,
        value: T,
        field: &mut T,
        dirty_bit: u64,
        on_changed: Option<Box<dyn FnMut(T, T)>>,
    ) where
        T: PartialEq + Copy,
    {
        if !self.sync_var_equal(value, field) {
            let old_value = *field;
            self.set_sync_var(value, field, dirty_bit);

            if let Some(on_changed) = on_changed {
                if NetworkServer::active_host() && !self.get_sync_var_hook_guard(dirty_bit) {
                    self.set_sync_var_hook_guard(dirty_bit, true);
                    on_changed(old_value, value);
                    self.set_sync_var_hook_guard(dirty_bit, false);
                }
            }
        }
    }

    pub fn generated_sync_var_setter_game_object(
        &self,
        value: Option<&GameObject>,
        field: &mut Option<GameObject>,
        dirty_bit: u64,
        on_changed: Option<Box<dyn FnMut(Option<GameObject>, Option<GameObject>)>>,
        net_id_field: &mut u32,
    ) {
        if !self.sync_var_game_object_equal(value, *net_id_field) {
            let old_value = field.clone();
            self.set_sync_var_game_object(value, field, dirty_bit, net_id_field);

            if let Some(on_changed) = on_changed {
                if NetworkServer::active_host() && !self.get_sync_var_hook_guard(dirty_bit) {
                    self.set_sync_var_hook_guard(dirty_bit, true);
                    on_changed(old_value, field.clone());
                    self.set_sync_var_hook_guard(dirty_bit, false);
                }
            }
        }
    }

    pub fn generated_sync_var_setter_network_identity(
        &self,
        value: Option<&NetworkIdentity>,
        field: &mut Option<NetworkIdentity>,
        dirty_bit: u64,
        on_changed: Option<Box<dyn FnMut(Option<NetworkIdentity>, Option<NetworkIdentity>)>>,
        net_id_field: &mut u32,
    ) {
        if !self.sync_var_network_identity_equal(value, *net_id_field) {
            let old_value = field.clone();
            self.set_sync_var_network_identity(value, field, dirty_bit, net_id_field);

            if let Some(on_changed) = on_changed {
                if NetworkServer::active_host() && !self.get_sync_var_hook_guard(dirty_bit) {
                    self.set_sync_var_hook_guard(dirty_bit, true);
                    on_changed(old_value, field.clone());
                    self.set_sync_var_hook_guard(dirty_bit, false);
                }
            }
        }
    }

    pub fn generated_sync_var_deserialize<T>(&self, field: &mut T, on_changed: Option<Box<dyn FnMut(T, T)>>, value: T)
    where
        T: PartialEq + Copy,
    {
        let previous = *field;
        *field = value;

        if let Some(on_changed) = on_changed && !self.sync_var_equal(previous, field) {
            on_changed(previous, *field);
        }
    }

    pub fn generated_sync_var_deserialize_game_object(
        &self,
        field: &mut Option<GameObject>,
        on_changed: Option<Box<dyn FnMut(Option<GameObject>, Option<GameObject>)>>,
        reader: &mut NetworkReader,
        net_id_field: &mut u32,
    ) {
        let previous_net_id = *net_id_field;
        *net_id_field = reader.read_u32();

        *field = self.get_sync_var_game_object(*net_id_field, field);

        if let Some(on_changed) = on_changed && !self.sync_var_equal(previous_net_id, net_id_field) {
            on_changed(self.get_sync_var_game_object(previous_net_id, field), *field);
        }
    }

    pub fn generated_sync_var_deserialize_network_identity(
        &self,
        field: &mut Option<NetworkIdentity>,
        on_changed: Option<Box<dyn FnMut(Option<NetworkIdentity>, Option<NetworkIdentity>)>>,
        reader: &mut NetworkReader,
        net_id_field: &mut u32,
    ) {
        let previous_net_id = *net_id_field;
        *net_id_field = reader.read_u32();

        *field = self.get_sync_var_network_identity(*net_id_field, field);

        if let Some(on_changed) = on_changed && !self.sync_var_equal(previous_net_id, net_id_field) {
            on_changed(self.get_sync_var_network_identity(previous_net_id, field), *field);
        }
    }

    pub fn generated_sync_var_deserialize_network_behaviour<T>(
        &self,
        field: &mut Option<T>,
        on_changed: Option<Box<dyn FnMut(Option<T>, Option<T>)>>,
        reader: &mut NetworkReader,
        net_id_field: &mut NetworkBehaviourSyncVar,
    ) where
        T: NetworkBehaviour,
    {
        let previous_net_id = *net_id_field;
        *net_id_field = reader.read_network_behaviour_sync_var();

        *field = self.get_sync_var_network_behaviour(*net_id_field, field);

        if let Some(on_changed) = on_changed && !self.sync_var_equal(previous_net_id, net_id_field) {
            on_changed(self.get_sync_var_network_behaviour(previous_net_id, field), *field);
        }
    }

    pub fn sync_var_equal<T>(
        &self,
        value: T,
        field: &T,
    ) -> bool
    where
        T: PartialEq + Copy,
    {
        value == *field
    }

    pub fn set_sync_var<T>(
        &self,
        value: T,
        field: &mut T,
        dirty_bit: u64,
    ) {
        self.set_sync_var_dirty_bit(dirty_bit);
        *field = value;
    }

    pub fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        self.serialize_sync_objects(writer, initial_state);
        self.serialize_sync_vars(writer, initial_state);
    }

    pub fn on_deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) {
        self.deserialize_sync_objects(reader, initial_state);
        self.deserialize_sync_vars(reader, initial_state);
    }
}