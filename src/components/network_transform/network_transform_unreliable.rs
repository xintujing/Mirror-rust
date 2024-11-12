use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait, SyncDirection, SyncMode};
use crate::components::network_transform::network_transform_base::NetworkTransformBase;
use crate::components::network_transform::transform_snapshot::TransformSnapshot;
use crate::components::network_transform::transform_sync_data::{Changed, SyncData};
use crate::core::backend_data::{NetworkBehaviourSetting, NetworkManagerSetting, NetworkTransformBaseSetting, NetworkTransformUnreliableSetting};
use crate::core::network_connection::NetworkConnectionTrait;
use crate::core::network_manager::{NetworkManagerStatic, Transform};
use crate::core::network_reader::{NetworkMessageReader, NetworkReader, NetworkReaderTrait};
use crate::core::network_server::NetworkServerStatic;
use crate::core::network_time::NetworkTime;
use crate::core::network_writer::{NetworkMessageWriter, NetworkWriter, NetworkWriterTrait};
use crate::core::network_writer_pool::NetworkWriterPool;
use crate::core::remote_calls::{RemoteCallDelegate, RemoteCallType, RemoteProcedureCalls};
use crate::core::snapshot_interpolation::snapshot_interpolation::SnapshotInterpolation;
use crate::core::transport::TransportChannel;
use dashmap::mapref::one::Ref;
use dashmap::try_result::TryResult;
use nalgebra::{Quaternion, UnitQuaternion, Vector3};
use ordered_float::OrderedFloat;
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use tklog::{debug, error};

#[derive(Debug)]
pub struct NetworkTransformUnreliable {
    network_transform_base: NetworkTransformBase,
    pub buffer_reset_multiplier: f32,
    pub changed_detection: bool,
    pub position_sensitivity: f32,
    pub rotation_sensitivity: f32,
    pub scale_sensitivity: f32,
    pub network_behaviour: NetworkBehaviour,
    pub sync_data: SyncData,
}

impl NetworkTransformUnreliable {
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkTransformUnreliable";
    pub fn new(network_transform_base_setting: NetworkTransformBaseSetting, network_transform_unreliable_setting: NetworkTransformUnreliableSetting, network_behaviour_setting: NetworkBehaviourSetting, component_index: u8, position: Vector3<f32>, quaternion: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        Self::call_register_delegate(Self::register_delegate);
        NetworkTransformUnreliable {
            network_transform_base: NetworkTransformBase::new(network_transform_base_setting),
            buffer_reset_multiplier: network_transform_unreliable_setting.buffer_reset_multiplier,
            changed_detection: network_transform_unreliable_setting.changed_detection,
            position_sensitivity: network_transform_unreliable_setting.position_sensitivity,
            rotation_sensitivity: network_transform_unreliable_setting.rotation_sensitivity,
            scale_sensitivity: network_transform_unreliable_setting.scale_sensitivity,
            network_behaviour: NetworkBehaviour::new(network_behaviour_setting, component_index),
            sync_data: SyncData::new(Changed::None, position, quaternion, scale),
        }
    }

    fn register_delegate() {
        // System.Void Mirror.NetworkTransformUnreliable::CmdClientToServerSync(System.Nullable`1<UnityEngine.Vector3>,System.Nullable`1<UnityEngine.Quaternion>,System.Nullable`1<UnityEngine.Vector3>)
        RemoteProcedureCalls::register_delegate("System.Void Mirror.NetworkTransformUnreliable::CmdClientToServerSync(System.Nullable`1<UnityEngine.Vector3>,System.Nullable`1<UnityEngine.Quaternion>,System.Nullable`1<UnityEngine.Vector3>)",
                                                RemoteCallType::Command,
                                                RemoteCallDelegate::new("invoke_user_code_cmd_client_to_server_sync__nullable_1__nullable_1__nullable_1", Box::new(NetworkTransformUnreliable::invoke_user_code_cmd_client_to_server_sync__nullable_1__nullable_1__nullable_1)), true);

        // System.Void Mirror.NetworkTransformUnreliable::CmdClientToServerSync(Mirror.SyncData)
        RemoteProcedureCalls::register_delegate("System.Void Mirror.NetworkTransformUnreliable::CmdClientToServerSync(Mirror.SyncData)",
                                                RemoteCallType::Command,
                                                RemoteCallDelegate::new("invoke_user_code_cmd_client_to_server_sync__sync_data", Box::new(NetworkTransformUnreliable::invoke_user_code_cmd_client_to_server_sync__sync_data)), true);
    }

