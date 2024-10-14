use dashmap::DashMap;

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

#[derive(Debug)]
pub struct NetworkIdentity {
    pub net_id: u32,
    pub scene_id: u64,
    pub asset_id: u32,
    pub owned_type: OwnedType,
    pub owned: u32,
    pub observers: Vec<i32>,
    pub is_destroy: bool,
    pub visibility: Visibility,
    pub objects: DashMap<String, Vec<u8>>,
    pub sync_vars: DashMap<String, SyncVar>,
}