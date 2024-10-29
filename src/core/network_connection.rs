use crate::core::batcher::{Batch, NetworkMessageReader, NetworkMessageWriter, UnBatch};
use crate::core::batching::batcher::Batcher;
use crate::core::messages::NetworkPingMessage;
use crate::core::network_identity::NetworkIdentity;
use crate::core::network_messages::NetworkMessages;
use crate::core::network_server::NetworkServer;
use crate::core::network_time::{ExponentialMovingAverage, NetworkTime};
use crate::core::network_writer::NetworkWriter;
use crate::core::network_writer_pool::NetworkWriterPool;
use crate::core::snapshot_interpolation::snapshot_interpolation::SnapshotInterpolation;
use crate::core::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::core::transport::{Transport, TransportChannel};
use crate::tools::utils::get_sec_timestamp_f64;
use bytes::Bytes;
use dashmap::mapref::one::RefMut;
use dashmap::DashMap;
use std::collections::BTreeSet;
use tklog::{debug, error};

#[derive(Clone)]
pub struct NetworkConnection {
    pub reliable_rpcs_batch: NetworkWriter,
    pub unreliable_rpcs_batch: NetworkWriter,
    pub batches: DashMap<u8, Batcher>,
    // pub un_batch:UnBatch,
    pub connection_id: u64,
    pub is_ready: bool,
    pub is_authenticated: bool,
    pub authentication_data: Vec<u8>,
    pub address: &'static str,
    pub identity: NetworkIdentity,
    pub owned_identities: Vec<NetworkIdentity>,
    pub observing_identities: Vec<NetworkIdentity>,
    pub last_message_time: f64,
    pub remote_time_stamp: f64,

    pub last_ping_time: f64,
    pub rtt: f64,
    // pub backend_data: Arc<BackendData>,
    pub snapshots: BTreeSet<TimeSnapshot>,
    pub snapshot_buffer_size_limit: i32,
    pub drift_ema: ExponentialMovingAverage,
    pub delivery_time_ema: ExponentialMovingAverage,
    pub remote_timeline: f64,
    pub remote_timescale: f64,
    pub buffer_time_multiplier: f64,
    pub buffer_time: f64,
    pub _rtt: ExponentialMovingAverage,
}

impl NetworkConnection {
    pub fn new(scene_id: u64, asset_id: u32) -> Self {
        let ts = get_sec_timestamp_f64();
        NetworkConnection {
            reliable_rpcs_batch: NetworkWriter::new(),
            unreliable_rpcs_batch: NetworkWriter::new(),
            batches: Default::default(),
            connection_id: 0,
            is_ready: false,
            is_authenticated: false,
            authentication_data: Default::default(),
            address: "",
            identity: NetworkIdentity::new(scene_id, asset_id),
            owned_identities: Default::default(),
            observing_identities: Default::default(),
            last_message_time: ts,
            remote_time_stamp: ts,
            last_ping_time: ts,
            rtt: 0.0,
            snapshots: Default::default(),
            snapshot_buffer_size_limit: 64,
            drift_ema: ExponentialMovingAverage::new(10),
            delivery_time_ema: ExponentialMovingAverage::new(10),
            remote_timeline: 0.0,
            remote_timescale: 0.0,
            buffer_time_multiplier: 2.0,
            buffer_time: 0.0,
            _rtt: ExponentialMovingAverage::new(NetworkTime::PING_WINDOW_SIZE),
        }
    }

    pub fn network_connection(connection_id: u64) -> Self {
        let ts = get_sec_timestamp_f64();
        NetworkConnection {
            reliable_rpcs_batch: NetworkWriter::new(),
            unreliable_rpcs_batch: NetworkWriter::new(),
            batches: Default::default(),
            connection_id,
            is_ready: false,
            is_authenticated: false,
            authentication_data: Default::default(),
            address: "",
            identity: NetworkIdentity::new(0, 3541431626),
            owned_identities: Default::default(),
            observing_identities: Default::default(),
            last_message_time: ts,
            remote_time_stamp: ts,
            last_ping_time: ts,
            rtt: 0.0,
            snapshots: Default::default(),
            snapshot_buffer_size_limit: 64,
            drift_ema: ExponentialMovingAverage::new(60),
            delivery_time_ema: ExponentialMovingAverage::new(10),
            remote_timeline: 0.0,
            remote_timescale: 0.0,
            buffer_time_multiplier: 2.0,
            buffer_time: 0.0,
            _rtt: ExponentialMovingAverage::new(NetworkTime::PING_WINDOW_SIZE),
        }
    }

    pub fn send(&mut self, segment: &[u8], channel: TransportChannel) {
        self.add_message_to_batcher_for_channel(segment, channel);
    }

