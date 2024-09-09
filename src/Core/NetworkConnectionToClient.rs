use std::collections::{HashSet, SortedList};

use crate::NetworkConnection;
use crate::NetworkIdentity;
use crate::NetworkMessage;
use crate::NetworkServer;
use crate::NetworkTime;
use crate::NetworkWriter;
use crate::SnapshotInterpolation;
use crate::Transport;

pub struct NetworkConnectionToClient {
    pub address: String,
    observing: HashSet<NetworkIdentity>,
    unbatcher: Unbatcher,
    drift_ema: ExponentialMovingAverage,
    delivery_time_ema: ExponentialMovingAverage,
    remote_timeline: f64,
    remote_timescale: f64,
    buffer_time_multiplier: f64,
    snapshots: SortedList<f64, TimeSnapshot>,
    snapshot_buffer_size_limit: usize,
    last_ping_time: f64,
    rtt: ExponentialMovingAverage,
    reliable_rpcs: NetworkWriter,
    unreliable_rpcs: NetworkWriter,
}

impl NetworkConnectionToClient {
    pub fn new(connection_id: u32, client_address: &str) -> Self {
        let address = client_address.to_string();
        let snapshot_buffer_size_limit = std::cmp::max(
            NetworkClient::snapshot_settings().buffer_time_multiplier as usize,
            64,
        );
        let drift_ema = ExponentialMovingAverage::new(
            NetworkServer::send_rate() * NetworkClient::snapshot_settings().drift_ema_duration,
        );
        let delivery_time_ema = ExponentialMovingAverage::new(
            NetworkServer::send_rate()
                * NetworkClient::snapshot_settings().delivery_time_ema_duration,
        );

        Self {
            address,
            observing: HashSet::new(),
            unbatcher: Unbatcher::new(),
            drift_ema,
            delivery_time_ema,
            remote_timeline: 0.0,
            remote_timescale: 1.0,
            buffer_time_multiplier: NetworkClient::snapshot_settings().buffer_time_multiplier,
            snapshots: SortedList::new(),
            snapshot_buffer_size_limit,
            last_ping_time: 0.0,
            rtt: ExponentialMovingAverage::new(NetworkTime::ping_window_size()),
            reliable_rpcs: NetworkWriter::new(),
            unreliable_rpcs: NetworkWriter::new(),
        }
    }

    pub fn on_time_snapshot(&mut self, snapshot: TimeSnapshot) {
        if self.snapshots.len() >= self.snapshot_buffer_size_limit {
            return;
        }

        if NetworkClient::snapshot_settings().dynamic_adjustment {
            self.buffer_time_multiplier = SnapshotInterpolation::dynamic_adjustment(
                NetworkServer::send_interval(),
                self.delivery_time_ema.standard_deviation(),
                NetworkClient::snapshot_settings().dynamic_adjustment_tolerance,
            );
        }

        SnapshotInterpolation::insert_and_adjust(
            &mut self.snapshots,
            NetworkClient::snapshot_settings().buffer_limit,
            snapshot,
            &mut self.remote_timeline,
            &mut self.remote_timescale,
            NetworkServer::send_interval(),
            self.buffer_time(),
            NetworkClient::snapshot_settings().catchup_speed,
            NetworkClient::snapshot_settings().slowdown_speed,
            &mut self.drift_ema,
            NetworkClient::snapshot_settings().catchup_negative_threshold,
            NetworkClient::snapshot_settings().catchup_positive_threshold,
            &mut self.delivery_time_ema,
        );
    }

    pub fn update_time_interpolation(&mut self) {
        if !self.snapshots.is_empty() {
            SnapshotInterpolation::step_time(
                std::time::Duration::from_secs_f32(NetworkTime::unscaled_delta_time()),
                &mut self.remote_timeline,
                self.remote_timescale,
            );

            SnapshotInterpolation::step_interpolation(&mut self.snapshots, self.remote_timeline, &mut _, &mut _, &mut _);
        }
    }

    fn update_ping(&mut self) {
        if NetworkTime::local_time() >= self.last_ping_time + NetworkTime::ping_interval() {
            let ping_message = NetworkPingMessage::new(NetworkTime::local_time(), 0.0);
            self.send(&ping_message, Channels::Unreliable);
            self.last_ping_time = NetworkTime::local_time();
        }
    }

    fn buffer_time(&self) -> f64 {
        NetworkServer::send_interval() * self.buffer_time_multiplier
    }

    fn send<T: NetworkMessage>(&mut self, message: &T, channel: u32) {
        self.reliable_rpcs.write(message);
        self.unreliable_rpcs.write(message);

        let segment = match channel {
            Channels::Reliable => self.reliable_rpcs.as_segment(),
            Channels::Unreliable => self.unreliable_rpcs.as_segment(),
            _ => todo!("Implement handling for other channel types"),
        };

        Transport::server_send(self.connection_id, segment, channel);
    }

    pub fn update(&mut self) {
        self.update_ping();
        NetworkConnection::update(self);
    }

    pub fn disconnect(&mut self) {
        self.is_ready = false;
        self.reliable_rpcs.clear();
        self.unreliable_rpcs.clear();
        Transport::server_disconnect(self.connection_id);
    }

    pub fn add_to_observing(&mut self, net_identity: &NetworkIdentity) {
        self.observing.insert(net_identity.clone());
        NetworkServer::show_for_connection(net_identity, self);
    }

    pub fn remove_from_observing(&mut self, net_identity: &NetworkIdentity, is_destroyed: bool) {
        self.observing.remove(net_identity);
        if !is_destroyed {
            NetworkServer::hide_for_connection(net_identity, self);
        }
    }

