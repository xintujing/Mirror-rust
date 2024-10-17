use crate::backend_data::BackendData;
use crate::batcher::Batch;
use crate::tools::to_hex_string;
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum Visibility { Default, Hidden, Shown }

#[derive(Debug, Clone, Copy)]
pub enum OwnedType { Client, Server }

#[derive(Debug, Clone)]
pub struct SyncVar {
    pub r#type: &'static str,
    pub data: Vec<u8>,
    pub is_dirty: bool,
    pub dirty_bit: u32,
}

impl SyncVar {
    pub fn new() -> Self {
        SyncVar {
            r#type: "",
            data: Vec::new(),
            is_dirty: false,
            dirty_bit: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkIdentity {
    pub net_id: u32,
    pub scene_id: u64,
    pub asset_id: u32,
    pub owned_type: OwnedType,
    pub owned: u32,
    pub is_destroy: bool,
    pub visibility: Visibility,
    pub sync_objs: DashMap<String, Vec<u8>>,
    pub sync_vars: DashMap<String, SyncVar>,
    pub backend_data: Arc<BackendData>,
}

impl NetworkIdentity {
    pub fn new(backend_data: Arc<BackendData>) -> Self {
        NetworkIdentity {
            net_id: 0,
            scene_id: 0,
            asset_id: 0,
            owned_type: OwnedType::Client,
            owned: 0,
            is_destroy: false,
            visibility: Visibility::Default,
            sync_objs: DashMap::new(),
            sync_vars: DashMap::new(),
            backend_data,
        }
    }

    pub fn serialize_server() {}

    pub fn create_spawn_message_payload(&self) {
        let mut mask = 0u64;
        if self.scene_id != 0 {} else if self.asset_id != 0 {
            if let Some(asset) = self.backend_data.assets.get(&self.asset_id) {
                println!("asset: key:{}, value:{}", asset.key(), asset.value());

                if let Some(components) = self.backend_data.components.get(asset.value()) {
                    println!("component: key:{}, value:{:?}", components.key(), components.value());

                    let components_len = components.value().len() as u8;

                    let mut batch_mask = Batch::new();

                    let mut batch_safety = Batch::new();

                    for i in 0..components_len {
                        let x = components.value().get(&i).unwrap();
                        mask |= 1 << (*x.key() as u64);
                        let mut content_batch = Batch::new();
                        if x.value() == "Mirror.NetworkTransformUnreliable" {
                            content_batch.write_f32_le(7.15);
                            content_batch.write_f32_le(0.0);
                            content_batch.write_f32_le(-4.03);

                            content_batch.write_f32_le(0.0);
                            content_batch.write_f32_le(0.0);
                            content_batch.write_f32_le(0.0);
                            content_batch.write_f32_le(1.0);

                            let safety = (content_batch.get_bytes().len() & 0xFF) as u8;
                            batch_safety.write_u8(safety);
                            batch_safety.write(&content_batch.get_bytes());
                        } else if x.value() == "QuickStart.PlayerScript" {
                            content_batch.write_u32_le(0);
                            content_batch.write_string_le("");
                            content_batch.write_f32_le(1.0);
                            content_batch.write_f32_le(1.0);
                            content_batch.write_f32_le(1.0);
                            content_batch.write_f32_le(1.0);

                            let safety = (content_batch.get_bytes().len() & 0xFF) as u8;
                            batch_safety.write_u8(safety);
                            batch_safety.write(&content_batch.get_bytes());
                        }
                    }
                    batch_mask.compress_var_u64_le(mask);
                    batch_mask.write(&batch_safety.get_bytes());
                    println!("batch: {}", to_hex_string(batch_mask.get_bytes().as_ref()));
                }
            }
        }
    }
}