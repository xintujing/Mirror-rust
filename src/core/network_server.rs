use crate::core::batching::un_batcher::UnBatcher;
use crate::core::messages::{ChangeOwnerMessage, CommandMessage, EntityStateMessage, NetworkMessageHandler, NetworkMessageHandlerFunc, NetworkPingMessage, NetworkPongMessage, ObjectDestroyMessage, ObjectHideMessage, ObjectSpawnFinishedMessage, ObjectSpawnStartedMessage, ReadyMessage, SpawnMessage, TimeSnapshotMessage};
use crate::core::network_connection::NetworkConnectionTrait;
use crate::core::network_connection_to_client::NetworkConnectionToClient;
use crate::core::network_identity::Visibility::ForceShown;
use crate::core::network_identity::{NetworkIdentity, Visibility};
use crate::core::network_manager::GameObject;
use crate::core::network_messages::NetworkMessages;
use crate::core::network_reader::{NetworkMessageReader, NetworkReader};
use crate::core::network_reader_pool::NetworkReaderPool;
use crate::core::network_time::NetworkTime;
use crate::core::network_writer_pool::NetworkWriterPool;
use crate::core::remote_calls::{RemoteCallType, RemoteProcedureCalls};
use crate::core::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::core::tools::time_sample::TimeSample;
use crate::core::transport::{Transport, TransportCallback, TransportCallbackType, TransportChannel, TransportError};
use atomic::Atomic;
use dashmap::mapref::multiple::RefMutMulti;
use dashmap::DashMap;
use lazy_static::lazy_static;
use std::sync::atomic::Ordering;
use std::sync::RwLock;
use tklog::{debug, error, warn};

pub enum RemovePlayerOptions {
    /// <summary>Player Object remains active on server and clients. Only ownership is removed</summary>
    KeepActive,
    /// <summary>Player Object is unspawned on clients but remains on server</summary>
    UnSpawn,
    /// <summary>Player Object is destroyed on server and clients</summary>
    Destroy,
}

// EventHandler 静态变量
type EventHandler = fn(&mut NetworkConnectionToClient, TransportError);

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum EventHandlerType {
    OnConnectedEvent,
    OnDisconnectedEvent,
    OnErrorEvent,
    OnTransportExceptionEvent,
}

// NetworkServer 静态变量
lazy_static! {
    static ref CONNECTED_EVENT: DashMap<EventHandlerType, Box<EventHandler>> = DashMap::new();

    static ref Initialized: Atomic<bool> = Atomic::new(false);
    static ref TickRate: Atomic<u32> = Atomic::new(60);
    static ref TICK_INTERVAL: Atomic<f32> = Atomic::new(1f32 / NetworkServerStatic::get_static_tick_rate() as f32);
    static ref SEND_RATE: Atomic<u32> = Atomic::new(NetworkServerStatic::get_static_tick_rate());
    static ref SEND_INTERVAL: Atomic<f32> = Atomic::new(1f32 / NetworkServerStatic::get_static_send_rate() as f32);
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

    static ref NETWORK_CONNECTIONS: DashMap<u64, NetworkConnectionToClient> = DashMap::new();
    static ref SPAWNED_NETWORK_IDENTITIES: DashMap<u32, NetworkIdentity> = DashMap::new();
    static ref NETWORK_MESSAGE_HANDLERS: DashMap<u16, NetworkMessageHandler> = DashMap::new();
}

