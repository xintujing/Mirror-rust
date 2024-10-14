use crate::batcher::{UnBatch, Writer};
use dashmap::DashMap;

pub struct Connect {
    connect_id: i32,
    is_ready: bool,
    is_authenticated: bool,
    authentication_data: Vec<u8>,
    address: &'static str,
    identity: u32,
    owned: Vec<u32>,
    observers: Vec<i32>,
    last_message_time: f32,
    last_ping_time: f64,
    rtt: f64,
    batches: DashMap<i32, Writer>,
    un_batcher: UnBatch,
    // snapshots: SortedList<f64, TimeSnapshot>,
}