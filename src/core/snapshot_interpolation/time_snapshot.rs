use crate::core::snapshot_interpolation::snapshot::Snapshot;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct TimeSnapshot {
    pub remote_time: f64,
    pub local_time: f64,
}

impl TimeSnapshot {
    pub fn new(remote_time: f64, local_time: f64) -> Self {
        Self {
            remote_time,
            local_time,
        }
    }
}

impl Eq for TimeSnapshot {}
impl Ord for TimeSnapshot {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.remote_time.partial_cmp(&other.remote_time).unwrap()
    }
}

impl Snapshot for TimeSnapshot {
    fn local_time(&self) -> f64 {
        self.local_time
    }

    fn remote_time(&self) -> f64 {
        self.remote_time
    }

    fn set_local_time(&mut self, local_time: f64) {
        self.local_time = local_time;
    }

    fn set_remote_time(&mut self, remote_time: f64) {
        self.remote_time = remote_time;
    }
}

