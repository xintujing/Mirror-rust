use crate::mirror::core::backend_data::BackendDataStatic;
use crate::mirror::core::batching::un_batcher::UnBatcher;
use crate::mirror::core::messages::{
    ChangeOwnerMessage, CommandMessage, EntityStateMessage, NetworkMessageHandler,
    NetworkMessageHandlerFunc, NetworkMessageTrait, NetworkPingMessage, NetworkPongMessage,
    NotReadyMessage, ObjectDestroyMessage, ObjectHideMessage, ObjectSpawnFinishedMessage,
    ObjectSpawnStartedMessage, ReadyMessage, SpawnMessage, TimeSnapshotMessage,
};
use crate::mirror::core::network_behaviour::{GameObject, NetworkBehaviourTrait};
use crate::mirror::core::network_connection::NetworkConnectionTrait;
use crate::mirror::core::network_connection_to_client::NetworkConnectionToClient;
use crate::mirror::core::network_identity::Visibility::ForceShown;
use crate::mirror::core::network_identity::{NetworkIdentity, Visibility};
use crate::mirror::core::network_manager::NetworkManagerStatic;
use crate::mirror::core::network_messages::NetworkMessages;
use crate::mirror::core::network_reader::NetworkReader;
use crate::mirror::core::network_reader_pool::NetworkReaderPool;
use crate::mirror::core::network_time::NetworkTime;
use crate::mirror::core::network_writer_pool::NetworkWriterPool;
use crate::mirror::core::remote_calls::{RemoteCallType, RemoteProcedureCalls};
use crate::mirror::core::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::mirror::core::tools::time_sample::TimeSample;
use crate::mirror::core::transport::{
    Transport, TransportCallback, TransportCallbackType, TransportChannel, TransportError,
};
use crate::{log_debug, log_error, log_info, log_warn};
use atomic::Atomic;
use dashmap::mapref::multiple::RefMutMulti;
use dashmap::try_result::TryResult;
use dashmap::{DashMap, DashSet};
use lazy_static::lazy_static;
use std::fmt::Debug;
use std::sync::atomic::Ordering;
use std::sync::RwLock;

pub enum ReplacePlayerOptions {
    KeepAuthority,
    KeepActive,
    UnSpawn,
    Destroy,
}

pub enum RemovePlayerOptions {
    // <summary>Player Object remains active on server and clients. Only ownership is removed</summary>
    KeepActive,
    // <summary>Player Object is un_spawned on clients but remains on server</summary>
    UnSpawn,
    // <summary>Player Object is destroyed on server and clients</summary>
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
    static ref TICK_INTERVAL: Atomic<f32> =
        Atomic::new(1f32 / NetworkServerStatic::tick_rate() as f32);
    static ref SEND_RATE: Atomic<u32> = Atomic::new(NetworkServerStatic::tick_rate());
    static ref SEND_INTERVAL: Atomic<f32> =
        Atomic::new(1f32 / NetworkServerStatic::send_rate() as f32);
    static ref LAST_SEND_TIME: Atomic<f64> = Atomic::new(0.0);
    static ref DONT_LISTEN: Atomic<bool> = Atomic::new(true);
    static ref ACTIVE: Atomic<bool> = Atomic::new(false);
    static ref IS_LOADING_SCENE: Atomic<bool> = Atomic::new(false);
    static ref EXCEPTIONS_DISCONNECT: Atomic<bool> = Atomic::new(false);
    static ref DISCONNECT_INACTIVE_CONNECTIONS: Atomic<bool> = Atomic::new(false);
    static ref DISCONNECT_INACTIVE_TIMEOUT: Atomic<f32> = Atomic::new(10.0);
    static ref ACTUAL_TICK_RATE: Atomic<u32> = Atomic::new(0);
    static ref ACTUAL_TICK_RATE_START: Atomic<f64> = Atomic::new(0.0);
    static ref ACTUAL_TICK_RATE_COUNTER: Atomic<u32> = Atomic::new(0);
    static ref MAX_CONNECTIONS: Atomic<usize> = Atomic::new(0);
    static ref EARLY_UPDATE_DURATION: RwLock<TimeSample> = RwLock::new(TimeSample::new(0));
    static ref LATE_UPDATE_DURATION: RwLock<TimeSample> = RwLock::new(TimeSample::new(0));
    static ref FULL_UPDATE_DURATION: RwLock<TimeSample> = RwLock::new(TimeSample::new(0));
    static ref NETWORK_CONNECTIONS: DashMap<u64, NetworkConnectionToClient> = DashMap::new();
    static ref SPAWNED_NETWORK_IDS: DashSet<u32> = DashSet::new();
    static ref SPAWNED_NETWORK_IDENTITIES: DashMap<u32, NetworkIdentity> = DashMap::new();
    pub static ref NETWORK_BEHAVIOURS: DashMap<String, Box<dyn NetworkBehaviourTrait>> =
        DashMap::new();
    static ref NETWORK_MESSAGE_HANDLERS: DashMap<u16, NetworkMessageHandler> = DashMap::new();
    static ref TRANSPORT_DATA_UN_BATCHER: RwLock<UnBatcher> = RwLock::new(UnBatcher::new());
}

// Box<dyn NetworkBehaviourTrait> 静态变量方法
impl NETWORK_BEHAVIOURS {
    // 添加 NetworkBehaviour
    pub fn add_behaviour(net_id: u32, index: u8, behaviour: Box<dyn NetworkBehaviourTrait>) {
        NETWORK_BEHAVIOURS.insert(format!("{}_{}", net_id, index), behaviour);
    }
    // 更新 NetworkBehaviour 的 NetId
    pub fn update_behaviour_net_id(o_net_id: u32, n_net_id: u32, count: u8) {
        for i in 0..count {
            let o_key = format!("{}_{}", o_net_id, i);
            let n_key = format!("{}_{}", n_net_id, i);
            if let Some((_, mut behaviour)) = NETWORK_BEHAVIOURS.remove(&o_key) {
                behaviour.set_net_id(n_net_id);
                NETWORK_BEHAVIOURS.insert(n_key, behaviour);
            }
        }
    }
    // 更新 NetworkBehaviour 的 ConnectionId
    pub fn update_behaviour_conn_id(net_id: u32, conn_id: u64, count: u8) {
        for i in 0..count {
            if let Some((_, mut behaviour)) =
                NETWORK_BEHAVIOURS.remove(&format!("{}_{}", net_id, i))
            {
                behaviour.set_connection_to_client(conn_id);
                NETWORK_BEHAVIOURS.insert(format!("{}_{}", net_id, i), behaviour);
            }
        }
    }
    // 移除 NetworkBehaviour
    pub fn remove_behaviour(net_id: u32, count: u8) {
        for i in 0..count {
            NETWORK_BEHAVIOURS.remove(&format!("{}_{}", net_id, i));
        }
    }
}

