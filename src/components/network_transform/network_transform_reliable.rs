use crate::components::network_transform::network_transform_base::{CoordinateSpace, NetworkTransformBase, NetworkTransformBaseTrait};
use crate::components::network_transform::transform_snapshot::TransformSnapshot;
use crate::core::backend_data::NetworkBehaviourComponent;
use crate::core::network_behaviour::{NetworkBehaviourTrait, SyncDirection, SyncMode};
use crate::core::network_manager::GameObject;
use crate::core::network_server::NetworkServerStatic;
use crate::core::network_time::NetworkTime;
use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::core::snapshot_interpolation::snapshot_interpolation::SnapshotInterpolation;
use crate::core::sync_object::SyncObject;
use crate::core::tools::accurateinterval::AccurateInterval;
use crate::core::tools::compress::{Compress, CompressTrait};
use crate::core::tools::delta_compression::DeltaCompression;
use nalgebra::{UnitQuaternion, Vector3};
use std::any::Any;
use std::fmt::Debug;
use std::sync::Once;

#[derive(Debug)]
pub struct NetworkTransformReliable {
    network_transform_base: NetworkTransformBase,

    // NetworkTransformReliableSetting start
    only_sync_on_change_correction_multiplier: f32,
    rotation_sensitivity: f32,
    position_precision: f32,
    scale_precision: f32,
    compress_rotation: bool,
    // NetworkTransformReliableSetting end

    send_interval_counter: u32,
    last_send_interval_time: f64,
    last_snapshot: TransformSnapshot,
    last_serialized_position: Vector3<i64>,
    last_deserialized_position: Vector3<i64>,
    last_serialized_scale: Vector3<i64>,
    last_deserialized_scale: Vector3<i64>,
}

impl NetworkTransformReliable {
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkTransformReliable";

    // UpdateServer()
    fn update_server(&mut self) {
        if *self.sync_direction() == SyncDirection::ClientToServer && self.connection_to_client() != 0 {
            if self.network_transform_base.server_snapshots.len() == 0 {
                return;
            }

            if let Some(conn) = NetworkServerStatic::get_static_network_connections().get(&self.connection_to_client()) {
                let (from, to, t) = SnapshotInterpolation::step_interpolation(&mut self.network_transform_base.server_snapshots, conn.remote_timeline);
                let computed = TransformSnapshot::transform_snapshot(from, to, t);
                self.apply(computed, to);
            }
        }
    }

    fn changed(&self, current: TransformSnapshot) -> bool {
        // 最后一次快照的旋转
        let last_rotation = UnitQuaternion::from_quaternion(self.last_snapshot.rotation);
        // 当前快照的旋转
        let current_rotation = UnitQuaternion::from_quaternion(current.rotation);
        // 计算角度差异
        let angle = last_rotation.angle_to(&current_rotation);
        Self::quantized_changed(self.last_snapshot.position, current.position, self.position_precision) ||
            angle > self.rotation_sensitivity ||
            Self::quantized_changed(self.last_snapshot.scale, current.scale, self.scale_precision)
    }
    fn quantized_changed(u: Vector3<f32>, v: Vector3<f32>, precision: f32) -> bool {
        let u_quantized = Compress::scale_to_long_0(u, precision);
        let v_quantized = Compress::scale_to_long_0(v, precision);
        u_quantized != v_quantized
    }

    // CheckLastSendTime
    fn check_last_send_time(&mut self) {
        if self.send_interval_counter >= self.network_transform_base.send_interval_multiplier {
            self.send_interval_counter = 0;
        }

        if AccurateInterval::elapsed(NetworkTime::local_time(), NetworkServerStatic::get_static_send_interval() as f64, &mut self.last_send_interval_time) {
            self.send_interval_counter += 1;
        }
    }
}


