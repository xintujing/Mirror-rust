use crate::mirror::authenticators::network_authenticator::{
    NetworkAuthenticatorTrait, NetworkAuthenticatorTraitStatic,
};
use crate::mirror::components::network_transform::network_transform_base::Transform;
use crate::mirror::core::backend_data::{BackendDataStatic, SnapshotInterpolationSetting};
use crate::mirror::core::connection_quality::ConnectionQualityMethod;
use crate::mirror::core::messages::{AddPlayerMessage, ReadyMessage, SceneMessage, SceneOperation};
use crate::mirror::core::network_behaviour::{GameObject, NetworkBehaviourFactory};
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
    pub fn get_network_manager_singleton() -> &'static mut Box<dyn NetworkManagerTrait> {
        unsafe {
            if let Some(ref mut singleton) = NETWORK_MANAGER_SINGLETON {
                return singleton;
            }
            panic!("NetworkManager singleton not found.");
        }
    }

    pub fn network_manager_singleton_exists() -> bool {
        unsafe { NETWORK_MANAGER_SINGLETON.is_some() }
    }

    pub fn set_network_manager_singleton(network_manager: Box<dyn NetworkManagerTrait>) {
        unsafe {
            NETWORK_MANAGER_SINGLETON.replace(network_manager);
        }
    }

    pub fn get_network_scene_name() -> String {
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

    pub fn get_start_positions_index() -> usize {
        START_POSITIONS_INDEX.load(Ordering::Relaxed)
    }

    pub fn set_start_positions_index(index: usize) {
        START_POSITIONS_INDEX.store(index, Ordering::Relaxed);
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
    Server,
}

// NetworkManager
pub struct NetworkManager {
    player_obj: GameObject,
    pub snapshot_interpolation_settings: SnapshotInterpolationSetting,
    pub mode: NetworkManagerMode,
    pub dont_destroy_on_load: bool,
    pub editor_auto_start: bool,
    pub send_rate: u32,
    offline_scene: String,
    pub online_scene: String,
    pub offline_scene_load_delay: f32,
    pub network_address: String,
    pub max_connections: usize,
    pub disconnect_inactive_connections: bool,
    pub disconnect_inactive_timeout: f32,
    authenticator: Option<Box<dyn NetworkAuthenticatorTrait>>,
    auto_create_player: bool,
    pub player_spawn_method: PlayerSpawnMethod,
    pub spawn_prefabs: Vec<GameObject>,
    pub exceptions_disconnect: bool,
    pub evaluation_method: ConnectionQualityMethod,
    pub evaluation_interval: f32,
    pub time_interpolation_gui: bool,
}

// NetworkManager 的默认实现
impl NetworkManager {
    fn initialize_singleton(&self) -> bool {
        if NetworkManagerStatic::network_manager_singleton_exists() {
            return true;
        }
        if self.dont_destroy_on_load {
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

    pub fn start_server(&mut self) {
        if NetworkServerStatic::active() {
            log_warn!("Server already started.");
            return;
        }

        self.mode = NetworkManagerMode::Server;

        self.setup_server();

        self.on_start_server();

        if self.is_server_online_scene_change_needed() {
            // TODO  SceneManager.LoadScene(onlineScene);
        } else {
            // TODO NetworkServer.SpawnObjects();
        }
    }

    fn setup_server(&mut self) {
        self.initialize_singleton();

        NetworkServerStatic::set_disconnect_inactive_connections(
            self.disconnect_inactive_connections,
        );
        NetworkServerStatic::set_disconnect_inactive_timeout(self.disconnect_inactive_timeout);
        NetworkServerStatic::set_exceptions_disconnect(self.exceptions_disconnect);

        if let Some(ref mut authenticator) = self.authenticator() {
            authenticator.on_start_server();
            NetworkAuthenticatorTraitStatic::set_on_server_authenticated(Box::new(
                Self::on_server_authenticated,
            ));
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
            Box::new(Self::on_server_add_player_internal),
            true,
        );
        // 添加 ReadyMessage 消息处理
        NetworkServer::replace_handler::<ReadyMessage>(
            Box::new(Self::on_server_ready_message_internal),
            true,
        );
    }

    fn on_server_error(conn: &mut NetworkConnectionToClient, error: TransportError) {
        let (_, _) = (conn, error);
    }

    fn on_server_authenticated(conn: &mut NetworkConnectionToClient) {
        // 获取 NetworkManagerTrait 的单例
        conn.set_authenticated(true);

        // 获取 NetworkManagerTrait 的单例
        let network_manager = NetworkManagerStatic::get_network_manager_singleton();
        // offline_scene
        let offline_scene = network_manager.offline_scene();

        // 获取 场景名称
        let network_scene_name = NetworkManagerStatic::get_network_scene_name();
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

    fn on_server_add_player_internal(
        conn_id: u64,
        _reader: &mut NetworkReader,
        _channel: TransportChannel,
    ) {
        // 获取 NetworkManagerTrait 的单例
        let network_manager = NetworkManagerStatic::get_network_manager_singleton();

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
            START_POSITIONS.write().unwrap().push(start.clone());
        }
    }

    fn update_scene() {
        // TODO  UpdateScene
    }

    fn is_server_online_scene_change_needed(&self) -> bool {
        self.online_scene != self.offline_scene
    }

    fn apply_configuration(&mut self) {
        NetworkServerStatic::set_tick_rate(self.send_rate);
    }

    fn stop_server(&mut self) {
        if !NetworkServerStatic::active() {
            log_warn!("Server already stopped.");
            return;
        }

        if let Some(ref mut authenticator) =
            NetworkManagerStatic::get_network_manager_singleton().authenticator()
        {
            authenticator.on_stop_server();
        }

        self.on_stop_server();

        NetworkServer::shutdown();

        self.mode = NetworkManagerMode::Offline;

        NetworkManagerStatic::set_start_positions_index(0);

        NetworkManagerStatic::set_network_scene_name("".to_string());
    }
}

pub trait NetworkManagerTrait {
    // 字段 get  set
    fn authenticator(&mut self) -> &mut Option<Box<dyn NetworkAuthenticatorTrait>>;
    fn set_authenticator(&mut self, authenticator: Box<dyn NetworkAuthenticatorTrait>);
    fn offline_scene(&self) -> &str;
    fn set_offline_scene(&mut self, scene_name: &'static str);
    fn auto_create_player(&self) -> bool;
    fn set_auto_create_player(&mut self, auto_create_player: bool);
    fn player_obj(&self) -> &GameObject;
    fn set_player_obj(&mut self, player_obj: GameObject);
    fn player_spawn_method(&self) -> &PlayerSpawnMethod;
    fn set_player_spawn_method(&mut self, player_spawn_method: PlayerSpawnMethod);
    fn snapshot_interpolation_settings(&self) -> &SnapshotInterpolationSetting;
    fn on_server_connect_internal(
        conn: &mut NetworkConnectionToClient,
        transport_error: TransportError,
    ) where
        Self: Sized;
    fn on_server_connect(conn: &mut NetworkConnectionToClient)
    where
        Self: Sized,
    {
        let _ = conn;
    }
    fn on_server_disconnect(conn: &mut NetworkConnectionToClient, transport_error: TransportError)
    where
        Self: Sized;
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

    fn awake()
    where
        Self: Sized;
    fn get_start_position(&mut self) -> Transform {
        if START_POSITIONS.read().unwrap().len() == 0 {
            return Transform::default();
        }

        if *self.player_spawn_method() == PlayerSpawnMethod::Random {
            let index = rand::random::<u32>() % START_POSITIONS.read().unwrap().len() as u32;
            return START_POSITIONS.read().unwrap()[index as usize].clone();
        }
        let index = NetworkManagerStatic::get_start_positions_index();
        NetworkManagerStatic::set_start_positions_index(
            index + 1 % START_POSITIONS.read().unwrap().len(),
        );
        START_POSITIONS.read().unwrap()[index].clone()
    }
    fn on_server_add_player(&mut self, conn_id: u64) {
        let mut player_obj = self.player_obj().clone();

        if player_obj.is_null() {
            log_error!("The PlayerPrefab is empty on the NetworkManager. Please setup a PlayerPrefab object.");
            return;
        }

        // 修改 player_obj 的 transform 属性
        player_obj.transform = self.get_start_position();

        NetworkServer::add_player_for_connection(conn_id, player_obj);
    }
    fn is_network_active(&self) -> bool {
        NetworkServerStatic::active()
    }
    fn set_network_manager_mode(&mut self, mode: NetworkManagerMode);
    fn get_network_manager_mode(&mut self) -> &NetworkManagerMode;
    fn on_validate(&mut self);
    fn reset(&mut self);
    fn start(&mut self);
    fn update(&mut self);
    fn late_update(&mut self);
    fn on_start_server(&mut self) {}
    fn on_stop_server(&mut self) {}
    fn server_change_scene(&mut self, new_scene_name: &str);
}

impl NetworkManagerTrait for NetworkManager {
    fn authenticator(&mut self) -> &mut Option<Box<dyn NetworkAuthenticatorTrait>> {
        &mut self.authenticator
    }

    fn set_authenticator(&mut self, authenticator: Box<dyn NetworkAuthenticatorTrait>) {
        self.authenticator.replace(authenticator);
    }

    fn offline_scene(&self) -> &str {
        self.offline_scene.as_str()
    }

    fn set_offline_scene(&mut self, scene_name: &'static str) {
        self.offline_scene = scene_name.to_string();
    }

    fn auto_create_player(&self) -> bool {
        self.auto_create_player
    }

    fn set_auto_create_player(&mut self, auto_create_player: bool) {
        self.auto_create_player = auto_create_player;
    }

    fn player_obj(&self) -> &GameObject {
        &self.player_obj
    }

    fn set_player_obj(&mut self, player_obj: GameObject) {
        self.player_obj = player_obj;
    }

    fn player_spawn_method(&self) -> &PlayerSpawnMethod {
        &self.player_spawn_method
    }

    fn set_player_spawn_method(&mut self, player_spawn_method: PlayerSpawnMethod) {
        self.player_spawn_method = player_spawn_method;
    }

    fn snapshot_interpolation_settings(&self) -> &SnapshotInterpolationSetting {
        &self.snapshot_interpolation_settings
    }

    // OnServerConnectInternal
    fn on_server_connect_internal(
        conn: &mut NetworkConnectionToClient,
        _transport_error: TransportError,
    ) where
        Self: Sized,
    {
        // 获取 NetworkManagerTrait 的单例
        let network_manager = NetworkManagerStatic::get_network_manager_singleton();

        // 如果 NetworkManager 的 authenticator 不为空
        if let Some(authenticator) = network_manager.authenticator() {
            // 调用 NetworkAuthenticatorTrait 的 on_server_connect 方法
            authenticator.on_server_authenticate(conn);
        } else {
            // 如果 NetworkManager 的 authenticator 为空
            Self::on_server_authenticated(conn);
        }
    }

    // OnServerDisconnect
    fn on_server_disconnect(conn: &mut NetworkConnectionToClient, _transport_error: TransportError)
    where
        Self: Sized,
    {
        NetworkServer::destroy_player_for_connection(conn);
    }

    fn awake() {
        let backend_data = BackendDataStatic::get_backend_data();
        if backend_data.network_manager_settings.len() == 0 {
            panic!("No NetworkManager settings found in the BackendData. Please add a NetworkManager setting.");
        }

        NetworkBehaviourFactory::register_network_behaviour_factory();

        let network_manager_setting = backend_data.network_manager_settings[0].clone();

        let mut spawn_prefabs = Vec::new();
        for spawn_prefab in &network_manager_setting.spawn_prefabs {
            spawn_prefabs.push(GameObject::new(spawn_prefab.clone()));
        }

        let manager = Self {
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
            player_obj: GameObject::new(network_manager_setting.player_prefab.clone()),
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
        };

        NetworkManagerStatic::set_network_manager_singleton(Box::new(manager));
    }

    fn set_network_manager_mode(&mut self, mode: NetworkManagerMode) {
        self.mode = mode;
    }

    fn get_network_manager_mode(&mut self) -> &NetworkManagerMode {
        &self.mode
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

    fn reset(&mut self) {
        log_debug!("NetworkManager reset");
    }

    fn start(&mut self) {
        if !self.initialize_singleton() {
            return;
        }
        self.apply_configuration();

        NetworkManagerStatic::set_network_scene_name(self.online_scene.to_string());

        // TODO SceneManager.sceneLoaded += OnSceneLoaded;

        self.start_server();
    }

    fn update(&mut self) {
        self.apply_configuration();
    }

    fn late_update(&mut self) {
        Self::update_scene();
    }

    fn on_start_server(&mut self) {}
    fn server_change_scene(&mut self, _new_scene_name: &str) {
        // TODO  SceneManager.LoadScene(newSceneName);
    }
}

#[cfg(test)]
mod tests {
    use crate::mirror::core::network_manager::NetworkManagerStatic;

    #[test]
    fn test_network_manager() {
        NetworkManagerStatic::set_network_scene_name("test".to_string());
        println!("{}", NetworkManagerStatic::get_network_scene_name());
    }
}
