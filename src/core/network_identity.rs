use crate::components::network_behaviour::NetworkBehaviourTrait;
use crate::components::network_common_component::NetworkCommonComponent;
use crate::components::network_transform::network_transform_reliable::NetworkTransformReliable;
use crate::components::network_transform::network_transform_unreliable::NetworkTransformUnreliable;
use crate::components::SyncVar;
use crate::core::backend_data::{BackendData, NetworkBehaviourComponent};
use crate::core::batcher::Batch;
use bytes::Bytes;
use dashmap::DashMap;
use nalgebra::Vector3;
use std::default::Default;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            for component in network_identity.backend_data.get_network_identity_data_network_behaviour_components_by_asset_id(network_identity.asset_id) {
                // 如果 component.component_type 包含 NetworkTransformUnreliable::COMPONENT_TAG
                if component.sub_class.contains(NetworkTransformUnreliable::COMPONENT_TAG) {
                    // scale
                    let scale = Vector3::new(1.0, 1.0, 1.0);
                    // 创建 NetworkTransform
                    let network_transform = NetworkTransformUnreliable::new(component.network_transform_base_setting, component.network_transform_unreliable_setting, component.network_behaviour_setting, component.index, Default::default(), Default::default(), scale);
                    // 添加到 components
                    network_identity.components.push(Box::new(network_transform));
                    continue;
                }
                // 如果 component.component_type 包含 NetworkTransformReliable::COMPONENT_TAG
                if component.sub_class.contains(NetworkTransformReliable::COMPONENT_TAG) {
                    // scale
                    let scale = Vector3::new(1.0, 1.0, 1.0);
                    // 创建 NetworkTransform
                    let network_transform = NetworkTransformReliable::new(component.network_transform_base_setting, component.network_transform_reliable_setting, component.network_behaviour_setting, component.index, Default::default(), Default::default(), scale);
                    // 添加到 components
                    network_identity.components.push(Box::new(network_transform));
                    continue;
                }
                if component.sub_class == "QuickStart.PlayerScript" {
                    // 创建 NetworkCommonComponent
                    let network_common = network_identity.new_network_common_component(component);
                    // 添加到 components
                    network_identity.components.push(Box::new(network_common));
                }
            }
        }
        network_identity
    }

    pub fn new_network_common_component(&self, network_behaviour_component: &NetworkBehaviourComponent) -> NetworkCommonComponent {
        let sync_vars = DashMap::new();
        for (index, sync_var) in self.backend_data.get_sync_var_data_s_by_sub_class(network_behaviour_component.sub_class.as_ref()).iter().enumerate() {
            sync_vars.insert(index as u8, SyncVar::new(
                sync_var.full_name.clone(),
                Bytes::copy_from_slice(sync_var.value.as_slice()),
                sync_var.dirty_bit,
            ));
        }
        NetworkCommonComponent::new(network_behaviour_component.network_behaviour_setting, network_behaviour_component.index, sync_vars)
    }

    pub fn new_spawn_message_payload(&mut self) -> Bytes {
        // mask
        let mut mask = 0u64;
        // 创建 Batch
        let mut batch = Batch::new();
        // 创建 所有 components 的 Batch
        let mut components_batch = Batch::new();
        // 遍历 components
        for mut component in self.components.iter_mut() {
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
        // 返回 batch 的 bytes
        batch.get_bytes()
    }
}