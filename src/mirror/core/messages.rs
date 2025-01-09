use crate::mirror::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::mirror::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::mirror::core::tools::stable_hash::StableHash;
use crate::mirror::core::transport::TransportChannel;
use nalgebra::{Quaternion, Vector3};
use std::any::Any;

pub type NetworkMessageHandlerFunc = fn(u64, &mut NetworkReader, TransportChannel);

pub struct NetworkMessageHandler {
    pub func: NetworkMessageHandlerFunc,
    pub require_authentication: bool,
}

impl NetworkMessageHandler {
    pub fn wrap_handler(func: NetworkMessageHandlerFunc, require_authentication: bool) -> Self {
        Self {
            func,
            require_authentication,
        }
    }
}

pub trait NetworkMessageTrait: Send + Sync {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized;
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized;
    fn get_hash_code() -> u16
    where
        Self: Sized,
    {
        Self::get_full_name().get_stable_hash_code16()
    }
    fn get_full_name() -> &'static str
    where
        Self: Sized;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct TimeSnapshotMessage;
impl NetworkMessageTrait for TimeSnapshotMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let _ = reader;
        Self
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 57097
        writer.write_ushort(Self::get_hash_code());
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        &"Mirror.TimeSnapshotMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct ReadyMessage;
impl NetworkMessageTrait for ReadyMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let _ = reader;
        Self
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 43708
        writer.write_ushort(Self::get_hash_code());
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.ReadyMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct NotReadyMessage;
impl NetworkMessageTrait for NotReadyMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let _ = reader;
        Self
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 43378
        writer.write_ushort(Self::get_hash_code());
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.NotReadyMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct AddPlayerMessage;
impl NetworkMessageTrait for AddPlayerMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let _ = reader;
        Self
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 49414
        writer.write_ushort(Self::get_hash_code());
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.AddPlayerMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(u8)]
pub enum SceneOperation {
    #[default]
    Normal = 0,
    LoadAdditive = 1,
    UnloadAdditive = 2,
}
impl SceneOperation {
    pub fn from(value: u8) -> SceneOperation {
        match value {
            0 => SceneOperation::Normal,
            1 => SceneOperation::LoadAdditive,
            2 => SceneOperation::UnloadAdditive,
            _ => SceneOperation::Normal,
        }
    }
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}
#[derive(Debug, PartialEq, Clone, Default)]
pub struct SceneMessage {
    pub scene_name: String,
    pub operation: SceneOperation,
    pub custom_handling: bool,
}
impl SceneMessage {
    #[allow(dead_code)]
    pub fn new(
        scene_name: String,
        operation: SceneOperation,
        custom_handling: bool,
    ) -> SceneMessage {
        SceneMessage {
            scene_name,
            operation,
            custom_handling,
        }
    }
}
impl NetworkMessageTrait for SceneMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let scene_name = reader.read_string();
        let operation = SceneOperation::from(reader.read_byte());
        let custom_handling = reader.read_bool();
        Self {
            scene_name,
            operation,
            custom_handling,
        }
    }
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 3552
        writer.write_ushort(Self::get_full_name().get_stable_hash_code16());
        writer.write_str(self.scene_name.as_str());
        writer.write_byte(self.operation.to_u8());
        writer.write_bool(self.custom_handling);
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.SceneMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct CommandMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Vec<u8>,
}
impl CommandMessage {
    #[allow(dead_code)]
    pub fn new(
        net_id: u32,
        component_index: u8,
        function_hash: u16,
        payload: Vec<u8>,
    ) -> CommandMessage {
        CommandMessage {
            net_id,
            component_index,
            function_hash,
            payload,
        }
    }
    #[allow(dead_code)]
    pub fn get_payload(&self) -> Vec<u8> {
        self.payload.to_vec()
    }
    #[allow(dead_code)]
    pub fn get_payload_no_len(&self) -> Vec<u8> {
        self.payload[4..].to_vec()
    }
}

