use crate::core::connection_quality::ConnectionQualityMethod;
use crate::core::network_authenticator::NetworkAuthenticatorTrait;
use crate::core::network_server::NetworkServer;
use crate::core::transport::Transport;
use atomic::Atomic;
use dashmap::DashMap;
use lazy_static::lazy_static;
use nalgebra::Vector3;
use std::sync::atomic::Ordering;
use tklog::{info, warn};

static mut SINGLETON: Option<Box<dyn NetworkManagerTrait>> = None;
static mut NETWORK_SCENE_NAME: &'static str = "";


lazy_static! {
    pub static ref START_POSITIONS: DashMap<u32,Vector3<f32>> = DashMap::new();
    pub static ref START_POSITIONS_INDEX: Atomic<u32> = Atomic::new(0);
}

#[derive(Clone)]
pub enum PlayerSpawnMethod {
    Random,
    RoundRobin,
}

#[derive(Clone)]
pub enum NetworkManagerMode {
    Offline,
    Server,
}


pub struct NetworkManager {
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
    // todo fix
    pub player_prefab: String,
    pub auto_create_player: bool,
    pub player_spawn_method: PlayerSpawnMethod,
    // todo fix
    pub spawn_prefabs: Vec<String>,
    pub exceptions_disconnect: bool,
    //  todo  add SnapshotSettings
    // pub snapshotSettings: SnapshotSettings,
    pub evaluation_method: ConnectionQualityMethod,
    pub evaluation_interval: f32,
    pub time_interpolation_gui: bool,
}

impl NetworkManager {
    pub fn new() -> Self {
        NetworkManager {
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
            player_prefab: "".to_string(),
            auto_create_player: true,
            player_spawn_method: PlayerSpawnMethod::Random,
            spawn_prefabs: Vec::new(),
            exceptions_disconnect: true,
            evaluation_method: ConnectionQualityMethod::Simple,
            evaluation_interval: 3.0,
            time_interpolation_gui: false,
        }
    }

    fn initialize_singleton(&self) -> bool {
        if Self::singleton_exists() {
            return true;
        }
        if self.dont_destroy_on_load {
            if Self::singleton_exists() {
                warn!("NetworkManager already exists in the scene. Deleting the new one.");
                return false;
            }
            // Self::set_singleton(Box::new(self.clone()));
        } else {
            // Self::set_singleton(Box::new(self.clone()));
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
        }

        NetworkServer::listen(self.max_connections);

        Self::register_server_messages();
    }

    fn register_server_messages() {
        // TODO NetworkServer.RegisterHandler<NetworkPingMessage>(OnPingMessage);
    }

    fn update_scene() {
        // TODO  UpdateScene
    }

    fn is_server_online_scene_change_needed(&self) -> bool {
        self.online_scene != self.offline_scene
    }

    fn apply_configuration(&self) {
        NetworkServer::set_static_tick_rate(self.send_rate);
        // NetworkClient.snapshotSettings = snapshotSettings;
        // NetworkClient.connectionQualityInterval = evaluationInterval;
        // NetworkClient.connectionQualityMethod = evaluationMethod;
    }

    // ********************************************************************
    pub fn get_singleton() -> &'static mut Box<dyn NetworkManagerTrait> {
        unsafe {
            if let Some(ref mut singleton) = SINGLETON {
                return singleton;
            }
            panic!("NetworkManager singleton not found.");
        }
    }

    pub fn singleton_exists() -> bool {
        unsafe {
            SINGLETON.is_some()
        }
    }

    pub fn set_singleton(network_manager: Box<dyn NetworkManagerTrait>) {
        unsafe {
            SINGLETON.replace(network_manager);
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
    pub fn get_next_start_position() -> Vector3<f32> {
        let index = START_POSITIONS_INDEX.fetch_add(1, Ordering::SeqCst);
        let pos = START_POSITIONS.get(&index);
        match pos {
            Some(p) => p.clone(),
            // todo 实现生成 位置
            None => Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

pub trait NetworkManagerTrait {
    fn is_network_active(&self) -> bool {
        NetworkServer::get_static_active()
    }
    fn set_network_manager_mode(&mut self, mode: NetworkManagerMode);
    fn get_network_manager_mode(&mut self) -> &NetworkManagerMode;
    fn on_validate(&mut self);
    fn reset(&mut self);
    fn awake(&mut self);
    fn start(&mut self);
    fn update(&mut self);
    fn late_update(&mut self);
    fn on_start_server(&mut self);
    fn server_change_scene(&mut self, new_scene_name: &str);
}

impl NetworkManagerTrait for NetworkManager {
    fn set_network_manager_mode(&mut self, mode: NetworkManagerMode) {
        self.mode = mode;
    }

    fn get_network_manager_mode(&mut self) -> &NetworkManagerMode {
        &self.mode
    }

    fn on_validate(&mut self) {
        self.max_connections = self.max_connections.max(0);
        // TODO   if (playerPrefab != null &&
    }

    fn reset(&mut self) {
        info!("NetworkManager reset");
    }

    fn awake(&mut self) {
        if !self.initialize_singleton() {
            return;
        }
        self.apply_configuration();

        Self::set_network_scene_name(self.online_scene)

        // TODO SceneManager.sceneLoaded += OnSceneLoaded;
    }

    fn start(&mut self) {
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