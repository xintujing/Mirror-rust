use crate::batcher::{Batch, UnBatch};
use dashmap::DashMap;

#[derive(Debug)]
pub struct Connect {
    pub connect_id: i32,
    pub is_ready: bool,
    pub is_authenticated: bool,
    pub authentication_data: Vec<u8>,
    pub address: &'static str,
    pub identity: u32,
    pub owned: Vec<u32>,
    pub observers: Vec<i32>,
    pub last_message_time: f32,
    pub last_ping_time: f64,
    pub rtt: f64,
    pub batches: DashMap<i32, Batch>,
    pub un_batch: UnBatch,
    // snapshots: SortedList<f64, TimeSnapshot>,
}