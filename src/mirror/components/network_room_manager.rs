use crate::mirror::authenticators::network_authenticator::{
    NetworkAuthenticatorTrait, NetworkAuthenticatorTraitStatic,
};
use crate::mirror::components::network_room_player::NetworkRoomPlayer;
use crate::mirror::core::backend_data::{BackendDataStatic, SnapshotInterpolationSetting};
use crate::mirror::core::messages::{AddPlayerMessage, ReadyMessage, SceneMessage, SceneOperation};
use crate::mirror::core::network_behaviour::{GameObject, NetworkBehaviourTrait};
use crate::mirror::core::network_connection::NetworkConnectionTrait;
use crate::mirror::core::network_connection_to_client::NetworkConnectionToClient;
use crate::mirror::core::network_manager::{
    NetworkManager, NetworkManagerMode, NetworkManagerStatic, NetworkManagerTrait,
};
use crate::mirror::core::network_reader::NetworkReader;
use crate::mirror::core::network_server::{
    EventHandlerType, NetworkServer, NetworkServerStatic, ReplacePlayerOptions,
};
use crate::mirror::core::transport::{TransportChannel, TransportError};
use crate::{log_debug, log_error, log_warn};
use dashmap::try_result::TryResult;

pub struct PendingPlayer {
    pub conn: u64,
    pub room_player: GameObject,
}

pub struct NetworkRoomManager {
    pub network_manager: NetworkManager,
    pub min_players: i32,
    pub room_scene: String,
    pub gameplay_scene: String,
    pub room_player_prefab: GameObject,
    pub pending_players: Vec<PendingPlayer>,
    _all_players_ready: bool,
    pub room_slots: Vec<u32>,
    pub client_index: i32,
}

impl NetworkRoomManager {
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
            NetworkManager::on_server_add_player_internal,
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

    fn on_server_authenticated(conn: &mut NetworkConnectionToClient) {
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
        reader: &mut NetworkReader,
        channel: TransportChannel,
    ) {
        let (_, _) = (reader, channel);
        Self::on_server_ready(conn_id)
    }

    fn scene_loaded_for_player(conn_id: u64, mut room_player: GameObject) {
        let network_manager = NetworkManagerStatic::network_manager_singleton();
        let room_scene = network_manager.online_scene().to_string();
        if NetworkManagerStatic::network_scene_name() == room_scene {
            let pending_player = PendingPlayer {
                conn: conn_id,
                room_player: room_player.clone(),
            };

            network_manager.pending_players().push(pending_player);
            return;
        }

        let transform = network_manager.get_start_position();
        room_player.transform = transform;

        NetworkServer::replace_player_for_connection(
            conn_id,
            room_player,
            ReplacePlayerOptions::KeepAuthority,
        );
    }

    fn is_server_online_scene_change_needed(&self) -> bool {
        self.network_manager.online_scene != self.network_manager.offline_scene
    }

    fn check_ready_to_begin(&mut self) {}
}

pub trait NetworkRoomManagerTrait: NetworkManagerTrait {
    fn on_room_server_create_room_player(conn_id: u64) -> Option<GameObject>;
    fn on_room_server_scene_changed(new_scene_name: String);
    fn on_room_start_server(&mut self);
    fn on_room_server_connect(conn: &mut NetworkConnectionToClient);
    fn on_room_server_disconnect(conn: &mut NetworkConnectionToClient);
    // OnRoomServerPlayersReady
    fn on_room_server_players_ready(&mut self);
    // OnRoomServerPlayersNotReady
    fn on_room_server_players_not_ready(&mut self);
}

impl NetworkManagerTrait for NetworkRoomManager {
    fn authenticator(&mut self) -> &mut Option<Box<dyn NetworkAuthenticatorTrait>> {
        self.network_manager.authenticator()
    }
    fn set_authenticator(&mut self, authenticator: Box<dyn NetworkAuthenticatorTrait>) {
        self.network_manager.set_authenticator(authenticator);
    }

    fn set_mode(&mut self, mode: NetworkManagerMode) {
        self.network_manager.set_mode(mode);
    }

    fn snapshot_interpolation_settings(&self) -> &SnapshotInterpolationSetting {
        self.network_manager.snapshot_interpolation_settings()
    }

    fn offline_scene(&self) -> &String {
        self.network_manager.offline_scene()
    }

