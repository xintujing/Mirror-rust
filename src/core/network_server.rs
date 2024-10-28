use crate::core::batcher::{Batch, NetworkMessageReader, NetworkMessageWriter, UnBatch};
use crate::core::messages::{CommandMessage, EntityStateMessage, NetworkMessageHandler, NetworkMessageHandlerFunc, NetworkPingMessage, NetworkPongMessage, ObjectSpawnFinishedMessage, ObjectSpawnStartedMessage, ReadyMessage, SpawnMessage, TimeSnapshotMessage};
use crate::core::network_connection::NetworkConnection;
use crate::core::network_identity::Visibility::Default;
use crate::core::network_identity::{NetworkIdentity, Visibility};
use crate::core::network_messages::NetworkMessages;
use crate::core::network_time::NetworkTime;
use crate::core::network_writer::NetworkWriter;
use crate::core::network_writer_pool::NetworkWriterPool;
use crate::core::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::core::tools::time_sample::TimeSample;
use crate::core::transport::{Transport, TransportCallback, TransportCallbackType, TransportChannel, TransportError};
use atomic::Atomic;
use bytes::Bytes;
use dashmap::mapref::multiple::RefMutMulti;
use dashmap::DashMap;
use lazy_static::lazy_static;
use nalgebra::{Quaternion, Vector3};
use std::sync::atomic::Ordering;
use std::sync::RwLock;
use tklog::{debug, error, warn};

lazy_static! {
    static ref INITIALIZED: Atomic<bool> = Atomic::new(false);
    static ref TICK_RATE: Atomic<u32> = Atomic::new(60);
    static ref TICK_INTERVAL: Atomic<f32> = Atomic::new(1f32 / NetworkServer::get_static_tick_rate() as f32);
    static ref SEND_RATE: Atomic<u32> = Atomic::new(NetworkServer::get_static_tick_rate());
    static ref SEND_INTERVAL: Atomic<f32> = Atomic::new(1f32 / NetworkServer::get_static_send_rate() as f32);
    static ref LAST_SEND_TIME: Atomic<f64> = Atomic::new(0.0);
    static ref DONT_LISTEN: Atomic<bool> = Atomic::new(true);
    static ref ACTIVE: Atomic<bool> = Atomic::new(false);
    static ref IS_LOADING_SCENE: Atomic<bool> = Atomic::new(false);
    static ref EXCEPTIONS_DISCONNECT: Atomic<bool> = Atomic::new(false);
    static ref DISCONNECT_INACTIVE_CONNECTIONS: Atomic<bool> = Atomic::new(false);
    static ref DISCONNECT_INACTIVE_TIMEOUT: Atomic<f32> = Atomic::new(60.0);
    static ref ACTUAL_TICK_RATE: Atomic<u32> = Atomic::new(0);
    static ref ACTUAL_TICK_RATE_START: Atomic<f64> = Atomic::new(0.0);
    static ref ACTUAL_TICK_RATE_COUNTER: Atomic<u32> = Atomic::new(0);
    static ref MAX_CONNECTIONS: Atomic<usize> = Atomic::new(0);
    static ref EARLY_UPDATE_DURATION: RwLock<TimeSample> = RwLock::new(TimeSample::new(0));
    static ref LATE_UPDATE_DURATION: RwLock<TimeSample> = RwLock::new(TimeSample::new(0));
    static ref FULL_UPDATE_DURATION: RwLock<TimeSample> = RwLock::new(TimeSample::new(0));

    static ref NETWORK_CONNECTIONS: DashMap<u64, NetworkConnection> = DashMap::new();
    static ref SPAWNED: DashMap<u64, NetworkIdentity> = DashMap::new();
    static ref NETWORK_MESSAGE_HANDLERS: DashMap<u16, NetworkMessageHandler>=DashMap::new();
}


pub enum RemovePlayerOptions {
    /// <summary>Player Object remains active on server and clients. Only ownership is removed</summary>
    KeepActive,
    /// <summary>Player Object is unspawned on clients but remains on server</summary>
    UnSpawn,
    /// <summary>Player Object is destroyed on server and clients</summary>
    Destroy,
}

pub struct NetworkServer;

