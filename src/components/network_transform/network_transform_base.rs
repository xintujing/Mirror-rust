use crate::components::network_transform::transform_snapshot::TransformSnapshot;
use std::cell::Cell;

#[derive(Default)]
pub struct NetworkTransformBase {
    pub is_client_with_authority: bool,

    pub client_snapshots: Cell<Vec<TransformSnapshot>>,
    pub server_snapshots: Cell<Vec<TransformSnapshot>>,

    pub sync_position: bool,
    pub sync_rotation: bool,
    pub sync_scale: bool,

    pub only_sync_on_change: bool,
    pub compress_rotation: bool,

    pub interpolate_position: bool,
    pub interpolate_rotation: bool,
    pub interpolate_scale: bool,

    // CoordinateSpace

    pub send_interval_multiplier: u32,

    pub timeline_offset: bool,

    pub time_stamp_adjustment: f64,

    pub offset: f64,
}

impl NetworkTransformBase {
    pub fn new() -> Self {
        NetworkTransformBase {
            is_client_with_authority: false,
            client_snapshots: Default::default(),
            server_snapshots: Default::default(),
            sync_position: true,
            sync_rotation: true,
            sync_scale: true,
            only_sync_on_change: true,
            compress_rotation: true,
            interpolate_position: true,
            interpolate_rotation: true,
            interpolate_scale: true,
            send_interval_multiplier: 1,
            timeline_offset: false,
            time_stamp_adjustment: 0.0,
            offset: 0.0,
        }
    }
}