    fn online_scene(&self) -> &String {
        self.network_manager.online_scene()
    }

    fn auto_create_player(&self) -> bool {
        self.network_manager.auto_create_player()
    }

    fn player_obj(&self) -> &GameObject {
        self.network_manager.player_obj()
    }

    fn dont_destroy_on_load(&self) -> bool {
        self.network_manager.dont_destroy_on_load()
    }

    fn network_address(&self) -> &String {
        self.network_manager.network_address()
    }

    fn on_validate(&mut self) {
        // base.OnValidate();
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

        if !self.room_player_prefab.is_null() {
            if !self.room_player_prefab.is_has_component() {
                log_error!("NetworkRoomManager - RoomPlayer Prefab must have a NetworkIdentity.");
            }
        }
    }

    fn ready_status_changed(&mut self, component: &mut NetworkRoomPlayer) {
        let mut current_players = 0;
        let mut ready_players = 0;

        for net_id in self.room_slots.to_vec().iter() {
            match NetworkServerStatic::spawned_network_identities().try_get(net_id) {
                TryResult::Present(identity) => {
                    if !identity.get_component::<NetworkRoomPlayer, _>(|room_player| {
                        current_players += 1;
                        if room_player.ready_to_begin {
                            ready_players += 1;
                        }
                    }) {
                        match net_id == &component.net_id() {
                            true => {
                                current_players += 1;
                                if component.ready_to_begin {
                                    ready_players += 1;
                                }
                            }
                            false => {
                                log_error!(
                                    "Failed to ready_status_changed for identity because of locked"
                                );
                            }
                        }
                    }
                }
                TryResult::Absent => {
                    log_error!("Failed to ready_status_changed for identity because of absent");
                }
                TryResult::Locked => {
                    log_error!("Failed to ready_status_changed for identity because of locked");
                }
            }

            if current_players == ready_players {
                self.check_ready_to_begin();
            } else {
                self.set_all_players_ready(false);
            }
        }
    }

    fn room_slots(&mut self) -> &mut Vec<u32> {
        &mut self.room_slots
    }