impl NetworkServer {
    fn initialize() {
        if Self::get_static_initialized() {
            return;
        }

        //Make sure connections are cleared in case any old connections references exist from previous sessions
        NETWORK_CONNECTIONS.clear();

        // TODO: if (aoi != null) aoi.ResetState();

        NetworkTime::reset_statics();


        if let Some(mut transport) = Transport::get_active_transport() {
            transport.set_transport_cb_fn(Box::new(Self::transport_callback));
        }


        if Self::get_static_initialized() {
            return;
        }

        if let Ok(mut early_update_duration) = EARLY_UPDATE_DURATION.write() {
            *early_update_duration = TimeSample::new(Self::get_static_send_rate());
        }
        if let Ok(mut full_update_duration) = FULL_UPDATE_DURATION.write() {
            *full_update_duration = TimeSample::new(Self::get_static_send_rate());
        }
        if let Ok(mut late_update_duration) = LATE_UPDATE_DURATION.write() {
            *late_update_duration = TimeSample::new(Self::get_static_send_rate());
        }
    }
    pub fn listen(max_connections: usize) {
        Self::initialize();

        Self::set_static_max_connections(max_connections);

        if Self::get_static_dont_listen() {
            if let Some(mut transport) = Transport::get_active_transport() {
                transport.server_start();
            }
        }

        Self::set_static_active(true);

        Self::register_message_handlers();
    }

    // 网络早期更新
    pub fn network_early_update() {
        if Self::get_static_active() {
            if let Ok(mut early_update_duration) = EARLY_UPDATE_DURATION.write() {
                early_update_duration.begin();
            }
            if let Ok(mut full_update_duration) = FULL_UPDATE_DURATION.write() {
                full_update_duration.begin();
            }
        }
        if let Some(mut active_transport) = Transport::get_active_transport() {
            active_transport.server_early_update();
        }

        //  step each connection's local time interpolation in early update. 1969
        Self::for_each_network_connection(|mut connection| {
            connection.update_time_interpolation();
        });


        if Self::get_static_active() {
            if let Ok(mut early_update_duration) = EARLY_UPDATE_DURATION.write() {
                early_update_duration.end();
            }
        }
    }

    // 网络更新
    pub fn network_late_update() {
        if Self::get_static_active() {
            if let Ok(mut late_update_duration) = LATE_UPDATE_DURATION.write() {
                late_update_duration.begin();
            }
            Self::broadcast();
        }
        if let Some(mut active_transport) = Transport::get_active_transport() {
            active_transport.server_late_update();
        }

        if Self::get_static_active() {
            let actual_tick_rate_counter = Self::get_static_actual_tick_rate_counter();
            Self::set_static_actual_tick_rate_counter(actual_tick_rate_counter + 1);

            let local_time = NetworkTime::local_time();

            if local_time - Self::get_static_actual_tick_rate_start() >= 1.0 {
                let elapsed = local_time - Self::get_static_actual_tick_rate_start();
                let actual_tick_rate_counter = Self::get_static_actual_tick_rate_counter();
                Self::set_static_actual_tick_rate(actual_tick_rate_counter / elapsed as u32);
                Self::set_static_actual_tick_rate_start(local_time);
                Self::set_static_actual_tick_rate_counter(0);
            }

            if let Ok(mut late_update_duration) = LATE_UPDATE_DURATION.write() {
                late_update_duration.end();
            }
            if let Ok(mut full_update_duration) = FULL_UPDATE_DURATION.write() {
                full_update_duration.end();
            }
        }
    }

    // Broadcast
    fn broadcast() {
        let mut connection_copy = NETWORK_CONNECTIONS.clone();
        connection_copy.iter_mut().for_each(|mut connection| {
            // check for inactivity. disconnects if necessary.
            if Self::disconnect_if_inactive(&mut connection) {
                return;
            }

            if connection.is_ready {
                // send messages
                connection.send_network_message(TimeSnapshotMessage::new(), TransportChannel::Unreliable);

                // broadcast to connection
                Self::broadcast_to_connection(&mut connection);
            }
            connection.update();
        });
    }

    fn broadcast_to_connection(connection: &mut NetworkConnection) {
        connection.observing_identities.iter_mut().for_each(|mut identity| {
            identity.new_spawn_message_payload();
            // TODO SerializeForConnection
        });
    }