    pub fn send_network_message<T>(&mut self, mut message: T, channel: TransportChannel)
    where
        T: NetworkMessageWriter + NetworkMessageReader + Send,
    {
        NetworkWriterPool::get_return(|mut writer| {
            message.serialize(&mut writer);
            let max = NetworkMessages::max_message_size(channel);
            if writer.get_position() > max {
                error!("Message too large to send: {}", writer.get_position());
                return;
            }
            // TODO NetworkDiagnostics.OnSend(message, channelId, writer.Position, 1);
            self.send(writer.to_array_segment(), channel);
        });
    }

    fn add_message_to_batcher_for_channel(&mut self, segment: &[u8], channel: TransportChannel) {
        if let Some(mut batch) = self.batches.get_mut(&channel.to_u8()) {
            batch.add_message(segment, NetworkTime::local_time());
            return;
        }
        let threshold = Transport::get_active_transport().unwrap().get_batcher_threshold(channel);
        let mut batcher = Batcher::new(threshold);
        batcher.add_message(segment, NetworkTime::local_time());
        self.batches.insert(channel.to_u8(), batcher);
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

    pub fn send_to_transport(&self, segment: Vec<u8>, channel: TransportChannel) {
        if let Some(transport) = Transport::get_active_transport() {
            transport.server_send(self.connection_id, segment, channel);
        }
    }

    pub fn update(&mut self) {
        self.update_ping();

        for mut batcher in self.batches.iter_mut() {
            // using
            NetworkWriterPool::get_return(|writer| {
                while batcher.get_batcher_writer(writer) {
                    self.send_to_transport(writer.get_data(), TransportChannel::from_u8(*batcher.key()));
                }
            });
        }
    }

    fn update_ping(&mut self) {
        let local_time = NetworkTime::local_time();
        if local_time >= self.last_ping_time + NetworkTime::get_ping_interval() {
            self.last_ping_time = local_time;
            self.send_network_message(NetworkPingMessage::new(local_time, 0.0), TransportChannel::Unreliable);
        }
    }

    pub fn on_time_snapshot(&mut self, snapshot: TimeSnapshot) {
        if self.snapshots.len() >= self.snapshot_buffer_size_limit as usize {
            return;
        }

        // TODO (optional) dynamic adjustment

        SnapshotInterpolation::insert_and_adjust(
            &mut self.snapshots,
            self.snapshot_buffer_size_limit as usize,
            snapshot,
            &mut self.remote_timeline,
            &mut self.remote_timescale,
            NetworkTime::get_ping_interval(),
            self.buffer_time,
            1.0,  // TODO NetworkClient.snapshotSettings.catchupSpeed,
            1.0, // TODO NetworkClient.snapshotSettings.slowdownSpeed,
            &mut self.drift_ema,
            0.1, // TODO NetworkClient.snapshotSettings.catchupNegativeThreshold,
            0.1, // TODO NetworkClient.snapshotSettings.catchupPositiveThreshold,
            &mut self.delivery_time_ema,
        );
    }

    pub fn is_alive(&self, timeout: f64) -> bool {
        let local_time = NetworkTime::local_time();
        local_time < self.last_message_time + timeout
    }

    pub fn disconnect(&mut self) {
        if let Some(transport) = Transport::get_active_transport() {
            self.is_ready = false;
            // self.reliable_rpcs_batch.clear();
            // self.unreliable_rpcs_batch.clear();
            transport.server_disconnect(self.connection_id);
        }
    }

    pub fn add_to_observing_identities(&mut self, mut identity: NetworkIdentity) {
        NetworkServer::show_for_connection(&mut identity, self.connection_id);
        self.observing_identities.push(identity);
    }

    pub fn remove_from_observing_identities(&mut self, identity: NetworkIdentity, is_destroyed: bool) {
        self.observing_identities.retain(|x| x.net_id != identity.net_id);
        if !is_destroyed {
            NetworkServer::hide_for_connection(&identity, self.connection_id);
        }
    }

    pub fn remove_from_observings_observers(&mut self) {
        for identity in self.observing_identities.iter_mut() {
            identity.remove_observer(self.connection_id);
        }
        self.observing_identities.clear();
    }

    pub fn add_owned_object(&mut self, identity: NetworkIdentity) {
        self.owned_identities.push(identity);
    }

    pub fn remove_owned_object(&mut self, identity: NetworkIdentity) {
        self.owned_identities.retain(|x| x.net_id != identity.net_id);
    }

    pub fn destroy_owned_objects(&mut self) {
        let mut tmp = self.owned_identities.clone();
        for identity in tmp.iter() {
            if identity.scene_id != 0 {
                // TODO NetworkServer.UnSpawn(netIdentity.gameObject);
            } else {
                // TODO NetworkServer.Destroy(netIdentity.gameObject);
            }
        }
        self.owned_identities.clear();
    }

    pub fn cleanup(&mut self) {
        for mut batch in self.batches.iter_mut() {
            batch.value_mut().clear();
        }
    }
}