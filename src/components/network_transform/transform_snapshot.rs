use crate::core::snapshot_interpolation::snapshot::Snapshot;
use nalgebra::{Quaternion, Vector3};

pub struct TransformSnapshot {
    pub snapshot: Snapshot,

    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl TransformSnapshot {
    pub fn new(remote_time: f64, local_time: f64, position: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        TransformSnapshot {
            snapshot: Snapshot::new(remote_time, local_time),
            position,
            rotation,
            scale,
        }
    }

    pub fn transform_snapshot(from: TransformSnapshot, to: TransformSnapshot, t: f64) -> TransformSnapshot {
        let position = Vector3::lerp(&from.position, &to.position, t as f32);
        let rotation = Quaternion::lerp(&from.rotation, &to.rotation, t as f32);
        let scale = Vector3::lerp(&from.scale, &to.scale, t as f32);
        TransformSnapshot::new(0.0, 0.0, position, rotation, scale)
    }
}