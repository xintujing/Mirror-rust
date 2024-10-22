use crate::core::snapshot_interpolation::snapshot::Snapshot;
use nalgebra::{Quaternion, Vector3};
use std::cell::Cell;

#[derive(Debug, Clone)]
pub struct TransformSnapshot {
    pub snapshot: Snapshot,

    pub position: Cell<Vector3<f32>>,
    pub rotation: Cell<Quaternion<f32>>,
    pub scale: Cell<Vector3<f32>>,
}

impl TransformSnapshot {
    pub fn new(remote_time: f64, local_time: f64, position: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        TransformSnapshot {
            snapshot: Snapshot::new(remote_time, local_time),
            position: Cell::new(position),
            rotation: Cell::new(rotation),
            scale: Cell::new(scale),
        }
    }

    pub fn transform_snapshot(from: TransformSnapshot, to: TransformSnapshot, t: f64) -> TransformSnapshot {
        let position = Vector3::lerp(&from.position.get(), &to.position.get(), t as f32);
        let rotation = Quaternion::lerp(&from.rotation.get(), &to.rotation.get(), t as f32);
        let scale = Vector3::lerp(&from.scale.get(), &to.scale.get(), t as f32);
        TransformSnapshot::new(0.0, 0.0, position, rotation, scale)
    }
}