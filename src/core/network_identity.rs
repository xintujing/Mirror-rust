use crate::components::network_behaviour::NetworkBehaviourTrait;
use crate::components::network_common::NetworkCommon;
use crate::components::network_transform::network_transform_reliable::NetworkTransformReliable;
use crate::components::network_transform::network_transform_unreliable::NetworkTransformUnreliable;
use crate::components::SyncVar;
use crate::core::backend_data::BackendData;
use crate::core::batcher::Batch;
use crate::tools::utils::get_timestamp;
use bytes::Bytes;
use dashmap::DashMap;
use nalgebra::Vector3;
use std::default::Default;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum Visibility { Default, Hidden, Shown }

#[derive(Debug, Clone, Copy)]
pub enum OwnedType { Client, Server }

pub struct NetworkIdentity {
    pub scene_id: u64,
    pub asset_id: u32,
    pub net_id: u32,
    pub owned_type: OwnedType,
    pub owned: u32,
    pub is_init: bool,
    pub is_destroy: bool,
    pub visibility: Visibility,
    pub components: Vec<Box<dyn NetworkBehaviourTrait>>,
    pub backend_data: Arc<BackendData>,
}

impl NetworkIdentity {
    pub fn new(backend_data: Arc<BackendData>, scene_id: u64, asset_id: u32) -> Self {
        let mut network_identity = NetworkIdentity {
            scene_id,
            asset_id,
            net_id: 0,
            owned_type: OwnedType::Client,
            owned: 0,
            is_init: true,
            is_destroy: false,
            visibility: Visibility::Default,
            components: Vec::new(),
            backend_data,
        };

        if network_identity.scene_id != 0 {
            // TODO: Implement this
        }
        // 如果 asset_id 不为 0
        if network_identity.asset_id != 0 {
            for component in network_identity.backend_data.get_network_identity_data_network_behaviour_components(network_identity.asset_id) {
                // 如果 component.component_type 包含 NetworkTransformUnreliable::COMPONENT_TAG
                if component.component_type.contains(NetworkTransformUnreliable::COMPONENT_TAG) {
                    // scale
                    let scale = Vector3::new(1.0, 1.0, 1.0);
                    // 创建 NetworkTransform
                    let network_transform = NetworkTransformUnreliable::new(component.network_transform_base_setting, component.network_transform_unreliable_setting, component.network_behaviour_setting, component.component_index, Default::default(), Default::default(), scale);
                    // 添加到 components
                    network_identity.components.push(Box::new(network_transform));
                    continue;
                }
                // 如果 component.component_type 包含 NetworkTransformReliable::COMPONENT_TAG
                if component.component_type.contains(NetworkTransformReliable::COMPONENT_TAG) {
                    // scale
                    let scale = Vector3::new(1.0, 1.0, 1.0);
                    // 创建 NetworkTransform
                    let network_transform = NetworkTransformReliable::new(component.network_behaviour_setting, component.component_index, true, true, false, Default::default(), Default::default(), scale);
                    // 添加到 components
                    network_identity.components.push(Box::new(network_transform));
                    continue;
                }
                // 如果 component.component_type 包含 NetworkTransformUnreliable::COMPONENT_TAG
                if component.component_type == "QuickStart.PlayerScript" {
                    let sync_vars = DashMap::new();
                    // 遍历 sync_vars
                    let mut sync_var1 = SyncVar::new();
                    let mut batch = Batch::new();
                    batch.write_i32_le(1);
                    sync_var1.data = batch.get_bytes();
                    sync_vars.insert(1, sync_var1);

                    let mut sync_var2 = SyncVar::new();
                    let mut batch = Batch::new();
                    batch.write_string_le(&format!("Player {}", get_timestamp()));
                    sync_var2.data = batch.get_bytes();
                    sync_vars.insert(2, sync_var2);

                    let mut sync_var3 = SyncVar::new();
                    let mut batch = Batch::new();
                    batch.write_f32_le(0.0);
                    batch.write_f32_le(0.0);
                    batch.write_f32_le(0.0);
                    batch.write_f32_le(1.0);
                    sync_var3.data = batch.get_bytes();
                    sync_vars.insert(3, sync_var3);

                    // sync_vars: DashMap<String, SyncVar>
                    let network_common = NetworkCommon::new(component.network_behaviour_setting, component.component_index, sync_vars);
                    // 添加到 components
                    network_identity.components.push(Box::new(network_common));
                }
            }
        }
        network_identity
    }

    pub fn serialize_server() {}

    pub fn create_spawn_message_payload(&self) -> Bytes {
        // mask
        let mut mask = 0u64;
        // 创建 Batch
        let mut batch = Batch::new();
        // 创建 所有 components 的 Batch
        let mut components_batch = Batch::new();
        // 遍历 components
        for component in self.components.iter() {
            mask |= 1 << component.get_network_behaviour().component_index;
            let component_bytes = component.serialize(true).get_bytes();
            let safety = (component_bytes.len() & 0xFF) as u8;
            components_batch.write_u8(safety);
            components_batch.write(&component_bytes);
        }
        // 写入 mask
        batch.compress_var_u64_le(mask);
        // 写入 components_batch
        batch.write(&components_batch.get_bytes());
        batch.get_bytes()
    }
}