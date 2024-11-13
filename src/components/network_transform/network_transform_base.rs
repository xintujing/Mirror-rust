use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait, SyncDirection, SyncMode};
use crate::components::network_transform::transform_snapshot::TransformSnapshot;
use crate::core::backend_data::{NetworkBehaviourSetting, NetworkTransformBaseSetting};
use crate::core::network_manager::{GameObject, NetworkManagerStatic};
use crate::core::network_reader::NetworkReader;
use crate::core::network_time::NetworkTime;
use crate::core::network_writer::NetworkWriter;
use crate::core::snapshot_interpolation::snapshot_interpolation::SnapshotInterpolation;
use nalgebra::{Quaternion, Vector3};
use ordered_float::OrderedFloat;
use std::any::Any;
use std::collections::BTreeMap;
use std::hash::Hash;

#[derive(Debug, PartialOrd, PartialEq)]
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
    network_behaviour: NetworkBehaviour,
    pub coordinate_space: CoordinateSpace,
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
    pub send_interval_multiplier: u32,
    pub timeline_offset: bool,
}

impl NetworkTransformBase {
    pub fn new(network_transform_base_setting: NetworkTransformBaseSetting, network_behaviour_setting: NetworkBehaviourSetting, component_index: u8) -> Self {
        NetworkTransformBase {
            network_behaviour: NetworkBehaviour::new(network_behaviour_setting, component_index),
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

    // void AddSnapshot
    pub fn add_snapshot(snapshots: &mut BTreeMap<OrderedFloat<f64>, TransformSnapshot>, timestamp: f64, mut position: Option<Vector3<f32>>, mut rotation: Option<Quaternion<f32>>, mut scale: Option<Vector3<f32>>) {
        let last_snapshot = snapshots.iter().last();
        if position.is_none() {
            if let Some((_, last_snapshot)) = last_snapshot {
                position = Some(last_snapshot.position);
            } else {
                // TODO
            }
        }
        if rotation.is_none() {
            if let Some((_, last_snapshot)) = last_snapshot {
                rotation = Some(last_snapshot.rotation);
            } else {
                // TODO
            }
        }
        if scale.is_none() {
            if let Some((_, last_snapshot)) = last_snapshot {
                scale = Some(last_snapshot.scale);
            } else {
                // TODO
            }
        }
        let new_snapshot = TransformSnapshot::new(timestamp, NetworkTime::local_time(), position.unwrap(), rotation.unwrap(), scale.unwrap());
        let snapshot_settings = NetworkManagerStatic::get_network_manager_singleton().snapshot_interpolation_settings();
        SnapshotInterpolation::insert_if_not_exists(snapshots, snapshot_settings.buffer_limit, new_snapshot);
    }
}

pub trait NetworkTransformBaseTrait {
    fn coordinate_space(&self) -> &CoordinateSpace;
    fn set_coordinate_space(&mut self, value: CoordinateSpace);
    fn get_game_object(&self) -> &GameObject;
    fn set_game_object(&mut self, value: GameObject);
    fn get_position(&self) -> Vector3<f32> {
        if self.coordinate_space() == &CoordinateSpace::Local {
            self.get_game_object().transform.local_position
        } else {
            self.get_game_object().transform.position
        }
    }
    fn set_position(&mut self, value: Vector3<f32>) {
        let mut game_object = self.get_game_object().clone();
        if self.coordinate_space() == &CoordinateSpace::Local {
            game_object.transform.local_position = value;
        } else {
            game_object.transform.position = value;
        }
        self.set_game_object(game_object);
    }
    fn get_rotation(&self) -> Quaternion<f32> {
        if self.coordinate_space() == &CoordinateSpace::Local {
            self.get_game_object().transform.local_rotation
        } else {
            self.get_game_object().transform.rotation
        }
    }
    fn set_rotation(&mut self, value: Quaternion<f32>) {
        let mut game_object = self.get_game_object().clone();
        if self.coordinate_space() == &CoordinateSpace::Local {
            game_object.transform.local_rotation = value;
        } else {
            game_object.transform.rotation = value;
        }
        self.set_game_object(game_object);
    }
    fn get_scale(&self) -> Vector3<f32> {
        if self.coordinate_space() == &CoordinateSpace::Local {
            self.get_game_object().transform.local_scale
        } else {
            self.get_game_object().transform.scale
        }
    }
    fn set_scale(&mut self, value: Vector3<f32>) {
        let mut game_object = self.get_game_object().clone();
        if self.coordinate_space() == &CoordinateSpace::Local {
            game_object.transform.local_scale = value;
        } else {
            game_object.transform.scale = value;
        }
        self.set_game_object(game_object);
    }
}

impl NetworkBehaviourTrait for NetworkTransformBase {
    fn sync_interval(&self) -> f64 {
        self.network_behaviour.sync_interval()
    }

    fn set_sync_interval(&mut self, value: f64) {
        self.network_behaviour.set_sync_interval(value)
    }

    fn last_sync_time(&self) -> f64 {
        self.network_behaviour.last_sync_time()
    }

    fn set_last_sync_time(&mut self, value: f64) {
        self.network_behaviour.set_last_sync_time(value)
    }

    fn sync_direction(&mut self) -> &SyncDirection {
        self.network_behaviour.sync_direction()
    }

    fn set_sync_direction(&mut self, value: SyncDirection) {
        self.network_behaviour.set_sync_direction(value)
    }

    fn sync_mode(&mut self) -> &SyncMode {
        self.network_behaviour.sync_mode()
    }

    fn set_sync_mode(&mut self, value: SyncMode) {
        self.network_behaviour.set_sync_mode(value)
    }

    fn component_index(&self) -> u8 {
        self.network_behaviour.component_index()
    }

    fn set_component_index(&mut self, value: u8) {
        self.network_behaviour.set_component_index(value)
    }

    fn sync_var_dirty_bits(&self) -> u64 {
        self.network_behaviour.sync_var_dirty_bits()
    }

    fn set_sync_var_dirty_bits(&mut self, value: u64) {
        self.network_behaviour.set_sync_var_dirty_bits(value)
    }

    fn sync_object_dirty_bits(&self) -> u64 {
        self.network_behaviour.sync_object_dirty_bits()
    }

    fn set_sync_object_dirty_bits(&mut self, value: u64) {
        self.network_behaviour.set_sync_object_dirty_bits(value)
    }

    fn net_id(&self) -> u32 {
        self.network_behaviour.net_id()
    }

    fn set_net_id(&mut self, value: u32) {
        self.network_behaviour.set_net_id(value)
    }

    fn connection_to_client(&self) -> u64 {
        self.network_behaviour.connection_to_client()
    }

    fn set_connection_to_client(&mut self, value: u64) {
        self.network_behaviour.set_connection_to_client(value)
    }

    fn observers(&self) -> &Vec<u64> {
        self.network_behaviour.observers()
    }

    fn set_observers(&mut self, value: Vec<u64>) {
        self.network_behaviour.set_observers(value)
    }

    fn game_object(&self) -> &GameObject {
        self.network_behaviour.game_object()
    }

    fn set_game_object(&mut self, value: GameObject) {
        self.network_behaviour.set_game_object(value)
    }

    fn is_dirty(&self) -> bool {
        self.network_behaviour.is_dirty()
    }

    fn deserialize_objects_all(&self, un_batch: NetworkReader, initial_state: bool) {}

    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {}

    fn deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool {
        true
    }

    fn on_start_server(&mut self) {}

    fn on_stop_server(&mut self) {}

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
