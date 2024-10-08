use crate::tools::generate_id;

#[allow(dead_code)]
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
}