    // InvokeUserCode_CmdClientToServerSync__Nullable\u00601__Nullable\u00601__Nullable\u00601
    pub fn invoke_user_code_cmd_client_to_server_sync__nullable_1__nullable_1__nullable_1(component: &mut Box<dyn NetworkBehaviourTrait>, reader: &mut NetworkReader, cmd_hash: u64) {
        if !NetworkServerStatic::get_static_active() {
            error!("Command CmdClientToServerSync called on client.");
            return;
        }
        component.as_any_mut().
            downcast_mut::<Self>().
            unwrap().
            user_code_cmd_client_to_server_sync__nullable_1__nullable_1__nullable_1(reader.read_vector3_nullable(), reader.read_quaternion_nullable(), reader.read_vector3_nullable());
    }

    // UserCode_CmdClientToServerSync__Nullable\u00601__Nullable\u00601__Nullable\u00601
    fn user_code_cmd_client_to_server_sync__nullable_1__nullable_1__nullable_1(&mut self, position: Option<Vector3<f32>>, rotation: Option<Quaternion<f32>>, scale: Option<Vector3<f32>>) {
        // TODO void OnClientToServerSync( Vector3? position, Quaternion? rotation, Vector3? scale)
    }

    // &mut Box<dyn NetworkBehaviourTrait>, &mut NetworkReader, u64
    pub fn invoke_user_code_cmd_client_to_server_sync__sync_data(component: &mut Box<dyn NetworkBehaviourTrait>, reader: &mut NetworkReader, cmd_hash: u64) {
        if !NetworkServerStatic::get_static_active() {
            error!("Command CmdClientToServerSync called on client.");
            return;
        }
        let sync_data = SyncData::deserialize(reader);
        component.as_any_mut().
            downcast_mut::<Self>().
            unwrap().
            user_code_cmd_client_to_server_sync__sync_data(sync_data);
    }

    // UserCode_CmdClientToServerSync__SyncData
    fn user_code_cmd_client_to_server_sync__sync_data(&mut self, sync_data: SyncData) {
        self.on_client_to_server_sync(sync_data);
        if *self.sync_direction() != SyncDirection::ClientToServer {
            return;
        }
        self.rpc_server_to_client_sync(sync_data);
    }

    // void OnClientToServerSync
    fn on_client_to_server_sync(&mut self, mut sync_data: SyncData) {
        match NetworkServerStatic::get_static_spawned_network_identities().try_get(&self.network_behaviour.net_id()) {
            TryResult::Present(_) => {
                println!("NetworkTransformUnreliable::OnClientToServerSync: object not spawned");
            }
            TryResult::Absent => {
                println!("NetworkTransformUnreliable::OnClientToServerSync: object not found");
            }
            TryResult::Locked => {
                println!("NetworkTransformUnreliable::OnClientToServerSync: object locked");
            }
        }


        // only apply if in client authority mode
        if *self.sync_direction() != SyncDirection::ClientToServer {
            return;
        }

        let mut timestamp = 0f64;
        if let Some(conn) = NetworkServerStatic::get_static_network_connections().get(&self.network_behaviour.connection_to_client()) {
            if self.network_transform_base.server_snapshots.len() >= conn.snapshot_buffer_size_limit as usize {
                return;
            }
            timestamp = conn.remote_time_stamp();
        }

        if self.network_transform_base.only_sync_on_change {
            let time_interval_check = self.buffer_reset_multiplier as f64 * self.network_transform_base.send_interval_multiplier as f64 * self.network_behaviour.sync_interval();

            if let Some((_, last_snapshot)) = self.network_transform_base.server_snapshots.iter().last() {
                if last_snapshot.remote_time + time_interval_check < timestamp {
                    self.network_transform_base.reset_state();
                }
            }
        }

        Self::update_sync_data(&mut sync_data, &mut self.network_transform_base.server_snapshots);
        NetworkTransformBase::add_snapshot(&mut self.network_transform_base.server_snapshots, timestamp, Some(sync_data.position), Some(sync_data.quat_rotation), Some(sync_data.scale));
    }

