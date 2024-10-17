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
}

impl NetworkIdentity {
    pub fn new() -> Self {
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
        }
    }

    pub fn serialize_server() {}
}