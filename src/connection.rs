use crate::tools::generate_id;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Connection {
    pub net_id: u32,
    /// 以下为 Mirror.Connection 类的属性
    pub connection_id: u64,
    pub address: String,
    pub is_authenticated: bool,
    /// TODO: Auth Data
    pub is_ready: bool,
    pub last_message_time: f32,
    /// TODO netid
    /// TODO 附属 netid
    pub remote_time_stamp: f64,
}

impl Connection {
    pub fn new(connection_id: u64, address: String) -> Self {
        Connection {
            net_id: generate_id(),
            connection_id,
            address,
            is_authenticated: false,
            is_ready: false,
            last_message_time: 0.0,
            remote_time_stamp: 0.0,
        }
    }
    pub fn set_ready(&mut self, ready: bool) {
        self.is_ready = ready;
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionsManager {
    con_map: HashMap<u64, Connection>,
}
impl<'a> ConnectionsManager {
    pub fn new() -> Self {
        ConnectionsManager {
            con_map: HashMap::new(),
        }
    }
    pub fn insert_connection(&mut self, connection: Connection) {
        self.con_map.insert(connection.connection_id, connection);
    }
    pub fn remove_connection(&mut self, connection_id: u64) {
        self.con_map.remove(&connection_id);
    }
    pub fn get_connection(&self, connection_id: u64) -> Option<&Connection> {
        self.con_map.get(&connection_id)
    }
    pub fn get_connection_mut(&mut self, connection_id: u64) -> Option<&mut Connection> {
        self.con_map.get_mut(&connection_id)
    }
    pub fn is_has_connection(&self, connection_id: u64) -> bool {
        self.con_map.contains_key(&connection_id)
    }
    pub fn get_connections(&mut self) -> &HashMap<u64, Connection> {
        &self.con_map
    }
}
