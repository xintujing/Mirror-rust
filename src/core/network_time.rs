use crate::core::batcher::{DataReader, UnBatch};
use crate::core::messages::{NetworkPingMessage, NetworkPongMessage};
use crate::core::network_connection::NetworkConnection;
use atomic::Atomic;
use kcp2k_rust::kcp2k_channel::Kcp2KChannel;
use lazy_static::lazy_static;
use std::sync::atomic::Ordering;
use std::sync::RwLock;
use std::thread::sleep;
use std::time::Instant;

lazy_static! {
    // 全局启动时间锚点
    static ref START_INSTANT: RwLock<Instant> = RwLock::new(Instant::now());
    static ref _RTT: RwLock<ExponentialMovingAverage> = RwLock::new(ExponentialMovingAverage::new(NetworkTime::PING_WINDOW_SIZE));
    static ref _PREDICTION_ERROR_UNADJUSTED: RwLock<ExponentialMovingAverage> = RwLock::new(ExponentialMovingAverage::new(NetworkTime::PREDICTION_ERROR_WINDOW_SIZE));
}
static LAST_PING_TIME: Atomic<f64> = Atomic::new(0.0);
static PING_INTERVAL: Atomic<f64> = Atomic::new(NetworkTime::DEFAULT_PING_INTERVAL * NetworkTime::PRECISION_FACTOR);


pub struct NetworkTime;

impl NetworkTime {
    const PRECISION_FACTOR: f64 = 1_000_000.0;
    const DEFAULT_PING_INTERVAL: f64 = 0.1;
    const PING_WINDOW_SIZE: u32 = 50;
    const PREDICTION_ERROR_WINDOW_SIZE: u32 = 20;

    #[allow(dead_code)]
    pub fn local_time() -> f64 {
        START_INSTANT.read().unwrap().elapsed().as_secs_f64()
    }

    #[allow(dead_code)]
    pub fn predicted_time() -> f64 {
        Self::local_time()
    }

    #[allow(dead_code)]
    pub fn prediction_error_unadjusted() -> f64 {
        _PREDICTION_ERROR_UNADJUSTED.read().unwrap().value()
    }

    #[allow(dead_code)]
    pub fn rtt() -> f64 {
        _RTT.read().unwrap().value()
    }

    #[allow(dead_code)]
    pub fn rtt_variance() -> f64 {
        _RTT.read().unwrap().variance()
    }

    #[allow(dead_code)]
    pub fn reset_statics() {
        if let Ok(mut start_instant) = START_INSTANT.write() {
            *start_instant = Instant::now();
        }
        if let Ok(mut rtt) = _RTT.write() {
            *rtt = ExponentialMovingAverage::new(Self::PING_WINDOW_SIZE);
        }
        if let Ok(mut prediction_error_unadjusted) = _PREDICTION_ERROR_UNADJUSTED.write() {
            *prediction_error_unadjusted = ExponentialMovingAverage::new(Self::PREDICTION_ERROR_WINDOW_SIZE);
        }
        PING_INTERVAL.store(Self::DEFAULT_PING_INTERVAL * Self::PRECISION_FACTOR, Ordering::Relaxed);
        LAST_PING_TIME.store(0.0, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub fn update_client() {
        if Self::local_time() >= LAST_PING_TIME.load(Ordering::Relaxed) + PING_INTERVAL.load(Ordering::Relaxed) {
            Self::send_ping();
        }
    }

    #[allow(dead_code)]
    fn send_ping() {
        let local_time = Self::local_time();
        let predicted_time = Self::predicted_time();
        let ping_message = NetworkPingMessage::new(local_time, predicted_time);
        // Simulate sending ping message
        LAST_PING_TIME.store(Instant::now().elapsed().as_secs_f64(), Ordering::Relaxed);
        // TODO: Send ping message
    }

    #[allow(dead_code)]
    pub fn on_server_ping(connection: &mut NetworkConnection, un_batch: &mut UnBatch, channel: Kcp2KChannel) {
        if let Ok(message) = NetworkPingMessage::deserialize(un_batch) {
            let local_time = Self::local_time();
            let unadjusted_error = local_time - message.local_time;
            let adjusted_error = local_time - message.predicted_time_adjusted;
            let pong_message = NetworkPongMessage::new(message.local_time, unadjusted_error, adjusted_error);
            // TODO: Send pong message
        }
    }

    #[allow(dead_code)]
    pub fn on_server_pong(connection: &mut NetworkConnection, un_batch: &mut UnBatch, channel: Kcp2KChannel) {
        if let Ok(message) = NetworkPongMessage::deserialize(un_batch) {
            if message.local_time > Self::local_time() {
                return;
            }
            let new_rtt = Self::local_time() - message.local_time;
            _RTT.write().unwrap().add(new_rtt);
        }
    }
}

struct ExponentialMovingAverage {
    n: u32,
    value: f64,
    variance: f64,
}

impl ExponentialMovingAverage {
    #[allow(dead_code)]
    fn new(n: u32) -> Self {
        Self { n, value: 0.0, variance: 0.0 }
    }
    #[allow(dead_code)]
    fn add(&mut self, sample: f64) {
        let alpha = 2.0 / (self.n as f64 + 1.0);
        let diff = sample - self.value;
        self.value = alpha * sample + (1.0 - alpha) * self.value;
        self.variance = alpha * diff.powi(2) + (1.0 - alpha) * self.variance;
    }

    #[allow(dead_code)]
    fn value(&self) -> f64 {
        self.value
    }

    #[allow(dead_code)]
    fn variance(&self) -> f64 {
        self.variance
    }
}

#[test]
fn test_network_time() {
    NetworkTime::reset_statics();
    sleep(std::time::Duration::from_secs(3));
    let local_time = NetworkTime::local_time();
    println!("local_time: {}", local_time);
    assert!(local_time > 0.0);
}