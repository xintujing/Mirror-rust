use crate::mirror::components::network_animator::Animator;
use crate::{log_error, log_info, stop_signal};
use config::Config;
use lazy_static::lazy_static;
use notify::event::{DataChange, ModifyKind};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashSet;
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::{OnceLock, RwLock};
use std::time::Duration;

lazy_static! {
    static ref BACKEND_DATA_FILE: String = "tobackend.json".to_string();
}

pub struct BackendDataStatic;

impl BackendDataStatic {
    fn tobackend() -> &'static RwLock<Config> {
        static BACKEND_DATA: OnceLock<RwLock<Config>> = OnceLock::new();
        BACKEND_DATA.get_or_init(|| {
            // 判断文件是否存在
            if !Path::new(BACKEND_DATA_FILE.as_str()).exists() {
                std::fs::write(BACKEND_DATA_FILE.as_str(), {
                    let backend_data = BackendData {
                        methods: Vec::new(),
                        network_identities: Vec::new(),
                        network_manager_settings: Vec::new(),
                        scene_ids: Vec::new(),
                        sync_vars: Vec::new(),
                        assets: Vec::new(),
                    };
                    serde_json::to_string_pretty(&backend_data).unwrap()
                })
                    .unwrap();
            }

            match Config::builder()
                .add_source(config::File::with_name(BACKEND_DATA_FILE.as_str()))
                .build()
            {
                Ok(backend_data) => RwLock::new(backend_data),
                Err(e) => {
                    panic!("Failed to read BackendData: {:?}", e);
                }
            }
        })
    }

    pub fn watch() {
        // Create a channel to receive the events.
        let (tx, rx) = channel();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher: RecommendedWatcher = Watcher::new(
            tx,
            notify::Config::default().with_poll_interval(Duration::from_secs(3)),
        )
            .unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher
            .watch(
                Path::new(BACKEND_DATA_FILE.as_str()),
                RecursiveMode::NonRecursive,
            )
            .unwrap_or_else(|_| {});

        // This is a simple loop, but you may want to use more complex logic here,
        // for example to handle I/O.
        while let Ok(event) = rx.recv() {
            if *stop_signal() {
                break;
            }
            match event {
                Ok(Event {
                       kind: notify::event::EventKind::Modify(ModifyKind::Data(DataChange::Content)),
                       ..
                   }) => {
                    match Config::builder()
                        .add_source(config::File::with_name(BACKEND_DATA_FILE.as_str()))
                        .build()
                    {
                        Ok(backend_data) => {
                            *Self::tobackend().write().unwrap() = backend_data;
                            *stop_signal() = true;
                            log_info!(format!("{} has been updated", BACKEND_DATA_FILE.as_str()));
                        }
                        Err(e) => {
                            log_error!(format!("watch error: {:?}", e));
                        }
                    }
                }
                Err(e) => {
                    log_error!(format!("watch error: {:?}", e));
                }
                _ => {}
            }
        }
    }

    pub fn get_backend_data() -> BackendData {
        match Self::tobackend().read().unwrap().clone().try_deserialize() {
            Ok(backend_data) => backend_data,
            Err(e) => {
                panic!("Failed to deserialize BackendData: {:?}", e);
            }
        }
    }

    pub fn import(path: &'static str) -> BackendData {
        // 读取 JSON 文件内容
        match std::fs::read_to_string(path) {
            Ok(data) => {
                // 将 JSON 文件内容反序列化为 Data 结构体
                match serde_json::from_str::<BackendData>(&data) {
                    Ok(backend_data) => backend_data,
                    Err(e) => {
                        panic!("Failed to deserialize BackendData: {:?}", e);
                    }
                }
            }
            Err(e) => {
                panic!("Failed to read BackendData: {:?}", e);
            }
        }
    }
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum MethodType {
    None = 0,
    Command = 1,
    TargetRpc = 2,
    ClientRpc = 3,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct KeyValue<KeyType, ValueType> {
    pub key: KeyType,
    pub value: ValueType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MethodData {
    #[serde(rename = "hashCode")]
    pub hash_code: u16,
    #[serde(rename = "subClass")]
    pub sub_class: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "requiresAuthority")]
    pub requires_authority: bool,
    #[serde(rename = "type")]
    pub r#type: MethodType,
    #[serde(rename = "parameters")]
    pub parameters: Vec<KeyValue<String, String>>,
    #[serde(rename = "rpcList")]
    pub rpc_list: Vec<String>,
    #[serde(rename = "varList")]
    pub var_list: Vec<KeyValue<u8, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncVarData {
    #[serde(rename = "fullname")]
    pub full_name: String,
    #[serde(rename = "subClass")]
    pub sub_class: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "initialValue")]
    pub value: Vec<u8>,
    #[serde(rename = "dirtyBit")]
    pub dirty_bit: u32,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct NetworkBehaviourSetting {
    #[serde(rename = "syncDirection")]
    /// need fix
    pub sync_direction: u8,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct NetworkTransformBaseSetting {
    #[serde(rename = "syncPosition")]
    pub sync_position: bool,
    #[serde(rename = "syncRotation")]
    pub sync_rotation: bool,
    #[serde(rename = "syncScale")]
    pub sync_scale: bool,

    #[serde(rename = "onlySyncOnChange")]
    pub only_sync_on_change: bool,
    #[serde(rename = "compressRotation")]
    pub compress_rotation: bool,

    #[serde(rename = "interpolatePosition")]
    pub interpolate_position: bool,
    #[serde(rename = "interpolateRotation")]
    pub interpolate_rotation: bool,
    #[serde(rename = "interpolateScale")]
    pub interpolate_scale: bool,

    /// need fix
    #[serde(rename = "coordinateSpace")]
    pub coordinate_space: u8,

    #[serde(rename = "sendIntervalMultiplier")]
    pub send_interval_multiplier: u32,

    #[serde(rename = "timelineOffset")]
    pub timeline_offset: bool,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct NetworkTransformReliableSetting {
    #[serde(rename = "onlySyncOnChangeCorrectionMultiplier")]
    pub only_sync_on_change_correction_multiplier: f32,

    #[serde(rename = "rotationSensitivity")]
    pub rotation_sensitivity: f32,

    #[serde(rename = "positionPrecision")]
    pub position_precision: f32,
    #[serde(rename = "scalePrecision")]
    pub scale_precision: f32,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct NetworkTransformUnreliableSetting {
    #[serde(rename = "bufferResetMultiplier")]
    pub buffer_reset_multiplier: f32,
    #[serde(rename = "positionSensitivity")]
    pub position_sensitivity: f32,
    #[serde(rename = "rotationSensitivity")]
    pub rotation_sensitivity: f32,
    #[serde(rename = "scaleSensitivity")]
    pub scale_sensitivity: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkAnimatorSetting {
    #[serde(rename = "clientAuthority")]
    pub client_authority: bool,
    #[serde(rename = "animator")]
    pub animator: Animator,
    #[serde(rename = "animatorSpeed")]
    pub animator_speed: f32,
    #[serde(rename = "previousSpeed")]
    pub previous_speed: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkBehaviourComponent {
    #[serde(rename = "componentIndex")]
    pub index: u8,
    #[serde(rename = "componentType")]
    pub sub_class: String,
    #[serde(rename = "networkBehaviourSetting")]
    pub network_behaviour_setting: NetworkBehaviourSetting,
    #[serde(rename = "networkTransformBaseSetting")]
    pub network_transform_base_setting: NetworkTransformBaseSetting,
    #[serde(rename = "networkTransformReliableSetting")]
    pub network_transform_reliable_setting: NetworkTransformReliableSetting,
    #[serde(rename = "networkTransformUnreliableSetting")]
    pub network_transform_unreliable_setting: NetworkTransformUnreliableSetting,
    #[serde(rename = "networkAnimatorSetting")]
    pub network_animator_setting: NetworkAnimatorSetting,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotInterpolationSetting {
    #[serde(rename = "bufferTimeMultiplier")]
    pub buffer_time_multiplier: f64,
    #[serde(rename = "bufferLimit")]
    pub buffer_limit: usize,
    #[serde(rename = "catchupNegativeThreshold")]
    pub catchup_negative_threshold: f32,
    #[serde(rename = "catchupPositiveThreshold")]
    pub catchup_positive_threshold: f32,
    #[serde(rename = "catchupSpeed")]
    pub catchup_speed: f64,
    #[serde(rename = "slowdownSpeed")]
    pub slowdown_speed: f64,
    #[serde(rename = "driftEmaDuration")]
    pub drift_ema_duration: i32,
    #[serde(rename = "dynamicAdjustment")]
    pub dynamic_adjustment: bool,
    #[serde(rename = "dynamicAdjustmentTolerance")]
    pub dynamic_adjustment_tolerance: f32,
    #[serde(rename = "deliveryTimeEmaDuration")]
    pub delivery_time_ema_duration: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkManagerSetting {
    #[serde(rename = "dontDestroyOnLoad")]
    pub dont_destroy_on_load: bool,
    #[serde(rename = "runInBackground")]
    pub run_in_background: bool,
    #[serde(rename = "headlessStartMode")]
    pub headless_start_mode: String,
    #[serde(rename = "editorAutoStart")]
    pub editor_auto_start: bool,
    #[serde(rename = "sendRate")]
    pub send_rate: u32,
    #[serde(rename = "offlineScene")]
    pub offline_scene: String,
    #[serde(rename = "onlineScene")]
    pub online_scene: String,
    #[serde(rename = "transport")]
    pub transport: String,
    #[serde(rename = "networkAddress")]
    pub network_address: String,
    #[serde(rename = "maxConnections")]
    pub max_connections: usize,
    #[serde(rename = "disconnectInactiveConnections")]
    pub disconnect_inactive_connections: bool,
    #[serde(rename = "disconnectInactiveTimeout")]
    pub disconnect_inactive_timeout: f32,
    #[serde(rename = "authenticator")]
    pub authenticator: String,
    #[serde(rename = "playerPrefab")]
    pub player_prefab: String,
    #[serde(rename = "autoCreatePlayer")]
    pub auto_create_player: bool,
    #[serde(rename = "playerSpawnMethod")]
    pub player_spawn_method: String,
    #[serde(rename = "spawnPrefabs")]
    pub spawn_prefabs: Vec<String>,
    #[serde(rename = "exceptionsDisconnect")]
    pub exceptions_disconnect: bool,
    #[serde(rename = "snapshotSettings")]
    pub snapshot_interpolation_setting: SnapshotInterpolationSetting,
    #[serde(rename = "evaluationMethod")]
    pub evaluation_method: String,
    #[serde(rename = "evaluationInterval")]
    pub evaluation_interval: f32,
    #[serde(rename = "timeInterpolationGui")]
    pub time_interpolation_gui: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkIdentityData {
    #[serde(rename = "assetId")]
    pub asset_id: u32,
    #[serde(rename = "sceneId")]
    pub scene_id: u64,
    /// need fix  dont need use KeyValue
    #[serde(rename = "networkBehaviourComponents")]
    pub network_behaviour_components: Vec<KeyValue<u8, NetworkBehaviourComponent>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BackendData {
    #[serde(rename = "methods")]
    pub methods: Vec<MethodData>,
    #[serde(rename = "networkIdentities")]
    pub network_identities: Vec<NetworkIdentityData>,
    #[serde(rename = "networkManagerSettings")]
    pub network_manager_settings: Vec<NetworkManagerSetting>,
    #[serde(rename = "sceneIds")]
    pub scene_ids: Vec<KeyValue<String, u64>>,
    #[serde(rename = "syncVars")]
    pub sync_vars: Vec<SyncVarData>,
    #[serde(rename = "assets")]
    pub assets: Vec<KeyValue<u32, String>>,
}
#[allow(dead_code)]
impl BackendData {
    #[allow(dead_code)]
    pub fn get_method_data_by_hash_code(&self, hash_code: u16) -> Option<&MethodData> {
        for method_data in self.methods.iter() {
            if method_data.hash_code == hash_code {
                return Some(method_data);
            }
        }
        None
    }
    #[allow(dead_code)]
    pub fn get_method_data_by_method_name(&self, method_name: &str) -> Option<&MethodData> {
        for method_data in self.methods.iter() {
            if method_data.name == method_name {
                return Some(method_data);
            }
        }
        None
    }
    #[allow(dead_code)]
    pub fn get_rpc_hash_code_s(&self, hash_code: u16) -> Vec<u16> {
        let mut hash_codes = Vec::new();
        // 拿到 MethodData
        if let Some(method_data) = self.get_method_data_by_hash_code(hash_code) {
            // 遍历 rpc_list
            for rpc in method_data.rpc_list.iter() {
                // 拿到 MethodData
                if let Some(rpc_method_data) = self.get_method_data_by_method_name(rpc) {
                    // 添加到 hash_codes
                    hash_codes.push(rpc_method_data.hash_code);
                }
            }
        }
        hash_codes
    }
    #[allow(dead_code)]
    pub fn get_network_identity_data_by_asset_id(
        &self,
        asset_id: u32,
    ) -> Option<NetworkIdentityData> {
        if asset_id == 0 {
            return None;
        }
        for network_identity_data in self.network_identities.iter() {
            if network_identity_data.asset_id == asset_id {
                return Some(network_identity_data.clone());
            }
        }
        None
    }
    #[allow(dead_code)]
    pub fn get_network_identity_data_by_scene_id(
        &self,
        scene_id: u64,
    ) -> Option<&NetworkIdentityData> {
        if scene_id == 0 {
            return None;
        }
        for network_identity_data in self.network_identities.iter() {
            if network_identity_data.scene_id == scene_id {
                return Some(network_identity_data);
            }
        }
        None
    }
    #[allow(dead_code)]
    pub fn get_network_identity_data_network_behaviour_components_by_asset_id(
        &self,
        asset_id: u32,
    ) -> Vec<NetworkBehaviourComponent> {
        if asset_id == 0 {
            return Vec::new();
        }
        let mut network_behaviour_components = Vec::new();
        if let Some(network_identity_data) = self.get_network_identity_data_by_asset_id(asset_id) {
            network_behaviour_components = network_identity_data
                .network_behaviour_components
                .iter()
                .map(|v| v.value.clone())
                .collect();
        }
        network_behaviour_components
    }
    #[allow(dead_code)]
    pub fn get_network_identity_data_network_behaviour_components_by_scene_id(
        &self,
        scene_id: u64,
    ) -> Vec<NetworkBehaviourComponent> {
        if scene_id == 0 {
            return Vec::new();
        }
        let mut network_behaviour_components = Vec::new();
        if let Some(network_identity_data) = self.get_network_identity_data_by_scene_id(scene_id) {
            network_behaviour_components = network_identity_data
                .network_behaviour_components
                .iter()
                .map(|v| v.value.clone())
                .collect();
        }
        network_behaviour_components
    }
    #[allow(dead_code)]
    pub fn get_scene_id_by_scene_name(&self, scene_name: &str) -> Option<u64> {
        for scene_id in self.scene_ids.iter() {
            if scene_id.key == scene_name {
                return Some(scene_id.value);
            }
        }
        None
    }
    pub fn get_asset_id_by_asset_name(&self, asset_name: &str) -> Option<u32> {
        for asset in self.assets.iter() {
            if asset.value == asset_name {
                return Some(asset.key);
            }
        }
        None
    }

    pub fn get_sync_var_data_s_by_sub_class(&self, sub_class: &str) -> Vec<SyncVarData> {
        let mut sync_var_data_s = Vec::new();
        let mut seen_full_names = HashSet::new();
        for sync_var_data in self.sync_vars.iter() {
            if sync_var_data.sub_class == sub_class {
                if seen_full_names.insert(sync_var_data.full_name.clone()) {
                    sync_var_data_s.push(sync_var_data.clone());
                }
            }
        }
        sync_var_data_s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mirror::core::tools::stable_hash::StableHash;

    #[test]
    fn test_import_data() {
        let backend_data = BackendDataStatic::get_backend_data();

        let vec = backend_data
            .get_network_identity_data_network_behaviour_components_by_asset_id(3606887665);
        for v in vec {
            println!("{:?}", v);
        }

        for x in backend_data.get_sync_var_data_s_by_sub_class("QuickStart.PlayerScript") {
            println!("{:?}", x);
        }

        println!("{:?}", backend_data.network_manager_settings);

        let method_data = backend_data.get_method_data_by_hash_code(
            "System.Void QuickStart.PlayerScript::CmdSetupPlayer(System.String,UnityEngine.Color)"
                .get_fn_stable_hash_code(),
        );
        println!("{:?}", method_data);
    }
}