    // void UpdateSyncData
    fn update_sync_data(sync_data: &mut SyncData, snapshots: &mut BTreeMap<OrderedFloat<f64>, TransformSnapshot>) {
        if sync_data.changed_data_byte == Changed::None.to_u8() ||
            sync_data.changed_data_byte == Changed::CompressRot.to_u8() {
            if let Some((_, last_snapshot)) = snapshots.iter().last() {
                sync_data.position = last_snapshot.position;
                sync_data.quat_rotation = last_snapshot.rotation;
                sync_data.scale = last_snapshot.scale;
            } else {
                // TODO
            }
            return;
        }

        // Just going to update these without checking if syncposition or not,
        // because if not syncing position, NT will not apply any position data
        // to the target during Apply().

        if let Some((_, last_snapshot)) = snapshots.iter().last() {
            // x
            if sync_data.changed_data_byte & Changed::PosX.to_u8() > 0 {
                sync_data.position.x = last_snapshot.position.x;
            }
            // y
            if sync_data.changed_data_byte & Changed::PosY.to_u8() > 0 {
                sync_data.position.y = last_snapshot.position.y;
            }
            // z
            if sync_data.changed_data_byte & Changed::PosZ.to_u8() > 0 {
                sync_data.position.z = last_snapshot.position.z;
            }
        } else {
            // x
            if sync_data.changed_data_byte & Changed::PosX.to_u8() > 0 {
                //  TODO
            }
            // y
            if sync_data.changed_data_byte & Changed::PosY.to_u8() > 0 {
                //  TODO
            }
            // z
            if sync_data.changed_data_byte & Changed::PosZ.to_u8() > 0 {
                //  TODO
            }
        }

        if sync_data.changed_data_byte & Changed::CompressRot.to_u8() == 0 {
            if let Some((_, last_snapshot)) = snapshots.iter().last() {
                let euler_angles = UnitQuaternion::from_quaternion(last_snapshot.rotation).euler_angles();
                // x
                if sync_data.changed_data_byte & Changed::RotX.to_u8() > 0 {
                    sync_data.vec_rotation.x = euler_angles.0;
                }
                // y
                if sync_data.changed_data_byte & Changed::RotY.to_u8() > 0 {
                    sync_data.vec_rotation.y = euler_angles.1;
                }
                // z
                if sync_data.changed_data_byte & Changed::RotZ.to_u8() > 0 {
                    sync_data.vec_rotation.z = euler_angles.2;
                }
                sync_data.quat_rotation = *UnitQuaternion::from_euler_angles(sync_data.vec_rotation.x, sync_data.vec_rotation.y, sync_data.vec_rotation.z).quaternion();
            } else {
                // x
                if sync_data.changed_data_byte & Changed::RotX.to_u8() > 0 {
                    //  TODO
                }
                // y
                if sync_data.changed_data_byte & Changed::RotY.to_u8() > 0 {
                    //  TODO
                }
                // z
                if sync_data.changed_data_byte & Changed::RotZ.to_u8() > 0 {
                    //  TODO
                }
            }
        } else {
            if let Some((_, last_snapshot)) = snapshots.iter().last() {
                sync_data.quat_rotation = last_snapshot.rotation;
            } else {
                // TODO
            }
        }


        if let Some((_, last_snapshot)) = snapshots.iter().last() {
            sync_data.scale = last_snapshot.scale;
        } else {
            // TODO
        }
    }

    // RpcServerToClientSync
    // [ClientRpc(channel = Un)]
    fn rpc_server_to_client_sync(&mut self, mut sync_data: SyncData) {
        NetworkWriterPool::get_return(|writer| {
            sync_data.serialize(writer);
            self.send_rpc_internal(
                "System.Void Mirror.NetworkTransformUnreliable::RpcServerToClientSync(Mirror.SyncData)",
                -1891602648,
                writer,
                TransportChannel::Unreliable,
                true,
            );
        });
    }
}
#[allow(dead_code)]
impl NetworkBehaviourTrait for NetworkTransformUnreliable {
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

    fn is_dirty(&self) -> bool {
        self.network_behaviour.is_dirty()
    }

    fn deserialize_objects_all(&self, un_batch: NetworkReader, initial_state: bool) {
        todo!()
    }

    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        if initial_state {
            if self.network_transform_base.sync_position {
                writer.write_vector3(self.sync_data.position);
            }
            if self.network_transform_base.sync_rotation {
                writer.write_quaternion(self.sync_data.quat_rotation);
            }
            if self.network_transform_base.sync_scale {
                writer.write_vector3(self.sync_data.scale);
            }
        }
    }

    fn deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool {
        todo!()
    }

    fn on_start_server(&mut self) {
        // TODO
    }

    fn on_stop_server(&mut self) {
        // TODO
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::backend_data::NetworkBehaviourSetting;

    #[test]
    fn test_network_behaviour_trait() {
        let a = NetworkTransformUnreliable::new(NetworkTransformBaseSetting::default(), NetworkTransformUnreliableSetting::default(), NetworkBehaviourSetting::default(), 0, Vector3::new(0.0, 0.0, 0.0), Quaternion::new(0.0, 0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
        let b = NetworkTransformUnreliable::new(NetworkTransformBaseSetting::default(), NetworkTransformUnreliableSetting::default(), NetworkBehaviourSetting::default(), 0, Vector3::new(0.0, 0.0, 0.0), Quaternion::new(0.0, 0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
        let c = NetworkTransformUnreliable::new(NetworkTransformBaseSetting::default(), NetworkTransformUnreliableSetting::default(), NetworkBehaviourSetting::default(), 0, Vector3::new(0.0, 0.0, 0.0), Quaternion::new(0.0, 0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
        let d = NetworkTransformUnreliable::new(NetworkTransformBaseSetting::default(), NetworkTransformUnreliableSetting::default(), NetworkBehaviourSetting::default(), 0, Vector3::new(0.0, 0.0, 0.0), Quaternion::new(0.0, 0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
    }
}