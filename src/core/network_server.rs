use crate::core::batcher::{DataReader, UnBatch};
use crate::core::messages::{CommandMessage, EntityStateMessage, NetworkMessageHandler, NetworkMessageHandlerFunc, NetworkPingMessage, NetworkPongMessage, ReadyMessage, TimeSnapshotMessage};
use crate::core::network_connection::NetworkConnection;
use crate::core::network_identity::NetworkIdentity;
use crate::core::network_time::NetworkTime;
use crate::core::tools::time_sample::TimeSample;
use dashmap::DashMap;
use kcp2k_rust::kcp2k_channel::Kcp2KChannel;
use tklog::warn;

pub enum RemovePlayerOptions {
    /// <summary>Player Object remains active on server and clients. Only ownership is removed</summary>
    KeepActive,
    /// <summary>Player Object is unspawned on clients but remains on server</summary>
    UnSpawn,
    /// <summary>Player Object is destroyed on server and clients</summary>
    Destroy,
}

pub struct NetworkServer {
    pub initialized: bool,
    pub max_connections: u32,
    pub tick_rate: u32,
    pub tick_interval: f32,
    pub send_rate: u32,
    pub last_send_time: f64,
    pub network_connections: DashMap<u64, NetworkConnection>,
    pub network_message_handlers: DashMap<u16, NetworkMessageHandler>,
    pub spawned: DashMap<u32, NetworkIdentity>,
    pub dont_listen: bool,
    pub active: bool,
    pub is_loading_scene: bool,
    // pub aoi:InterestManagementBase
    pub exceptions_disconnect: bool,
    pub disconnect_inactive_connections: bool,
    pub disconnect_inactive_timeout: f32,

    pub actual_tick_rate: u32,
    pub actual_tick_rate_start: f64,
    pub actual_tick_rate_counter: u32,

    pub early_update_duration: TimeSample,
    pub late_update_duration: TimeSample,
    pub full_update_duration: TimeSample,
}

impl NetworkServer {
    pub fn new() -> Self {
        Self {
            initialized: false,
            max_connections: 0,
            tick_rate: 0,
            tick_interval: 0.0,
            send_rate: 0,
            last_send_time: 0.0,
            network_connections: DashMap::new(),
            network_message_handlers: DashMap::new(),
            spawned: DashMap::new(),
            dont_listen: false,
            active: false,
            is_loading_scene: false,
            exceptions_disconnect: false,
            disconnect_inactive_connections: false,
            disconnect_inactive_timeout: 0.0,
            actual_tick_rate: 0,
            actual_tick_rate_start: 0.0,
            actual_tick_rate_counter: 0,
            early_update_duration: TimeSample::new(0),
            late_update_duration: TimeSample::new(0),
            full_update_duration: TimeSample::new(0),
        }
    }
    pub fn initialize(&mut self) {
        if self.initialized {
            return;
        }

        //Make sure connections are cleared in case any old connections references exist from previous sessions
        self.network_connections.clear();

        // TODO: if (aoi != null) aoi.ResetState();

        NetworkTime::reset_statics();

        // TODO AddTransportHandlers();

        self.initialized = true;

        self.early_update_duration = TimeSample::new(self.send_rate);
        self.late_update_duration = TimeSample::new(self.send_rate);
        self.full_update_duration = TimeSample::new(self.send_rate);
    }
    pub fn listen(&mut self, max_connections: u32) {
        self.initialize();
        self.max_connections = max_connections;

        if !self.dont_listen {
            // TODO Transport.active.ServerStart()
        }
        self.active = true;

        self.register_message_handlers();
    }

    fn register_message_handlers(&self) {
        // 注册 ReadyMessage 处理程序
        self.register_handler::<ReadyMessage>(Box::new(Self::on_client_ready_message), true);
        // 注册 CommandMessage 处理程序
        self.register_handler::<CommandMessage>(Box::new(Self::on_command_message), true);

        // 注册 NetworkPingMessage 处理程序
        self.register_handler::<NetworkPingMessage>(Box::new(NetworkTime::on_server_ping), false);
        // 注册 NetworkPongMessage 处理程序
        self.register_handler::<NetworkPongMessage>(Box::new(NetworkTime::on_server_pong), false);

        // 注册 EntityStateMessage 处理程序
        self.register_handler::<EntityStateMessage>(Box::new(Self::on_entity_state_message), true);
        // 注册 TimeSnapshotMessage 处理程序
        self.register_handler::<TimeSnapshotMessage>(Box::new(Self::on_time_snapshot_message), true);
    }

    // 处理 ReadyMessage 消息
    fn on_client_ready_message(connection: &mut NetworkConnection, reader: &mut UnBatch, channel: Kcp2KChannel) {
        let message = ReadyMessage::deserialize(reader);
        if let Ok(message) = message {
            println!("on_client_ready_message: {:?}", message);
        }
    }

    // 处理 OnCommandMessage 消息
    fn on_command_message(connection: &mut NetworkConnection, reader: &mut UnBatch, channel: Kcp2KChannel) {
        let message = CommandMessage::deserialize(reader);
        if let Ok(message) = message {
            println!("on_command_message: {:?}", message);
        }
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
    pub fn register_handler<T>(&self, network_message_handler: NetworkMessageHandlerFunc, require_authentication: bool)
    where
        T: DataReader + Send + Sync + 'static,
    {
        let hash_code = T::get_hash_code();

        if self.network_message_handlers.contains_key(&hash_code) {
            warn!(format!("NetworkServer.RegisterHandler replacing handler for id={}. If replacement is intentional, use ReplaceHandler instead to avoid this warning.", hash_code));
            return;
        }

        self.network_message_handlers.insert(hash_code, NetworkMessageHandler::wrap_handler(network_message_handler, require_authentication));
    }
}
