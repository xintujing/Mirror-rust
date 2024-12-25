use crate::mirror::authenticators::network_authenticator::{
    NetworkAuthenticatorTrait, NetworkAuthenticatorTraitStatic,
};
use crate::mirror::components::network_room_player::NetworkRoomPlayer;
use crate::mirror::components::network_transform::network_transform_base::Transform;
use crate::mirror::core::backend_data::{BackendDataStatic, SnapshotInterpolationSetting};
use crate::mirror::core::connection_quality::ConnectionQualityMethod;
use crate::mirror::core::messages::{AddPlayerMessage, ReadyMessage, SceneMessage, SceneOperation};
use crate::mirror::core::network_behaviour::{
    GameObject, NetworkBehaviour, NetworkBehaviourTrait, SyncDirection, SyncMode,
};
use crate::mirror::core::network_connection::NetworkConnectionTrait;
use crate::mirror::core::network_connection_to_client::NetworkConnectionToClient;
use crate::mirror::core::network_manager::{
    NetworkManager, NetworkManagerMode, NetworkManagerStatic, NetworkManagerTrait,
    PlayerSpawnMethod,
};
use crate::mirror::core::network_reader::NetworkReader;
use crate::mirror::core::network_server::{EventHandlerType, NetworkServer, NetworkServerStatic};
use crate::mirror::core::transport::{Transport, TransportChannel, TransportError};
use crate::{log_debug, log_error, log_warn};
use dashmap::try_result::TryResult;
use std::any::Any;

pub struct PendingPlayer {
    pub conn: u64,
    pub room_player: GameObject,
}

pub struct NetworkRoomManager {
    pub network_manager: NetworkManager,
    pub min_players: i32,
    pub room_player_prefab: NetworkRoomPlayer,
    pub room_scene: String,
    pub gameplay_scene: String,
    pub pending_players: Vec<PendingPlayer>,
    _all_players_ready: bool,
    pub room_slots: Vec<u32>,
    pub client_index: i32,
}

impl NetworkRoomManager {
    pub fn all_players_ready(&self) -> bool {
        self._all_players_ready
    }

    pub fn set_all_players_ready(&mut self, value: bool) {
        let was_ready = self._all_players_ready;
        let now_ready = value;
        if was_ready != now_ready {
            self._all_players_ready = value;
            match now_ready {
                true => {
                    self.on_room_server_players_ready();
                }
                false => {
                    self.on_room_server_players_not_ready();
                }
            }
        }
    }

    // OnRoomServerPlayersReady
    fn on_room_server_players_ready(&mut self) {
        self.server_change_scene(self.gameplay_scene.to_string());
    }

    // OnRoomServerPlayersNotReady
    fn on_room_server_players_not_ready(&mut self) {}

