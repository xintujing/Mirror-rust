use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum MethodType {
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
    /// need fix
    pub r#type: String,
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
    pub initial_value: Vec<u8>,
    #[serde(rename = "dirtyBit")]
    pub dirty_bit: i32,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct NetworkBehaviourSetting {
    #[serde(rename = "syncDirection")]
    /// need fix
    pub sync_direction: u8,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
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
    #[serde(rename = "coordinateSpace")]
    /// need fix
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct NetworkTransformUnreliableSetting {
    #[serde(rename = "bufferResetMultiplier")]
    pub buffer_reset_multiplier: f32,
    #[serde(rename = "changedDetection")]
    pub changed_detection: bool,
    #[serde(rename = "positionSensitivity")]
    pub position_sensitivity: f32,
    #[serde(rename = "rotationSensitivity")]
    pub rotation_sensitivity: f32,
    #[serde(rename = "scaleSensitivity")]
    pub scale_sensitivity: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkBehaviourComponent {
    #[serde(rename = "componentIndex")]
    pub component_index: u8,
    #[serde(rename = "componentType")]
    pub component_type: String,
    #[serde(rename = "networkBehaviourSetting")]
    pub network_behaviour_setting: NetworkBehaviourSetting,
    #[serde(rename = "networkTransformBaseSetting")]
    pub network_transform_base_setting: NetworkTransformBaseSetting,
    #[serde(rename = "networkTransformReliableSetting")]
    pub network_transform_reliable_setting: NetworkTransformReliableSetting,
    #[serde(rename = "networkTransformUnreliableSetting")]
    pub network_transform_unreliable_setting: NetworkTransformUnreliableSetting,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkIdentityData {
    #[serde(rename = "assetId")]
    pub asset_id: u32,
    #[serde(rename = "sceneId")]
    pub scene_id: u64,
    #[serde(rename = "networkBehaviourComponents")]
    pub network_behaviour_components: Vec<KeyValue<u8, NetworkBehaviourComponent>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackendData {
    #[serde(rename = "methods")]
    pub methods: Vec<MethodData>,
    #[serde(rename = "networkIdentities")]
    pub network_identities: Vec<NetworkIdentityData>,
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
    pub fn import(path: &'static str) -> Self {
        // 读取 JSON 文件内容
        if let Ok(data) = std::fs::read_to_string(path) {
            // 将 JSON 文件内容反序列化为 Data 结构体
            let data: BackendData = serde_json::from_str(&data).unwrap();
            return data;
        }
        panic!("Failed to import BackData");
    }
    #[allow(dead_code)]
    pub fn get_method_data_by_hash_code(&self, hash_code: u16) -> Option<&MethodData> {
        for method_data in &self.methods {
            if method_data.hash_code == hash_code {
                return Some(method_data);
            }
        }
        None
    }
    #[allow(dead_code)]
    pub fn get_method_data_by_method_name(&self, method_name: &str) -> Option<&MethodData> {
        for method_data in &self.methods {
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
            for rpc in &method_data.rpc_list {
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
    pub fn get_network_identity_data(&self, asset_id: u32) -> Option<&NetworkIdentityData> {
        for network_identity_data in &self.network_identities {
            if network_identity_data.asset_id == asset_id {
                return Some(network_identity_data);
            }
        }
        None
    }
    #[allow(dead_code)]
    pub fn get_network_identity_data_network_behaviour_components(&self, asset_id: u32) -> Vec<&NetworkBehaviourComponent> {
        let mut network_behaviour_components = Vec::new();
        if let Some(network_identity_data) = self.get_network_identity_data(asset_id) {
            network_behaviour_components = network_identity_data.network_behaviour_components.iter().map(|v| &v.value).collect();
        }
        network_behaviour_components
    }
    #[allow(dead_code)]
    pub fn get_scene_id(&self, scene_name: &str) -> Option<u64> {
        for scene_id in &self.scene_ids {
            if scene_id.key == scene_name {
                return Some(scene_id.value);
            }
        }
        None
    }
    #[allow(dead_code)]
    pub fn get_sync_var_data(&self, full_name: &str) -> Option<&SyncVarData> {
        for sync_var_data in &self.sync_vars {
            if sync_var_data.full_name == full_name {
                return Some(sync_var_data);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_data() {
        let backend_data = BackendData::import("backend_data.json");

        if let Some(method_data) = backend_data.get_method_data_by_hash_code(42311) {
            println!("{:?}", method_data);
            let rpc_method_datas = backend_data.get_method_data_by_method_name(method_data.rpc_list.as_ref());
            for rpc_method_data in rpc_method_datas {
                println!("{:?}", rpc_method_data);
            }
        }
        println!("{:?}", backend_data.get_rpc_hash_code_s(42311));

        let vec = backend_data.get_network_identity_data_network_behaviour_components(3541431626);
        for v in vec {
            println!("{:?}", v);
        }
    }
}