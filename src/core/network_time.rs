use crate::core::messages::{NetworkPingMessage, NetworkPongMessage};
use crate::core::network_connection::NetworkConnection;
use kcp2k_rust::kcp2k_channel::Kcp2KChannel;
use std::sync::atomic::AtomicU64;
use std::time::Instant;

pub struct NetworkTime {
    ping_interval: f64,
    last_ping_time: Instant,
    rtt: ExponentialMovingAverage,
    prediction_error_unadjusted: ExponentialMovingAverage,
    prediction_error_adjusted: AtomicU64,
    local_time: Instant,
}

impl NetworkTime {
    const PRECISION_FACTOR: f64 = 1_000_000.0; // 1 million for microseconds precision
    pub fn new(ping_interval: f64, ping_window_size: usize, prediction_error_window_size: usize) -> Self {
        Self {
            ping_interval,
            last_ping_time: Instant::now(),
            rtt: ExponentialMovingAverage::new(ping_window_size),
            prediction_error_unadjusted: ExponentialMovingAverage::new(prediction_error_window_size),
            prediction_error_adjusted: AtomicU64::new(0),
            local_time: Instant::now(),
        }
    }

    pub fn local_time(&self) -> f64 {
        self.local_time.elapsed().as_secs_f64()
    }

    pub fn time(&self) -> f64 {
        self.local_time()
    }

    pub fn predicted_time(&self) -> f64 {
        self.local_time() + self.prediction_error_unadjusted.value()
    }

    pub fn offset(&self) -> f64 {
        self.local_time() - self.time()
    }

    pub fn rtt(&self) -> f64 {
        self.rtt.value()
    }

    pub fn rtt_variance(&self) -> f64 {
        self.rtt.variance()
    }

    pub fn update_client(&mut self) {
        if self.local_time.elapsed().as_secs_f64() >= self.last_ping_time.elapsed().as_secs_f64() + self.ping_interval {
            self.send_ping();
        }
    }

    fn send_ping(&mut self) {
        let local_time = self.local_time();
        let predicted_time = self.predicted_time();
        let ping_message = NetworkPingMessage::new(local_time, predicted_time);
        // Simulate sending ping message
        self.last_ping_time = Instant::now();
    }

    pub fn on_server_ping(&mut self, connection: &mut NetworkConnection, message: NetworkPingMessage, channel: Kcp2KChannel) {
        let unadjusted_error = self.local_time() - message.local_time;
        let adjusted_error = self.local_time() - message.predicted_time_adjusted;
        let pong_message = NetworkPongMessage {
            local_time: message.local_time,
            prediction_error_unadjusted: unadjusted_error,
            prediction_error_adjusted: adjusted_error,
        };
        // Simulate sending pong message
    }
    pub fn on_server_pong(&mut self, connection: &mut NetworkConnection, message: NetworkPongMessage, channel: Kcp2KChannel) {
        if message.local_time > self.local_time() {
            return;
        }
        let new_rtt = self.local_time() - message.local_time;
        self.rtt.add(new_rtt);
    }


    // pub fn on_client_pong(&mut self, message: NetworkPongMessage) {
    //     if message.local_time > self.local_time() {
    //         return;
    //     }
    //     let new_rtt = self.local_time() - message.local_time;
    //     self.rtt.add(new_rtt);
    //     self.prediction_error_unadjusted.add(message.prediction_error_unadjusted);
    //     self.prediction_error_adjusted.store(
    //         (message.prediction_error_adjusted * Self::PRECISION_FACTOR) as u64,
    //         Ordering::Relaxed,
    //     );
    // }
    //
    // pub fn on_client_ping(&mut self, message: NetworkPingMessage) {
    //     let pong_message = NetworkPongMessage {
    //         local_time: message.local_time,
    //         prediction_error_unadjusted: 0.0,
    //         prediction_error_adjusted: 0.0,
    //     };
    //     // Simulate sending pong message
    // }

}

struct ExponentialMovingAverage {
    n: usize,
    value: f64,
    variance: f64,
}

impl ExponentialMovingAverage {
    fn new(n: usize) -> Self {
        Self { n, value: 0.0, variance: 0.0 }
    }

    fn add(&mut self, sample: f64) {
        let alpha = 2.0 / (self.n as f64 + 1.0);
        let diff = sample - self.value;
        self.value = alpha * sample + (1.0 - alpha) * self.value;
        self.variance = alpha * diff.powi(2) + (1.0 - alpha) * self.variance;
    }

    fn value(&self) -> f64 {
        self.value
    }

    fn variance(&self) -> f64 {
        self.variance
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_network_time() {}
}