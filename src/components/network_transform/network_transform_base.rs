use crate::components::network_transform::transform_snapshot::TransformSnapshot;
use crate::core::backend_data::NetworkTransformBaseSetting;

pub struct NetworkTransformBase {
    pub is_client_with_authority: bool,

    pub client_snapshots: Vec<TransformSnapshot>,
    pub server_snapshots: Vec<TransformSnapshot>,
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
    pub coordinate_space: u8,
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
            coordinate_space: network_transform_base_setting.coordinate_space,
            send_interval_multiplier: network_transform_base_setting.send_interval_multiplier,
            timeline_offset: network_transform_base_setting.timeline_offset,
        }
    }
}