impl NetworkBehaviourTrait for NetworkTransformReliable {
    fn new(game_object: GameObject, network_behaviour_component: &NetworkBehaviourComponent) -> Self
    where
        Self: Sized,
    {
        Self::call_register_delegate();
        Self {
            network_transform_base: NetworkTransformBase::new(game_object, network_behaviour_component.network_transform_base_setting, network_behaviour_component.network_behaviour_setting, network_behaviour_component.index),
            only_sync_on_change_correction_multiplier: network_behaviour_component.network_transform_reliable_setting.only_sync_on_change_correction_multiplier,
            rotation_sensitivity: network_behaviour_component.network_transform_reliable_setting.rotation_sensitivity,
            position_precision: network_behaviour_component.network_transform_reliable_setting.position_precision,
            scale_precision: network_behaviour_component.network_transform_reliable_setting.scale_precision,
            compress_rotation: true,
            send_interval_counter: 0,
            last_send_interval_time: f64::MIN,
            last_snapshot: TransformSnapshot::default(),
            last_serialized_position: Default::default(),
            last_deserialized_position: Default::default(),
            last_serialized_scale: Default::default(),
            last_deserialized_scale: Default::default(),
        }
    }

    fn register_delegate()
    where
        Self: Sized,
    {
        todo!()
    }

    fn get_once() -> &'static Once
    where
        Self: Sized,
    {
        static ONCE: Once = Once::new();
        &ONCE
    }

    fn sync_interval(&self) -> f64 {
        self.network_transform_base.network_behaviour.sync_interval
    }

    fn set_sync_interval(&mut self, value: f64) {
        self.network_transform_base.network_behaviour.sync_interval = value
    }

    fn last_sync_time(&self) -> f64 {
        self.network_transform_base.network_behaviour.last_sync_time
    }

    fn set_last_sync_time(&mut self, value: f64) {
        self.network_transform_base.network_behaviour.last_sync_time = value
    }

    fn sync_direction(&mut self) -> &SyncDirection {
        &self.network_transform_base.network_behaviour.sync_direction
    }

    fn set_sync_direction(&mut self, value: SyncDirection) {
        self.network_transform_base.network_behaviour.sync_direction = value
    }

    fn sync_mode(&mut self) -> &SyncMode {
        &self.network_transform_base.network_behaviour.sync_mode
    }

    fn set_sync_mode(&mut self, value: SyncMode) {
        self.network_transform_base.network_behaviour.sync_mode = value
    }

    fn index(&self) -> u8 {
        self.network_transform_base.network_behaviour.index
    }

    fn set_index(&mut self, value: u8) {
        self.network_transform_base.network_behaviour.index = value
    }

    fn sync_var_dirty_bits(&self) -> u64 {
        self.network_transform_base.network_behaviour.sync_var_dirty_bits
    }

    fn __set_sync_var_dirty_bits(&mut self, value: u64) {
        self.network_transform_base.network_behaviour.sync_var_dirty_bits = value
    }

    fn sync_object_dirty_bits(&self) -> u64 {
        self.network_transform_base.network_behaviour.sync_object_dirty_bits
    }

    fn __set_sync_object_dirty_bits(&mut self, value: u64) {
        self.network_transform_base.network_behaviour.sync_object_dirty_bits = value
    }

    fn net_id(&self) -> u32 {
        self.network_transform_base.network_behaviour.net_id
    }

    fn set_net_id(&mut self, value: u32) {
        self.network_transform_base.network_behaviour.net_id = value
    }

    fn connection_to_client(&self) -> u64 {
        self.network_transform_base.network_behaviour.connection_to_client
    }

    fn set_connection_to_client(&mut self, value: u64) {
        self.network_transform_base.network_behaviour.connection_to_client = value
    }

    fn observers(&self) -> &Vec<u64> {
        &self.network_transform_base.network_behaviour.observers
    }

    fn set_observers(&mut self, value: Vec<u64>) {
        self.network_transform_base.network_behaviour.observers = value
    }

    fn game_object(&self) -> &GameObject {
        &self.network_transform_base.network_behaviour.game_object
    }

    fn set_game_object(&mut self, value: GameObject) {
        self.network_transform_base.network_behaviour.game_object = value
    }

