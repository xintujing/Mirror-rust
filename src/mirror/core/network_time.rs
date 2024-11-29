use crate::mirror::core::messages::{NetworkMessageTrait, NetworkPingMessage, NetworkPongMessage};
use crate::mirror::core::network_connection::NetworkConnectionTrait;
use crate::mirror::core::network_reader::NetworkReader;
use crate::mirror::core::network_server::NetworkServerStatic;
use crate::mirror::core::transport::TransportChannel;
use crate::{log_error, log_warn};
use atomic::Atomic;
use dashmap::try_result::TryResult;
use lazy_static::lazy_static;
use std::sync::atomic::Ordering;
use std::sync::RwLock;
use std::time::Instant;

lazy_static! {
    // 全局启动时间锚点
    static ref START_INSTANT: RwLock<Instant> = RwLock::new(Instant::now());
    static ref LAST_PING_TIME: Atomic<f64> = Atomic::new(0.0);
    static ref PING_INTERVAL: Atomic<f64> = Atomic::new(NetworkTime::DEFAULT_PING_INTERVAL);
    static ref FRAME_COUNT: Atomic<u32> = Atomic::new(0);
    static ref _RTT: RwLock<ExponentialMovingAverage> = RwLock::new(ExponentialMovingAverage::new(NetworkTime::PING_WINDOW_SIZE));
    static ref _PREDICTION_ERROR_UNADJUSTED: RwLock<ExponentialMovingAverage> = RwLock::new(ExponentialMovingAverage::new(NetworkTime::PREDICTION_ERROR_WINDOW_SIZE));
}

pub struct NetworkTime;

impl NetworkTime {
    pub const DEFAULT_PING_INTERVAL: f64 = 0.1;
    pub const PING_WINDOW_SIZE: u32 = 50;
    pub const PREDICTION_ERROR_WINDOW_SIZE: u32 = 20;

    pub fn frame_count() -> u32 {
        FRAME_COUNT.load(Ordering::Relaxed)
    }

