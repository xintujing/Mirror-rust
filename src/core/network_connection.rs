use crate::core::batcher::Batch;
use crate::core::network_identity::NetworkIdentity;
use crate::core::network_time::{ExponentialMovingAverage, NetworkTime};
use crate::core::snapshot_interpolation::snapshot_interpolation::SnapshotInterpolation;
use crate::core::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::core::transport::TransportChannel;
use crate::tools::utils::get_sec_timestamp_f64;
use std::collections::BTreeSet;

#[derive(Clone)]
pub struct NetworkConnection {
    pub reliable_rpcs_batch: Batch,
    pub unreliable_rpcs_batch: Batch,
    // pub un_batch:UnBatch,
    pub connection_id: u64,
    pub is_ready: bool,
    pub is_authenticated: bool,
    pub authentication_data: Vec<u8>,
    pub address: &'static str,
    pub identity: Option<NetworkIdentity>,
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
            reliable_rpcs_batch: Batch::new(),
            unreliable_rpcs_batch: Batch::new(),
            connection_id: 0,
            is_ready: false,
            is_authenticated: false,
            authentication_data: Default::default(),
            address: "",
            identity: Some(NetworkIdentity::new(scene_id, asset_id)),
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
            reliable_rpcs_batch: Batch::new(),
            unreliable_rpcs_batch: Batch::new(),
            connection_id,
            is_ready: false,
            is_authenticated: false,
            authentication_data: Default::default(),
            address: "",
            identity: None,
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

    pub fn send(&self, batch: &Batch, channel: TransportChannel) {
        // TODO NetworkDiagnostics.OnSend(message, channelId, writer.Position, 1);

        // TODO GetBatchForChannelId(channelId).AddMessage(segment, NetworkTime.localTime);
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
}