impl NetworkMessageTrait for CommandMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let net_id = reader.decompress_var_uint();
        let component_index = reader.read_byte();
        let function_hash = reader.read_ushort();
        let payload = reader.read_bytes_and_size();
        Self {
            net_id,
            component_index,
            function_hash,
            payload,
        }
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 39124
        writer.write_ushort(Self::get_full_name().get_stable_hash_code16());
        writer.compress_var_uint(self.net_id);
        writer.write_byte(self.component_index);
        writer.write_ushort(self.function_hash);
        writer.write_array_segment_and_size(self.payload.as_slice());
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.CommandMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct RpcMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Vec<u8>,
}
impl RpcMessage {
    #[allow(dead_code)]
    pub fn new(
        net_id: u32,
        component_index: u8,
        function_hash: u16,
        payload: Vec<u8>,
    ) -> RpcMessage {
        RpcMessage {
            net_id,
            component_index,
            function_hash,
            payload,
        }
    }

    #[allow(dead_code)]
    pub fn get_payload_no_len(&self) -> Vec<u8> {
        self.payload[4..].to_vec()
    }
}
impl NetworkMessageTrait for RpcMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let net_id = reader.decompress_var_uint();
        let component_index = reader.read_byte();
        let function_hash = reader.read_ushort();
        let payload = reader.read_bytes_and_size();
        Self {
            net_id,
            component_index,
            function_hash,
            payload,
        }
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 40238
        writer.write_ushort(Self::get_full_name().get_stable_hash_code16());
        writer.compress_var_uint(self.net_id);
        writer.write_byte(self.component_index);
        writer.write_ushort(self.function_hash);
        writer.write_array_segment_and_size(self.payload.as_slice());
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.RpcMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct SpawnMessage {
    pub net_id: u32,
    pub is_local_player: bool,
    pub is_owner: bool,
    pub scene_id: u64,
    pub asset_id: u32,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub payload: Vec<u8>,
}
impl SpawnMessage {
    #[allow(dead_code)]
    pub fn new(
        net_id: u32,
        is_local_player: bool,
        is_owner: bool,
        scene_id: u64,
        asset_id: u32,
        position: Vector3<f32>,
        rotation: Quaternion<f32>,
        scale: Vector3<f32>,
        payload: Vec<u8>,
    ) -> SpawnMessage {
        SpawnMessage {
            net_id,
            is_local_player,
            is_owner,
            scene_id,
            asset_id,
            position,
            rotation,
            scale,
            payload,
        }
    }
    #[allow(dead_code)]
    pub fn get_payload(&self) -> Vec<u8> {
        self.payload.to_vec()
    }
}
impl NetworkMessageTrait for SpawnMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let net_id = reader.decompress_var_uint();
        let is_local_player = reader.read_bool();
        let is_owner = reader.read_bool();
        let scene_id = reader.decompress_var_ulong();
        let asset_id = reader.decompress_var_uint();
        let position = reader.read_vector3();
        let rotation = reader.read_quaternion();
        let scale = reader.read_vector3();
        let payload = reader.read_bytes_and_size();
        Self {
            net_id,
            is_local_player,
            is_owner,
            scene_id,
            asset_id,
            position,
            rotation,
            scale,
            payload,
        }
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 12504
        writer.write_ushort(Self::get_full_name().get_stable_hash_code16());
        writer.compress_var_uint(self.net_id);
        writer.write_bool(self.is_local_player);
        writer.write_bool(self.is_owner);
        writer.compress_var_ulong(self.scene_id);
        writer.compress_var_uint(self.asset_id);
        writer.write_vector3(self.position);
        writer.write_quaternion(self.rotation);
        writer.write_vector3(self.scale);
        writer.write_array_segment_and_size(self.payload.as_slice());
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.SpawnMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ChangeOwnerMessage {
    pub net_id: u32,
    pub is_owner: bool,
    pub is_local_player: bool,
}
impl ChangeOwnerMessage {
    #[allow(dead_code)]
    pub fn new(net_id: u32, is_owner: bool, is_local_player: bool) -> Self {
        Self {
            net_id,
            is_owner,
            is_local_player,
        }
    }
}
impl NetworkMessageTrait for ChangeOwnerMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let net_id = reader.decompress_var_uint();
        let is_owner = reader.read_bool();
        let is_local_player = reader.read_bool();
        Self {
            net_id,
            is_owner,
            is_local_player,
        }
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        writer.write_ushort(Self::get_full_name().get_stable_hash_code16());
        writer.compress_var_uint(self.net_id);
        writer.write_bool(self.is_owner);
        writer.write_bool(self.is_local_player);
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.ChangeOwnerMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ObjectSpawnStartedMessage;
impl NetworkMessageTrait for ObjectSpawnStartedMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let _ = reader;
        Self
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 12504
        writer.write_ushort(Self::get_full_name().get_stable_hash_code16());
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.ObjectSpawnStartedMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ObjectSpawnFinishedMessage;
impl NetworkMessageTrait for ObjectSpawnFinishedMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let _ = reader;
        Self
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 43444
        writer.write_ushort(Self::get_full_name().get_stable_hash_code16());
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.ObjectSpawnFinishedMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ObjectDestroyMessage {
    pub net_id: u32,
}
impl ObjectDestroyMessage {
    #[allow(dead_code)]
    pub fn new(net_id: u32) -> ObjectDestroyMessage {
        ObjectDestroyMessage { net_id }
    }
}
impl NetworkMessageTrait for ObjectDestroyMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let net_id = reader.decompress_var_uint();
        Self { net_id }
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 12504
        writer.write_ushort(Self::get_full_name().get_stable_hash_code16());
        writer.compress_var_uint(self.net_id);
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.ObjectDestroyMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ObjectHideMessage {
    pub net_id: u32,
}
impl ObjectHideMessage {
    #[allow(dead_code)]
    pub fn new(net_id: u32) -> Self {
        Self { net_id }
    }
}
impl NetworkMessageTrait for ObjectHideMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let net_id = reader.decompress_var_uint();
        Self { net_id }
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 12504
        writer.write_ushort(Self::get_full_name().get_stable_hash_code16());
        writer.compress_var_uint(self.net_id);
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.ObjectHideMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct EntityStateMessage {
    pub net_id: u32,
    pub payload: Vec<u8>,
}
impl EntityStateMessage {
    #[allow(dead_code)]
    pub fn new(net_id: u32, payload: Vec<u8>) -> EntityStateMessage {
        Self { net_id, payload }
    }

