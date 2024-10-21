use crate::backend_data::BackendData;
use crate::batcher::{Batch, DataWriter};
use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait};
use crate::components::network_common::NetworkCommon;
use crate::components::network_transform_unreliable::NetworkTransformUnreliable;
use crate::components::SyncVar;
use crate::network_identity;
use crate::tools::utils::to_hex_string;
use bytes::Bytes;
use dashmap::DashMap;
use nalgebra::Vector3;
use std::default::Default;
use std::sync::Arc;
use tklog::debug;

#[derive(Debug, Clone, Copy)]
pub enum Visibility { Default, Hidden, Shown }

#[derive(Debug, Clone, Copy)]
pub enum OwnedType { Client, Server }

#[derive(Debug)]
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
            // 如果 asset_id 在 assets 中
            if let Some(asset) = network_identity.backend_data.assets.get(&network_identity.asset_id) {
                // 如果 components 中有 asset.value()
                if let Some(components) = network_identity.backend_data.components.get(asset.value()) {
                    // 遍历 components
                    for i in 0..components.value().len() as u8 {
                        // 如果 component 存在
                        if let Some(component) = components.value().get(&i) {
                            // 如果 component.value() 包含 Mirror.NetworkTransform
                            if component.value().contains(NetworkTransformUnreliable::COMPONENT_TAG) {
                                // scale
                                let scale = Vector3::new(1.0, 1.0, 1.0);
                                // 创建 NetworkTransform
                                let network_transform = NetworkTransformUnreliable::new(i, true, true, false, Default::default(), Default::default(), scale);
                                // 添加到 components
                                network_identity.components.push(Box::new(network_transform));
                                continue;
                            }
                            if component.value() == "QuickStart.PlayerScript" {
                                let sync_vars = DashMap::new();
                                // 遍历 sync_vars
                                let mut sync_var1 = SyncVar::new();
                                let mut batch = Batch::new();
                                batch.write_i32_le(0);
                                sync_var1.data = batch.get_bytes().to_vec();
                                sync_vars.insert(1, sync_var1);
                                let mut sync_var2 = SyncVar::new();
                                let mut batch = Batch::new();
                                batch.write_string_le("");
                                sync_var2.data = batch.get_bytes().to_vec();
                                sync_vars.insert(2, sync_var2);
                                let mut sync_var3 = SyncVar::new();
                                let mut batch = Batch::new();
                                batch.write_f32_le(1.0);
                                batch.write_f32_le(1.0);
                                batch.write_f32_le(1.0);
                                batch.write_f32_le(1.0);
                                sync_var3.data = batch.get_bytes().to_vec();
                                sync_vars.insert(3, sync_var3);

                                // sync_vars: DashMap<String, SyncVar>
                                let network_common = NetworkCommon::new(i, sync_vars);
                                // 添加到 components
                                network_identity.components.push(Box::new(network_common));
                            }
                        }
                    }
                }
            }
        }
        // for network_identity in network_identity.components.iter() {
        //     debug!(format!("network_identity: {:?}", network_identity));
        // }
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
            let component_batch = component.serialize();
            let component_bytes = component_batch.get_bytes();
            let safety = (component_bytes.len() & 0xFF) as u8;
            components_batch.write_u8(safety);
            components_batch.write(&component_bytes);
        }

        // if self.scene_id != 0 {
        //     // TODO: Implement this
        // } else if self.asset_id != 0 {
        //     if let Some(asset) = self.backend_data.assets.get(&self.asset_id) {
        //         // println!("asset: key:{}, value:{}", asset.key(), asset.value());
        //         if let Some(components) = self.backend_data.components.get(asset.value()) {
        //             // println!("component: key:{}, value:{:?}", components.key(), components.value());
        //             for i in 0..components.value().len() as u8 {
        //                 if let Some(component) = components.value().get(&i) {
        //                     mask |= 1 << (*component.key() as u64);
        //                     if component.value().contains("Mirror.NetworkTransform") {
        //                         let mut network_transform = NetworkTransformUnreliable::new(0, true, true, false, Default::default(), Default::default(), Default::default());
        //                         // 序列化 NetworkTransform
        //                         let mut content_batch = network_transform.serialize();
        //                         let safety = (content_batch.get_bytes().len() & 0xFF) as u8;
        //                         components_batch.write_u8(safety);
        //                         components_batch.write(&content_batch.get_bytes());
        //                     } else if component.value() == "QuickStart.PlayerScript" {
        //                         let mut content_batch = Batch::new();
        //                         content_batch.write_u32_le(0);
        //                         content_batch.write_string_le("");
        //                         content_batch.write_f32_le(1.0);
        //                         content_batch.write_f32_le(1.0);
        //                         content_batch.write_f32_le(1.0);
        //                         content_batch.write_f32_le(1.0);
        //                         let safety = (content_batch.get_bytes().len() & 0xFF) as u8;
        //                         components_batch.write_u8(safety);
        //                         components_batch.write(&content_batch.get_bytes());
        //                     } else {
        //                         for sync_var_data in self.backend_data.sync_vars.iter() {
        //                             // println!("sync_var_data: {} {}", sync_var_data.sub_class, component.value());
        //                             if sync_var_data.sub_class.contains(component.value().as_str()) {
        //                                 println!("sync_var_data: {:?}", sync_var_data);
        //                             }
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }
        // 写入 mask
        batch.compress_var_u64_le(mask);
        // 写入 components_batch
        batch.write(&components_batch.get_bytes());
        debug!("batch: ", to_hex_string(batch.get_bytes().as_ref()));
        batch.get_bytes()
    }
}