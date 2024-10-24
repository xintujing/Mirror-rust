use crate::core::batcher::{DataReader, UnBatch};
use crate::core::messages::{CommandMessage, EntityStateMessage, NetworkMessageHandler, NetworkMessageHandlerFunc, NetworkPingMessage, NetworkPongMessage, ReadyMessage, TimeSnapshotMessage};
use crate::core::network_connection::NetworkConnection;
use crate::core::network_identity::{NetworkIdentity, Visibility};
use crate::core::network_time::NetworkTime;
use crate::core::tools::time_sample::TimeSample;
use atomic::Atomic;
use dashmap::mapref::multiple::RefMutMulti;
use dashmap::DashMap;
use kcp2k_rust::kcp2k_channel::Kcp2KChannel;
use lazy_static::lazy_static;
use std::cell::RefMut;
use std::sync::atomic::Ordering;
use std::sync::RwLock;
use tklog::{error, warn};


lazy_static! {
    static ref INITIALIZED: Atomic<bool> = Atomic::new(false);
    static ref TICK_RATE: Atomic<u32> = Atomic::new(0);
    static ref TICK_INTERVAL: Atomic<f32> = Atomic::new(0.0);
    static ref SEND_RATE: Atomic<u32> = Atomic::new(0);
    static ref LAST_SEND_TIME: Atomic<f64> = Atomic::new(0.0);
    static ref DONT_LISTEN: Atomic<bool> = Atomic::new(false);
    static ref ACTIVE: Atomic<bool> = Atomic::new(false);
    static ref IS_LOADING_SCENE: Atomic<bool> = Atomic::new(false);
    static ref EXCEPTIONS_DISCONNECT: Atomic<bool> = Atomic::new(false);
    static ref DISCONNECT_INACTIVE_CONNECTIONS: Atomic<bool> = Atomic::new(false);
    static ref DISCONNECT_INACTIVE_TIMEOUT: Atomic<f32> = Atomic::new(0.0);
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
    pub fn initialize() {
        if Self::get_static_initialized() {
            return;
        }

        //Make sure connections are cleared in case any old connections references exist from previous sessions
        NETWORK_CONNECTIONS.clear();

        // TODO: if (aoi != null) aoi.ResetState();

        NetworkTime::reset_statics();

        // TODO AddTransportHandlers();

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
            // TODO Transport.active.ServerStart()
        }

        Self::set_static_active(true);

        Self::register_message_handlers();
    }

    pub fn on_transport_connected(connection_id: u64) {
        if connection_id == 0 {
            error!(format!("Server.HandleConnect: invalid connectionId: {}. Needs to be != 0, because 0 is reserved for local player.", connection_id));
            // TODO Transport.active.ServerDisconnect(connectionId);
            return;
        }

        if NETWORK_CONNECTIONS.contains_key(&connection_id) {
            error!(format!("Server.HandleConnect: connectionId {} already exists.", connection_id));
            // TODO Transport.active.ServerDisconnect(connectionId);
            return;
        }

        if Self::get_static_network_connections_size() >= Self::get_static_max_connections() {
            error!(format!("Server.HandleConnect: maxConnections reached: {}. Disconnecting connectionId: {}", Self::get_static_max_connections(), connection_id));
            // TODO Transport.active.ServerDisconnect(connectionId);
            return;
        }
        let connection = NetworkConnection::network_connection(connection_id);
        // TODO
        // Self::on_connected(&mut connection);
    }

    fn on_connected(connection: NetworkConnection) {
        Self::add_connection(connection);
        // TODO: OnConnectedEvent?.Invoke(conn);
    }

    fn add_connection(connection: NetworkConnection) -> bool {
        if NETWORK_CONNECTIONS.contains_key(&connection.connection_id) {
            return false;
        }
        NETWORK_CONNECTIONS.insert(connection.connection_id, connection);
        true
    }

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
    fn on_client_ready_message(connection: &mut NetworkConnection, reader: &mut UnBatch, channel: Kcp2KChannel) {
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

        // TODO: conn.Send(new ObjectSpawnStartedMessage()); 1227

        // add connection to each nearby NetworkIdentity's observers, which
        // internally sends a spawn message for each one to the connection.

        SPAWNED.iter().for_each(|identity| {
            if identity.visibility == Visibility::Shown {
                // TODO: identity.AddObserver(connection);
            } else if identity.visibility == Visibility::Hidden {
                // do nothing
            } else if identity.visibility == Visibility::Default {
                // TODO aoi system
                // TODO: identity.AddObserver(connection);
            }
        });
        // TODO: conn.Send(new ObjectSpawnFinishedMessage()); 1279
    }

    // 处理 OnCommandMessage 消息
    fn on_command_message(connection: &mut NetworkConnection, reader: &mut UnBatch, channel: Kcp2KChannel) {
        if let Ok(message) = CommandMessage::deserialize(reader) {}
    }

    // 处理 OnEntityStateMessage 消息
    fn on_entity_state_message(connection: &mut NetworkConnection, reader: &mut UnBatch, channel: Kcp2KChannel) {
        let message = EntityStateMessage::deserialize(reader);
        if let Ok(message) = message {
            println!("on_entity_state_message: {:?}", message);
        }
    }

    // 处理 OnTimeSnapshotMessage 消息
    fn on_time_snapshot_message(connection: &mut NetworkConnection, reader: &mut UnBatch, channel: Kcp2KChannel) {
        let message = TimeSnapshotMessage::deserialize(reader);
        if let Ok(message) = message {
            println!("on_time_snapshot_message: {:?}", message);
        }
    }

    // 定义一个函数来注册处理程序
    pub fn register_handler<T>(network_message_handler: NetworkMessageHandlerFunc, require_authentication: bool)
    where
        T: DataReader + Send + Sync + 'static,
    {
        let hash_code = T::get_hash_code();

        if NETWORK_MESSAGE_HANDLERS.contains_key(&hash_code) {
            warn!(format!("NetworkServer.RegisterHandler replacing handler for id={}. If replacement is intentional, use ReplaceHandler instead to avoid this warning.", hash_code));
            return;
        }
        // if let Ok(mut network_message_handlers) = NETWORK_MESSAGE_HANDLERS.write() {
        //     if network_message_handlers.contains_key(&hash_code) {
        //         warn!(format!("NetworkServer.RegisterHandler replacing handler for id={}. If replacement is intentional, use ReplaceHandler instead to avoid this warning.", hash_code));
        //         return;
        //     }
        //     network_message_handlers.insert(hash_code, NetworkMessageHandler::wrap_handler(network_message_handler, require_authentication));
        // }
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