// NetworkServer 静态结构体
pub struct NetworkServerStatic;
// NetworkServer 静态结构体方法
impl NetworkServerStatic {
    pub fn get_exceptions_disconnect() -> bool {
        EXCEPTIONS_DISCONNECT.load(Ordering::Relaxed)
    }
    pub fn set_exceptions_disconnect(value: bool) {
        EXCEPTIONS_DISCONNECT.store(value, Ordering::Relaxed);
    }
    pub fn get_connected_event() -> &'static DashMap<EventHandlerType, Box<EventHandler>> {
        &CONNECTED_EVENT
    }
    pub fn get_static_initialized() -> bool {
        Initialized.load(Ordering::Relaxed)
    }
    pub fn set_static_initialized(value: bool) {
        Initialized.store(value, Ordering::Relaxed);
    }
    pub fn get_static_max_connections() -> usize {
        MAX_CONNECTIONS.load(Ordering::Relaxed)
    }
    pub fn set_static_max_connections(value: usize) {
        MAX_CONNECTIONS.store(value, Ordering::Relaxed);
    }
    pub fn get_static_spawned_size() -> usize {
        NetworkServerStatic::get_static_spawned_network_identities().len()
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
        TickRate.load(Ordering::Relaxed)
    }
    pub fn set_static_tick_rate(value: u32) {
        TickRate.store(value, Ordering::Relaxed);
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
    // 获取 NetworkConnections
    pub fn get_static_network_connections() -> &'static DashMap<u64, NetworkConnectionToClient> {
        &NETWORK_CONNECTIONS
    }
    // 添加连接
    fn static_network_connections_add_connection(connection: NetworkConnectionToClient) -> bool {
        if NETWORK_CONNECTIONS.contains_key(&connection.connection_id()) {
            return false;
        }
        NETWORK_CONNECTIONS.insert(connection.connection_id(), connection);
        true
    }
    pub fn get_static_spawned_network_identities() -> &'static DashMap<u32, NetworkIdentity> {
        &SPAWNED_NETWORK_IDENTITIES
    }
    // 遍历NETWORK_CONNECTIONS
    pub fn for_each_network_connection<F>(mut f: F)
    where
        F: FnMut(RefMutMulti<u64, NetworkConnectionToClient>),
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
        SPAWNED_NETWORK_IDENTITIES.iter_mut().for_each(|item| {
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

// NetworkServer 结构体
pub struct NetworkServer;

// NetworkServer 结构体方法
impl NetworkServer {
    fn initialize() {
        if NetworkServerStatic::get_static_initialized() {
            return;
        }

        //Make sure connections are cleared in case any old connections references exist from previous sessions
        NETWORK_CONNECTIONS.clear();

        // TODO: if (aoi != null) aoi.reset_state();

        NetworkTime::reset_statics();

        // 设置 TransportCallback
        if let Some(transport) = Transport::get_active_transport() {
            transport.set_transport_cb_fn(Box::new(Self::transport_callback));
        }

        // NetworkServer 是初始化的
        if NetworkServerStatic::get_static_initialized() {
            return;
        }

        if let Ok(mut early_update_duration) = EARLY_UPDATE_DURATION.write() {
            *early_update_duration = TimeSample::new(NetworkServerStatic::get_static_send_rate());
        }
        if let Ok(mut full_update_duration) = FULL_UPDATE_DURATION.write() {
            *full_update_duration = TimeSample::new(NetworkServerStatic::get_static_send_rate());
        }
        if let Ok(mut late_update_duration) = LATE_UPDATE_DURATION.write() {
            *late_update_duration = TimeSample::new(NetworkServerStatic::get_static_send_rate());
        }
    }
    pub fn listen(max_connections: usize) {
        // 初始化
        Self::initialize();

        // 设置最大连接数
        NetworkServerStatic::set_static_max_connections(max_connections);

        // 如果不监听
        if NetworkServerStatic::get_static_dont_listen() {
            if let Some(transport) = Transport::get_active_transport() {
                transport.server_start();
            }
        }
        // 设置 NetworkServer 为激活状态
        NetworkServerStatic::set_static_active(true);

        // 注册消息处理器
        Self::register_message_handlers();
    }

    // 网络早期更新
    pub fn network_early_update() {
        if NetworkServerStatic::get_static_active() {
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
        NetworkServerStatic::for_each_network_connection(|mut connection| {
            connection.update_time_interpolation();
        });

        if NetworkServerStatic::get_static_active() {
            if let Ok(mut early_update_duration) = EARLY_UPDATE_DURATION.write() {
                early_update_duration.end();
            }
        }

        // NetworkBehaviour update  模拟
        NetworkServerStatic::get_static_spawned_network_identities().iter_mut().for_each(|mut identity| {
            identity.network_behaviours.iter_mut().for_each(|behaviour| {
                behaviour.update();
            });
        });
    }

    // 网络更新
    pub fn network_late_update() {
        if NetworkServerStatic::get_static_active() {
            if let Ok(mut late_update_duration) = LATE_UPDATE_DURATION.write() {
                late_update_duration.begin();
            }
            Self::broadcast();
        }
        if let Some(mut active_transport) = Transport::get_active_transport() {
            active_transport.server_late_update();
        }

        if NetworkServerStatic::get_static_active() {
            let actual_tick_rate_counter = NetworkServerStatic::get_static_actual_tick_rate_counter();
            NetworkServerStatic::set_static_actual_tick_rate_counter(actual_tick_rate_counter + 1);

            let local_time = NetworkTime::local_time();

            if local_time - NetworkServerStatic::get_static_actual_tick_rate_start() >= 1.0 {
                let elapsed = local_time - NetworkServerStatic::get_static_actual_tick_rate_start();
                let actual_tick_rate_counter = NetworkServerStatic::get_static_actual_tick_rate_counter();
                NetworkServerStatic::set_static_actual_tick_rate(actual_tick_rate_counter / elapsed as u32);
                NetworkServerStatic::set_static_actual_tick_rate_start(local_time);
                NetworkServerStatic::set_static_actual_tick_rate_counter(0);
            }

            if let Ok(mut late_update_duration) = LATE_UPDATE_DURATION.write() {
                late_update_duration.end();
            }
            if let Ok(mut full_update_duration) = FULL_UPDATE_DURATION.write() {
                full_update_duration.end();
            }
        }

        // NetworkBehaviour late_update
        NetworkServerStatic::get_static_spawned_network_identities().iter_mut().for_each(|mut identity| {
            identity.network_behaviours.iter_mut().for_each(|behaviour| {
                behaviour.late_update()
            });
        });
    }

    // Broadcast
    fn broadcast() {
        NetworkServerStatic::for_each_network_connection(|mut connection| {
            if Self::disconnect_if_inactive(&mut connection) {
                return;
            }

            if connection.is_ready() {
                connection.send_network_message(&mut TimeSnapshotMessage::new(), TransportChannel::Unreliable);
                Self::broadcast_to_connection(&mut connection);
            }
            connection.update();
        });
    }

    // BroadcastToConnection(NetworkConnectionToClient connection)
    fn broadcast_to_connection(conn: &mut NetworkConnectionToClient) {
        for i in 0..conn.observing.len() {
            let net_id = conn.observing[i];
            if net_id != 0 {
                if let Some(mut message) = Self::serialize_for_connection(net_id, conn.connection_id()) {
                    // debug!(format!("Server.broadcast_to_connection: connectionId: {}, netId: {}", conn.connection_id(), net_id));
                    conn.send_network_message(&mut message, TransportChannel::Reliable);
                }
            } else {
                warn!(format!("Server.broadcast_to_connection: identity is null. Removing from observing list. connectionId: {}, netId: {}", conn.connection_id(), net_id));
                conn.observing.retain(|id| *id != net_id);
            }
        }
    }
    fn serialize_for_connection(net_id: u32, conn_id: u64) -> Option<EntityStateMessage> {
        if let Some(mut identity) = NetworkServerStatic::get_static_spawned_network_identities().get_mut(&net_id) {
            let owned = identity.connection_to_client() == conn_id;
            let net_id = identity.net_id();
            let serialization = identity.get_server_serialization_at_tick(NetworkTime::frame_count());
            if owned {
                if serialization.owner_writer.get_position() > 0 {
                    return Some(EntityStateMessage::new(net_id, serialization.owner_writer.to_bytes()));
                }
            } else {
                if serialization.observers_writer.get_position() > 0 {
                    return Some(EntityStateMessage::new(net_id, serialization.observers_writer.to_bytes()));
                }
            }
        }
        None
    }

    fn disconnect_if_inactive(connection: &mut NetworkConnectionToClient) -> bool {
        if NetworkServerStatic::get_static_disconnect_inactive_connections() &&
            !connection.is_alive(NetworkServerStatic::get_static_disconnect_inactive_timeout() as f64) {
            warn!(format!("Server.DisconnectIfInactive: connectionId: {} is inactive. Disconnecting.", connection.connection_id()));
            connection.disconnect();
            return true;
        }
        false
    }

    // show / hide for connection //////////////////////////////////////////
    pub fn show_for_connection(network_identity: &mut NetworkIdentity, conn: &mut NetworkConnectionToClient) {
        if conn.is_ready() {
            Self::send_spawn_message(network_identity, conn);
        }
    }

    pub fn hide_for_connection(identity: &mut NetworkIdentity, conn: &mut NetworkConnectionToClient) {
        if conn.is_ready() {
            println!("hide_for_connection: {:?}", identity);
            let mut message = ObjectHideMessage::new(identity.net_id());
            conn.send_network_message(&mut message, TransportChannel::Reliable);
        }
    }

    fn send_spawn_message(identity: &mut NetworkIdentity, conn: &mut NetworkConnectionToClient) {
        // 找到 NetworkIdentity
        if identity.server_only {
            return;
        }

        // 是否是所有者
        let is_owner = identity.connection_to_client() == conn.connection_id();
        // 是否是本地玩家
        let is_local_player = conn.net_id() == identity.net_id();
        // 创建 SpawnMessage 的 payload
        let payload = Self::create_spawn_message_payload(is_owner, identity);
        // 发送 SpawnMessage
        let mut spawn_message = SpawnMessage::new(identity.net_id(),
                                                  is_local_player,
                                                  is_owner,
                                                  identity.scene_id,
                                                  identity.asset_id,
                                                  identity.game_object().transform.local_position,
                                                  identity.game_object().transform.local_rotation,
                                                  identity.game_object().transform.local_scale,
                                                  payload);
        // 发送 SpawnMessage
        conn.send_network_message(&mut spawn_message, TransportChannel::Reliable);
    }

    fn create_spawn_message_payload(is_owner: bool, identity: &mut NetworkIdentity) -> Vec<u8> {
        let mut payload = Vec::new();
        // 如果没有 NetworkBehaviours
        if identity.network_behaviours.len() == 0 {
            return payload;
        }

        NetworkWriterPool::get_return(|owner_writer| {
            NetworkWriterPool::get_return(|observers_writer| {
                // 序列化 NetworkIdentity
                identity.serialize_server(true, owner_writer, observers_writer);
                // 如果是所有者
                if is_owner {
                    payload = owner_writer.to_bytes();
                } else { // 如果不是所有者
                    payload = observers_writer.to_bytes();
                }
            });
        });
        payload
    }

    // 处理 TransportCallback   AddTransportHandlers(
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
                Self::on_transport_exception(tbc.connection_id, tbc.error)
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

        if NetworkServerStatic::get_static_network_connections_size() >= NetworkServerStatic::get_static_max_connections() {
            error!(format!("Server.HandleConnect: max_connections reached: {}. Disconnecting connectionId: {}", NetworkServerStatic::get_static_max_connections(), connection_id));
            if let Some(mut transport) = Transport::get_active_transport() {
                transport.server_disconnect(connection_id);
            }
            return;
        }
        let connection = NetworkConnectionToClient::new(connection_id);
        Self::on_connected(connection);
    }

    // 处理 TransportData 消息
    fn on_transport_data(connection_id: u64, data: Vec<u8>, channel: TransportChannel) {
        let mut transport_data_un_batcher = UnBatcher::new();

        if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
            // 添加数据到 transport_data_un_batcher
            if !transport_data_un_batcher.add_batch_with_bytes(data) {
                if NetworkServerStatic::get_static_exceptions_disconnect() {
                    error!(format!("Server.HandleData: connectionId: {} failed to add batch. Disconnecting.", connection_id));
                    connection.disconnect();
                } else {
                    warn!(format!("Server.HandleData: connectionId: {} failed to add batch.", connection_id));
                }
                return;
            }
        } else {
            error!(format!("Server.HandleData: Unknown connectionId: {}", connection_id));
        }

        // 如果没有加载场景
        if !NetworkServerStatic::get_static_is_loading_scene() {
            // 处理消息
            while let Some((message, remote_time_stamp)) = transport_data_un_batcher.get_next_message() {
                NetworkReaderPool::get_with_array_segment_return(&message, |reader| {
                    // 如果消息长度大于 NetworkMessages::ID_SIZE
                    if reader.remaining() >= NetworkMessages::ID_SIZE {
                        // 更新远程时间戳
                        if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
                            connection.set_remote_time_stamp(remote_time_stamp);
                        }
                        // 处理消息
                        if !Self::unpack_and_invoke(connection_id, reader, channel) {
                            if NetworkServerStatic::get_static_exceptions_disconnect() {
                                error!(format!("Server.HandleData: connectionId: {} failed to unpack and invoke message. Disconnecting.", connection_id));
                                if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
                                    connection.disconnect();
                                }
                            } else {
                                warn!(format!("Server.HandleData: connectionId: {} failed to unpack and invoke message.", connection_id));
                            }
                            return;
                        }
                    } else {
                        if NetworkServerStatic::get_static_exceptions_disconnect() {
                            error!(format!("Server.HandleData: connectionId: {} message too small. Disconnecting.", connection_id));
                            if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
                                connection.disconnect();
                            }
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
    }

    fn unpack_and_invoke(connection_id: u64, reader: &mut NetworkReader, channel: TransportChannel) -> bool {
        // 解包消息id
        let message_id = NetworkMessages::unpack_id(reader);
        // 如果消息id在 NETWORK_MESSAGE_HANDLERS 中
        if let Some(handler) = NETWORK_MESSAGE_HANDLERS.get(&message_id) {
            (handler.func)(connection_id, reader, channel);
            if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
                connection.set_last_message_time(NetworkTime::local_time());
            }
            return true;
        }
        warn!(format!("Server.HandleData: connectionId: {} unknown message id: {}", connection_id, message_id));
        false
    }

    // 处理 TransportDisconnected 消息
    fn on_transport_disconnected(connection_id: u64) {
        if let Some((_, mut connection)) = NetworkServerStatic::get_static_network_connections().remove(&connection_id) {
            connection.cleanup();
            if let Some(on_disconnected_event) = NetworkServerStatic::get_connected_event().get(&EventHandlerType::OnDisconnectedEvent) {
                on_disconnected_event(&mut connection, TransportError::None);
            } else {
                Self::destroy_player_for_connection(&mut connection);
            }
        }
    }

    pub fn destroy_player_for_connection(conn: &mut NetworkConnectionToClient) {
        conn.destroy_owned_objects();
        conn.remove_from_observings_observers();
        conn.set_net_id(0);
    }

    pub fn destroy(identity: &mut NetworkIdentity) {
        if !NetworkServerStatic::get_static_active() {
            error!("Server.Destroy: NetworkServer is not active. Cannot destroy objects without an active server.");
            return;
        }

        if identity.game_object().is_null() {
            warn!("Server.Destroy: game object is null.");
            return;
        }

        if identity.scene_id == 0 {
            Self::un_spawn_internal(identity, true);
        } else {
            Self::un_spawn_internal(identity, false);
            identity.destroy_called = true;
        }
    }

    pub fn remove_player_for_connection(conn: &mut NetworkConnectionToClient, options: RemovePlayerOptions) {
        if conn.net_id() == 0 {
            return;
        }

        match options {
            RemovePlayerOptions::KeepActive => {
                if let Some(mut identity) = NetworkServerStatic::get_static_spawned_network_identities().get_mut(&conn.net_id()) {
                    identity.set_connection_to_client(0);
                    conn.owned().retain(|id| *id != identity.net_id());
                    Self::send_change_owner_message(&mut identity, conn);
                } else {
                    error!(format!("Server.RemovePlayer: netId {} not found in spawned.", conn.net_id()));
                    return;
                }
            }
            RemovePlayerOptions::UnSpawn => {
                if let Some(mut identity) = NetworkServerStatic::get_static_spawned_network_identities().get_mut(&conn.net_id()) {
                    Self::un_spawn(&mut identity);
                } else {
                    error!(format!("Server.RemovePlayer: netId {} not found in spawned.", conn.net_id()));
                    return;
                }
            }
            RemovePlayerOptions::Destroy => {
                if let Some(mut identity) = NetworkServerStatic::get_static_spawned_network_identities().get_mut(&conn.net_id()) {
                    Self::destroy(&mut identity);
                } else {
                    error!(format!("Server.RemovePlayer: netId {} not found in spawned.", conn.net_id()));
                    return;
                }
            }
        }
        conn.set_net_id(0);
    }

    // SendChangeOwnerMessage(
    pub fn send_change_owner_message(identity: &mut NetworkIdentity, conn: &mut NetworkConnectionToClient) {
        // 如果 net_id 为 0
        if identity.net_id() == 0 {
            return;
        }

        // 如果连接没有观察者
        if !conn.observing.contains(&identity.net_id()) {
            return;
        }

        // 如果 NetworkIdentity 存在
        if identity.server_only {
            return;
        }
        let is_owner = identity.connection_to_client() == conn.connection_id();
        let is_local_player = conn.net_id() == identity.net_id() && is_owner;
        let mut change_owner_message = ChangeOwnerMessage::new(identity.net_id(), is_owner, is_local_player);
        conn.send_network_message(&mut change_owner_message, TransportChannel::Reliable);
    }

    // UnSpawn
    pub fn un_spawn(identity: &mut NetworkIdentity) {
        Self::un_spawn_internal(identity, true);
    }

    fn un_spawn_internal(identity: &mut NetworkIdentity, reset_state: bool) {
        if !NetworkServerStatic::get_static_active() {
            error!("UnSpawn: NetworkServer is not active. Cannot unspawn objects without an active server.".to_string());
            return;
        }

        // TODO aoi

        if identity.game_object().is_null() {
            warn!("UnSpawn: game object is null.".to_string());
            return;
        }

        if let Some(mut connection) = NetworkServerStatic::get_static_network_connections().get_mut(&identity.connection_to_client()) {
            connection.remove_owned_object(identity.net_id());
        }
        let net_id = identity.net_id();
        Self::send_to_observers(identity, ObjectDestroyMessage::new(net_id), TransportChannel::Reliable);

        identity.clear_observers();

        identity.on_stop_server();

        if reset_state {
            identity.reset_state();
        }
    }

    fn send_to_observers(identity: &mut NetworkIdentity, mut message: ObjectDestroyMessage, channel: TransportChannel) {
        if identity.is_null() || identity.observers.len() == 0 {
            return;
        }

        NetworkWriterPool::get_return(|writer| {
            NetworkMessages::pack(&mut message, writer);
            let segment = writer.to_array_segment();

            let max = NetworkMessages::max_message_size(channel);
            if writer.get_position() > max {
                warn!("Server.SendToObservers: message is too large to send. Consider using a higher channel or splitting the message into smaller parts.");
                return;
            }

            for i in 0..identity.observers.len() {
                let conn_id = identity.observers[i];
                if let Some(mut connection) = NetworkServerStatic::get_static_network_connections().get_mut(&conn_id) {
                    connection.send(segment, channel);
                }
            }

            // TODO NetworkDiagnostics.OnSend(message, channelId, segment.Count, identity.observers.Count);
        });
    }

    pub fn add_player_for_connection(conn_id: u64, mut player: GameObject) -> bool {
        if let Some(mut identity) = player.get_component() {
            if let Some(mut connection) = NetworkServerStatic::get_static_network_connections().get_mut(&conn_id) {
                if connection.net_id() != 0 {
                    warn!(format!("AddPlayer: connection already has a player GameObject. Please remove the current player GameObject from {}", connection.is_ready()));
                    return false;
                }
                // 设置连接的 NetworkIdentity
                connection.set_net_id(identity.net_id());
                // 修改 NetworkIdentity 的 client_owner
                identity.set_client_owner(conn_id);
            } else {
                warn!(format!("AddPlayer: connectionId {} not found in connections", conn_id));
                return false;
            }
            Self::set_client_ready(conn_id);
            Self::respawn(identity);
            return true;
        }
        warn!(format!("AddPlayer: player GameObject has no NetworkIdentity. Please add a NetworkIdentity to {:?}",1));
        false
    }

    fn spawn(identity: NetworkIdentity, conn_id: u64) {
        Self::spawn_object(identity, conn_id);
    }

    // SpawnObject(
    fn spawn_object(mut identity: NetworkIdentity, conn_id: u64) {
        if !NetworkServerStatic::get_static_active() {
            error!(format!("SpawnObject for {:?}, NetworkServer is not active. Cannot spawn objects without an active server.", identity.game_object()));
            return;
        }

        if identity.spawned_from_instantiate {
            return;
        }

        if NetworkServerStatic::get_static_spawned_network_identities().contains_key(&identity.net_id()) {
            warn!(format!("SpawnObject for {:?}, netId {} already exists. Use UnSpawnObject first.", identity.game_object(), identity.net_id()));
            return;
        }

        // 如果 identity 的 net_id 为 0
        if identity.net_id() == 0 {
            /// 必须先分配 NetworkIdentity 的 net_id 再设置连接的 NetworkIdentity

            // 分配 NetworkIdentity 的 net_id
            identity.set_net_id(NetworkIdentity::get_static_next_network_id());

            // 设置连接的 NetworkIdentity
            identity.set_connection_to_client(conn_id);

            // identity.on_start_server();
            identity.on_start_server();

            // 重建观察者
            Self::rebuild_observers(&mut identity, true);

            // 添加到 SPAWNED 中
            NetworkServerStatic::get_static_spawned_network_identities().insert(identity.net_id(), identity);

            return;
        }

        // TODO aoi
        Self::rebuild_observers(&mut identity, true);
    }

    fn rebuild_observers(identity: &mut NetworkIdentity, initialize: bool) {
        // TODO aoi
        if "aoi" == "aoi" || identity.visibility == ForceShown {
            Self::rebuild_observers_default(identity, initialize);
        } else {
            // TODO aoi
        }
    }

    fn rebuild_observers_default(identity: &mut NetworkIdentity, initialize: bool) {
        if initialize {
            if identity.visibility != Visibility::ForceHidden {
                Self::add_all_ready_server_connections_to_observers(identity);
            } else if identity.connection_to_client() != 0 {
                // force hidden, but add owner connection
                identity.add_observer(identity.connection_to_client());
            }
        }
    }

    fn add_all_ready_server_connections_to_observers(identity: &mut NetworkIdentity) {
        let mut conn_ids = Vec::new();
        NetworkServerStatic::get_static_network_connections().iter_mut().for_each(|mut connection| {
            if connection.is_ready() {
                conn_ids.push(connection.connection_id());
            }
        });
        for conn_id in conn_ids {
            identity.add_observer(conn_id);
        }
    }

    fn respawn(mut identity: NetworkIdentity) {
        if identity.net_id() == 0 {
            let conn_id = identity.connection_to_client();
            Self::spawn(identity, conn_id);
        } else {
            // 找到连接
            if let Some(mut connection) = NetworkServerStatic::get_static_network_connections().get_mut(&identity.connection_to_client()) {
                Self::send_spawn_message(&mut identity, &mut connection);
            }
        }
    }

    // 处理 TransportError 消息
    fn on_transport_error(connection_id: u64, transport_error: TransportError) {
        if let Some(mut connection) = NetworkServerStatic::get_static_network_connections().get_mut(&connection_id) {
            if let Some(on_error_event) = NetworkServerStatic::get_connected_event().get(&EventHandlerType::OnErrorEvent) {
                on_error_event(&mut connection, transport_error);
            }
        }
    }

    // 处理 ServerTransportException 消息
    fn on_transport_exception(connection_id: u64, transport_error: TransportError) {
        warn!(format!("Server.HandleTransportException: connectionId: {}, error: {:?}", connection_id, transport_error));
        if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
            if let Some(on_exception_event) = NetworkServerStatic::get_connected_event().get(&EventHandlerType::OnTransportExceptionEvent) {
                on_exception_event(&mut connection, transport_error);
            }
        }
    }


    // 处理 Connected 消息
    fn on_connected(mut conn: NetworkConnectionToClient) {
        // 如果有 OnConnectedEvent
        if let Some(on_connected_event) = NetworkServerStatic::get_connected_event().get(&EventHandlerType::OnConnectedEvent) {
            on_connected_event(&mut conn, TransportError::None);
        } else {
            warn!("OnConnectedEvent is null");
        }
        // 添加连接 到 NETWORK_CONNECTIONS
        NetworkServerStatic::static_network_connections_add_connection(conn);
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
    fn on_client_ready_message(connection_id: u64, reader: &mut NetworkReader, channel: TransportChannel) {
        let _ = channel;
        let _ = ReadyMessage::deserialize(reader);
        Self::set_client_ready(connection_id);
    }
    // 设置客户端准备就绪
    pub fn set_client_ready(conn_id: u64) {
        if let Some(mut connection) = NetworkServerStatic::get_static_network_connections().get_mut(&conn_id) {
            connection.set_ready(true);
        }
        // 为连接生成观察者
        Self::spawn_observers_for_connection(conn_id);
    }
    // 为连接生成观察者
    fn spawn_observers_for_connection(conn_id: u64) {
        // 发送 ObjectSpawnStartedMessage 消息
        if let Some(mut connection) = NetworkServerStatic::get_static_network_connections().get_mut(&conn_id) {
            if !connection.is_ready() {
                return;
            }
            connection.send_network_message(&mut ObjectSpawnStartedMessage::new(), TransportChannel::Reliable);
        }
        // add connection to each nearby NetworkIdentity's observers, which
        // internally sends a spawn message for each one to the connection.
        NetworkServerStatic::get_static_spawned_network_identities().iter_mut().for_each(|mut identity| {
            if identity.visibility == ForceShown {
                identity.add_observer(conn_id);
            } else if identity.visibility == Visibility::ForceHidden {
                // do nothing
            } else if identity.visibility == Visibility::Default {
                // TODO aoi system
                identity.add_observer(conn_id);
            }
        });

        // 发送 ObjectSpawnFinishedMessage 消息
        if let Some(mut connection) = NetworkServerStatic::get_static_network_connections().get_mut(&conn_id) {
            connection.send_network_message(&mut ObjectSpawnFinishedMessage::new(), TransportChannel::Reliable);
        }
    }

    // 处理 OnCommandMessage 消息
    fn on_command_message(connection_id: u64, reader: &mut NetworkReader, channel: TransportChannel) {
        let message = CommandMessage::deserialize(reader);

        if let Some(mut connection) = NetworkServerStatic::get_static_network_connections().get_mut(&connection_id) {
            // connection 没有准备好
            if !connection.is_ready() {
                // 如果 channel 是 Reliable
                if channel == TransportChannel::Reliable {
                    let mut network_behaviours_len = u32::MAX;
                    // 如果 SPAWNED 中有 message.net_id
                    if let Some(identity) = NetworkServerStatic::get_static_spawned_network_identities().get(&message.net_id) {
                        // 如果 message.component_index 小于 net_identity.network_behaviours.len()
                        if (message.component_index as usize) < identity.network_behaviours.len() {
                            // 如果 message.function_hash 在 RemoteProcedureCalls 中
                            if let Some(method_name) = RemoteProcedureCalls::get_function_method_name(message.function_hash) {
                                warn!(format!("Command {} received for {} [netId={}] component  [index={}] when client not ready.\nThis may be ignored if client intentionally set NotReady.", method_name, identity.net_id(), message.net_id, message.component_index));
                                return;
                            }
                        }
                    }
                    warn!("Command received while client is not ready. This may be ignored if client intentionally set NotReady.".to_string());
                }
                return;
            }
        }

        if let Some(mut identity) = NetworkServerStatic::get_static_spawned_network_identities().get_mut(&message.net_id) {
            // 是否需要权限
            let requires_authority = RemoteProcedureCalls::command_requires_authority(message.function_hash);
            // 如果需要权限并且 identity.connection_id_to_client != connection.connection_id
            if requires_authority && identity.connection_to_client() != connection_id {
                // Attempt to identify the component and method to narrow down the cause of the error.
                if identity.network_behaviours.len() > message.component_index as usize {
                    if let Some(method_name) = RemoteProcedureCalls::get_function_method_name(message.function_hash) {
                        warn!(format!("Command {} received for {} [netId={}] component [index={}] without authority", method_name, identity.net_id(), message.net_id,  message.component_index));
                        return;
                    }
                }
                warn!(format!("Command received for {} [netId={}] without authority", identity.net_id(), message.net_id));
                return;
            }

            NetworkReaderPool::get_with_bytes_return(message.payload, |reader| {
                identity.handle_remote_call(message.component_index, message.function_hash, RemoteCallType::Command, reader, connection_id);
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
    fn on_entity_state_message(connection_id: u64, reader: &mut NetworkReader, channel: TransportChannel) {
        let message = EntityStateMessage::deserialize(reader);
        if let Some(mut identity) = NetworkServerStatic::get_static_spawned_network_identities().get_mut(&message.net_id) {
            if identity.connection_to_client() == connection_id {
                NetworkReaderPool::get_with_bytes_return(message.payload, |reader| {
                    if !identity.deserialize_server(reader) {
                        if NetworkServerStatic::get_static_exceptions_disconnect() {
                            error!(format!("Server failed to deserialize client state for {} with netId={}, Disconnecting.", identity.net_id(), identity.net_id()));
                            if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
                                connection.disconnect();
                            }
                        } else {
                            warn!(format!("Server failed to deserialize client state for {} with netId={}", identity.net_id(), identity.net_id()));
                        }
                    }
                });
            } else {
                warn!(format!("EntityStateMessage from {} for {} without authority.", connection_id, identity.net_id()));
            }
        }
    }

    // 处理 OnTimeSnapshotMessage 消息
    fn on_time_snapshot_message(connection_id: u64, reader: &mut NetworkReader, channel: TransportChannel) {
        if let Some(mut connection) = NETWORK_CONNECTIONS.get_mut(&connection_id) {
            let snapshot = TimeSnapshot::new(connection.remote_time_stamp(), NetworkTime::local_time());
            connection.on_time_snapshot(snapshot);
        }
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
    // 定义一个函数来替换处理程序
    pub fn replace_handler<T>(network_message_handler: NetworkMessageHandlerFunc, require_authentication: bool)
    where
        T: NetworkMessageReader + Send + Sync + 'static,
    {
        let hash_code = T::get_hash_code();
        NETWORK_MESSAGE_HANDLERS.insert(hash_code, NetworkMessageHandler::wrap_handler(network_message_handler, require_authentication));
    }
}
