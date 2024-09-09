use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct ExponentialMovingAverage {
    alpha: f64,
    average: f64,
    variance: f64,
}

impl ExponentialMovingAverage {
    fn new(window_size: usize) -> Self {
        ExponentialMovingAverage {
            alpha: 2.0 / (window_size as f64 + 1.0),
            average: 0.0,
            variance: 0.0,
        }
    }

    fn add(&mut self, value: f64) {
        let delta = value - self.average;
        self.average += self.alpha * delta;
        self.variance = (1.0 - self.alpha) * (self.variance + self.alpha * delta * delta);
    }
}

static NETWORK_TIME: Lazy<Mutex<NetworkTime>> = Lazy::new(|| Mutex::new(NetworkTime::new()));

struct NetworkTime {
    last_ping_time: Instant,
    ping_interval: Duration,
    rtt: ExponentialMovingAverage,
    prediction_error: ExponentialMovingAverage,
    local_start_time: Instant,
}

impl NetworkTime {
    fn new() -> Self {
        NetworkTime {
            last_ping_time: Instant::now(),
            ping_interval: Duration::from_millis(100),
            rtt: ExponentialMovingAverage::new(50),
            prediction_error: ExponentialMovingAverage::new(20),
            local_start_time: Instant::now(),
        }
    }

    fn local_time(&self) -> f64 {
        self.local_start_time.elapsed().as_secs_f64()
    }

    fn time(&self) -> f64 {
        // This would ideally be the server's time, obtained through network messages.
        self.local_time()
    }

    fn predicted_time(&self) -> f64 {
        // This should add the server's correction received via network.
        self.time() + self.prediction_error.average
    }

    fn update(&mut self) {
        if self.last_ping_time.elapsed() >= self.ping_interval {
            self.send_ping();
        }
    }

    fn send_ping(&mut self) {
        // Simulate sending a ping message and updating last ping time.
        self.last_ping_time = Instant::now();
        // Imagine this sends a network request to the server, and then receives an answer
        self.receive_pong(self.local_time());
    }

    fn receive_pong(&mut self, sent_time: f64) {
        let rtt = self.local_time() - sent_time;
        self.rtt.add(rtt);
        // Simulate server sending a correction factor back.
        self.prediction_error.add(rtt / 2.0); // Example: server could calculate the error.
    }

    fn rtt(&self) -> f64 {
        self.rtt.average
    }

    fn rtt_variance(&self) -> f64 {
        self.rtt.variance
    }
}

fn main() {
    let mut network_time = NETWORK_TIME.lock().unwrap();
    network_time.update();

    println!("RTT: {}", network_time.rtt());
    println!("RTT Variance: {}", network_time.rtt_variance());
    println!("Predicted Time: {}", network_time.predicted_time());
}
