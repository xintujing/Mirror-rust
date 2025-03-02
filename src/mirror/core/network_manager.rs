use crate::mirror::authenticators::network_authenticator::{
    NetworkAuthenticatorTrait, NetworkAuthenticatorTraitStatic,
};
use crate::mirror::components::network_room_manager::PendingPlayer;
use crate::mirror::components::network_room_player::NetworkRoomPlayer;
use crate::mirror::components::network_transform::network_transform_base::Transform;
use crate::mirror::core::backend_data::{
    BackendDataStatic, NetworkManagerSetting, SnapshotInterpolationSetting,
};
use crate::mirror::core::connection_quality::ConnectionQualityMethod;
use crate::mirror::core::messages::{AddPlayerMessage, ReadyMessage, SceneMessage, SceneOperation};
use crate::mirror::core::network_behaviour::GameObject;
use crate::mirror::core::network_connection::NetworkConnectionTrait;
use crate::mirror::core::network_connection_to_client::NetworkConnectionToClient;
use crate::mirror::core::network_reader::NetworkReader;
use crate::mirror::core::network_server::{EventHandlerType, NetworkServer, NetworkServerStatic};
use crate::mirror::core::transport::{Transport, TransportChannel, TransportError};
use crate::{log_debug, log_error, log_warn};
use atomic::Atomic;
use dashmap::try_result::TryResult;
use lazy_static::lazy_static;
use nalgebra::Vector3;
use rand::Rng;
use std::any::Any;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};

static mut NETWORK_MANAGER_SINGLETON: Option<Box<dyn NetworkManagerTrait>> = None;

lazy_static! {
    static ref START_POSITIONS: Arc<RwLock<Vec<Transform>>> = Arc::new(RwLock::new(Vec::new()));
    static ref START_POSITIONS_INDEX: Atomic<usize> = Atomic::new(0);
    static ref NETWORK_SCENE_NAME: RwLock<String> = RwLock::new("".to_string());
}

// NetworkManagerStatic
pub struct NetworkManagerStatic;