    fn disconnect_if_inactive(connection: &mut NetworkConnection) -> bool {
        if Self::get_static_disconnect_inactive_connections() &&
            !connection.is_alive(Self::get_static_disconnect_inactive_timeout() as f64) {
            warn!(format!("Server.DisconnectIfInactive: connectionId: {} is inactive. Disconnecting.", connection.connection_id));
            connection.disconnect();
            return true;
        }
        false
    }

    // show / hide for connection //////////////////////////////////////////
    pub fn show_for_connection(identity: &mut NetworkIdentity, connection_id: u64) {
        if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
            if connection.is_ready {
                Self::send_spawn_message(identity, &mut connection);
            }
        }
    }

    fn send_spawn_message(identity: &mut NetworkIdentity, connection: &mut NetworkConnection) {
        let is_local_player = identity.net_id == connection.identity.net_id;
        let is_owner = identity.connection_id_to_client == connection.connection_id;
        // position
        let position = Vector3::new(0.0, 0.0, 0.0);

        // rotation
        let rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);

        // scale
        let scale = Vector3::new(1.0, 1.0, 1.0);

        let payload = identity.new_spawn_message_payload();

        let message = SpawnMessage::new(identity.net_id,
                                        is_local_player,
                                        is_owner,
                                        identity.scene_id,
                                        identity.asset_id,
                                        position,
                                        rotation,
                                        scale,
                                        payload);
        connection.send_network_message(message, TransportChannel::Reliable);
    }

    // 处理 TransportCallback
    fn transport_callback(tbc: TransportCallback) {
        match tbc.r#type {
            TransportCallbackType::OnServerConnected => {
                Self::on_transport_connected(tbc.connection_id)
            }
            TransportCallbackType::OnServerDataReceived => {
                Self::on_transport_data(tbc.connection_id, tbc.data, tbc.channel)
            }
            TransportCallbackType::OnServerDisconnected => {
                Self::on_transport_disconnected(tbc.connection_id)
            }
            TransportCallbackType::OnServerError => {
                error!(format!("Server.HandleError: connectionId: {}, error: {:?}", tbc.connection_id, tbc.error));
                Self::on_transport_error(tbc.connection_id, tbc.error)
            }
            TransportCallbackType::OnServerTransportException => {
                Self::on_server_transport_exception(tbc.connection_id, tbc.error)
            }
            _ => {}
        }
    }

    // 处理 TransportConnected 消息
    fn on_transport_connected(connection_id: u64) {
        if connection_id == 0 {
            error!(format!("Server.HandleConnect: invalid connectionId: {}. Needs to be != 0, because 0 is reserved for local player.", connection_id));
            if let Some(mut transport) = Transport::get_active_transport() {
                transport.server_disconnect(connection_id);
            }
            return;
        }

        if NETWORK_CONNECTIONS.contains_key(&connection_id) {
            error!(format!("Server.HandleConnect: connectionId {} already exists.", connection_id));
            if let Some(mut transport) = Transport::get_active_transport() {
                transport.server_disconnect(connection_id);
            }
            return;
        }

        if Self::get_static_network_connections_size() >= Self::get_static_max_connections() {
            error!(format!("Server.HandleConnect: maxConnections reached: {}. Disconnecting connectionId: {}", Self::get_static_max_connections(), connection_id));
            if let Some(mut transport) = Transport::get_active_transport() {
                transport.server_disconnect(connection_id);
            }
            return;
        }
        let connection = NetworkConnection::network_connection(connection_id);
        Self::on_connected(connection);
    }

    // 处理 TransportData 消息
    fn on_transport_data(connection_id: u64, data: Vec<u8>, channel: TransportChannel) {
        //
        let mut reader = UnBatch::new(Bytes::copy_from_slice(data.as_slice()));
        let remote_time_stamp = reader.read_f64_le().unwrap_or_else(|_| 0.0);
        if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
            while let Ok(mut batch) = reader.read_next() {
                let message_id = batch.read_u16_le().unwrap();
                // println!("remote_time_stamp: {}, message_id: {}", remote_time_stamp, message_id);
                if let Some(handler) = NETWORK_MESSAGE_HANDLERS.get(&message_id) {
                    (handler.func)(&mut connection, &mut batch, channel);
                }
                if IS_LOADING_SCENE.load(Ordering::Relaxed) && batch.remaining() > NetworkMessages::ID_SIZE {
                    connection.remote_time_stamp = remote_time_stamp;
                }
            }
        }
    }

    // 处理 TransportDisconnected 消息
    fn on_transport_disconnected(connection_id: u64) {
        if let Some((_, mut connection)) = NETWORK_CONNECTIONS.remove(&connection_id) {
            connection.cleanup();
            if false {
                // TODO: OnDisconnectedEvent?.Invoke(conn); 844
            } else {
                Self::destroy_player_for_connection(&mut connection);
            }
        }
    }

    fn destroy_player_for_connection(conn: &mut NetworkConnection) {
        conn.destroy_owned_objects();
        conn.remove_from_observings_observers();
        conn.identity = NetworkIdentity::default();
    }

    // 处理 TransportError 消息
    fn on_transport_error(connection_id: u64, transport_error: TransportError) {
        warn!(format!("Server.HandleError: connectionId: {}, error: {:?}", connection_id, transport_error));
        if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
            // TODO OnErrorEvent?.Invoke(conn, error, reason);
        }
    }

    // 处理 ServerTransportException 消息
    fn on_server_transport_exception(connection_id: u64, transport_error: TransportError) {
        warn!(format!("Server.HandleTransportException: connectionId: {}, error: {:?}", connection_id, transport_error));
        if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
            // TODO OnTransportExceptionEvent?.Invoke(conn, error, reason);
        }
    }

    // 处理 Connected 消息
    fn on_connected(connection: NetworkConnection) {
        Self::add_connection(connection);
        // TODO: OnConnectedEvent?.Invoke(conn);
    }

    // 添加连接
    fn add_connection(connection: NetworkConnection) -> bool {
        if NETWORK_CONNECTIONS.contains_key(&connection.connection_id) {
            return false;
        }
        NETWORK_CONNECTIONS.insert(connection.connection_id, connection);
        true
    }

    // 注册消息处理程序
    fn register_message_handlers() {
        // 注册 ReadyMessage 处理程序
        Self::register_handler::<ReadyMessage>(Box::new(Self::on_client_ready_message), true);
        // 注册 CommandMessage 处理程序
        Self::register_handler::<CommandMessage>(Box::new(Self::on_command_message), true);

        // 注册 NetworkPingMessage 处理程序
        Self::register_handler::<NetworkPingMessage>(Box::new(NetworkTime::on_server_ping), false);
        // 注册 NetworkPongMessage 处理程序
        Self::register_handler::<NetworkPongMessage>(Box::new(NetworkTime::on_server_pong), false);

        // 注册 EntityStateMessage 处理程序
        Self::register_handler::<EntityStateMessage>(Box::new(Self::on_entity_state_message), true);
        // 注册 TimeSnapshotMessage 处理程序
        Self::register_handler::<TimeSnapshotMessage>(Box::new(Self::on_time_snapshot_message), true);
    }

    // 处理 ReadyMessage 消息
    fn on_client_ready_message(connection: &mut NetworkConnection, reader: &mut UnBatch, channel: TransportChannel) {
        let _ = channel;
        if let Ok(_) = ReadyMessage::deserialize(reader) {
            Self::set_client_ready(connection);
        }
    }
    // 设置客户端准备就绪
    fn set_client_ready(connection: &mut NetworkConnection) {
        connection.is_ready = true;

        Self::spawn_observers_for_connection(connection);
    }
    // 为连接生成观察者
    fn spawn_observers_for_connection(connection: &mut NetworkConnection) {
        if !connection.is_ready {
            return;
        }
        debug!("spawn_observers_for_connection: ", connection.connection_id);

        connection.send_network_message(ObjectSpawnStartedMessage::new(), TransportChannel::Reliable);

        // add connection to each nearby NetworkIdentity's observers, which
        // internally sends a spawn message for each one to the connection.
        SPAWNED.iter_mut().for_each(|mut identity| {
            if identity.visibility == Visibility::Shown {
                identity.add_observing_network_connection(connection.connection_id);
            } else if identity.visibility == Visibility::Hidden {
                // do nothing
            } else if identity.visibility == Visibility::Default {
                // TODO aoi system
                identity.add_observing_network_connection(connection.connection_id);
            }
        });

        connection.send_network_message(ObjectSpawnFinishedMessage::new(), TransportChannel::Reliable);
    }

    // 处理 OnCommandMessage 消息
    fn on_command_message(connection: &mut NetworkConnection, reader: &mut UnBatch, channel: TransportChannel) {
        if let Ok(message) = CommandMessage::deserialize(reader) {
            // TODO: on_command_message
        }
    }

    // 处理 OnEntityStateMessage 消息
    fn on_entity_state_message(connection: &mut NetworkConnection, reader: &mut UnBatch, channel: TransportChannel) {
        let message = EntityStateMessage::deserialize(reader);
        if let Ok(message) = message {
            // TODO: on_entity_state_message
            println!("on_entity_state_message: {:?}", message);
        }
    }

    // 处理 OnTimeSnapshotMessage 消息
    fn on_time_snapshot_message(connection: &mut NetworkConnection, reader: &mut UnBatch, channel: TransportChannel) {
        let message = TimeSnapshotMessage::deserialize(reader);
        if let Ok(message) = message {
            println!("on_time_snapshot_message: {:?}", message);
        }
        let snapshot = TimeSnapshot::new(connection.remote_time_stamp, NetworkTime::local_time());
        connection.on_time_snapshot(snapshot);
    }

    // 定义一个函数来注册处理程序
    pub fn register_handler<T>(network_message_handler: NetworkMessageHandlerFunc, require_authentication: bool)
    where
        T: NetworkMessageReader + Send + Sync + 'static,
    {
        let hash_code = T::get_hash_code();

        if NETWORK_MESSAGE_HANDLERS.contains_key(&hash_code) {
            warn!(format!("NetworkServer.RegisterHandler replacing handler for id={}. If replacement is intentional, use ReplaceHandler instead to avoid this warning.", hash_code));
            return;
        }
        NETWORK_MESSAGE_HANDLERS.insert(hash_code, NetworkMessageHandler::wrap_handler(network_message_handler, require_authentication));
    }


    // *****************************************************
    pub fn get_static_initialized() -> bool {
        INITIALIZED.load(Ordering::Relaxed)
    }
    pub fn set_static_initialized(value: bool) {
        INITIALIZED.store(value, Ordering::Relaxed);
    }
    pub fn get_static_max_connections() -> usize {
        MAX_CONNECTIONS.load(Ordering::Relaxed)
    }
    pub fn set_static_max_connections(value: usize) {
        MAX_CONNECTIONS.store(value, Ordering::Relaxed);
    }
    pub fn get_static_spawned_size() -> usize {
        SPAWNED.len()
    }
    pub fn get_static_network_connections_size() -> usize {
        NETWORK_CONNECTIONS.len()
    }
    pub fn get_static_send_rate() -> u32 {
        SEND_RATE.load(Ordering::Relaxed)
    }
    pub fn set_static_send_rate(value: u32) {
        SEND_RATE.store(value, Ordering::Relaxed);
        Self::set_static_send_interval(1f32 / value as f32);
    }
    pub fn get_static_send_interval() -> f32 {
        SEND_INTERVAL.load(Ordering::Relaxed)
    }
    pub fn set_static_send_interval(value: f32) {
        SEND_INTERVAL.store(value, Ordering::Relaxed);
    }
    pub fn get_static_dont_listen() -> bool {
        DONT_LISTEN.load(Ordering::Relaxed)
    }
    pub fn set_static_dont_listen(value: bool) {
        DONT_LISTEN.store(value, Ordering::Relaxed);
    }
    pub fn get_static_active() -> bool {
        ACTIVE.load(Ordering::Relaxed)
    }
    pub fn set_static_active(value: bool) {
        ACTIVE.store(value, Ordering::Relaxed);
    }
    pub fn get_static_is_loading_scene() -> bool {
        IS_LOADING_SCENE.load(Ordering::Relaxed)
    }
    pub fn set_static_is_loading_scene(value: bool) {
        IS_LOADING_SCENE.store(value, Ordering::Relaxed);
    }
    pub fn get_static_exceptions_disconnect() -> bool {
        EXCEPTIONS_DISCONNECT.load(Ordering::Relaxed)
    }
    pub fn set_static_exceptions_disconnect(value: bool) {
        EXCEPTIONS_DISCONNECT.store(value, Ordering::Relaxed);
    }
    pub fn get_static_disconnect_inactive_connections() -> bool {
        DISCONNECT_INACTIVE_CONNECTIONS.load(Ordering::Relaxed)
    }
    pub fn set_static_disconnect_inactive_connections(value: bool) {
        DISCONNECT_INACTIVE_CONNECTIONS.store(value, Ordering::Relaxed);
    }
    pub fn get_static_disconnect_inactive_timeout() -> f32 {
        DISCONNECT_INACTIVE_TIMEOUT.load(Ordering::Relaxed)
    }
    pub fn set_static_disconnect_inactive_timeout(value: f32) {
        DISCONNECT_INACTIVE_TIMEOUT.store(value, Ordering::Relaxed);
    }
    pub fn get_static_actual_tick_rate() -> u32 {
        ACTUAL_TICK_RATE.load(Ordering::Relaxed)
    }
    pub fn set_static_actual_tick_rate(value: u32) {
        ACTUAL_TICK_RATE.store(value, Ordering::Relaxed);
    }
    pub fn get_static_actual_tick_rate_start() -> f64 {
        ACTUAL_TICK_RATE_START.load(Ordering::Relaxed)
    }
    pub fn set_static_actual_tick_rate_start(value: f64) {
        ACTUAL_TICK_RATE_START.store(value, Ordering::Relaxed);
    }
    pub fn get_static_actual_tick_rate_counter() -> u32 {
        ACTUAL_TICK_RATE_COUNTER.load(Ordering::Relaxed)
    }
    pub fn set_static_actual_tick_rate_counter(value: u32) {
        ACTUAL_TICK_RATE_COUNTER.store(value, Ordering::Relaxed);
    }
    pub fn get_static_tick_rate() -> u32 {
        TICK_RATE.load(Ordering::Relaxed)
    }
    pub fn set_static_tick_rate(value: u32) {
        TICK_RATE.store(value, Ordering::Relaxed);
        Self::set_static_tick_interval(1f32 / value as f32);
        Self::set_static_send_rate(value);
    }
    pub fn get_static_tick_interval() -> f32 {
        TICK_INTERVAL.load(Ordering::Relaxed)
    }
    pub fn set_static_tick_interval(value: f32) {
        TICK_INTERVAL.store(value, Ordering::Relaxed);
    }
    pub fn get_static_last_send_time() -> f64 {
        LAST_SEND_TIME.load(Ordering::Relaxed)
    }
    pub fn set_static_last_send_time(value: f64) {
        LAST_SEND_TIME.store(value, Ordering::Relaxed);
    }
    pub fn set_static_early_update_duration(value: TimeSample) {
        if let Ok(mut early_update_duration) = EARLY_UPDATE_DURATION.write() {
            *early_update_duration = value;
        }
    }
    pub fn set_static_full_update_duration(value: TimeSample) {
        if let Ok(mut full_update_duration) = FULL_UPDATE_DURATION.write() {
            *full_update_duration = value;
        }
    }
    pub fn set_static_late_update_duration(value: TimeSample) {
        if let Ok(mut late_update_duration) = LATE_UPDATE_DURATION.write() {
            *late_update_duration = value;
        }
    }

    // 遍历NETWORK_CONNECTIONS
    pub fn for_each_network_connection<F>(mut f: F)
    where
        F: FnMut(RefMutMulti<u64, NetworkConnection>),
    {
        NETWORK_CONNECTIONS.iter_mut().for_each(|item| {
            f(item);
        });
    }
    // 遍历SPAWNED
    pub fn for_each_spawned<F>(mut f: F)
    where
        F: FnMut(RefMutMulti<u64, NetworkIdentity>),
    {
        SPAWNED.iter_mut().for_each(|item| {
            f(item);
        });
    }
    // 遍历NETWORK_MESSAGE_HANDLERS
    pub fn for_each_network_message_handler<F>(mut f: F)
    where
        F: FnMut(RefMutMulti<u16, NetworkMessageHandler>),
    {
        NETWORK_MESSAGE_HANDLERS.iter_mut().for_each(|item| {
            f(item);
        });
    }
}
