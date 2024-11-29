use crate::log_warn;
use crate::mirror::core::snapshot_interpolation::snapshot::Snapshot;
use nalgebra::{Quaternion, Vector3};

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct TransformSnapshot {
    pub remote_time: f64,
    pub local_time: f64,

    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl TransformSnapshot {
    pub fn new(remote_time: f64, local_time: f64, position: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        Self {
            remote_time,
            local_time,
            position,
            rotation,
            scale,
        }
    }

    pub fn default() -> Self {
        Self {
            remote_time: 0.0,
            local_time: 0.0,
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn transform_snapshot(from: TransformSnapshot, to: TransformSnapshot, t: f64) -> TransformSnapshot {
        let position = Vector3::lerp(&from.position, &to.position, t as f32);
        let rotation = Quaternion::lerp(&from.rotation, &to.rotation, t as f32);
        let scale = Vector3::lerp(&from.scale, &to.scale, t as f32);
        TransformSnapshot::new(0.0, 0.0, position, rotation, scale)
    }
}

impl Eq for TransformSnapshot {}
impl Ord for TransformSnapshot {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if let Some(ordering) = self.remote_time.partial_cmp(&other.remote_time) {
            ordering
        } else {
            log_warn!("TransformSnapshot::cmp() failed to compare remote_time");
            std::cmp::Ordering::Equal
        }
    }
}
impl PartialOrd for TransformSnapshot {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.remote_time.partial_cmp(&other.remote_time)
    }
}


impl Snapshot for TransformSnapshot {
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