    pub fn increment_frame_count() {
        FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub fn local_time() -> f64 {
        if let Ok(start_instant) = START_INSTANT.read() {
            start_instant.elapsed().as_secs_f64()
        } else {
            log_warn!("NetworkTime::local_time() failed to get start_instant");
            Instant::now().elapsed().as_secs_f64()
        }
    }

    #[allow(dead_code)]
    pub fn predicted_time() -> f64 {
        Self::local_time()
    }

    #[allow(dead_code)]
    pub fn prediction_error_unadjusted() -> f64 {
        if let Ok(prediction_error_unadjusted) = _PREDICTION_ERROR_UNADJUSTED.read() {
            prediction_error_unadjusted.value
        } else {
            log_warn!("NetworkTime::prediction_error_unadjusted() failed to get prediction_error_unadjusted");
            0.0
        }
    }

    #[allow(dead_code)]
    pub fn rtt() -> f64 {
        if let Ok(rtt) = _RTT.read() {
            rtt.value
        } else {
            log_warn!("NetworkTime::rtt() failed to get rtt");
            0.0
        }
    }

    #[allow(dead_code)]
    pub fn rtt_variance() -> f64 {
        if let Ok(rtt) = _RTT.read() {
            rtt.variance
        } else {
            log_warn!("NetworkTime::rtt_variance() failed to get rtt");
            0.0
        }
    }

    #[allow(dead_code)]
    pub fn reset_statics() {
        Self::set_static_instant(Instant::now());
        if let Ok(mut rtt) = _RTT.write() {
            *rtt = ExponentialMovingAverage::new(Self::PING_WINDOW_SIZE);
        }
        if let Ok(mut prediction_error_unadjusted) = _PREDICTION_ERROR_UNADJUSTED.write() {
            *prediction_error_unadjusted =
                ExponentialMovingAverage::new(Self::PREDICTION_ERROR_WINDOW_SIZE);
        }
        Self::set_ping_interval(Self::DEFAULT_PING_INTERVAL);
        Self::set_last_ping_time(0.0);
    }

    #[allow(dead_code)]
    pub fn on_server_ping(
        connection_id: u64,
        un_batch: &mut NetworkReader,
        channel: TransportChannel,
    ) {
        let _ = channel;
        let message = NetworkPingMessage::deserialize(un_batch);
        let local_time = Self::local_time();
        let unadjusted_error = local_time - message.local_time;
        let adjusted_error = local_time - message.predicted_time_adjusted;
        // new prediction error
        let mut pong_message =
            NetworkPongMessage::new(message.local_time, unadjusted_error, adjusted_error);
        match NetworkServerStatic::network_connections().try_get_mut(&connection_id) {
            TryResult::Present(mut connection) => {
                // send pong message
                connection.send_network_message(&mut pong_message, TransportChannel::Reliable);
            }
            TryResult::Absent => {
                log_error!(format!(
                    "NetworkTime::on_server_ping() failed to get connection: {}",
                    connection_id
                ));
            }
            TryResult::Locked => {
                log_error!(format!(
                    "NetworkTime::on_server_ping() failed to get connection: {}",
                    connection_id
                ));
            }
        }
    }

    pub fn on_server_pong(
        _connection_id: u64,
        un_batch: &mut NetworkReader,
        _channel: TransportChannel,
    ) {
        let message = NetworkPongMessage::deserialize(un_batch);
        if message.local_time > Self::local_time() {
            return;
        }
        let new_rtt = Self::local_time() - message.local_time;
        if let Ok(mut rtt) = _RTT.write() {
            rtt.add(new_rtt);
        } else {
            log_warn!("NetworkTime::on_server_pong() failed to get rtt");
        }
    }

    #[allow(dead_code)]
    pub fn get_static_instant() -> Instant {
        if let Ok(start_instant) = START_INSTANT.read() {
            *start_instant
        } else {
            log_warn!("NetworkTime::get_static_instant() failed to get start_instant");
            Instant::now()
        }
    }

    #[allow(dead_code)]
    pub fn set_static_instant(instant: Instant) {
        if let Ok(mut start_instant) = START_INSTANT.write() {
            *start_instant = instant;
        } else {
            log_warn!("NetworkTime::set_static_instant() failed to get start_instant");
        }
    }

    #[allow(dead_code)]
    pub fn get_last_ping_time() -> f64 {
        LAST_PING_TIME.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub fn set_last_ping_time(time: f64) {
        LAST_PING_TIME.store(time, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub fn get_ping_interval() -> f64 {
        PING_INTERVAL.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub fn set_ping_interval(interval: f64) {
        PING_INTERVAL.store(interval, Ordering::Relaxed);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ExponentialMovingAverage {
    alpha: f64,
    initialized: bool,
    pub value: f64,
    pub variance: f64,
    pub standard_deviation: f64,
}

impl ExponentialMovingAverage {
    #[allow(dead_code)]
    pub fn new(n: u32) -> Self {
        Self {
            alpha: 2.0 / (n as f64 + 1.0),
            value: 0.0,
            variance: 0.0,
            standard_deviation: 0.0,
            initialized: false,
        }
    }
    #[allow(dead_code)]
    pub fn add(&mut self, new_value: f64) {
        if self.initialized {
            let delta = new_value - self.value;
            self.value += self.alpha * delta;
            self.variance = (1.0 - self.alpha) * (self.variance + self.alpha * delta.powi(2));
            self.standard_deviation = self.variance.sqrt();
        } else {
            self.value = new_value;
            self.initialized = true;
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.value = 0.0;
        self.variance = 0.0;
        self.standard_deviation = 0.0;
        self.initialized = false;
    }
}

#[test]
fn test_network_time() {
    NetworkTime::reset_statics();
    let local_time = NetworkTime::local_time();
    println!("local_time: {}", local_time);
    assert!(local_time > 0.0);

    let predicted_time = NetworkTime::predicted_time();
    println!("predicted_time: {}", predicted_time);
    assert!(predicted_time > 0.0);
}