    fn recalculate_room_player_indices(&mut self) -> (i32, u32) {
        let mut index = 0;
        let mut id = 0;
        for (i, net_id) in self.room_slots.iter().enumerate() {
            match NetworkServerStatic::spawned_network_identities().try_get(&net_id) {
                TryResult::Present(identity) => {
                    if !identity.get_component::<NetworkRoomPlayer, _>(|player| {
                        player.index = i as i32;
                        player.set_sync_var_dirty_bits(1 << 1);
                    }) {
                        index = i as i32;
                        id = *net_id;
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
        (index, id)
    }

    fn pending_players(&mut self) -> &mut Vec<PendingPlayer> {
        &mut self.pending_players
    }

    fn room_scene(&self) -> &String {
        &self.room_scene
    }

    fn gameplay_scene(&self) -> &String {
        &self.gameplay_scene
    }

    fn set_all_players_ready(&mut self, value: bool) {
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

    fn reset(&mut self) {
        log_debug!("NetworkManager reset");
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        // 获取 BackendData
        let backend_data = BackendDataStatic::get_backend_data();
        if backend_data.network_manager_settings.len() == 0 {
            panic!("No NetworkRoomManager settings found in the BackendData. Please add a NetworkRoomManager setting.");
        }

        // 获取 NetworkRoomManagerSetting
        let network_room_manager_setting = backend_data.network_room_manager_settings[0].clone();

        // 新建 NetworkManager
        let network_manager = NetworkManager::new_with_network_manager_setting(
            network_room_manager_setting.network_manager_setting,
        );

        // 获取 room_player_prefab_game_object
        let room_player_prefab_game_object =
            GameObject::new_with_prefab(network_room_manager_setting.room_player_prefab.clone());

        Self {
            network_manager,
            min_players: network_room_manager_setting.min_players,
            room_player_prefab: room_player_prefab_game_object,
            room_scene: network_room_manager_setting.room_scene.to_string(),
            gameplay_scene: network_room_manager_setting.gameplay_scene.to_string(),
            pending_players: Vec::new(),
            _all_players_ready: false,
            room_slots: Vec::new(),
            client_index: 0,
        }
    }

    fn start(&mut self) {
        if !Self::initialize_singleton() {
            return;
        }
        self.network_manager.apply_configuration();

        NetworkManagerStatic::set_network_scene_name(
            self.network_manager.offline_scene.to_string(),
        );

        if NetworkServerStatic::active() {
            log_warn!("Server already started.");
            return;
        }

        self.network_manager.mode = NetworkManagerMode::ServerOnly;

        NetworkServerStatic::set_disconnect_inactive_connections(
            self.network_manager.disconnect_inactive_connections,
        );
        NetworkServerStatic::set_disconnect_inactive_timeout(
            self.network_manager.disconnect_inactive_timeout,
        );
        NetworkServerStatic::set_exceptions_disconnect(self.network_manager.exceptions_disconnect);

        if let Some(authenticator) = self.network_manager.authenticator() {
            authenticator.on_start_server();
            NetworkAuthenticatorTraitStatic::set_on_server_authenticated(
                Self::on_server_authenticated,
            );
        }

        NetworkServer::listen(self.network_manager.max_connections);

        Self::register_server_messages();

        self.on_start_server();

        if self.is_server_online_scene_change_needed() {
            self.server_change_scene(self.network_manager.online_scene.to_string());
        } else {
            NetworkServer::spawn_objects();
        }
    }

    fn update(&mut self) {
        self.network_manager.apply_configuration();
    }

    fn late_update(&mut self) {
        self.network_manager.late_update();
    }

    fn on_destroy(&mut self) {
        self.network_manager.stop_server();
    }

    fn server_change_scene(&mut self, new_scene_name: String) {
        if new_scene_name == self.room_scene {
            for net_id in self.room_slots.iter() {
                match NetworkServerStatic::spawned_network_identities().try_get(net_id) {
                    TryResult::Present(identity) => {
                        if !identity.get_component::<NetworkRoomPlayer, _>(|network_room_player| {
                            network_room_player.ready_to_begin = false;
                            network_room_player.set_sync_var_dirty_bits(1 << 0);
                            NetworkServer::replace_player_for_connection(
                                identity.connection_to_client(),
                                identity.game_object().clone(),
                                ReplacePlayerOptions::KeepAuthority,
                            );
                        }) {
                            log_error!(format!(
                                "Failed to on_server_disconnect for identity {} because of absent",
                                net_id
                            ));
                        }
                    }
                    TryResult::Absent => {
                        log_error!("Failed to on_server_disconnect for identity because of absent");
                    }
                    TryResult::Locked => {
                        log_error!("Failed to on_server_disconnect for identity because of locked");
                    }
                }
            }
        }

        // base.OnServerChangeScene(newSceneName);
        self.network_manager.server_change_scene(new_scene_name);
    }

    fn on_server_connect(conn: &mut NetworkConnectionToClient)
    where
        Self: Sized,
    {
        NetworkManager::on_server_connect(conn);
        Self::on_room_server_connect(conn);
    }

    // OnServerDisconnect
    fn on_server_disconnect(conn: &mut NetworkConnectionToClient, _transport_error: TransportError)
    where
        Self: Sized,
    {
        let network_room_manager = NetworkManagerStatic::network_manager_singleton();
        network_room_manager
            .room_slots()
            .retain(|&x| x != conn.net_id());

        for net_id in conn.owned().iter() {
            network_room_manager.room_slots().retain(|x| x != net_id);
        }

        network_room_manager.set_all_players_ready(false);

        for net_id in network_room_manager.room_slots().iter() {
            match NetworkServerStatic::spawned_network_identities().try_get(net_id) {
                TryResult::Present(identity) => {
                    if !identity.get_component::<NetworkRoomPlayer, _>(|player| {
                        player.ready_to_begin = false;
                    }) {
                        log_error!(format!(
                            "Failed to on_server_disconnect for identity {} because of absent",
                            net_id
                        ));
                    }
                }
                TryResult::Absent => {
                    log_error!("Failed to on_server_disconnect for identity because of absent");
                }
                TryResult::Locked => {
                    log_error!("Failed to on_server_disconnect for identity because of locked");
                }
            }
        }

        if NetworkManagerStatic::network_scene_name() == *network_room_manager.room_scene() {
            let (index, net_id) = network_room_manager.recalculate_room_player_indices();
            if index != 0 && net_id != 0 {
                log_error!("Fix index for net_id: {},this should not happen", net_id);
            }
        }

        Self::on_room_server_disconnect(conn);
        NetworkManager::on_server_disconnect(conn, _transport_error);

        if network_room_manager.num_players() < 1 {
            network_room_manager.stop_server();
        }
    }

    fn on_server_ready(conn_id: u64)
    where
        Self: Sized,
    {
        // base OnServerReady
        NetworkManager::on_server_ready(conn_id);

        // 如果 conn_id 为 0
        if conn_id == 0 {
            return;
        }

        let mut net_id = 0;
        let mut room_player = GameObject::default();
        // 获取 NetworkConnectionToClient
        match NetworkServerStatic::network_connections().try_get(&conn_id) {
            TryResult::Present(conn) => {
                net_id = conn.net_id();
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
        // 如果 net_id 为 0
        if net_id == 0 {
            return;
        }
        // 获取 NetworkIdentity
        match NetworkServerStatic::spawned_network_identities().try_get(&net_id) {
            TryResult::Present(identity) => {
                if !identity.get_component::<NetworkRoomPlayer, _>(|player| {
                    room_player = player.game_object().clone();
                }) {
                    log_error!(format!(
                        "Failed to on_server_ready for identity {} because of absent",
                        net_id
                    ));
                }
            }
            TryResult::Absent => {
                log_error!(format!(
                    "Failed to on_server_ready for identity {} because of absent",
                    net_id
                ));
            }
            TryResult::Locked => {
                log_error!(format!(
                    "Failed to on_server_ready for identity {} because of locked",
                    net_id
                ));
            }
        }
        // 如果 room_player 不为空
        if !room_player.is_null() {
            Self::scene_loaded_for_player(conn_id, room_player);
        }
    }

    fn on_server_add_player(&mut self, conn_id: u64) {
        self.client_index += 1;
        self.set_all_players_ready(false);

        // TODO fix Utils.IsSceneActive(RoomScene)

        let mut player_obj: GameObject;
        // 自定义方法创建 player_obj
        match Self::on_room_server_create_room_player(conn_id) {
            None => {
                // 拿到 player_obj
                player_obj = self.room_player_prefab.clone();
                if player_obj.is_null() {
                    log_error!("The PlayerPrefab is empty on the NetworkManager. Please setup a PlayerPrefab object.");
                    return;
                }

                // 修改 player_obj 的 transform 属性
                player_obj.transform = self.get_start_position();
            }
            Some(player) => {
                player_obj = player;
            }
        }
        NetworkServer::add_player_for_connection(conn_id, player_obj);
    }

    fn on_server_error(conn: &mut NetworkConnectionToClient, error: TransportError)
    where
        Self: Sized,
    {
        NetworkManager::on_server_error(conn, error);
    }

    fn on_server_transport_exception(conn: &mut NetworkConnectionToClient, error: TransportError)
    where
        Self: Sized,
    {
        NetworkManager::on_server_transport_exception(conn, error);
    }

    fn on_server_change_scene(&mut self, new_scene_name: String) {
        self.network_manager.on_server_change_scene(new_scene_name);
    }

    fn on_server_scene_changed(&mut self, new_scene_name: String) {
        if new_scene_name != self.room_scene {
            for pp in self.pending_players.iter() {
                Self::scene_loaded_for_player(pp.conn, pp.room_player.clone());
            }
            self.pending_players.clear();
        }
        Self::on_room_server_scene_changed(new_scene_name);
    }

    fn on_start_server(&mut self) {
        self.on_room_start_server();
    }

    fn on_stop_server(&mut self) {
        self.network_manager.on_stop_server();
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
}

impl NetworkRoomManagerTrait for NetworkRoomManager {
    fn on_room_server_create_room_player(conn_id: u64) -> Option<GameObject> {
        let _ = conn_id;
        None
    }

    fn on_room_server_scene_changed(new_scene_name: String) {
        let _ = new_scene_name;
    }

    fn on_room_start_server(&mut self) {}

    fn on_room_server_connect(conn: &mut NetworkConnectionToClient) {
        let _ = conn;
    }

    fn on_room_server_disconnect(conn: &mut NetworkConnectionToClient) {
        let _ = conn;
    }

    fn on_room_server_players_ready(&mut self) {
        self.server_change_scene(self.gameplay_scene.to_string());
    }

    fn on_room_server_players_not_ready(&mut self) {}
}