// NetworkServer 静态结构体
pub struct NetworkServerStatic;
// NetworkServer 静态结构体方法
impl NetworkServerStatic {
    pub fn exceptions_disconnect() -> bool {
        EXCEPTIONS_DISCONNECT.load(Ordering::Relaxed)
    }
    pub fn set_exceptions_disconnect(value: bool) {
        EXCEPTIONS_DISCONNECT.store(value, Ordering::Relaxed);
    }
    pub fn connected_event() -> &'static DashMap<EventHandlerType, Box<EventHandler>> {
        &CONNECTED_EVENT
    }
    pub fn initialized() -> bool {
        Initialized.load(Ordering::Relaxed)
    }
    pub fn set_initialized(value: bool) {
        Initialized.store(value, Ordering::Relaxed);
    }
    pub fn max_connections() -> usize {
        MAX_CONNECTIONS.load(Ordering::Relaxed)
    }
    pub fn set_max_connections(value: usize) {
        MAX_CONNECTIONS.store(value, Ordering::Relaxed);
    }
    pub fn network_connections_size() -> usize {
        NETWORK_CONNECTIONS.len()
    }
    pub fn send_rate() -> u32 {
        SEND_RATE.load(Ordering::Relaxed)
    }
    pub fn set_send_rate(value: u32) {
        SEND_RATE.store(value, Ordering::Relaxed);
        Self::set_send_interval(1f32 / value as f32);
    }
    pub fn send_interval() -> f32 {
        SEND_INTERVAL.load(Ordering::Relaxed)
    }
    pub fn set_send_interval(value: f32) {
        SEND_INTERVAL.store(value, Ordering::Relaxed);
    }
    pub fn dont_listen() -> bool {
        DONT_LISTEN.load(Ordering::Relaxed)
    }
    pub fn set_dont_listen(value: bool) {
        DONT_LISTEN.store(value, Ordering::Relaxed);
    }
    pub fn active() -> bool {
        ACTIVE.load(Ordering::Relaxed)
    }
    pub fn set_active(value: bool) {
        ACTIVE.store(value, Ordering::Relaxed);
    }
    pub fn is_loading_scene() -> bool {
        IS_LOADING_SCENE.load(Ordering::Relaxed)
    }
    pub fn set_is_loading_scene(value: bool) {
        IS_LOADING_SCENE.store(value, Ordering::Relaxed);
    }
    pub fn disconnect_inactive_connections() -> bool {
        DISCONNECT_INACTIVE_CONNECTIONS.load(Ordering::Relaxed)
    }
    pub fn set_disconnect_inactive_connections(value: bool) {
        DISCONNECT_INACTIVE_CONNECTIONS.store(value, Ordering::Relaxed);
    }
    pub fn disconnect_inactive_timeout() -> f32 {
        DISCONNECT_INACTIVE_TIMEOUT.load(Ordering::Relaxed)
    }
    pub fn set_disconnect_inactive_timeout(value: f32) {
        DISCONNECT_INACTIVE_TIMEOUT.store(value, Ordering::Relaxed);
    }
    pub fn actual_tick_rate() -> u32 {
        ACTUAL_TICK_RATE.load(Ordering::Relaxed)
    }
    pub fn set_actual_tick_rate(value: u32) {
        ACTUAL_TICK_RATE.store(value, Ordering::Relaxed);
    }
    pub fn actual_tick_rate_start() -> f64 {
        ACTUAL_TICK_RATE_START.load(Ordering::Relaxed)
    }
    pub fn set_actual_tick_rate_start(value: f64) {
        ACTUAL_TICK_RATE_START.store(value, Ordering::Relaxed);
    }
    pub fn actual_tick_rate_counter() -> u32 {
        ACTUAL_TICK_RATE_COUNTER.load(Ordering::Relaxed)
    }
    pub fn set_actual_tick_rate_counter(value: u32) {
        ACTUAL_TICK_RATE_COUNTER.store(value, Ordering::Relaxed);
    }
    pub fn tick_rate() -> u32 {
        TickRate.load(Ordering::Relaxed)
    }
    pub fn set_tick_rate(value: u32) {
        TickRate.store(value, Ordering::Relaxed);
        Self::set_tick_interval(1f32 / value as f32);
        Self::set_send_rate(value);
    }
    pub fn tick_interval() -> f32 {
        TICK_INTERVAL.load(Ordering::Relaxed)
    }
    pub fn set_tick_interval(value: f32) {
        TICK_INTERVAL.store(value, Ordering::Relaxed);
    }
    pub fn last_send_time() -> f64 {
        LAST_SEND_TIME.load(Ordering::Relaxed)
    }
    pub fn set_last_send_time(value: f64) {
        LAST_SEND_TIME.store(value, Ordering::Relaxed);
    }
    pub fn set_early_update_duration(value: TimeSample) {
        if let Ok(mut early_update_duration) = EARLY_UPDATE_DURATION.write() {
            *early_update_duration = value;
        }
    }
    pub fn set_full_update_duration(value: TimeSample) {
        if let Ok(mut full_update_duration) = FULL_UPDATE_DURATION.write() {
            *full_update_duration = value;
        }
    }
    pub fn set_late_update_duration(value: TimeSample) {
        if let Ok(mut late_update_duration) = LATE_UPDATE_DURATION.write() {
            *late_update_duration = value;
        }
    }
    // FULL_UPDATE_DURATION
    pub fn full_update_duration() -> &'static RwLock<TimeSample> {
        &FULL_UPDATE_DURATION
    }
    // TRANSPORT_DATA_UN_BATCHER
    fn transport_data_un_batcher() -> &'static RwLock<UnBatcher> {
        &TRANSPORT_DATA_UN_BATCHER
    }
    // 获取 NetworkConnections
    pub fn network_connections() -> &'static DashMap<u64, NetworkConnectionToClient> {
        &NETWORK_CONNECTIONS
    }
    // 添加连接
    fn add_network_connection(connection: NetworkConnectionToClient) -> bool {
        if NETWORK_CONNECTIONS.contains_key(&connection.connection_id()) {
            return false;
        }
        NETWORK_CONNECTIONS.insert(connection.connection_id(), connection);
        true
    }
    pub fn spawned_network_ids() -> &'static DashSet<u32> {
        &SPAWNED_NETWORK_IDS
    }
    pub fn spawned_network_identities() -> &'static DashMap<u32, NetworkIdentity> {
        &SPAWNED_NETWORK_IDENTITIES
    }
    pub fn add_spawned_network_identity(identity: NetworkIdentity) {
        Self::spawned_network_ids().insert(identity.net_id());
        SPAWNED_NETWORK_IDENTITIES.insert(identity.net_id(), identity);
    }
    pub fn remove_spawned_network_identity(net_id: &u32) {
        if let Some((net_id, sni)) = SPAWNED_NETWORK_IDENTITIES.remove(net_id) {
            for i in 0..sni.network_behaviours_count {
                NETWORK_BEHAVIOURS.remove(&format!("{}_{}", net_id, i));
            }
        }
        SPAWNED_NETWORK_IDS.remove(net_id);
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
        if NetworkServerStatic::initialized() {
            return;
        }

        //Make sure connections are cleared in case any old connections references to exist from previous sessions
        NetworkServerStatic::network_connections().clear();

        // TODO: if (aoi != null) aoi.reset_state();

        NetworkTime::reset_statics();

        // 设置 TransportCallback
        if let Some(transport) = Transport::active_transport() {
            transport.set_transport_cb_fn(Self::transport_callback);
        }

        // NetworkServer 是初始化的
        if NetworkServerStatic::initialized() {
            return;
        }

        NetworkServerStatic::set_initialized(true);

        NetworkServerStatic::set_early_update_duration(TimeSample::new(
            NetworkServerStatic::send_rate(),
        ));
        NetworkServerStatic::set_full_update_duration(TimeSample::new(
            NetworkServerStatic::send_rate(),
        ));
        NetworkServerStatic::set_late_update_duration(TimeSample::new(
            NetworkServerStatic::send_rate(),
        ));
    }
    pub fn listen(max_connections: usize) {
        // 初始化
        Self::initialize();

        // 设置最大连接数
        NetworkServerStatic::set_max_connections(max_connections);

        // 如果不监听
        if NetworkServerStatic::dont_listen() {
            if let Some(transport) = Transport::active_transport() {
                transport.server_start();
            }
        }
        // 设置 NetworkServer 为激活状态
        NetworkServerStatic::set_active(true);

        // 注册消息处理器
        Self::register_message_handlers();
    }

    pub fn shutdown() {
        if NetworkServerStatic::initialized() {
            Self::disconnect_all();

            if let Some(transport) = Transport::active_transport() {
                transport.server_stop();
            }

            NetworkServerStatic::set_active(false);
            NetworkServerStatic::set_initialized(false);
        }
        NETWORK_MESSAGE_HANDLERS.clear();
        NetworkServerStatic::network_connections().clear();
        NetworkServerStatic::spawned_network_ids().clear();
        NetworkServerStatic::spawned_network_identities().clear();
        NetworkServerStatic::transport_data_un_batcher()
            .write()
            .unwrap()
            .clear();
    }

    fn disconnect_all() {
        NetworkServerStatic::for_each_network_connection(|mut connection| {
            connection.disconnect();
        });
    }

    // 网络早期更新
    pub fn network_early_update() {
        if NetworkServerStatic::active() {
            match EARLY_UPDATE_DURATION.try_write() {
                Ok(mut early_update_duration) => {
                    early_update_duration.begin();
                }
                Err(e) => {
                    log_warn!(format!(
                        "Server.network_early_update() failed to get EARLY_UPDATE_DURATION: {:?}",
                        e
                    ));
                }
            }
            match FULL_UPDATE_DURATION.try_write() {
                Ok(mut full_update_duration) => {
                    full_update_duration.begin();
                }
                Err(e) => {
                    log_warn!(format!(
                        "Server.network_early_update() failed to get FULL_UPDATE_DURATION: {:?}",
                        e
                    ));
                }
            }
        }

        if let Some(active_transport) = Transport::active_transport() {
            active_transport.server_early_update();
        }

        //  step each connection's local time interpolation in early update. 1969
        NetworkServerStatic::for_each_network_connection(|mut connection| {
            connection.update_time_interpolation();
        });

        if NetworkServerStatic::active() {
            match EARLY_UPDATE_DURATION.try_write() {
                Ok(mut early_update_duration) => {
                    early_update_duration.end();
                }
                Err(e) => {
                    log_warn!(format!(
                        "Server.network_early_update() failed to get EARLY_UPDATE_DURATION: {:?}",
                        e
                    ));
                }
            }
        }
    }

    // 网络更新
    pub fn network_late_update() {
        if NetworkServerStatic::active() {
            match LATE_UPDATE_DURATION.try_write() {
                Ok(mut late_update_duration) => {
                    late_update_duration.begin();
                }
                Err(e) => {
                    log_warn!(format!(
                        "Server.network_late_update() failed to get LATE_UPDATE_DURATION: {:?}",
                        e
                    ));
                }
            }
            Self::broadcast();
        }
        if let Some(active_transport) = Transport::active_transport() {
            active_transport.server_late_update();
        }

        if NetworkServerStatic::active() {
            let actual_tick_rate_counter = NetworkServerStatic::actual_tick_rate_counter();
            NetworkServerStatic::set_actual_tick_rate_counter(actual_tick_rate_counter + 1);

            let local_time = NetworkTime::local_time();

            if local_time - NetworkServerStatic::actual_tick_rate_start() >= 1.0 {
                let elapsed = local_time - NetworkServerStatic::actual_tick_rate_start();
                let actual_tick_rate_counter = NetworkServerStatic::actual_tick_rate_counter();
                NetworkServerStatic::set_actual_tick_rate(
                    actual_tick_rate_counter / elapsed as u32,
                );
                NetworkServerStatic::set_actual_tick_rate_start(local_time);
                NetworkServerStatic::set_actual_tick_rate_counter(0);
            }

            match LATE_UPDATE_DURATION.try_write() {
                Ok(mut late_update_duration) => {
                    late_update_duration.end();
                }
                Err(e) => {
                    log_warn!(format!(
                        "Server.network_late_update() failed to get LATE_UPDATE_DURATION: {:?}",
                        e
                    ));
                }
            }
            match FULL_UPDATE_DURATION.try_write() {
                Ok(mut full_update_duration) => {
                    full_update_duration.end();
                }
                Err(e) => {
                    log_warn!(format!(
                        "Server.network_late_update() failed to get FULL_UPDATE_DURATION: {:?}",
                        e
                    ));
                }
            }
        }
    }

    // Broadcast
    fn broadcast() {
        NetworkServerStatic::for_each_network_connection(|mut connection| {
            // 如果连接不活跃
            if Self::disconnect_if_inactive(&mut connection) {
                return;
            }

            // 如果连接没有认证并且没有准备好
            if Self::disconnect_if_no_auth_not_ready(&mut connection) {
                return;
            }

            if connection.is_ready() {
                connection.send_network_message(
                    &mut TimeSnapshotMessage::default(),
                    TransportChannel::Unreliable,
                );
                Self::broadcast_to_connection(&mut connection);
            }
            connection.update();
        });
    }

    // BroadcastToConnection(NetworkConnectionToClient connection)
    fn broadcast_to_connection(conn: &mut NetworkConnectionToClient) {
        for net_id in conn.observing.to_vec().iter() {
            if *net_id != 0 {
                if let Some(mut message) =
                    Self::serialize_for_connection(*net_id, conn.connection_id())
                {
                    // debug!(format!("Server.broadcast_to_connection: connectionId: {}, netId: {}", conn.connection_id(), net_id));
                    conn.send_network_message(&mut message, TransportChannel::Reliable);
                }
            } else {
                log_warn!(format!("Server.broadcast_to_connection: identity is null. Removing from observing list. connectionId: {}, netId: {}", conn.connection_id(), net_id));
                conn.observing.retain(|id| id != net_id);
            }
        }
    }

    // SerializeForConnection
    fn serialize_for_connection(net_id: u32, conn_id: u64) -> Option<EntityStateMessage> {
        match NetworkServerStatic::spawned_network_identities().try_get_mut(&net_id) {
            TryResult::Present(mut identity) => {
                let owned = identity.connection_to_client() == conn_id;
                let net_id = identity.net_id();
                let serialization =
                    identity.get_server_serialization_at_tick(NetworkTime::frame_count());
                match owned {
                    true => {
                        if serialization.owner_writer.get_position() > 0 {
                            return Some(EntityStateMessage::new(
                                net_id,
                                serialization.owner_writer.to_bytes(),
                            ));
                        }
                    }
                    false => {
                        if serialization.observers_writer.get_position() > 0 {
                            return Some(EntityStateMessage::new(
                                net_id,
                                serialization.observers_writer.to_bytes(),
                            ));
                        }
                    }
                }
            }
            TryResult::Absent => {
                log_warn!(format!(
                    "Server.SerializeForConnection: netId {} not found in spawned.",
                    net_id
                ));
            }
            TryResult::Locked => {
                log_warn!(format!(
                    "Server.SerializeForConnection: netId {} is locked.",
                    net_id
                ));
            }
        }
        None
    }

    // DisconnectIfInactive
    fn disconnect_if_inactive(connection: &mut NetworkConnectionToClient) -> bool {
        if NetworkServerStatic::disconnect_inactive_connections()
            && !connection.is_alive(NetworkServerStatic::disconnect_inactive_timeout() as f64)
        {
            log_warn!(format!(
                "Server.DisconnectIfInactive: connectionId: {} is inactive. Disconnecting.",
                connection.connection_id()
            ));
            connection.disconnect();
            return true;
        }
        false
    }

    // 关闭5 秒没有认证（链接认证）并且没有准备（链接准备，非游戏准备）好的连接
    fn disconnect_if_no_auth_not_ready(connection: &mut NetworkConnectionToClient) -> bool {
        if !connection.is_authenticated()
            && !connection.is_ready()
            && NetworkTime::local_time() - connection.first_conn_loc_time_stamp() > 5.0
        {
            log_warn!(format!(
                "Server.DisconnectIfNoAuthNotReady: connectionId: {} is not authenticated and not ready. Disconnecting.",
                connection.connection_id()
            ));
            connection.disconnect();
            return true;
        }
        false
    }

    // show / hide for connection //////////////////////////////////////////
    pub fn show_for_connection(
        identity: &mut NetworkIdentity,
        conn: &mut NetworkConnectionToClient,
    ) {
        if conn.is_ready() {
            Self::send_spawn_message(identity, conn);
        }
    }

    pub fn hide_for_connection(
        conn: &mut NetworkConnectionToClient,
        identity: &mut NetworkIdentity,
    ) {
        if conn.is_ready() {
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
        let mut spawn_message = SpawnMessage::new(
            identity.net_id(),
            is_local_player,
            is_owner,
            identity.scene_id,
            identity.asset_id,
            identity.game_object().transform.local_position,
            identity.game_object().transform.local_rotation,
            identity.game_object().transform.local_scale,
            payload,
        );
        // 发送 SpawnMessage
        conn.send_network_message(&mut spawn_message, TransportChannel::Reliable);
    }

    fn create_spawn_message_payload(is_owner: bool, identity: &mut NetworkIdentity) -> Vec<u8> {
        let mut payload = Vec::new();
        // 如果没有 NetworkBehaviours
        if identity.network_behaviours_count == 0 {
            return payload;
        }

        NetworkWriterPool::get_return(|owner_writer| {
            NetworkWriterPool::get_return(|observers_writer| {
                // 序列化 NetworkIdentity
                identity.serialize_server(true, owner_writer, observers_writer);
                // 如果是所有者
                if is_owner {
                    payload = owner_writer.to_bytes();
                } else {
                    // 如果不是所有者
                    payload = observers_writer.to_bytes();
                }
            });
        });
        payload
    }

    // 处理 TransportCallback   AddTransportHandlers(
    fn transport_callback(tcb: TransportCallback) {
        match tcb.r#type {
            TransportCallbackType::OnServerConnected => {
                log_info!(format!(
                    "Server.HandleConnect: connectionId: {}",
                    tcb.conn_id
                ));
                Self::on_transport_connected(tcb.conn_id)
            }
            TransportCallbackType::OnServerDataReceived => {
                Self::on_transport_data(tcb.conn_id, tcb.data, tcb.channel)
            }
            TransportCallbackType::OnServerDisconnected => {
                log_info!(format!(
                    "Server.HandleDisconnect: connectionId: {}",
                    tcb.conn_id
                ));
                Self::on_transport_disconnected(tcb.conn_id)
            }
            TransportCallbackType::OnServerError => {
                log_error!(format!(
                    "Server.HandleError: connectionId: {} error: {:?}",
                    tcb.conn_id, tcb.error
                ));
                Self::on_transport_error(tcb.conn_id, tcb.error)
            }
            TransportCallbackType::OnServerTransportException => {
                log_error!(format!(
                    "Server.HandleTransportException: connectionId: {} error: {:?}",
                    tcb.conn_id, tcb.error
                ));
                Self::on_transport_exception(tcb.conn_id, tcb.error)
            }
            TransportCallbackType::OnServerDataSent => {}
        }
    }

    // 处理 TransportConnected 消息
    fn on_transport_connected(connection_id: u64) {
        if connection_id == 0 {
            log_error!(format!("Server.HandleConnect: invalid connectionId: {}. Needs to be != 0, because 0 is reserved for local player.", connection_id));
            if let Some(transport) = Transport::active_transport() {
                transport.server_disconnect(connection_id);
            }
            return;
        }

        if NetworkServerStatic::network_connections().contains_key(&connection_id) {
            log_error!(format!(
                "Server.HandleConnect: connectionId {} already exists.",
                connection_id
            ));
            if let Some(transport) = Transport::active_transport() {
                transport.server_disconnect(connection_id);
            }
            return;
        }

        if NetworkServerStatic::network_connections_size() >= NetworkServerStatic::max_connections()
        {
            log_error!(format!(
                "Server.HandleConnect: max_connections reached: {}. Disconnecting connectionId: {}",
                NetworkServerStatic::max_connections(),
                connection_id
            ));
            if let Some(transport) = Transport::active_transport() {
                transport.server_disconnect(connection_id);
            }
            return;
        }
        let connection = NetworkConnectionToClient::new(connection_id);
        Self::on_connected(connection);
    }

    // 处理 TransportData 消息
    fn on_transport_data(connection_id: u64, data: Vec<u8>, channel: TransportChannel) {
        // 获取 transport_data_un_batcher
        if let Ok(mut transport_data_un_batcher) =
            NetworkServerStatic::transport_data_un_batcher().write()
        {
            match NetworkServerStatic::network_connections().try_get_mut(&connection_id) {
                // 如果有连接
                TryResult::Present(mut connection) => {
                    // 添加数据到 transport_data_un_batcher
                    if !transport_data_un_batcher.add_batch_with_bytes(data) {
                        if NetworkServerStatic::exceptions_disconnect() {
                            log_error!(format!(
                            "Server.HandleData: connectionId: {} failed to add un_batch. Disconnecting.",
                            connection_id
                        ));
                            connection.disconnect();
                            return;
                        }
                        log_warn!(format!(
                            "Server.HandleData: connectionId: {} failed to add un_batch.",
                            connection_id
                        ));
                        return;
                    }
                }
                // 如果没有找到连接
                TryResult::Absent => {
                    log_error!(format!(
                        "Server.HandleData: connectionId: {} not found.",
                        connection_id
                    ));
                    return;
                }
                TryResult::Locked => {
                    log_error!(format!(
                        "Server.HandleData: connectionId: {} is locked.",
                        connection_id
                    ));
                    return;
                }
            }

            // 如果正在加载场景
            if NetworkServerStatic::is_loading_scene() {
                log_error!(format!(
                    "Server.HandleData: connectionId: {} is loading scene. Ignoring message.",
                    connection_id
                ));
                return;
            }

            // 处理消息
            while let Some((message, remote_time_stamp)) =
                transport_data_un_batcher.get_next_message()
            {
                NetworkReaderPool::get_with_array_segment_return(&message, |reader| {
                    match reader.remaining() >= NetworkMessages::ID_SIZE {
                        // 如果消息长度大于 NetworkMessages::ID_SIZE
                        true => {
                            // 更新远程时间戳
                            match NetworkServerStatic::network_connections()
                                .try_get_mut(&connection_id)
                            {
                                TryResult::Present(mut connection) => {
                                    connection.set_remote_time_stamp(remote_time_stamp);
                                }
                                TryResult::Absent => {
                                    log_error!(format!(
                                        "Server.HandleData: connectionId: {} not found.",
                                        connection_id
                                    ));
                                    return;
                                }
                                TryResult::Locked => {
                                    log_error!(format!(
                                        "Server.HandleData: connectionId: {} is locked.",
                                        connection_id
                                    ));
                                    return;
                                }
                            }
                            // 处理消息
                            if !Self::unpack_and_invoke(connection_id, reader, channel) {
                                if NetworkServerStatic::exceptions_disconnect() {
                                    log_error!(format!("Server.HandleData: connectionId: {} failed to unpack and invoke message. Disconnecting.", connection_id));
                                    match NetworkServerStatic::network_connections()
                                        .try_get_mut(&connection_id)
                                    {
                                        TryResult::Present(mut connection) => {
                                            connection.disconnect();
                                        }
                                        TryResult::Absent => {
                                            log_error!(format!(
                                                "Server.HandleData: connectionId: {} not found.",
                                                connection_id
                                            ));
                                        }
                                        TryResult::Locked => {
                                            log_error!(format!(
                                                "Server.HandleData: connectionId: {} is locked.",
                                                connection_id
                                            ));
                                        }
                                    }
                                } else {
                                    log_warn!(format!("Server.HandleData: connectionId: {} failed to unpack and invoke message.", connection_id));
                                }
                                return;
                            }
                        }
                        // 如果消息长度小于 NetworkMessages::ID_SIZE
                        false => {
                            if NetworkServerStatic::exceptions_disconnect() {
                                log_error!(format!("Server.HandleData: connectionId: {} message too small. Disconnecting.", connection_id));
                                match NetworkServerStatic::network_connections()
                                    .try_get_mut(&connection_id)
                                {
                                    TryResult::Present(mut connection) => {
                                        connection.disconnect();
                                    }
                                    TryResult::Absent => {
                                        log_error!(format!(
                                            "Server.HandleData: connectionId: {} not found.",
                                            connection_id
                                        ));
                                    }
                                    TryResult::Locked => {
                                        log_error!(format!(
                                            "Server.HandleData: connectionId: {} is locked.",
                                            connection_id
                                        ));
                                    }
                                }
                            } else {
                                log_warn!(format!(
                                    "Server.HandleData: connectionId: {} message too small.",
                                    connection_id
                                ));
                            }
                            return;
                        }
                    }
                });
            }

            if transport_data_un_batcher.batches_count() > 0 {
                log_error!(format!(
                    "Server.HandleData: connectionId: {} unprocessed batches: {}",
                    connection_id,
                    transport_data_un_batcher.batches_count()
                ));
            }
        }
    }

    fn unpack_and_invoke(
        connection_id: u64,
        reader: &mut NetworkReader,
        channel: TransportChannel,
    ) -> bool {
        // 解包消息id
        let message_id = NetworkMessages::unpack_id(reader);
        // 如果消息id在 NETWORK_MESSAGE_HANDLERS 中
        if let Some(handler) = NETWORK_MESSAGE_HANDLERS.get(&message_id) {
            (handler.func)(connection_id, reader, channel);
            match NetworkServerStatic::network_connections().try_get_mut(&connection_id) {
                TryResult::Present(mut connection) => {
                    connection.set_last_message_time(NetworkTime::local_time());
                }
                TryResult::Absent => {
                    log_error!(format!(
                        "Server.HandleData: connectionId: {} not found.",
                        connection_id
                    ));
                }
                TryResult::Locked => {
                    log_error!(format!(
                        "Server.HandleData: connectionId: {} is locked.",
                        connection_id
                    ));
                }
            }
            return true;
        }
        log_warn!(format!(
            "Server.HandleData: connectionId: {} unknown message id: {}",
            connection_id, message_id
        ));
        false
    }

    // 处理 TransportDisconnected 消息
    fn on_transport_disconnected(connection_id: u64) {
        if let Some((_, mut connection)) =
            NetworkServerStatic::network_connections().remove(&connection_id)
        {
            if let Some(on_disconnected_event) =
                NetworkServerStatic::connected_event().get(&EventHandlerType::OnDisconnectedEvent)
            {
                on_disconnected_event(&mut connection, TransportError::None);
            } else {
                Self::destroy_player_for_connection(&mut connection);
            }
            connection.cleanup();
        }
    }

    pub fn destroy_player_for_connection(conn: &mut NetworkConnectionToClient) {
        conn.remove_from_observings_observers();
        // 销毁 owned 对象
        conn.destroy_owned_objects();
        // 设置 net_id 为 0
        conn.set_net_id(0);
    }

    pub fn destroy(conn: &mut NetworkConnectionToClient, identity: &mut NetworkIdentity) {
        if !NetworkServerStatic::active() {
            log_error!("Server.Destroy: NetworkServer is not active. Cannot destroy objects without an active server.");
            return;
        }

        if identity.game_object().is_null() {
            log_warn!("Server.Destroy: game object is null.");
            return;
        }

        if identity.scene_id != 0 {
            Self::un_spawn_internal(conn, identity, true);
        } else {
            Self::un_spawn_internal(conn, identity, false);
            identity.destroy_called = true;
        }
    }

    pub fn remove_player_for_connection(
        conn: &mut NetworkConnectionToClient,
        options: RemovePlayerOptions,
    ) {
        if conn.net_id() == 0 {
            return;
        }

        match options {
            RemovePlayerOptions::KeepActive => {
                match NetworkServerStatic::spawned_network_identities().try_get_mut(&conn.net_id())
                {
                    TryResult::Present(mut identity) => {
                        identity.set_connection_to_client(0);
                        conn.owned().retain(|id| *id != identity.net_id());
                        Self::send_change_owner_message(&mut identity, conn);
                    }
                    TryResult::Absent => {
                        log_error!(format!(
                            "Server.RemovePlayer: netId {} not found in spawned.",
                            conn.net_id()
                        ));
                        return;
                    }
                    TryResult::Locked => {
                        log_error!(format!(
                            "Server.RemovePlayer: netId {} is locked.",
                            conn.net_id()
                        ));
                        return;
                    }
                }
            }
            RemovePlayerOptions::UnSpawn => {
                match NetworkServerStatic::spawned_network_identities().remove(&conn.net_id()) {
                    Some((_, mut identity)) => {
                        Self::un_spawn(conn, &mut identity);
                    }
                    None => {
                        log_error!(format!(
                            "Server.RemovePlayer: netId {} not found in spawned.",
                            conn.net_id()
                        ));
                        return;
                    }
                }
            }
            RemovePlayerOptions::Destroy => {
                match NetworkServerStatic::spawned_network_identities().try_get_mut(&conn.net_id())
                {
                    TryResult::Present(mut identity) => {
                        Self::destroy(conn, &mut identity);
                    }
                    TryResult::Absent => {
                        log_error!(format!(
                            "Server.RemovePlayer: netId {} not found in spawned.",
                            conn.net_id()
                        ));
                        return;
                    }
                    TryResult::Locked => {
                        log_error!(format!(
                            "Server.RemovePlayer: netId {} is locked.",
                            conn.net_id()
                        ));
                        return;
                    }
                }
            }
        }
        conn.set_net_id(0);
    }

    // SendChangeOwnerMessage(
    pub fn send_change_owner_message(
        identity: &mut NetworkIdentity,
        conn: &mut NetworkConnectionToClient,
    ) {
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
        let mut change_owner_message =
            ChangeOwnerMessage::new(identity.net_id(), is_owner, is_local_player);
        conn.send_network_message(&mut change_owner_message, TransportChannel::Reliable);
    }

    // UnSpawn
    pub fn un_spawn(conn: &mut NetworkConnectionToClient, identity: &mut NetworkIdentity) {
        Self::un_spawn_internal(conn, identity, true);
    }

    fn un_spawn_internal(
        conn: &mut NetworkConnectionToClient,
        identity: &mut NetworkIdentity,
        reset_state: bool,
    ) {
        if !NetworkServerStatic::active() {
            log_error!("UnSpawn: NetworkServer is not active. Cannot un_spawn objects without an active server.".to_string());
            return;
        }

        // TODO aoi

        if identity.game_object().is_null() {
            log_warn!("UnSpawn: game object is null.".to_string());
            return;
        }

        // 移除 NetworkIdentity
        conn.remove_owned_object(identity.net_id());

        Self::send_to_observers(
            conn,
            identity,
            ObjectDestroyMessage::new(identity.net_id()),
            TransportChannel::Reliable,
        );

        identity.clear_observers();

        identity.on_stop_server();

        if reset_state {
            identity.reset_state();
            identity.set_active(false);
        }
    }

    fn send_to_observers(
        conn: &mut NetworkConnectionToClient,
        identity: &mut NetworkIdentity,
        mut message: ObjectDestroyMessage,
        channel: TransportChannel,
    ) {
        if identity.is_null() || identity.observers().len() == 0 {
            return;
        }

        NetworkWriterPool::get_return(|writer| {
            NetworkMessages::pack(&mut message, writer);
            let segment = writer.to_array_segment();

            let max = NetworkMessages::max_message_size(channel);
            if writer.get_position() > max {
                log_warn!("Server.SendToObservers: message is too large to send. Consider using a higher channel or splitting the message into smaller parts.");
                return;
            }

            for conn_id in identity.observers().iter() {
                match NetworkServerStatic::network_connections().try_get_mut(conn_id) {
                    TryResult::Present(mut connection) => {
                        connection.send(segment, channel);
                    }
                    TryResult::Absent => {
                        conn.send(segment, channel);
                    }
                    TryResult::Locked => {
                        log_error!("Server.SendToObservers: connection is locked.");
                    }
                }
            }
        });
    }

    fn init_identity_by_game_obj(conn_id: u64, player: &GameObject) -> Option<NetworkIdentity> {
        if let Some(mut identity) = player.get_identity_by_prefab() {
            match NetworkServerStatic::network_connections().try_get_mut(&conn_id) {
                TryResult::Present(mut connection) => {
                    if connection.net_id() != 0 {
                        log_warn!(format!("AddPlayer: connection already has a player GameObject. Please remove the current player GameObject from {}", connection.is_ready()));
                        return None;
                    }
                    // 设置连接的 NetworkIdentity
                    connection.set_net_id(identity.net_id());
                    // 修改 NetworkIdentity 的 client_owner
                    identity.set_client_owner(conn_id);
                }
                TryResult::Absent => {
                    log_warn!(format!(
                        "AddPlayer: connectionId {} not found in connections",
                        conn_id
                    ));
                    return None;
                }
                TryResult::Locked => {
                    log_error!(format!("AddPlayer: connectionId {} is locked", conn_id));
                    return None;
                }
            }
            return Some(identity);
        }
        None
    }

    pub fn add_player_for_connection(conn_id: u64, player: &GameObject) -> bool {
        match Self::init_identity_by_game_obj(conn_id, &player) {
            None => {
                log_warn!(format!("AddPlayer: player GameObject has no NetworkIdentity. Please add a NetworkIdentity to {:?}",1));
                false
            }
            Some(identity) => {
                Self::set_client_ready(conn_id);
                Self::respawn(identity);
                true
            }
        }
    }

    pub fn replace_player_for_connection(
        conn_id: u64,
        player: &GameObject,
        replace_player_options: ReplacePlayerOptions,
    ) -> bool {
        // 确认 是本人
        match NetworkServerStatic::network_connections().try_get(&conn_id) {
            TryResult::Present(conn) => {
                match NetworkServerStatic::spawned_network_identities().try_get(&conn.net_id()) {
                    TryResult::Present(identity) => {
                        if identity.connection_to_client() != 0
                            && identity.connection_to_client() != conn_id
                        {
                            log_error!(format!(
                                "Cannot replace player for connection. New player is already owned by a different connection. netId: {}, connId: {}",
                                conn.net_id(),
                                conn.is_ready()
                            ));
                            return false;
                        }
                    }
                    TryResult::Absent => {
                        log_error!(format!(
                            "ReplacePlayer: netId {} not found in spawned",
                            conn.net_id()
                        ));
                        return false;
                    }
                    TryResult::Locked => {
                        log_error!(format!("ReplacePlayer: netId {} is locked", conn.net_id()));
                        return false;
                    }
                }
            }
            TryResult::Absent => {
                log_error!(format!(
                    "ReplacePlayer: connectionId {} not found in connections",
                    conn_id
                ));
                return false;
            }
            TryResult::Locked => {
                log_error!(format!("ReplacePlayer: connectionId {} is locked", conn_id));
                return false;
            }
        }

        log_debug!(format!(
            "ReplacePlayer: replacing player for connectionId: {} {}",
            conn_id,
            player.prefab
        ));
        // 初始化 NetworkIdentity
        match player.get_identity_by_prefab() {
            None => {
                log_warn!(format!("ReplacePlayer: player GameObject has no NetworkIdentity. Please add a NetworkIdentity to {:?}",1));
                return false;
            }
            Some(identity) => {
                Self::set_client_ready(conn_id);
                Self::respawn(identity);
            }
        }

        // 处理旧的 NetworkIdentity
        match NetworkServerStatic::network_connections().try_get_mut(&conn_id) {
            TryResult::Present(mut conn) => {
                match replace_player_options {
                    // 保留所有权
                    ReplacePlayerOptions::KeepAuthority => {
                        match NetworkServerStatic::spawned_network_identities()
                            .try_get_mut(&conn.net_id())
                        {
                            TryResult::Present(mut identity) => {
                                Self::send_change_owner_message(&mut identity, conn.value_mut());
                            }
                            TryResult::Absent => {
                                log_error!(
                                    "Failed to on_server_ready for identity because of absent"
                                );
                            }
                            TryResult::Locked => {
                                log_error!(
                                    "Failed to on_server_ready for identity because of locked"
                                );
                            }
                        }
                    }
                    // 保留激活状态
                    ReplacePlayerOptions::KeepActive => {
                        match NetworkServerStatic::spawned_network_identities()
                            .try_get_mut(&conn.net_id())
                        {
                            TryResult::Present(mut identity) => {
                                identity.remove_client_authority();
                            }
                            TryResult::Absent => {
                                log_error!(
                                    "Failed to on_server_ready for identity because of absent"
                                );
                            }
                            TryResult::Locked => {
                                log_error!(
                                    "Failed to on_server_ready for identity because of locked"
                                );
                            }
                        }
                    }
                    // 保留激活状态
                    ReplacePlayerOptions::UnSpawn => {
                        match NetworkServerStatic::spawned_network_identities()
                            .remove(&conn.net_id())
                        {
                            Some((_, mut identity)) => {
                                Self::un_spawn(conn.value_mut(), &mut identity);
                            }
                            None => {
                                log_error!(
                                    "Failed to on_server_ready for identity because of absent"
                                );
                            }
                        }
                    }
                    // 销毁
                    ReplacePlayerOptions::Destroy => {
                        match NetworkServerStatic::spawned_network_identities()
                            .try_get_mut(&conn.net_id())
                        {
                            TryResult::Present(mut identity) => {
                                Self::destroy(conn.value_mut(), &mut identity);
                            }
                            TryResult::Absent => {
                                log_error!(
                                    "Failed to on_server_ready for identity because of absent"
                                );
                            }
                            TryResult::Locked => {
                                log_error!(
                                    "Failed to on_server_ready for identity because of locked"
                                );
                            }
                        }
                    }
                }
            }
            TryResult::Absent => {
                log_error!(format!(
                    "Failed to on_server_ready for conn {} because of absent",
                    conn_id
                ));
            }
            TryResult::Locked => {
                log_error!(format!(
                    "Failed to on_server_ready for conn {} because of locked",
                    conn_id
                ));
            }
        }
        true
    }

    pub fn spawn_objects() {
        if !NetworkServerStatic::active() {
            log_error!("SpawnObjects: NetworkServer is not active. Cannot spawn objects without an active server.".to_string());
            return;
        }

        let mut deque = BackendDataStatic::get_backend_data().find_scene_network_identity_all();
        while let Some(mut identity) = deque.pop_front() {
            // 获取活动的场景id
            let scene_id = BackendDataStatic::get_backend_data()
                .get_scene_id_by_scene_name(NetworkManagerStatic::network_scene_name().as_str())
                .unwrap_or_else(|| 0);
            if identity.scene_id != 0 && identity.scene_id == scene_id {
                identity.set_active(true);
                let conn_id = identity.connection_to_client();
                Self::spawn(identity, conn_id);
            }
        }
    }

    fn spawn(identity: NetworkIdentity, conn_id: u64) {
        Self::spawn_object(identity, conn_id);
    }

    // SpawnObject(
    fn spawn_object(mut identity: NetworkIdentity, conn_id: u64) {
        if !NetworkServerStatic::active() {
            log_error!(format!("SpawnObject for {:?}, NetworkServer is not active. Cannot spawn objects without an active server.", identity.game_object()));
            return;
        }

        if identity.spawned_from_instantiate {
            return;
        }

        if NetworkServerStatic::spawned_network_identities().contains_key(&identity.net_id()) {
            log_warn!(format!(
                "SpawnObject for {:?}, netId {} already exists. Use UnSpawnObject first.",
                identity.game_object(),
                identity.net_id()
            ));
            return;
        }

        // 如果 identity 的 net_id 为 0
        if identity.net_id() == 0 {
            // 必须先分配 NetworkIdentity 的 net_id 再设置连接的 NetworkIdentity
            // 分配 NetworkIdentity 的 net_id
            identity.set_net_id(NetworkIdentity::get_static_next_network_id());

            // 设置连接的 NetworkIdentity
            identity.set_connection_to_client(conn_id);

            // identity.on_start_server();
            identity.on_start_server();

            // 重建观察者
            Self::rebuild_observers(&mut identity, true);

            // 添加到 SPAWNED 中
            NetworkServerStatic::add_spawned_network_identity(identity);

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
        NetworkServerStatic::for_each_network_connection(|connection| {
            if connection.is_ready() {
                conn_ids.push(connection.connection_id());
            }
        });
        for id in conn_ids {
            identity.add_observer(id);
        }
    }

    fn respawn(mut identity: NetworkIdentity) {
        match identity.net_id() {
            0 => {
                let conn_id = identity.connection_to_client();
                Self::spawn(identity, conn_id);
            }
            _ => {
                // 找到连接
                match NetworkServerStatic::network_connections()
                    .try_get_mut(&identity.connection_to_client())
                {
                    TryResult::Present(mut connection) => {
                        Self::send_spawn_message(&mut identity, &mut connection);
                    }
                    TryResult::Absent => {
                        log_error!(format!(
                            "Server.Respawn: connectionId {} not found in connections",
                            identity.connection_to_client()
                        ));
                    }
                    TryResult::Locked => {
                        log_error!(format!(
                            "Server.Respawn: connectionId {} is locked",
                            identity.connection_to_client()
                        ));
                    }
                }
            }
        }
    }

    // 处理 TransportError 消息
    fn on_transport_error(connection_id: u64, transport_error: TransportError) {
        match NetworkServerStatic::network_connections().try_get_mut(&connection_id) {
            TryResult::Present(mut connection) => {
                if let Some(on_error_event) =
                    NetworkServerStatic::connected_event().get(&EventHandlerType::OnErrorEvent)
                {
                    on_error_event(&mut connection, transport_error);
                }
            }
            _ => {}
        }
    }

    // 处理 ServerTransportException 消息
    fn on_transport_exception(connection_id: u64, transport_error: TransportError) {
        log_warn!(format!(
            "Server.HandleTransportException: connectionId: {}, error: {:?}",
            connection_id, transport_error
        ));
        match NetworkServerStatic::network_connections().try_get_mut(&connection_id) {
            TryResult::Present(mut connection) => {
                if let Some(on_exception_event) = NetworkServerStatic::connected_event()
                    .get(&EventHandlerType::OnTransportExceptionEvent)
                {
                    on_exception_event(&mut connection, transport_error);
                }
            }
            _ => {}
        }
    }

    // 处理 Connected 消息
    fn on_connected(mut conn: NetworkConnectionToClient) {
        // 如果有 OnConnectedEvent
        if let Some(on_connected_event) =
            NetworkServerStatic::connected_event().get(&EventHandlerType::OnConnectedEvent)
        {
            on_connected_event(&mut conn, TransportError::None);
        } else {
            log_warn!("OnConnectedEvent is null");
        }
        // 添加连接 到 NETWORK_CONNECTIONS
        NetworkServerStatic::add_network_connection(conn);
    }

    // 注册消息处理程序
    fn register_message_handlers() {
        // 注册 ReadyMessage 处理程序
        Self::register_handler::<ReadyMessage>(Self::on_client_ready_message, true);
        // 注册 CommandMessage 处理程序
        Self::register_handler::<CommandMessage>(Self::on_command_message, true);

        // 注册 NetworkPingMessage 处理程序
        Self::register_handler::<NetworkPingMessage>(NetworkTime::on_server_ping, false);
        // 注册 NetworkPongMessage 处理程序
        Self::register_handler::<NetworkPongMessage>(NetworkTime::on_server_pong, false);

        // 注册 EntityStateMessage 处理程序
        Self::register_handler::<EntityStateMessage>(Self::on_entity_state_message, true);
        // 注册 TimeSnapshotMessage 处理程序
        Self::register_handler::<TimeSnapshotMessage>(Self::on_time_snapshot_message, true);
    }

    // 处理 ReadyMessage 消息
    fn on_client_ready_message(
        connection_id: u64,
        _reader: &mut NetworkReader,
        _channel: TransportChannel,
    ) {
        Self::set_client_ready(connection_id);
    }
    // 设置客户端准备就绪
    pub fn set_client_ready(conn_id: u64) {
        match NetworkServerStatic::network_connections().try_get_mut(&conn_id) {
            TryResult::Present(mut connection) => {
                connection.set_ready(true);
            }
            TryResult::Absent => {
                log_error!(format!(
                    "Server.SetClientReady: connectionId {} not found in connections",
                    conn_id
                ));
            }
            TryResult::Locked => {
                log_error!(format!(
                    "Server.SetClientReady: connectionId {} is locked",
                    conn_id
                ));
            }
        }
        // 为连接生成观察者
        Self::spawn_observers_for_connection(conn_id);
    }
    // 发送给所有客户端
    pub fn send_to_all<T>(message: &mut T, channel: TransportChannel, send_to_ready_only: bool)
    where
        T: NetworkMessageTrait + Send,
    {
        if !NetworkServerStatic::active() {
            log_error!("Server.SendToAll: NetworkServer is not active. Cannot send messages without an active server.");
            return;
        }

        NetworkWriterPool::get_return(|writer| {
            message.serialize(writer);
            let max = NetworkMessages::max_message_size(channel);
            if writer.get_position() > max {
                log_error!("Message too large to send: ", writer.get_position());
                return;
            }
            NetworkServerStatic::for_each_network_connection(|mut connection| {
                if send_to_ready_only && !connection.is_ready() {
                    return;
                }
                connection.send(writer.to_array_segment(), channel);
            });
        });
    }
    // 设置所有客户端未准备就绪
    pub fn set_all_clients_not_ready() {
        NetworkServerStatic::for_each_network_connection(|mut connection| {
            connection.set_ready(false);
            connection.remove_from_observings_observers();
            connection
                .send_network_message(&mut NotReadyMessage::default(), TransportChannel::Reliable);
        });
    }
    // 为连接生成观察者
    fn spawn_observers_for_connection(conn_id: u64) {
        // 发送 ObjectSpawnStartedMessage 消息
        match NetworkServerStatic::network_connections().try_get_mut(&conn_id) {
            TryResult::Present(mut connection) => {
                if !connection.is_ready() {
                    return;
                }
                connection.send_network_message(
                    &mut ObjectSpawnStartedMessage::default(),
                    TransportChannel::Reliable,
                );
            }
            TryResult::Absent => {
                log_error!(format!(
                    "Server.SpawnObserversForConnection: connectionId {} not found in connections",
                    conn_id
                ));
            }
            TryResult::Locked => {
                log_error!(format!(
                    "Server.SpawnObserversForConnection: connectionId {} is locked",
                    conn_id
                ));
            }
        }

        // add connection to each nearby NetworkIdentity's observers, which
        // internally sends a spawn message for each one to the connection.
        NetworkServerStatic::for_each_spawned(|mut identity| {
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
        match NetworkServerStatic::network_connections().try_get_mut(&conn_id) {
            TryResult::Present(mut connection) => {
                connection.send_network_message(
                    &mut ObjectSpawnFinishedMessage::default(),
                    TransportChannel::Reliable,
                );
            }
            TryResult::Absent => {
                log_error!(format!(
                    "Server.SpawnObserversForConnection: connectionId {} not found in connections",
                    conn_id
                ));
            }
            TryResult::Locked => {
                log_error!(format!(
                    "Server.SpawnObserversForConnection: connectionId {} is locked",
                    conn_id
                ));
            }
        }
    }

    // 处理 OnCommandMessage 消息
    fn on_command_message(
        connection_id: u64,
        reader: &mut NetworkReader,
        channel: TransportChannel,
    ) {
        let message = CommandMessage::deserialize(reader);

        // 如果 connection_id 在 NETWORK_CONNECTIONS 中
        match NetworkServerStatic::network_connections().try_get_mut(&connection_id) {
            TryResult::Present(connection) => {
                // connection 没有准备好
                if !connection.is_ready() {
                    // 如果 channel 是 Reliable
                    if channel == TransportChannel::Reliable {
                        // 如果 SPAWNED 中有 message.net_id
                        match NetworkServerStatic::spawned_network_identities()
                            .try_get(&message.net_id)
                        {
                            TryResult::Present(identity) => {
                                // 如果 message.component_index 小于 net_identity.network_behaviours.len()
                                if message.component_index < identity.network_behaviours_count {
                                    // 如果 message.function_hash 在 RemoteProcedureCalls 中
                                    if let Some(method_name) =
                                        RemoteProcedureCalls::get_function_method_name(
                                            message.function_hash,
                                        )
                                    {
                                        log_warn!(format!("Command {} received for {} [netId={}] component  [index={}] when client not ready.\nThis may be ignored if client intentionally set NotReady.", method_name, identity.net_id(), message.net_id, message.component_index));
                                        return;
                                    }
                                }
                            }
                            TryResult::Absent => {
                                log_error!(format!(
                                    "Server.HandleCommand: connectionId {} not found in connections",
                                    connection_id
                                ));
                                return;
                            }
                            TryResult::Locked => {
                                log_error!(format!(
                                    "Server.HandleCommand: connectionId {} is locked",
                                    connection_id
                                ));
                                return;
                            }
                        }
                        log_warn!("Command received while client is not ready. This may be ignored if client intentionally set NotReady.".to_string());
                    }
                    return;
                }
            }
            TryResult::Absent => {
                log_error!(format!(
                    "Server.HandleCommand: connectionId {} not found in connections",
                    connection_id
                ));
                return;
            }
            TryResult::Locked => {
                log_error!(format!(
                    "Server.HandleCommand: connectionId {} is locked",
                    connection_id
                ));
                return;
            }
        }

        // 如果 message.net_id 在 SPAWNED 中
        match NetworkServerStatic::spawned_network_identities().try_get(&message.net_id) {
            TryResult::Present(identity) => {
                // 是否需要权限
                let requires_authority =
                    RemoteProcedureCalls::command_requires_authority(message.function_hash);
                // 如果需要权限并且 identity.connection_id_to_client != connection.connection_id
                if requires_authority && identity.connection_to_client() != connection_id {
                    // Attempt to identify the component and method to narrow down the cause of the log_error.
                    if identity.network_behaviours_count > message.component_index {
                        if let Some(method_name) =
                            RemoteProcedureCalls::get_function_method_name(message.function_hash)
                        {
                            log_warn!(format!("Command {} received for {} [netId={}] component [index={}] without authority", method_name, identity.net_id(), message.net_id,  message.component_index));
                            return;
                        }
                    }
                    log_warn!(format!(
                        "Command received for {} [netId={}] without authority",
                        identity.net_id(),
                        message.net_id
                    ));
                    return;
                }
            }
            TryResult::Absent => {
                // over reliable channel, commands should always come after spawn.
                // over unreliable, they might come in before the object was spawned.
                // for example, NetworkTransform.
                // let's not spam the console for unreliable out-of-order messages.
                if channel == TransportChannel::Reliable {
                    log_warn!(format!(
                        "Spawned object not found when handling Command message netId={}",
                        message.net_id
                    ));
                }
                return;
            }
            TryResult::Locked => {
                log_error!(format!(
                    "Server.HandleCommand: netId {} is locked",
                    message.net_id
                ));
                return;
            }
        }

        // 处理远程调用
        NetworkReaderPool::get_with_bytes_return(message.payload, |reader| {
            NetworkIdentity::handle_remote_call(
                connection_id,
                message.net_id,
                message.component_index,
                message.function_hash,
                reader,
                RemoteCallType::Command,
            );
        });
    }

    // 处理 OnEntityStateMessage 消息
    fn on_entity_state_message(
        connection_id: u64,
        reader: &mut NetworkReader,
        _channel: TransportChannel,
    ) {
        let message = EntityStateMessage::deserialize(reader);
        match NetworkServerStatic::spawned_network_identities().try_get_mut(&message.net_id) {
            TryResult::Present(mut identity) => {
                if identity.connection_to_client() == connection_id {
                    NetworkReaderPool::get_with_bytes_return(message.payload, |reader| {
                        if !identity.deserialize_server(reader) {
                            if NetworkServerStatic::exceptions_disconnect() {
                                log_error!(format!("Server failed to deserialize client state for {} with netId={}, Disconnecting.", identity.connection_to_client(), identity.net_id()));
                                match NetworkServerStatic::network_connections()
                                    .try_get_mut(&connection_id)
                                {
                                    TryResult::Present(mut connection) => {
                                        connection.disconnect();
                                    }
                                    TryResult::Absent => {
                                        log_error!(format!(
                                            "Server.HandleEntityState: connectionId {} not found.",
                                            connection_id
                                        ));
                                    }
                                    TryResult::Locked => {
                                        log_error!(format!(
                                            "Server.HandleEntityState: connectionId {} is locked.",
                                            connection_id
                                        ));
                                    }
                                }
                            } else {
                                log_warn!(format!(
                                "Server failed to deserialize client state for {} with netId={}",
                                identity.connection_to_client(),
                                identity.net_id()
                            ));
                            }
                        }
                    });
                } else {
                    log_warn!(format!(
                        "EntityStateMessage from {} for {} without authority.",
                        connection_id,
                        identity.net_id()
                    ));
                }
            }
            TryResult::Absent => {
                log_warn!(format!(
                    "EntityStateMessage for netId={} not found in spawned.",
                    message.net_id
                ));
            }
            TryResult::Locked => {
                log_error!(format!(
                    "Server.HandleEntityState: netId {} is locked",
                    message.net_id
                ));
            }
        }
    }

    // 处理 OnTimeSnapshotMessage 消息
    fn on_time_snapshot_message(
        connection_id: u64,
        _reader: &mut NetworkReader,
        _channel: TransportChannel,
    ) {
        match NetworkServerStatic::network_connections().try_get_mut(&connection_id) {
            TryResult::Present(mut connection) => {
                let snapshot =
                    TimeSnapshot::new(connection.remote_time_stamp(), NetworkTime::local_time());
                connection.on_time_snapshot(snapshot);
            }
            TryResult::Absent => {
                log_error!(format!(
                    "Server.HandleTimeSnapshot: connectionId {} not found.",
                    connection_id
                ));
            }
            TryResult::Locked => {
                log_error!(format!(
                    "Server.HandleTimeSnapshot: connectionId {} is locked.",
                    connection_id
                ));
            }
        }
    }

    // 定义一个函数来注册处理程序
    pub fn register_handler<T>(
        network_message_handler: NetworkMessageHandlerFunc,
        require_authentication: bool,
    ) where
        T: NetworkMessageTrait + Send + Sync + 'static,
    {
        let hash_code = T::get_hash_code();

        if NETWORK_MESSAGE_HANDLERS.contains_key(&hash_code) {
            log_warn!(format!("NetworkServer.RegisterHandler replacing handler for id={}. If replacement is intentional, use ReplaceHandler instead to avoid this log_warning.", hash_code));
            return;
        }
        NETWORK_MESSAGE_HANDLERS.insert(
            hash_code,
            NetworkMessageHandler::wrap_handler(network_message_handler, require_authentication),
        );
    }
    // 定义一个函数来替换处理程序
    pub fn replace_handler<T>(
        network_message_handler: NetworkMessageHandlerFunc,
        require_authentication: bool,
    ) where
        T: NetworkMessageTrait + Send + Sync + 'static,
    {
        let hash_code = T::get_hash_code();
        NETWORK_MESSAGE_HANDLERS.insert(
            hash_code,
            NetworkMessageHandler::wrap_handler(network_message_handler, require_authentication),
        );
    }
    pub fn unregister_handler<T>()
    where
        T: NetworkMessageTrait + Send + Sync + 'static,
    {
        let hash_code = T::get_hash_code();
        NETWORK_MESSAGE_HANDLERS.remove(&hash_code);
    }
}
