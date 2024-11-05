use crate::core::network_connection::{NetworkConnection, NetworkConnectionTrait};
use crate::core::network_identity::NetworkIdentity;
use crate::core::network_server::NetworkServer;
use crate::core::network_time::{ExponentialMovingAverage, NetworkTime};
use crate::core::network_writer::NetworkWriter;
use crate::core::snapshot_interpolation::snapshot_interpolation::SnapshotInterpolation;
use crate::core::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::core::transport::{Transport, TransportChannel};
use crate::tools::logger::warn;
use std::collections::BTreeSet;

pub struct NetworkConnectionToClient {
    pub network_connection: NetworkConnection,
    pub reliable_rpcs_batch: NetworkWriter,
    pub unreliable_rpcs_batch: NetworkWriter,
    pub address: String,
    pub observing: Vec<u32>,
    pub drift_ema: ExponentialMovingAverage,
    pub delivery_time_ema: ExponentialMovingAverage,
    pub remote_timeline: f64,
    pub remote_timescale: f64,
    pub buffer_time_multiplier: f64,
    pub buffer_time: f64,
    pub snapshots: BTreeSet<TimeSnapshot>,
    pub snapshot_buffer_size_limit: i32,
    last_ping_time: f64,
    pub _rtt: ExponentialMovingAverage,
}
impl NetworkConnectionTrait for NetworkConnectionToClient {
    fn new(conn_id: u64) -> Self {
        let ts = NetworkTime::local_time();
        let mut network_connection_to_client = Self {
            network_connection: NetworkConnection::new(conn_id),
            reliable_rpcs_batch: NetworkWriter::new(),
            unreliable_rpcs_batch: NetworkWriter::new(),
            address: "".to_string(),
            observing: Vec::new(),
            drift_ema: ExponentialMovingAverage::new(60),
            delivery_time_ema: ExponentialMovingAverage::new(10),
            remote_timeline: ts,
            remote_timescale: ts,
            buffer_time_multiplier: 2.0,
            buffer_time: 0.0,
            snapshots: BTreeSet::new(),
            snapshot_buffer_size_limit: 64,
            last_ping_time: ts,
            _rtt: ExponentialMovingAverage::new(NetworkTime::PING_WINDOW_SIZE),
        };
        if let Some(mut transport) = Transport::get_active_transport() {
            network_connection_to_client.address = transport.server_get_client_address(conn_id);
        }
        network_connection_to_client
    }

    fn connection_id(&self) -> u64 {
        self.network_connection.connection_id()
    }

    fn last_ping_time(&self) -> f64 {
        self.network_connection.last_ping_time()
    }

    fn set_last_ping_time(&mut self, time: f64) {
        self.network_connection.set_last_ping_time(time);
    }

    fn last_message_time(&self) -> f64 {
        self.network_connection.last_message_time()
    }

    fn is_ready(&self) -> bool {
        self.network_connection.is_ready()
    }

    fn set_ready(&mut self, ready: bool) {
        self.network_connection.set_ready(ready);
    }

    fn send(&mut self, segment: &[u8], channel: TransportChannel) {
        self.network_connection.send(segment, channel);
    }

    fn update(&mut self) {
        self.network_connection.update();
    }

    fn disconnect(&mut self) {
        self.reliable_rpcs_batch.reset();
        self.unreliable_rpcs_batch.reset();
        self.network_connection.disconnect();
    }

    fn cleanup(&mut self) {
        self.network_connection.cleanup();
    }
}

impl NetworkConnectionToClient {
    pub fn on_time_snapshot(&mut self, snapshot: TimeSnapshot) {
        if self.snapshots.len() >= self.snapshot_buffer_size_limit as usize {
            return;
        }


        if let Ok(snapshot_settings) = NetworkServer::get_static_snapshot_settings().read() {

            // dynamic adjustment
            if snapshot_settings.dynamic_adjustment {
                self.buffer_time_multiplier = SnapshotInterpolation::dynamic_adjustment(
                    NetworkServer::get_static_send_interval() as f64,
                    self.delivery_time_ema.standard_deviation,
                    snapshot_settings.dynamic_adjustment_tolerance as f64,
                )
            }

            SnapshotInterpolation::insert_and_adjust(
                &mut self.snapshots,
                self.snapshot_buffer_size_limit as usize,
                snapshot,
                &mut self.remote_timeline,
                &mut self.remote_timescale,
                NetworkTime::get_ping_interval(),
                self.buffer_time,
                snapshot_settings.catchup_speed,
                snapshot_settings.slowdown_speed,
                &mut self.drift_ema,
                snapshot_settings.catchup_negative_threshold as f64,
                snapshot_settings.catchup_positive_threshold as f64,
                &mut self.delivery_time_ema,
            );
        } else {
            warn("on_time_snapshot failed to get snapshot_settings");
        }
    }
    pub fn update_time_interpolation(&mut self) {
        if self.snapshots.len() > 0 {
            SnapshotInterpolation::step_time(
                NetworkTime::get_ping_interval(),
                &mut self.remote_timeline,
                self.remote_timescale,
            );

            SnapshotInterpolation::step_interpolation(
                &mut self.snapshots,
                self.remote_timeline,
            );
        }
    }
    pub fn add_to_observing(&mut self, network_identity: &mut NetworkIdentity) {
        self.observing.push(network_identity.net_id);
        NetworkServer::show_for_connection(network_identity, self);
    }
    pub fn remove_from_observing_identities(&mut self, network_identity: &mut NetworkIdentity, is_destroyed: bool) {
        self.observing.retain(|net_id| *net_id != network_identity.net_id);
        if !is_destroyed {
            NetworkServer::hide_for_connection(network_identity, self);
        }
    }
    pub fn remove_from_observings_observers(&mut self) {
        let connection_id = self.connection_id();
        for identity in self.observing.iter_mut() {
            // TODO NetworkServer.RemoveObserverForConnection(this, identity);
            // identity.remove_observer(connection_id);
        }
        self.observing.clear();
    }

    pub fn add_owned_object(&mut self, net_id: u32) {
        self.network_connection.owned.push(net_id);
    }
    pub fn remove_owned_object(&mut self, net_id: u32) {
        self.network_connection.owned.retain(|x| net_id != net_id);
    }

    pub fn destroy_owned_objects(&mut self) {
        for identity_id in self.network_connection.owned.iter() {
            if *identity_id != 0 {
                if let Some(identity) = NetworkServer::get_static_spawned_network_identities().get_mut(identity_id) {
                    if identity.scene_id != 0 {
                        // TODO NetworkServer.RemovePlayerForConnection(this, RemovePlayerOptions.KeepActive);
                    } else {
                        // TODO NetworkServer.Destroy(netIdentity.gameObject);
                    }
                }
            }
            NetworkServer::remove_static_spawned_network_identity(*identity_id);
        }
        self.network_connection.owned.clear();
    }
}