    fn sync_objects(&mut self) -> &mut Vec<Box<dyn SyncObject>> {
        &mut self.network_transform_base.network_behaviour.sync_objects
    }

    fn set_sync_objects(&mut self, value: Vec<Box<dyn SyncObject>>) {
        self.network_transform_base.network_behaviour.sync_objects = value
    }

    fn sync_var_hook_guard(&self) -> u64 {
        self.network_transform_base.network_behaviour.sync_var_hook_guard
    }

    fn __set_sync_var_hook_guard(&mut self, value: u64) {
        self.network_transform_base.network_behaviour.sync_var_hook_guard = value
    }


    fn is_dirty(&self) -> bool {
        self.network_transform_base.network_behaviour.is_dirty()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    // OnSerialize()
    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        let mut snapshot = self.construct();
        if initial_state {
            if self.last_snapshot.remote_time > 0.0 {
                snapshot = self.last_snapshot;
            }
            // 写入位置
            if self.sync_position() {
                writer.write_vector3(snapshot.position);
            }
            // 写入旋转
            if self.sync_rotation() {
                if self.compress_rotation {
                    writer.write_uint(snapshot.rotation.compress())
                } else {
                    writer.write_quaternion(snapshot.rotation);
                }
            }
            // 写入缩放
            if self.sync_scale() {
                writer.write_vector3(snapshot.scale);
            }
        } else {
            if self.sync_position() {
                let (_, quantized) = Compress::scale_to_long_0(snapshot.position, self.position_precision);
                DeltaCompression::compress_vector3long(writer, self.last_serialized_position, quantized);
            }
            if self.sync_rotation() {
                if self.compress_rotation {
                    writer.write_uint(snapshot.rotation.compress());
                } else {
                    writer.write_quaternion(snapshot.rotation);
                }
            }
            if self.sync_scale() {
                let (_, quantized) = Compress::scale_to_long_0(snapshot.scale, self.scale_precision);
                DeltaCompression::compress_vector3long(writer, self.last_serialized_scale, quantized);
            }
            // save serialized as 'last' for next delta compression
            if self.sync_position() {
                self.last_serialized_position = Compress::scale_to_long_0(snapshot.position, self.position_precision).1;
            }
            if self.sync_scale() {
                self.last_serialized_scale = Compress::scale_to_long_0(snapshot.scale, self.scale_precision).1;
            }
            // set 'last'
            self.last_snapshot = snapshot;
        }
    }
    fn update(&mut self) {
        self.update_server();
    }
    fn late_update(&mut self) {
        if self.send_interval_counter == self.network_transform_base.send_interval_multiplier && (!self.network_transform_base.only_sync_on_change || self.changed(self.construct())) {
            self.set_dirty()
        }
        self.check_last_send_time();
    }
}

impl NetworkTransformBaseTrait for NetworkTransformReliable {
    fn coordinate_space(&self) -> &CoordinateSpace {
        &self.network_transform_base.coordinate_space
    }

    fn set_coordinate_space(&mut self, value: CoordinateSpace) {
        self.network_transform_base.coordinate_space = value;
    }

    fn get_game_object(&self) -> &GameObject {
        &self.network_transform_base.network_behaviour.game_object
    }

    fn set_game_object(&mut self, value: GameObject) {
        self.network_transform_base.network_behaviour.game_object = value;
    }

    fn sync_position(&self) -> bool {
        self.network_transform_base.sync_position
    }

    fn sync_rotation(&self) -> bool {
        self.network_transform_base.sync_rotation
    }

    fn interpolate_position(&self) -> bool {
        self.network_transform_base.interpolate_position
    }

    fn interpolate_rotation(&self) -> bool {
        self.network_transform_base.interpolate_rotation
    }

    fn interpolate_scale(&self) -> bool {
        self.network_transform_base.interpolate_scale
    }

    fn sync_scale(&self) -> bool {
        self.network_transform_base.sync_scale
    }
}