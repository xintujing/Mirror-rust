use crate::components::network_transform::transform_snapshot::TransformSnapshot;
use crate::core::backend_data::NetworkTransformBaseSetting;
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;
use std::hash::Hash;

#[derive(Debug)]
pub enum CoordinateSpace {
    Local,
    World,
}

impl CoordinateSpace {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => CoordinateSpace::Local,
            1 => CoordinateSpace::World,
            _ => CoordinateSpace::Local,
        }
    }
}

#[derive(Debug)]
pub struct NetworkTransformBase {
    pub is_client_with_authority: bool,

    pub client_snapshots: BTreeMap<OrderedFloat<f64>, TransformSnapshot>,
    pub server_snapshots: BTreeMap<OrderedFloat<f64>, TransformSnapshot>,
    pub time_stamp_adjustment: f64,
    pub offset: f64,
    // pub network_behaviour_setting: NetworkBehaviourSetting,
    pub sync_position: bool,
    pub sync_rotation: bool,
    pub sync_scale: bool,
    pub only_sync_on_change: bool,
    pub compress_rotation: bool,
    pub interpolate_position: bool,
    pub interpolate_rotation: bool,
    pub interpolate_scale: bool,
    pub coordinate_space: CoordinateSpace,
    pub send_interval_multiplier: u32,
    pub timeline_offset: bool,
}

impl NetworkTransformBase {
    pub fn new(network_transform_base_setting: NetworkTransformBaseSetting) -> Self {
        NetworkTransformBase {
            is_client_with_authority: false,
            client_snapshots: Default::default(),
            server_snapshots: Default::default(),
            time_stamp_adjustment: 0.0,
            offset: 0.0,
            // network_behaviour_setting: NetworkBehaviourSetting::new(network_behaviour_setting),
            sync_position: network_transform_base_setting.sync_position,
            sync_rotation: network_transform_base_setting.sync_rotation,
            sync_scale: network_transform_base_setting.sync_scale,
            only_sync_on_change: network_transform_base_setting.only_sync_on_change,
            compress_rotation: network_transform_base_setting.compress_rotation,
            interpolate_position: network_transform_base_setting.interpolate_position,
            interpolate_rotation: network_transform_base_setting.interpolate_rotation,
            interpolate_scale: network_transform_base_setting.interpolate_scale,
            coordinate_space: CoordinateSpace::from_u8(network_transform_base_setting.coordinate_space),
            send_interval_multiplier: network_transform_base_setting.send_interval_multiplier,
            timeline_offset: network_transform_base_setting.timeline_offset,
        }
    }

    pub fn reset_state(&mut self) {
        self.client_snapshots.clear();
        self.server_snapshots.clear();
    }
}