#[derive(Debug, Clone)]
pub struct Connect {
    pub connect_id: u64,
    pub is_ready: bool,
    pub is_authenticated: bool,
    pub authentication_data: Vec<u8>,
    pub address: &'static str,
    pub identity: u32,
    pub owned: Vec<u32>,
    pub observers: Vec<i32>,
    pub last_message_time: f64,
    pub last_ping_time: f64,
    pub rtt: f64,
    // snapshots: SortedList<f64, TimeSnapshot>,
}

impl Connect {
    pub fn new() -> Self {
        Connect {
            connect_id: 0,
            is_ready: false,
            is_authenticated: false,
            authentication_data: Vec::new(),
            address: "",
            identity: 0,
            owned: Vec::new(),
            observers: Vec::new(),
            last_message_time: 0.0,
            last_ping_time: 0.0,
            rtt: 0.0,
        }
    }
}