    fn initialize_singleton(&self) -> bool {
        if NetworkManagerStatic::network_manager_singleton_exists() {
            return true;
        }
        if self.network_manager.dont_destroy_on_load {
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

        self.network_manager.mode = NetworkManagerMode::ServerOnly;

        self.network_manager.setup_server();

        self.network_manager.on_start_server();

        if self.is_server_online_scene_change_needed() {
            self.server_change_scene(self.online_scene().to_string());
        } else {
            // TODO NetworkServer.SpawnObjects();
            NetworkServer::spawn_objects();
        }
    }

    fn setup_server(&mut self) {
        self.initialize_singleton();

        NetworkServerStatic::set_disconnect_inactive_connections(
            self.network_manager.disconnect_inactive_connections,
        );
        NetworkServerStatic::set_disconnect_inactive_timeout(
            self.network_manager.disconnect_inactive_timeout,
        );
        NetworkServerStatic::set_exceptions_disconnect(self.network_manager.exceptions_disconnect);

        if let Some(ref mut authenticator) = self.authenticator() {
            authenticator.on_start_server();
            NetworkAuthenticatorTraitStatic::set_on_server_authenticated(
                Self::on_server_authenticated,
            );
        }

        NetworkServer::listen(self.network_manager.max_connections);

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
        let offline_scene = network_manager.offline_scene();

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

    // OnServerReady(
    fn on_server_ready(conn_id: u64) {
        NetworkServer::set_client_ready(conn_id);

        // TODO SceneLoadedForPlayer
    }

    fn on_server_add_player_internal(
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

    fn is_server_online_scene_change_needed(&self) -> bool {
        self.online_scene() != self.offline_scene()
    }

    fn apply_configuration(&mut self) {
        NetworkServerStatic::set_tick_rate(self.network_manager.send_rate);
    }

    pub fn ready_status_changed(&mut self) {
        let mut current_players = 0;
        let mut ready_players = 0;

        for net_id in self.room_slots.to_vec().iter() {
            match NetworkServerStatic::spawned_network_identities().try_get_mut(net_id) {
                TryResult::Present(mut identity) => {
                    let room_player = identity.get_component::<NetworkRoomPlayer>();
                    if room_player.is_some() {
                        current_players += 1;
                        if room_player.unwrap().ready_to_begin {
                            ready_players += 1;
                        }
                    }
                }
                TryResult::Absent => {
                    log_error!("Failed to on_server_disconnect for identity because of absent");
                }
                TryResult::Locked => {
                    log_error!("Failed to on_server_disconnect for identity because of locked");
                }
            }

            if current_players == ready_players {
                self.check_ready_to_begin();
            } else {
                self.set_all_players_ready(false);
            }
        }
    }

    fn check_ready_to_begin(&mut self) {}

    pub fn recalculate_room_player_indices(&mut self) {
        for (i, net_id) in self.room_slots.iter().enumerate() {
            match NetworkServerStatic::spawned_network_identities().try_get_mut(&net_id) {
                TryResult::Present(mut identity) => {
                    if let Some(player) = identity.get_component::<NetworkRoomPlayer>() {
                        player.index = i as i32
                    }
                }
                TryResult::Absent => {
                    log_error!(
                        "Failed to recalculate_room_player_indices for identity because of absent"
                    );
                }
                TryResult::Locked => {
                    log_error!(
                        "Failed to recalculate_room_player_indices for identity because of locked"
                    );
                }
            }
        }
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

        self.network_manager.mode = NetworkManagerMode::Offline;

        NetworkManagerStatic::set_start_positions_index(0);

        NetworkManagerStatic::set_network_scene_name("".to_string());

        #[allow(warnings)]
        unsafe {
            // NetworkManagerStatic::network_manager_singleton().take();
        }
    }

    fn update_scene(&mut self) {
        if NetworkServerStatic::is_loading_scene() {
            self.finish_load_scene();
        }
    }

    fn finish_load_scene(&mut self) {
        NetworkServerStatic::set_is_loading_scene(false);

        match self.network_manager.mode {
            NetworkManagerMode::ServerOnly => {
                self.finish_load_scene_server_only();
            }
            _ => {}
        }
    }

    fn finish_load_scene_server_only(&mut self) {
        // TODO NetworkServer.SpawnObjects();
        NetworkServer::spawn_objects();
        self.on_server_change_scene(NetworkManagerStatic::network_scene_name());
    }
}

impl NetworkManagerTrait for NetworkRoomManager {
    fn authenticator(&mut self) -> &mut Option<Box<dyn NetworkAuthenticatorTrait>> {
        &mut self.network_manager.authenticator
    }

    fn set_authenticator(&mut self, authenticator: Box<dyn NetworkAuthenticatorTrait>) {
        self.network_manager.set_authenticator(authenticator);
    }

    fn network_address(&self) -> &str {
        self.network_manager.network_address()
    }

    fn offline_scene(&self) -> &str {
        self.room_scene.as_str()
    }

    fn set_offline_scene(&mut self, scene_name: &'static str) {
        self.room_scene = scene_name.to_string();
    }

    fn online_scene(&self) -> &str {
        self.gameplay_scene.as_str()
    }

    fn set_online_scene(&mut self, scene_name: &'static str) {
        self.gameplay_scene = scene_name.to_string();
    }

    fn auto_create_player(&self) -> bool {
        self.network_manager.auto_create_player()
    }

    fn set_auto_create_player(&mut self, auto_create_player: bool) {
        self.network_manager
            .set_auto_create_player(auto_create_player);
    }

    fn player_obj(&self) -> &GameObject {
        self.network_manager.player_obj()
    }

    fn set_player_obj(&mut self, player_obj: GameObject) {
        self.network_manager.set_player_obj(player_obj);
    }

    fn player_spawn_method(&self) -> &PlayerSpawnMethod {
        self.network_manager.player_spawn_method()
    }

    fn set_player_spawn_method(&mut self, player_spawn_method: PlayerSpawnMethod) {
        self.network_manager
            .set_player_spawn_method(player_spawn_method);
    }

    fn snapshot_interpolation_settings(&self) -> &SnapshotInterpolationSetting {
        self.network_manager.snapshot_interpolation_settings()
    }

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

    // OnServerDisconnect
    fn on_server_disconnect(conn: &mut NetworkConnectionToClient, _transport_error: TransportError)
    where
        Self: Sized,
    {
        let network_room_manager = NetworkManagerStatic::network_manager_singleton()
            .as_any_mut()
            .downcast_mut::<Self>()
            .unwrap();
        network_room_manager
            .room_slots
            .retain(|&x| x != conn.net_id());
    }

    fn awake() {
        let backend_data = BackendDataStatic::get_backend_data();
        if backend_data.network_manager_settings.len() == 0 {
            panic!("No NetworkManager settings found in the BackendData. Please add a NetworkManager setting.");
        }

        let network_manager_setting = backend_data.network_manager_settings[0].clone();

        let mut spawn_prefabs = Vec::new();
        for spawn_prefab in &network_manager_setting.spawn_prefabs {
            spawn_prefabs.push(GameObject::new_with_prefab(spawn_prefab.clone()));
        }

        let network_manager = NetworkManager {
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
        };

        let network_room_manager = NetworkRoomManager {
            network_manager,
            min_players: 1,
            // TODO fix
            room_player_prefab: NetworkRoomPlayer {
                network_behaviour: NetworkBehaviour {
                    sync_interval: 0.0,
                    last_sync_time: 0.0,
                    sync_direction: SyncDirection::ServerToClient,
                    sync_mode: SyncMode::Observers,
                    index: 0,
                    sync_var_dirty_bits: 0,
                    sync_object_dirty_bits: 0,
                    net_id: 0,
                    connection_to_client: 0,
                    observers: vec![],
                    game_object: GameObject {
                        scene_name: "".to_string(),
                        prefab: "".to_string(),
                        transform: Transform::default(),
                        active: false,
                    },
                    sync_objects: vec![],
                    sync_var_hook_guard: 0,
                },
                ready_to_begin: false,
                index: 0,
            },
            room_scene: "".to_string(),
            gameplay_scene: "".to_string(),
            pending_players: vec![],
            _all_players_ready: false,
            room_slots: vec![],
            client_index: 0,
        };

        NetworkManagerStatic::set_network_manager_singleton(Box::new(network_room_manager));
    }

    fn on_server_add_player(&mut self, conn_id: u64) {
        self.client_index += 1;
        self.set_all_players_ready(false);

        // 拿到 player_obj
        let mut player_obj = self.room_player_prefab.game_object().clone();
        if player_obj.is_null() {
            log_error!("The PlayerPrefab is empty on the NetworkManager. Please setup a PlayerPrefab object.");
            return;
        }

        // 修改 player_obj 的 transform 属性
        player_obj.transform = self.get_start_position();

        NetworkServer::add_player_for_connection(conn_id, player_obj);
    }

    fn set_network_manager_mode(&mut self, mode: NetworkManagerMode) {
        self.network_manager.set_network_manager_mode(mode);
    }

    fn get_network_manager_mode(&mut self) -> &NetworkManagerMode {
        &self.network_manager.get_network_manager_mode()
    }

    fn on_validate(&mut self) {
        self.network_manager.max_connections = self.network_manager.max_connections.max(0);

        if !self.network_manager.player_obj.is_null()
            && !self.network_manager.player_obj.is_has_component()
        {
            log_error!("NetworkManager - Player Prefab must have a NetworkIdentity.");
        }

        if !self.network_manager.player_obj.is_null()
            && self
            .network_manager
            .spawn_prefabs
            .contains(&self.network_manager.player_obj)
        {
            log_warn!("NetworkManager - Player Prefab doesn't need to be in Spawnable Prefabs list too. Removing it.");
            self.network_manager
                .spawn_prefabs
                .retain(|x| x != &self.network_manager.player_obj);
        }

        // NetworkRoomManager start

        // always <= maxConnections
        self.min_players = self
            .network_manager
            .max_connections
            .max(self.min_players as usize) as i32;
        // always >= 0
        self.min_players = self.min_players.max(0);

        if !self.room_player_prefab.game_object().is_null() {
            if !self.room_player_prefab.game_object().is_has_component() {
                log_error!("NetworkRoomManager - RoomPlayer Prefab must have a NetworkIdentity.");
            }
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

        NetworkManagerStatic::set_network_scene_name(self.offline_scene().to_string());

        self.start_server();
    }

    fn update(&mut self) {
        self.apply_configuration();
    }
    fn late_update(&mut self) {
        self.update_scene();
    }
    fn on_start_server(&mut self) {}

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

        if !NetworkServerStatic::active() && new_scene_name == self.online_scene() {
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
                &mut SceneMessage::new(new_scene_name.to_string(), SceneOperation::Normal, true),
                TransportChannel::Reliable,
                false,
            );
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
