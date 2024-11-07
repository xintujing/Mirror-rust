use crate::core::backend_data::BACKEND_DATA;
use crate::core::connection_quality::ConnectionQualityMethod;
use crate::core::messages::{AddPlayerMessage, ReadyMessage};
use crate::core::network_authenticator::NetworkAuthenticatorTrait;
use crate::core::network_connection::NetworkConnectionTrait;
use crate::core::network_connection_to_client::NetworkConnectionToClient;
use crate::core::network_identity::NetworkIdentity;
use crate::core::network_reader::NetworkReader;
use crate::core::network_server::NetworkServer;
use crate::core::snapshot_interpolation::snapshot_interpolation_settings::SnapshotInterpolationSettings;
use crate::core::transport::{Transport, TransportChannel};
use atomic::Atomic;
use dashmap::DashMap;
use lazy_static::lazy_static;
use nalgebra::{Quaternion, Vector3};
use std::sync::atomic::Ordering;
use tklog::{debug, error, info, warn};

static mut NETWORK_MANAGER_SINGLETON: Option<Box<dyn NetworkManagerTrait>> = None;
static mut NETWORK_SCENE_NAME: &'static str = "";


lazy_static! {
    static ref START_POSITIONS: Vec<Transform> = Vec::new();
    static ref START_POSITIONS_INDEX: Atomic<u32> = Atomic::new(0);
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

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub positions: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn new(positions: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        Self {
            positions,
            rotation,
            scale,
        }
    }

    pub fn default() -> Self {
        Self {
            positions: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameObject {
    pub name: String,
    pub prefab: String,
    pub transform: Transform,
}

impl GameObject {
    pub fn default() -> Self {
        Self {
            name: "".to_string(),
            prefab: "".to_string(),
            transform: Transform::default(),
        }
    }
    pub fn is_has_component(&self) -> bool {
        if self.prefab == "" {
            return false;
        }
        if let None = BACKEND_DATA.get_asset_id_by_asset_name(self.prefab.as_str()) {
            return false;
        }
        true
    }
    pub fn get_component(&mut self) -> Option<NetworkIdentity> {
        if let Some(asset_id) = BACKEND_DATA.get_asset_id_by_asset_name(self.prefab.as_str()) {
            let mut identity = NetworkIdentity::new(asset_id);
            identity.game_object = self.clone();
            return Some(identity);
        };
        None
    }
    pub fn is_null(&self) -> bool {
        self.name == "" && self.prefab == ""
    }
}

impl PartialEq for GameObject {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.prefab == other.prefab
    }
}

pub struct NetworkManager {
    player_obj: GameObject,
    pub mode: NetworkManagerMode,
    pub dont_destroy_on_load: bool,
    pub editor_auto_start: bool,
    pub send_rate: u32,
    pub offline_scene: &'static str,
    pub online_scene: &'static str,
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
    pub evaluation_method: ConnectionQualityMethod,
    pub evaluation_interval: f32,
    pub time_interpolation_gui: bool,
}

impl NetworkManager {
    fn initialize_singleton(&self) -> bool {
        if Self::network_manager_singleton_exists() {
            return true;
        }
        if self.dont_destroy_on_load {
            if Self::network_manager_singleton_exists() {
                warn!("NetworkManager already exists in the scene. Deleting the new one.");
                return false;
            }
        }

        if !Transport::active_transport_exists() {
            panic!("No transport found in the scene. Add a transport component to the scene.");
        }
        true
    }

    pub fn start_server(&mut self) {
        if NetworkServer::get_static_active() {
            warn!("Server already started.");
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

        NetworkServer::set_static_disconnect_inactive_connections(self.disconnect_inactive_connections);
        NetworkServer::set_static_disconnect_inactive_timeout(self.disconnect_inactive_timeout);
        //  TODO  exceptionsDisconnect

        if let Some(ref mut authenticator) = self.authenticator {
            authenticator.on_start_server();
            // TODO authenticator.OnServerAuthenticated.AddListener(OnServerAuthenticated);
        }

        NetworkServer::listen(self.max_connections);

        Self::register_server_messages();
    }

    // zhuce
    fn register_server_messages() {
        // TODO NetworkServer.RegisterHandler<NetworkPingMessage>(OnPingMessage);
        NetworkServer::register_handler::<AddPlayerMessage>(Box::new(Self::on_server_add_player_internal), true);
        NetworkServer::replace_handler::<ReadyMessage>(Box::new(Self::on_server_ready_message_internal), true);
    }

    fn on_server_ready_message_internal(conn_id: u64, reader: &mut NetworkReader, channel: TransportChannel) {
        Self::on_server_ready(conn_id)
    }

    fn on_server_ready(conn_id: u64) {
        NetworkServer::set_client_ready(conn_id);
    }

    fn on_server_add_player_internal(conn_id: u64, reader: &mut NetworkReader, channel: TransportChannel) {
        debug!("on_server_add_player_internal");
        // 获取 NetworkManagerTrait 的单例
        let singleton = Self::get_network_manager_singleton();

        // 获取 NetworkManager 的 NetworkManager 对象
        let network_manager = singleton.get_network_manager();

        // 如果 NetworkManager 的 auto_create_player 为 true 且 player_obj.prefab 为空
        if network_manager.auto_create_player && network_manager.player_obj.prefab == "" {
            error!("The PlayerPrefab is empty on the NetworkManager. Please setup a PlayerPrefab object.");
            return;
        }

        // 如果 NetworkManager 的 auto_create_player 为 true 且 player_obj.prefab 不为空
        if network_manager.auto_create_player {
            // 如果 player_obj.prefab 不为空，且 player_obj.prefab 不存在于 BACKEND_DATA 的 asset_id 中
            if let Some(asset_id) = BACKEND_DATA.get_asset_id_by_asset_name(network_manager.player_obj.prefab.as_str()) {
                if let None = BACKEND_DATA.get_network_identity_data_by_asset_id(asset_id) {
                    error!("The PlayerPrefab does not have a NetworkIdentity. Please add a NetworkIdentity to the player prefab.");
                    return;
                }
            }
        }

        // 如果 NetworkManager 的 auto_create_player 为 false
        if let Some(mut connection) = NetworkServer::get_static_network_connections().get_mut(&conn_id) {
            if connection.net_id() != 0 {
                error!("There is already a player for this connection.");
                return;
            }
        }
        // 调用 NetworkManagerTrait 的 on_server_add_player 方法
        network_manager.on_server_add_player(conn_id);
    }

    fn update_scene() {
        // TODO  UpdateScene
    }

    fn is_server_online_scene_change_needed(&self) -> bool {
        self.online_scene != self.offline_scene
    }

    fn apply_configuration(&mut self) {
        NetworkServer::set_static_tick_rate(self.send_rate);
        // NetworkClient.snapshot_settings = snapshot_settings;
        // NetworkClient.connectionQualityInterval = evaluationInterval;
        // NetworkClient.connectionQualityMethod = evaluationMethod;
    }

    // ********************************************************************
    pub fn get_network_manager_singleton() -> &'static mut Box<dyn NetworkManagerTrait> {
        unsafe {
            if let Some(ref mut singleton) = NETWORK_MANAGER_SINGLETON {
                return singleton;
            }
            panic!("NetworkManager singleton not found.");
        }
    }

    pub fn network_manager_singleton_exists() -> bool {
        unsafe {
            NETWORK_MANAGER_SINGLETON.is_some()
        }
    }

    pub fn set_network_manager_singleton(network_manager: Box<dyn NetworkManagerTrait>) {
        unsafe {
            NETWORK_MANAGER_SINGLETON.replace(network_manager);
        }
    }
    pub fn get_network_scene_name() -> &'static str {
        unsafe {
            NETWORK_SCENE_NAME
        }
    }
    pub fn set_network_scene_name(name: &'static str) {
        unsafe {
            NETWORK_SCENE_NAME = name;
        }
    }
}

pub trait NetworkManagerTrait {
    fn awake()
    where
        Self: Sized;
    fn get_start_position(&mut self) -> Option<Transform> {
        if START_POSITIONS.len() == 0 {
            return None;
        }

        if self.get_network_manager().player_spawn_method == PlayerSpawnMethod::Random {
            let index = rand::random::<u32>() % START_POSITIONS.len() as u32;
            return Some(START_POSITIONS[index as usize].clone());
        }
        let index = START_POSITIONS_INDEX.load(Ordering::Relaxed);
        START_POSITIONS_INDEX.store(index + 1 % START_POSITIONS.len() as u32, Ordering::Relaxed);
        Some(START_POSITIONS[index as usize].clone())
    }
    fn on_server_add_player(&mut self, conn_id: u64) {
        let mut player_obj = self.get_network_manager().player_obj.clone();

        if player_obj.is_null() {
            error!("The PlayerPrefab is empty on the NetworkManager. Please setup a PlayerPrefab object.");
            return;
        }

        let mut start_position = Transform::default();
        if let Some(sp) = self.get_start_position() {
            start_position = sp;
        }

        // 修改 player_obj 的 transform 属性
        player_obj.transform = start_position;

        NetworkServer::add_player_for_connection(conn_id, player_obj);
    }
    fn get_network_manager(&mut self) -> &mut NetworkManager;
    fn is_network_active(&self) -> bool {
        NetworkServer::get_static_active()
    }
    fn set_network_manager_mode(&mut self, mode: NetworkManagerMode);
    fn get_network_manager_mode(&mut self) -> &NetworkManagerMode;
    fn on_validate(&mut self);
    fn reset(&mut self);
    fn start(&mut self);
    fn update(&mut self);
    fn late_update(&mut self);
    fn on_start_server(&mut self);
    fn server_change_scene(&mut self, new_scene_name: &str);
}

impl NetworkManagerTrait for NetworkManager {
    fn awake() {
        let mut manager = Self {
            mode: NetworkManagerMode::Offline,
            dont_destroy_on_load: true,
            editor_auto_start: false,
            send_rate: 60,
            offline_scene: "",
            online_scene: "",
            offline_scene_load_delay: 0.0,
            network_address: "0.0.0.0".to_string(),
            max_connections: 100,
            disconnect_inactive_connections: false,
            disconnect_inactive_timeout: 60.0,
            authenticator: None,
            player_obj: GameObject::default(),
            auto_create_player: true,
            player_spawn_method: PlayerSpawnMethod::Random,
            spawn_prefabs: Vec::new(),
            exceptions_disconnect: true,
            evaluation_method: ConnectionQualityMethod::Simple,
            evaluation_interval: 3.0,
            time_interpolation_gui: false,
        };
        // TODO  fix  NetworkManager cfg
        manager.player_obj.prefab = "Assets/QuickStart/Player.prefab".to_string();

        NetworkManager::set_network_manager_singleton(Box::new(manager));
    }

    fn get_network_manager(&mut self) -> &mut NetworkManager {
        self
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
            error!("NetworkManager - Player Prefab must have a NetworkIdentity.");
        }

        if !self.player_obj.is_null() && self.spawn_prefabs.contains(&self.player_obj) {
            warn!("NetworkManager - Player Prefab doesn't need to be in Spawnable Prefabs list too. Removing it.");
            self.spawn_prefabs.retain(|x| x != &self.player_obj);
        }
    }

    fn reset(&mut self) {
        info!("NetworkManager reset");
    }

    fn start(&mut self) {
        if !self.initialize_singleton() {
            return;
        }
        self.apply_configuration();

        Self::set_network_scene_name(self.online_scene);

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
    fn server_change_scene(&mut self, new_scene_name: &str) {
        // TODO  SceneManager.LoadScene(newSceneName);
    }
}

#[cfg(test)]
mod tests {
    use crate::core::network_manager::NetworkManager;

    #[test]
    fn test_network_manager() {
        NetworkManager::set_network_scene_name("test");
        println!("{}", NetworkManager::get_network_scene_name());
    }
}