    pub fn remove_from_observings_observers(&mut self) {
        for net_identity in self.observing.iter() {
            net_identity.remove_observer(self);
        }
        self.observing.clear();
    }

    pub fn add_owned_object(&mut self, obj: &NetworkIdentity) {
        self.owned.insert(obj.clone());
    }

    pub fn remove_owned_object(&mut self, obj: &NetworkIdentity) {
        self.owned.remove(obj);
    }

    pub fn destroy_owned_objects(&mut self) {
        let mut tmp = self.owned.clone();
        for net_identity in tmp.iter() {
            if let Some(net_identity) = net_identity {
                if net_identity.scene_id != 0 {
                    NetworkServer::remove_player_for_connection(self, RemovePlayerOptions::KeepActive);
                } else {
                    NetworkServer::destroy(net_identity.game_object());
                }
            }
        }
        self.owned.clear();
    }
}

impl NetworkConnection for NetworkConnectionToClient {
    fn send_to_transport(&self, segment: ArraySegment<u8>, channel: u32) {
        Transport::server_send(self.connection_id, segment, channel);
    }
}

struct Unbatcher {}

impl Unbatcher {
    fn new() -> Self {
        Self {}
    }
}

struct ExponentialMovingAverage {
    value: f64,
    alpha: f64,
}

impl ExponentialMovingAverage {
    fn new(window_size: f64) -> Self {
        let alpha = 2.0 / (window_size + 1.0);
        Self { value: 0.0, alpha }
    }

    fn update(&mut self, new_value: f64) {
        self.value = new_value * self.alpha + self.value * (1.0 - self.alpha);
    }

    fn value(&self) -> f64 {
        self.value
    }

    fn standard_deviation(&self) -> f64 {
        self.value.sqrt()
    }
}

struct TimeSnapshot {
    // Add necessary fields for a time snapshot
}

enum Channels {
    Reliable,
    Unreliable,
    // Add more channel types as needed
}

struct RemovePlayerOptions {
    KeepActive,
    // Add more options as needed
}

mod NetworkClient {
    pub fn snapshot_settings() -> SnapshotSettings {
        // Return the current snapshot settings
        SnapshotSettings::default()
    }
}

mod SnapshotInterpolation {
    pub fn dynamic_adjustment(
        send_interval: f64,
        delivery_time_std_dev: f64,
        dynamic_adjustment_tolerance: f64,
    ) -> f64 {
        // Implement the dynamic adjustment logic
        2.0
    }

    pub fn insert_and_adjust(
        snapshots: &mut SortedList<f64, TimeSnapshot>,
        buffer_limit: usize,
        snapshot: TimeSnapshot,
        remote_timeline: &mut f64,
        remote_timescale: &mut f64,
        send_interval: f64,
        buffer_time: f64,
        catchup_speed: f64,
        slowdown_speed: f64,
        drift_ema: &mut ExponentialMovingAverage,
        catchup_negative_threshold: f64,
        catchup_positive_threshold: f64,
        delivery_time_ema: &mut ExponentialMovingAverage,
    ) {
        // Implement the insert and adjust logic
    }

    pub fn step_time(
        delta_time: std::time::Duration,
        remote_timeline: &mut f64,
        remote_timescale: f64,
    ) {
        // Implement the step time logic
    }

    pub fn step_interpolation(
        snapshots: &mut SortedList<f64, TimeSnapshot>,
        remote_timeline: f64,
        _out_t: &mut f64,
        _out_local_time: &mut f64,
        _out_remote_time: &mut f64,
    ) {
        // Implement the step interpolation logic
    }
}

mod NetworkServer {
    pub fn send_interval() -> f64 {
        // Return the current send interval
        0.1
    }

    pub fn show_for_connection(net_identity: &NetworkIdentity, conn: &NetworkConnectionToClient) {
        // Implement the show for connection logic
    }

    pub fn hide_for_connection(net_identity: &NetworkIdentity, conn: &NetworkConnectionToClient) {
        // Implement the hide for connection logic
    }

    pub fn remove_player_for_connection(
        conn: &NetworkConnectionToClient,
        options: RemovePlayerOptions,
    ) {
        // Implement the remove player for connection logic
    }

    pub fn destroy(game_object: &GameObject) {
        // Implement the destroy logic
    }
}

mod NetworkTime {
    pub fn local_time() -> f64 {
        // Return the current local time
        Instant::now().as_secs_f64()
    }

    pub fn unscaled_delta_time() -> f32 {
        // Return the current unscaled delta time
        0.016
    }

    pub fn ping_interval() -> f64 {
        // Return the current ping interval
        1.0
    }

    pub fn ping_window_size() -> usize {
        // Return the current ping window size
        64
    }
}

struct GameObject {}

struct SnapshotSettings {
    buffer_time_multiplier: f64,
    drift_ema_duration: f64,
    delivery_time_ema_duration: f64,
    buffer_limit: usize,
    catchup_speed: f64,
    slowdown_speed: f64,
    catchup_negative_threshold: f64,
    catchup_positive_threshold: f64,
    dynamic_adjustment: bool,
    dynamic_adjustment_tolerance: f64,
}

impl Default for SnapshotSettings {
    fn default() -> Self {
        Self {
            buffer_time_multiplier: 2.0,
            drift_ema_duration: 1.0,
            delivery_time_ema_duration: 1.0,
            buffer_limit: 64,
            catchup_speed: 0.1,
            slowdown_speed: 0.01,
            catchup_negative_threshold: -0.1,
            catchup_positive_threshold: 0.1,
            dynamic_adjustment: true,
            dynamic_adjustment_tolerance: 0.1,
        }
    }
}