    #[allow(dead_code)]
    pub fn get_payload_no_len(&self) -> Vec<u8> {
        self.payload[4..].to_vec()
    }
}
impl NetworkMessageTrait for EntityStateMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let net_id = reader.decompress_var_uint();
        let payload = reader.read_bytes_and_size();
        Self { net_id, payload }
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 12504
        writer.write_ushort(Self::get_full_name().get_stable_hash_code16());
        writer.compress_var_uint(self.net_id);
        writer.write_array_segment_and_size(self.payload.as_slice());
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.EntityStateMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct NetworkPingMessage {
    pub local_time: f64,
    pub predicted_time_adjusted: f64,
}
impl NetworkPingMessage {
    #[allow(dead_code)]
    pub fn new(local_time: f64, predicted_time_adjusted: f64) -> Self {
        Self {
            local_time,
            predicted_time_adjusted,
        }
    }
}

impl NetworkMessageTrait for NetworkPingMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let local_time = reader.read_double();
        let predicted_time_adjusted = reader.read_double();
        Self {
            local_time,
            predicted_time_adjusted,
        }
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 17487
        writer.write_ushort(Self::get_full_name().get_stable_hash_code16());
        writer.write_double(self.local_time);
        writer.write_double(self.predicted_time_adjusted);
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.NetworkPingMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct NetworkPongMessage {
    pub local_time: f64,
    pub prediction_error_unadjusted: f64,
    pub prediction_error_adjusted: f64,
}
impl NetworkPongMessage {
    #[allow(dead_code)]
    pub fn new(
        local_time: f64,
        prediction_error_unadjusted: f64,
        prediction_error_adjusted: f64,
    ) -> NetworkPongMessage {
        Self {
            local_time,
            prediction_error_unadjusted,
            prediction_error_adjusted,
        }
    }
}
impl NetworkMessageTrait for NetworkPongMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let local_time = reader.read_double();
        let prediction_error_unadjusted = reader.read_double();
        let prediction_error_adjusted = reader.read_double();
        Self {
            local_time,
            prediction_error_unadjusted,
            prediction_error_adjusted,
        }
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 27095
        writer.write_ushort(Self::get_full_name().get_stable_hash_code16());
        writer.write_double(self.local_time);
        writer.write_double(self.prediction_error_unadjusted);
        writer.write_double(self.prediction_error_adjusted);
    }

    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.NetworkPongMessage"
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
