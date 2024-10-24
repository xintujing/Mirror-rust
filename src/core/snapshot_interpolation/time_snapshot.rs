use crate::core::snapshot_interpolation::snapshot::Snapshot;

pub struct TimeSnapshot {
    pub snapshot: Snapshot,
}

impl TimeSnapshot {
    pub fn new(remote_time: f64, local_time: f64) -> Self {
        TimeSnapshot {
            snapshot: Snapshot::new(remote_time, local_time),
        }
    }
}