// NetworkManagerStatic 的默认实现
impl NetworkManagerStatic {
    pub fn network_manager_singleton() -> &'static mut Box<dyn NetworkManagerTrait> {
        unsafe {
            if let Some(ref mut singleton) = NETWORK_MANAGER_SINGLETON {
                return singleton;
            }
            panic!("NetworkManager singleton not found.");
        }
    }

    #[allow(warnings)]
    pub fn network_manager_singleton_exists() -> bool {
        unsafe { NETWORK_MANAGER_SINGLETON.is_some() }
    }

    #[allow(warnings)]
    pub fn set_network_manager_singleton(network_manager: Box<dyn NetworkManagerTrait>) {
        unsafe {
            NETWORK_MANAGER_SINGLETON.replace(network_manager);
        }
    }

    pub fn network_scene_name() -> String {
        if let Ok(name) = NETWORK_SCENE_NAME.try_read() {
            return name.to_string();
        }
        log_error!("Network scene name is not set or locked.");
        "".to_string()
    }

    pub fn set_network_scene_name(name: String) {
        if let Ok(mut scene_name) = NETWORK_SCENE_NAME.try_write() {
            *scene_name = name
        } else {
            log_error!("Network scene name is locked and cannot be set.");
        }
    }

    pub fn start_positions_index() -> usize {
        START_POSITIONS_INDEX.load(Ordering::Relaxed)
    }

    pub fn set_start_positions_index(index: usize) {
        START_POSITIONS_INDEX.store(index, Ordering::Relaxed);
    }

    pub fn start_positions() -> &'static Arc<RwLock<Vec<Transform>>> {
        &START_POSITIONS
    }

    pub fn add_start_position(start: Transform) {
        match START_POSITIONS.write() {
            Ok(mut sps) => {
                sps.push(start);
            }
            Err(e) => {
                log_error!(format!("Failed to add start position: {:?}", e));
            }
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub enum PlayerSpawnMethod {
    Random,
    RoundRobin,
}

#[derive(Debug)]
pub enum NetworkManagerMode {
    Offline,
    ServerOnly,
}

// NetworkManager
pub struct NetworkManager {
    pub player_obj: GameObject,
    pub snapshot_interpolation_settings: SnapshotInterpolationSetting,
    pub mode: NetworkManagerMode,
    pub dont_destroy_on_load: bool,
    #[allow(warnings)]
    pub editor_auto_start: bool,
    pub send_rate: u32,
    pub offline_scene: String,
    pub online_scene: String,
    #[allow(warnings)]
    pub offline_scene_load_delay: f32,
    pub network_address: String,
    pub max_connections: usize,
    pub disconnect_inactive_connections: bool,
    pub disconnect_inactive_timeout: f32,
    pub authenticator: Option<Box<dyn NetworkAuthenticatorTrait>>,
    pub auto_create_player: bool,
    pub player_spawn_method: PlayerSpawnMethod,
    pub spawn_prefabs: Vec<GameObject>,
    pub exceptions_disconnect: bool,
    #[allow(warnings)]
    pub evaluation_method: ConnectionQualityMethod,
    #[allow(warnings)]
    pub evaluation_interval: f32,
    #[allow(warnings)]
    pub time_interpolation_gui: bool,
}

// NetworkManager 的默认实现
impl NetworkManager {
    pub fn new_with_network_manager_setting(
        network_manager_setting: NetworkManagerSetting,
    ) -> Self {
        let mut spawn_prefabs = Vec::new();
        for spawn_prefab in &network_manager_setting.spawn_prefabs {
            spawn_prefabs.push(GameObject::new_with_prefab(spawn_prefab.clone()));
        }
        Self {
            mode: NetworkManagerMode::Offline,
            dont_destroy_on_load: network_manager_setting.dont_destroy_on_load,
            editor_auto_start: network_manager_setting.editor_auto_start,
            send_rate: network_manager_setting.send_rate,
            offline_scene: network_manager_setting.offline_scene,
            online_scene: network_manager_setting.online_scene,
            offline_scene_load_delay: 0.0,
            network_address: network_manager_setting.network_address.clone(),
            max_connections: network_manager_setting.max_connections,
            disconnect_inactive_connections: network_manager_setting
                .disconnect_inactive_connections,
            disconnect_inactive_timeout: network_manager_setting.disconnect_inactive_timeout,
            authenticator: None,
            player_obj: GameObject::new_with_prefab(network_manager_setting.player_prefab.clone()),
            auto_create_player: network_manager_setting.auto_create_player,
            player_spawn_method: PlayerSpawnMethod::Random,
            spawn_prefabs,
            exceptions_disconnect: network_manager_setting.exceptions_disconnect,
            evaluation_method: ConnectionQualityMethod::Simple,
            evaluation_interval: network_manager_setting.evaluation_interval,
            time_interpolation_gui: network_manager_setting.time_interpolation_gui,
            snapshot_interpolation_settings: network_manager_setting
                .snapshot_interpolation_setting
                .clone(),
        }
    }

    pub fn setup_server(&mut self) {
        Self::initialize_singleton();

        NetworkServerStatic::set_disconnect_inactive_connections(
            self.disconnect_inactive_connections,
        );
        NetworkServerStatic::set_disconnect_inactive_timeout(self.disconnect_inactive_timeout);
        NetworkServerStatic::set_exceptions_disconnect(self.exceptions_disconnect);

        if let Some(ref mut authenticator) = self.authenticator {
            authenticator.on_start_server();
            NetworkAuthenticatorTraitStatic::set_on_server_authenticated(
                Self::on_server_authenticated,
            );
        }

        NetworkServer::listen(self.max_connections);

        Self::register_server_messages();
    }

    // zhuce
    fn register_server_messages() {
        // 添加连接事件
        NetworkServerStatic::connected_event().insert(
            EventHandlerType::OnConnectedEvent,
            Box::new(Self::on_server_connect_internal),
        );
        // 添加断开连接事件
        NetworkServerStatic::connected_event().insert(
            EventHandlerType::OnDisconnectedEvent,
            Box::new(Self::on_server_disconnect),
        );
        // 添加错误事件
        NetworkServerStatic::connected_event().insert(
            EventHandlerType::OnErrorEvent,
            Box::new(Self::on_server_error),
        );
        // 添加异常事件
        NetworkServerStatic::connected_event().insert(
            EventHandlerType::OnTransportExceptionEvent,
            Box::new(Self::on_server_transport_exception),
        );

        // 添加 AddPlayerMessage 消息处理
        NetworkServer::register_handler::<AddPlayerMessage>(
            Self::on_server_add_player_internal,
            true,
        );
        // 添加 ReadyMessage 消息处理
        NetworkServer::replace_handler::<ReadyMessage>(
            Self::on_server_ready_message_internal,
            true,
        );
    }

    fn on_server_error(conn: &mut NetworkConnectionToClient, error: TransportError) {
        let (_, _) = (conn, error);
    }

    pub fn on_server_authenticated(conn: &mut NetworkConnectionToClient) {
        // 获取 NetworkManagerTrait 的单例
        conn.set_authenticated(true);

        // 获取 NetworkManagerTrait 的单例
        let network_manager = NetworkManagerStatic::network_manager_singleton();
        // offline_scene
        let offline_scene = network_manager.offline_scene().to_string();

        // 获取 场景名称
        let network_scene_name = NetworkManagerStatic::network_scene_name();
        // 如果 场景名称不为空 且 场景名称不等于 NetworkManager 的 offline_scene
        if network_scene_name != "" && network_scene_name != offline_scene {
            // 创建 SceneMessage 消息
            let mut scene_message = SceneMessage::new(
                network_scene_name.to_string(),
                SceneOperation::Normal,
                false,
            );
            // 发送 SceneMessage 消息
            conn.send_network_message(&mut scene_message, TransportChannel::Reliable);
        }

        Self::on_server_connect(conn);
    }

    fn on_server_ready_message_internal(
        conn_id: u64,
        _reader: &mut NetworkReader,
        _channel: TransportChannel,
    ) {
        Self::on_server_ready(conn_id)
    }

    fn on_server_ready(conn_id: u64) {
        NetworkServer::set_client_ready(conn_id);
    }

    pub fn on_server_add_player_internal(
        conn_id: u64,
        _reader: &mut NetworkReader,
        _channel: TransportChannel,
    ) {
        // 获取 NetworkManagerTrait 的单例
        let network_manager = NetworkManagerStatic::network_manager_singleton();

        // 如果 NetworkManager 的 auto_create_player 为 true 且 player_obj.prefab 为空
        if network_manager.auto_create_player() && network_manager.player_obj().prefab == "" {
            log_error!("The PlayerPrefab is empty on the NetworkManager. Please setup a PlayerPrefab object.");
            return;
        }

        // 如果 NetworkManager 的 auto_create_player 为 true 且 player_obj.prefab 不为空
        if network_manager.auto_create_player() {
            // 如果 player_obj.prefab 不为空，且 player_obj.prefab 不存在于 BACKEND_DATA 的 asset_id 中
            if let Some(asset_id) = BackendDataStatic::get_backend_data()
                .get_asset_id_by_asset_name(network_manager.player_obj().prefab.as_str())
            {
                if let None = BackendDataStatic::get_backend_data()
                    .get_network_identity_data_by_asset_id(asset_id)
                {
                    log_error!("The PlayerPrefab does not have a NetworkIdentity. Please add a NetworkIdentity to the player prefab.");
                    return;
                }
            }
        }

        // 如果 NetworkManager 的 auto_create_player 为 false
        match NetworkServerStatic::network_connections().try_get(&conn_id) {
            TryResult::Present(coon) => {
                if coon.net_id() != 0 {
                    log_error!("There is already a player for this connection.");
                    return;
                }
            }
            TryResult::Absent => {
                log_error!(format!(
                    "Failed to on_server_add_player_internal for coon {} because of absent",
                    conn_id
                ));
                return;
            }
            TryResult::Locked => {
                log_error!(format!(
                    "Failed to on_server_add_player_internal for coon {} because of locked",
                    conn_id
                ));
                return;
            }
        }
        // 调用 NetworkManagerTrait 的 on_server_add_player 方法
        network_manager.on_server_add_player(conn_id);
    }

    pub fn register_start_position(mut start: Transform) {
        // 在生成 五个随机位置
        for _ in 0..5 {
            let x = rand::rng().random_range(-8.0..8.0);
            let y = 0.5;
            let z = rand::rng().random_range(-8.0..8.0);
            let v3 = Vector3::new(x, y, z);
            start.position = v3;
            start.local_position = v3;
            NetworkManagerStatic::add_start_position(start);
        }
    }

    fn is_server_online_scene_change_needed(&self) -> bool {
        self.online_scene != self.offline_scene
    }

    pub fn apply_configuration(&mut self) {
        NetworkServerStatic::set_tick_rate(self.send_rate);
    }

    fn update_scene(&mut self) {
        if NetworkServerStatic::is_loading_scene() {
            self.finish_load_scene();
        }
    }

    fn finish_load_scene(&mut self) {
        NetworkServerStatic::set_is_loading_scene(false);

        match self.mode {
            NetworkManagerMode::ServerOnly => {
                self.finish_load_scene_server_only();
            }
            _ => {}
        }
    }

    fn finish_load_scene_server_only(&mut self) {
        NetworkServer::spawn_objects();
        self.on_server_change_scene(NetworkManagerStatic::network_scene_name());
    }
}

pub trait NetworkManagerTrait: Any {
    fn initialize_singleton() -> bool
    where
        Self: Sized,
    {
        if NetworkManagerStatic::network_manager_singleton_exists() {
            return true;
        }
        if NetworkManagerStatic::network_manager_singleton().dont_destroy_on_load() {
            if NetworkManagerStatic::network_manager_singleton_exists() {
                log_warn!("NetworkManager already exists in the scene. Deleting the new one.");
                return false;
            }
        }

        if !Transport::active_transport_exists() {
            panic!("No transport found, Add a transport component.");
        }
        true
    }
    fn authenticator(&mut self) -> &mut Option<Box<dyn NetworkAuthenticatorTrait>>;
    fn set_authenticator(&mut self, authenticator: Box<dyn NetworkAuthenticatorTrait>);
    fn set_mode(&mut self, mode: NetworkManagerMode);
    fn snapshot_interpolation_settings(&self) -> &SnapshotInterpolationSetting;
    fn offline_scene(&self) -> &String;
    fn online_scene(&self) -> &String;
    fn auto_create_player(&self) -> bool;
    fn player_obj(&self) -> &GameObject;
    fn dont_destroy_on_load(&self) -> bool;
    fn network_address(&self) -> &String;
    fn on_validate(&mut self);
    fn ready_status_changed(&mut self, component: &mut NetworkRoomPlayer);
    fn room_slots(&mut self) -> &mut Vec<u32>;
    fn recalculate_room_player_indices(&mut self) -> (i32, u32);
    fn pending_players(&mut self) -> &mut Vec<PendingPlayer>;
    fn set_all_players_ready(&mut self, value: bool);
    fn room_scene(&self) -> &String;
    fn gameplay_scene(&self) -> &String;
    fn num_players(&self) -> usize {
        let mut num_players = 0;
        NetworkServerStatic::for_each_network_connection(|conn| {
            if conn.net_id() != 0 {
                num_players += 1;
            }
        });
        num_players
    }
    fn stop_server(&mut self) {
        if !NetworkServerStatic::active() {
            log_warn!("Server already stopped.");
            return;
        }

        if let Some(ref mut authenticator) =
            NetworkManagerStatic::network_manager_singleton().authenticator()
        {
            authenticator.on_stop_server();
        }

        self.on_stop_server();

        NetworkManagerStatic::start_positions()
            .write()
            .unwrap()
            .clear();
        NetworkManagerStatic::set_start_positions_index(0);
        NetworkManagerStatic::set_network_scene_name("".to_string());

        NetworkServer::shutdown();

        self.set_mode(NetworkManagerMode::Offline);

        NetworkManagerStatic::set_start_positions_index(0);

        NetworkManagerStatic::set_network_scene_name("".to_string());
    }
    fn reset(&mut self);
    fn new() -> Self
    where
        Self: Sized;
    fn awake()
    where
        Self: Sized,
    {
        let manager = Self::new();
        NetworkManagerStatic::set_network_manager_singleton(Box::new(manager));
    }
    fn start(&mut self);
    fn update(&mut self);
    fn late_update(&mut self);
    fn on_destroy(&mut self);
    fn server_change_scene(&mut self, new_scene_name: String);
    fn get_start_position(&mut self) -> Transform {
        Transform::default()
    }
    fn on_server_connect(conn: &mut NetworkConnectionToClient)
    where
        Self: Sized,
    {
        let _ = conn;
    }
    fn on_server_disconnect(conn: &mut NetworkConnectionToClient, transport_error: TransportError)
    where
        Self: Sized;
    fn on_server_ready(conn_id: u64)
    where
        Self: Sized;
    fn on_server_add_player(&mut self, conn_id: u64);
    fn on_server_error(conn: &mut NetworkConnectionToClient, error: TransportError)
    where
        Self: Sized;
    fn on_server_transport_exception(conn: &mut NetworkConnectionToClient, error: TransportError)
    where
        Self: Sized;
    fn on_server_change_scene(&mut self, new_scene_name: String);
    fn on_server_scene_changed(&mut self, new_scene_name: String);
    fn on_start_server(&mut self);
    fn on_stop_server(&mut self);

    fn on_server_connect_internal(
        conn: &mut NetworkConnectionToClient,
        transport_error: TransportError,
    ) where
        Self: Sized;
}

impl NetworkManagerTrait for NetworkManager {
    fn authenticator(&mut self) -> &mut Option<Box<dyn NetworkAuthenticatorTrait>> {
        &mut self.authenticator
    }

    fn set_authenticator(&mut self, authenticator: Box<dyn NetworkAuthenticatorTrait>) {
        self.authenticator.replace(authenticator);
    }

    fn set_mode(&mut self, mode: NetworkManagerMode) {
        self.mode = mode;
    }

    fn snapshot_interpolation_settings(&self) -> &SnapshotInterpolationSetting {
        &self.snapshot_interpolation_settings
    }

    fn offline_scene(&self) -> &String {
        &self.offline_scene
    }

    fn online_scene(&self) -> &String {
        &self.online_scene
    }

    fn auto_create_player(&self) -> bool {
        self.auto_create_player
    }

    fn player_obj(&self) -> &GameObject {
        &self.player_obj
    }

    fn dont_destroy_on_load(&self) -> bool {
        self.dont_destroy_on_load
    }

    fn network_address(&self) -> &String {
        &self.network_address
    }

    fn on_validate(&mut self) {
        self.max_connections = self.max_connections.max(0);

        if !self.player_obj.is_null() && !self.player_obj.is_has_component() {
            log_error!("NetworkManager - Player Prefab must have a NetworkIdentity.");
        }

        if !self.player_obj.is_null() && self.spawn_prefabs.contains(&self.player_obj) {
            log_warn!("NetworkManager - Player Prefab doesn't need to be in Spawnable Prefabs list too. Removing it.");
            self.spawn_prefabs.retain(|x| x != &self.player_obj);
        }
    }

    fn ready_status_changed(&mut self, component: &mut NetworkRoomPlayer) {
        let _ = component;
    }

    #[allow(warnings)]
    fn room_slots(&mut self) -> &mut Vec<u32> {
        static mut ROOM_SLOTS: Vec<u32> = Vec::new();
        unsafe { &mut ROOM_SLOTS }
    }

    fn recalculate_room_player_indices(&mut self) -> (i32, u32) {
        (0, 0)
    }

    #[allow(warnings)]
    fn pending_players(&mut self) -> &mut Vec<PendingPlayer> {
        static mut PENDING_PLAYERS: Vec<PendingPlayer> = Vec::new();
        unsafe { &mut PENDING_PLAYERS }
    }

    fn set_all_players_ready(&mut self, value: bool) {
        let _ = value;
    }

    fn room_scene(&self) -> &String {
        &self.offline_scene
    }

    fn gameplay_scene(&self) -> &String {
        &self.online_scene
    }

    fn reset(&mut self) {
        log_debug!("NetworkManager reset");
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        let backend_data = BackendDataStatic::get_backend_data();
        if backend_data.network_manager_settings.len() == 0 {
            panic!("No NetworkManager settings found in the BackendData. Please add a NetworkManager setting.");
        }
        let network_manager_setting = backend_data.network_manager_settings[0].clone();
        Self::new_with_network_manager_setting(network_manager_setting)
    }

    fn start(&mut self) {
        if !Self::initialize_singleton() {
            return;
        }
        self.apply_configuration();

        NetworkManagerStatic::set_network_scene_name(self.online_scene.to_string());

        if NetworkServerStatic::active() {
            log_warn!("Server already started.");
            return;
        }

        self.mode = NetworkManagerMode::ServerOnly;

        self.setup_server();

        self.on_start_server();

        if self.is_server_online_scene_change_needed() {
            self.server_change_scene(self.online_scene.to_string());
        } else {
            NetworkServer::spawn_objects();
        }
    }

    fn update(&mut self) {
        self.apply_configuration();
    }

    fn late_update(&mut self) {
        self.update_scene();
    }

    fn on_destroy(&mut self) {
        self.stop_server();
    }

    fn server_change_scene(&mut self, new_scene_name: String) {
        if new_scene_name == "" {
            log_error!("ServerChangeScene newSceneName is empty");
            return;
        }

        if NetworkServerStatic::is_loading_scene()
            && new_scene_name == NetworkManagerStatic::network_scene_name()
        {
            log_error!(format!(
                "Scene change is already in progress for scene: {}",
                new_scene_name
            ));
            return;
        }

        if !NetworkServerStatic::active() && new_scene_name == self.offline_scene {
            log_error!(
                "ServerChangeScene called when server is not active. Call StartServer first."
            );
            return;
        }

        NetworkServer::set_all_clients_not_ready();
        NetworkManagerStatic::set_network_scene_name(new_scene_name.to_string());

        self.on_server_change_scene(new_scene_name.to_string());

        NetworkServerStatic::set_is_loading_scene(true);

        if NetworkServerStatic::active() {
            NetworkServer::send_to_all(
                &mut SceneMessage::new(new_scene_name.to_string(), SceneOperation::Normal, false),
                TransportChannel::Reliable,
                false,
            );
        }

        NetworkManagerStatic::set_start_positions_index(0);
        NetworkManagerStatic::start_positions().write().unwrap().clear();
    }

    fn get_start_position(&mut self) -> Transform {
        if NetworkManagerStatic::start_positions()
            .read()
            .unwrap()
            .len()
            == 0
        {
            return Transform::default();
        }

        if self.player_spawn_method == PlayerSpawnMethod::Random {
            let index = rand::random::<u32>()
                % NetworkManagerStatic::start_positions()
                .read()
                .unwrap()
                .len() as u32;
            return NetworkManagerStatic::start_positions().read().unwrap()[index as usize].clone();
        }
        let index = NetworkManagerStatic::start_positions_index();
        NetworkManagerStatic::set_start_positions_index(
            index
                + 1 % NetworkManagerStatic::start_positions()
                .read()
                .unwrap()
                .len(),
        );
        NetworkManagerStatic::start_positions().read().unwrap()[index].clone()
    }

    // OnServerDisconnect
    fn on_server_disconnect(conn: &mut NetworkConnectionToClient, _transport_error: TransportError)
    where
        Self: Sized,
    {
        NetworkServer::destroy_player_for_connection(conn);
    }

    fn on_server_ready(conn_id: u64)
    where
        Self: Sized,
    {
        NetworkServer::set_client_ready(conn_id);
    }

    fn on_server_add_player(&mut self, conn_id: u64) {
        if self.player_obj.is_null() {
            log_error!("The PlayerPrefab is empty on the NetworkManager. Please setup a PlayerPrefab object.");
            return;
        }

        // 修改 player_obj 的 transform 属性
        self.player_obj.transform = self.get_start_position();

        NetworkServer::add_player_for_connection(conn_id, &self.player_obj);
    }

    fn on_server_error(conn: &mut NetworkConnectionToClient, error: TransportError)
    where
        Self: Sized,
    {
        let (_, _) = (conn, error);
    }

    fn on_server_transport_exception(conn: &mut NetworkConnectionToClient, error: TransportError)
    where
        Self: Sized,
    {
        let (_, _) = (conn, error);
    }

    fn on_server_change_scene(&mut self, _new_scene_name: String) {}

    fn on_server_scene_changed(&mut self, _new_scene_name: String) {}

    fn on_start_server(&mut self) {}

    fn on_stop_server(&mut self) {}
    // OnServerConnectInternal
    fn on_server_connect_internal(
        conn: &mut NetworkConnectionToClient,
        _transport_error: TransportError,
    ) where
        Self: Sized,
    {
        // 获取 NetworkManagerTrait 的单例
        let network_manager = NetworkManagerStatic::network_manager_singleton();

        // 如果 NetworkManager 的 authenticator 不为空
        if let Some(authenticator) = network_manager.authenticator() {
            // 调用 NetworkAuthenticatorTrait 的 on_server_connect 方法
            authenticator.on_server_authenticate(conn);
        } else {
            // 如果 NetworkManager 的 authenticator 为空
            Self::on_server_authenticated(conn);
        }
    }
}
