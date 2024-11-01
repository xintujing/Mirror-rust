use crate::core::batching::un_batcher::UnBatcher;
use crate::core::messages::{CommandMessage, EntityStateMessage, NetworkMessageHandler, NetworkMessageHandlerFunc, NetworkPingMessage, NetworkPongMessage, ObjectHideMessage, ObjectSpawnFinishedMessage, ObjectSpawnStartedMessage, ReadyMessage, SpawnMessage, TimeSnapshotMessage};
use crate::core::network_connection::NetworkConnection;
use crate::core::network_identity::{NetworkIdentity, Visibility};
use crate::core::network_manager::GameObject;
use crate::core::network_messages::NetworkMessages;
use crate::core::network_reader::{NetworkMessageReader, NetworkReader};
use crate::core::network_reader_pool::NetworkReaderPool;
use crate::core::network_time::NetworkTime;
use crate::core::network_writer::NetworkWriter;
use crate::core::network_writer_pool::NetworkWriterPool;
use crate::core::remote_calls::{RemoteCallType, RemoteProcedureCalls};
use crate::core::snapshot_interpolation::snapshot_interpolation_settings::SnapshotInterpolationSettings;
use crate::core::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::core::tools::time_sample::TimeSample;
use crate::core::transport::{Transport, TransportCallback, TransportCallbackType, TransportChannel, TransportError};
use atomic::Atomic;
use bytes::Bytes;
use dashmap::mapref::multiple::RefMutMulti;
use dashmap::DashMap;
use lazy_static::lazy_static;
use nalgebra::{Quaternion, Vector3};
use std::convert::identity;
use std::sync::atomic::Ordering;
use std::sync::{RwLock, RwLockReadGuard};
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
    static ref SNAPSHOT_SETTINGS: RwLock<SnapshotInterpolationSettings> = RwLock::new(SnapshotInterpolationSettings::default());

    static ref NETWORK_CONNECTIONS: DashMap<u64, NetworkConnection> = DashMap::new();
    static ref SPAWNED: DashMap<u32, NetworkIdentity> = DashMap::new();
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


        if let Some(transport) = Transport::get_active_transport() {
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
            if let Some(transport) = Transport::get_active_transport() {
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
        if let Some(active_transport) = Transport::get_active_transport() {
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
        NETWORK_CONNECTIONS.iter_mut().for_each(|mut connection| {
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
        let len = connection.observing_identities.len();
        for i in 0..len {
            let identity = &mut connection.observing_identities[i];
            let net_id = identity.net_id; // 提取 `net_id`
            if true {
                if let Some(serialization) = Self::serialize_for_connection(identity, connection.connection_id) {
                    let message = EntityStateMessage::new(net_id, serialization.to_bytes());
                    connection.send_network_message(message, TransportChannel::Reliable);
                }
            } else {
                warn!(format!("Server.BroadcastToConnection: identity is null. Removing from observing list. connectionId: {}, netId: {}", connection.connection_id, identity.net_id));
                connection.observing_identities.retain(|idy| idy.net_id != net_id);
            }
        }
    }
    fn serialize_for_connection(identity: &mut NetworkIdentity, connection_id: u64) -> Option<&mut NetworkWriter> {
        let owned = identity.connection_id_to_client == connection_id;

        let serialization = identity.get_server_serialization_at_tick(Self::get_static_tick_rate());

        if owned {
            if serialization.owner_writer.get_position() > 0 {
                return Some(&mut serialization.owner_writer);
            }
        } else {
            if serialization.observers_writer.get_position() > 0 {
                return Some(&mut serialization.observers_writer);
            }
        }
        None
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

    pub fn hide_for_connection(identity: &NetworkIdentity, connection_id: u64) {
        if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
            let object_hide_message = ObjectHideMessage::new(identity.net_id);
            connection.send_network_message(object_hide_message, TransportChannel::Reliable);
        }
    }

    fn send_spawn_message(identity: &mut NetworkIdentity, connection: &mut NetworkConnection) {
        if identity.server_only {
            return;
        }
        // TODO  fix pos

        let is_local_player = identity.net_id == connection.identity.net_id;
        let is_owner = identity.connection_id_to_client == connection.connection_id;
        // position
        let position = Vector3::new(0.0, 0.0, 0.0);

        // rotation
        let rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);

        // scale
        let scale = Vector3::new(1.0, 1.0, 1.0);

        NetworkWriterPool::get_return(|owner_writer| {
            NetworkWriterPool::get_return(|observers_writer| {
                let payload = Self::create_spawn_message_payload(identity.connection_id_to_client == connection.connection_id, identity, owner_writer, observers_writer);
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
            });
        });
    }

    fn create_spawn_message_payload(is_owner: bool, identity: &mut NetworkIdentity, owner_writer: &mut NetworkWriter, observers_writer: &mut NetworkWriter) -> Vec<u8> {
        if identity.network_behaviours.len() == 0 {
            return Vec::new();
        }

        identity.serialize_server(true, owner_writer, observers_writer);

        let owner_segment = owner_writer.to_bytes();
        let observers_segment = observers_writer.to_bytes();

        if is_owner {
            return owner_segment;
        }
        observers_segment
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
            error!(format!("Server.HandleConnect: max_connections reached: {}. Disconnecting connectionId: {}", Self::get_static_max_connections(), connection_id));
            if let Some(mut transport) = Transport::get_active_transport() {
                transport.server_disconnect(connection_id);
            }
            return;
        }
        let connection = NetworkConnection::new(connection_id);
        Self::on_connected(connection);
    }

    // 处理 TransportData 消息
    fn on_transport_data(connection_id: u64, data: Vec<u8>, channel: TransportChannel) {
        let mut transport_data_un_batcher = UnBatcher::new();

        if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
            // 添加数据到 transport_data_un_batcher
            if !transport_data_un_batcher.add_batch_with_bytes(data) {
                if Self::get_static_exceptions_disconnect() {
                    error!(format!("Server.HandleData: connectionId: {} failed to add batch. Disconnecting.", connection_id));
                    connection.disconnect();
                } else {
                    warn!(format!("Server.HandleData: connectionId: {} failed to add batch.", connection_id));
                }
                return;
            }

            // 如果没有加载场景
            if !Self::get_static_is_loading_scene() {
                // 处理消息
                while let Some((message, remote_time_stamp)) = transport_data_un_batcher.get_next_message() {
                    NetworkReaderPool::get_with_array_segment_return(&message, |reader| {
                        // 如果消息长度大于 NetworkMessages::ID_SIZE
                        if reader.remaining() >= NetworkMessages::ID_SIZE {
                            // 更新远程时间戳
                            connection.remote_time_stamp = remote_time_stamp;
                            // 处理消息
                            if !Self::unpack_and_invoke(&mut connection, reader, channel) {
                                if Self::get_static_exceptions_disconnect() {
                                    error!(format!("Server.HandleData: connectionId: {} failed to unpack and invoke message. Disconnecting.", connection_id));
                                    connection.disconnect();
                                } else {
                                    warn!(format!("Server.HandleData: connectionId: {} failed to unpack and invoke message.", connection_id));
                                }
                                return;
                            }
                        } else {
                            if Self::get_static_exceptions_disconnect() {
                                error!(format!("Server.HandleData: connectionId: {} message too small. Disconnecting.", connection_id));
                                connection.disconnect();
                            } else {
                                warn!(format!("Server.HandleData: connectionId: {} message too small.", connection_id));
                            }
                            return;
                        }
                    });
                }

                if transport_data_un_batcher.batches_count() > 0 {
                    error!(format!("Server.HandleData: connectionId: {} unprocessed batches: {}", connection_id, transport_data_un_batcher.batches_count()));
                }
            }
        } else {
            error!(format!("Server.HandleData: Unknown connectionId: {}", connection_id));
        }
    }

    fn unpack_and_invoke(connection: &mut NetworkConnection, reader: &mut NetworkReader, channel: TransportChannel) -> bool {
        // 解包消息id
        let message_id = NetworkMessages::unpack_id(reader);
        // 如果消息id在 NETWORK_MESSAGE_HANDLERS 中
        if let Some(handler) = NETWORK_MESSAGE_HANDLERS.get(&message_id) {
            (handler.func)(connection, reader, channel);
            connection.last_message_time = NetworkTime::local_time();
            return true;
        }
        warn!(format!("Server.HandleData: connectionId: {} unknown message id: {}", connection.connection_id, message_id));
        false
    }

    // 处理 TransportDisconnected 消息
    fn on_transport_disconnected(connection_id: u64) {
        if let Some((_, mut connection)) = NETWORK_CONNECTIONS.remove(&connection_id) {
            connection.cleanup();
            if false {
                // TODO: OnDisconnectedEvent?.invoke(conn); 844
            } else {
                Self::destroy_player_for_connection(&mut connection);
            }
        }
    }

    fn destroy_player_for_connection(connection: &mut NetworkConnection) {
        connection.destroy_owned_objects();
        connection.remove_from_observings_observers();
        connection.identity = NetworkIdentity::default();
    }

    pub fn add_player_for_connection(connection: &mut NetworkConnection, player: &mut GameObject) -> bool {
        if let Some(mut identity) = player.get_component() {
            if !connection.identity.is_null() {
                warn!(format!("AddPlayer: connection already has a player GameObject. Please remove the current player GameObject from {}", connection.is_ready));
                return false;
            }
            identity.set_client_owner(connection.connection_id);
            connection.identity = identity;
            Self::set_client_ready(connection);
            Self::respawn(&mut connection.identity);
            return true;
        }
        warn!(format!("AddPlayer: player GameObject has no NetworkIdentity. Please add a NetworkIdentity to {:?}",player));
        false
    }

    fn spawn(obj: &mut GameObject, connection: &mut NetworkConnection) {
        Self::spawn_obj(obj, connection)
    }

    fn spawn_obj(obj: &mut GameObject, connection: &mut NetworkConnection) {
        // TODO SpawnObject

    }

    fn respawn(identity: &mut NetworkIdentity) {
        if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&identity.connection_id_to_client) {
            if identity.net_id == 0 {
                warn!(format!("Server.Respawn: netId is zero. Please set a valid netId on {:?}",identity.game_object));
                Self::spawn(&mut identity.game_object, &mut connection);
                return;
            }
            Self::send_spawn_message(identity, &mut connection);
        }
    }

    // 处理 TransportError 消息
    fn on_transport_error(connection_id: u64, transport_error: TransportError) {
        warn!(format!("Server.HandleError: connectionId: {}, error: {:?}", connection_id, transport_error));
        if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
            // TODO OnErrorEvent?.invoke(conn, error, reason);
        }
    }

    // 处理 ServerTransportException 消息
    fn on_server_transport_exception(connection_id: u64, transport_error: TransportError) {
        warn!(format!("Server.HandleTransportException: connectionId: {}, error: {:?}", connection_id, transport_error));
        if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
            // TODO OnTransportExceptionEvent?.invoke(conn, error, reason);
        }
    }

    // 处理 Connected 消息
    fn on_connected(connection: NetworkConnection) {
        Self::add_connection(connection);
        // TODO: OnConnectedEvent?.invoke(conn);
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
    fn on_client_ready_message(connection: &mut NetworkConnection, reader: &mut NetworkReader, channel: TransportChannel) {
        let _ = channel;
        let _ = ReadyMessage::deserialize(reader);
        Self::set_client_ready(connection);
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
    fn on_command_message(connection: &mut NetworkConnection, reader: &mut NetworkReader, channel: TransportChannel) {
        let message = CommandMessage::deserialize(reader);
        // connection 没有准备好
        if !connection.is_ready {
            // 如果 channel 是 Reliable
            if channel == TransportChannel::Reliable {
                // 如果 SPAWNED 中有 message.net_id
                if let Some(net_identity) = SPAWNED.get(&message.net_id) {
                    // 如果 message.component_index 小于 net_identity.network_behaviours.len()
                    if net_identity.network_behaviours.len() > message.component_index as usize {
                        // 如果 message.function_hash 在 RemoteProcedureCalls 中
                        if let Some(method_name) = RemoteProcedureCalls::get_function_method_name(message.function_hash) {
                            warn!(format!("Command {} received for {} [netId={}] component  [index={}] when client not ready.\nThis may be ignored if client intentionally set NotReady.", method_name, net_identity.net_id, message.net_id, message.component_index));
                            return;
                        }
                    }
                }
                warn!("Command received while client is not ready. This may be ignored if client intentionally set NotReady.".to_string());
            }
            return;
        }

        if let Some(mut identity) = SPAWNED.get_mut(&message.net_id) {
            // 是否需要权限
            let requires_authority = RemoteProcedureCalls::command_requires_authority(message.function_hash);
            // 如果需要权限并且 identity.connection_id_to_client != connection.connection_id
            if requires_authority && identity.connection_id_to_client != connection.connection_id {
                // Attempt to identify the component and method to narrow down the cause of the error.
                if identity.network_behaviours.len() > message.component_index as usize {
                    if let Some(method_name) = RemoteProcedureCalls::get_function_method_name(message.function_hash) {
                        warn!(format!("Command {} received for {} [netId={}] component [index={}] without authority", method_name, identity.net_id, message.net_id,  message.component_index));
                        return;
                    }
                }
                warn!(format!("Command received for {} [netId={}] without authority", identity.net_id, message.net_id));
                return;
            }

            NetworkReaderPool::get_with_bytes_return(message.payload, |reader| {
                identity.handle_remote_call(message.component_index, message.function_hash, RemoteCallType::Command, reader, connection.connection_id);
            });
        } else {
            // over reliable channel, commands should always come after spawn.
            // over unreliable, they might come in before the object was spawned.
            // for example, NetworkTransform.
            // let's not spam the console for unreliable out of order messages.
            if channel == TransportChannel::Reliable {
                warn!(format!("Spawned object not found when handling Command message netId={}", message.net_id));
            }
            return;
        }
    }

    // 处理 OnEntityStateMessage 消息
    fn on_entity_state_message(connection: &mut NetworkConnection, reader: &mut NetworkReader, channel: TransportChannel) {
        let message = EntityStateMessage::deserialize(reader);
        if let Some(mut identity) = SPAWNED.get_mut(&message.net_id) {
            if identity.connection_id_to_client == connection.connection_id {
                NetworkReaderPool::get_with_bytes_return(message.payload, |reader| {
                    if !identity.deserialize_server(reader) {
                        if Self::get_static_exceptions_disconnect() {
                            error!(format!("Server failed to deserialize client state for {} with netId={}, Disconnecting.", identity.net_id, identity.net_id));
                            connection.disconnect();
                        } else {
                            warn!(format!("Server failed to deserialize client state for {} with netId={}", identity.net_id, identity.net_id));
                        }
                    }
                });
            } else {
                warn!(format!("EntityStateMessage from {} for {} without authority.", connection.connection_id, identity.net_id));
            }
        }
    }

    // 处理 OnTimeSnapshotMessage 消息
    fn on_time_snapshot_message(connection: &mut NetworkConnection, reader: &mut NetworkReader, channel: TransportChannel) {
        let message = TimeSnapshotMessage::deserialize(reader);
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
    pub fn get_static_snapshot_settings() -> &'static RwLock<SnapshotInterpolationSettings> {
        &SNAPSHOT_SETTINGS
    }
    pub fn set_static_snapshot_settings(value: SnapshotInterpolationSettings) {
        if let Ok(mut snapshot_settings) = SNAPSHOT_SETTINGS.write() {
            *snapshot_settings = value;
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
        F: FnMut(RefMutMulti<u32, NetworkIdentity>),
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
