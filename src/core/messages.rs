use crate::core::batcher::{NetworkMessageReader, NetworkMessageWriter, UnBatch};
use crate::core::network_connection::NetworkConnection;
use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::core::tools::stable_hash::StableHash;
use crate::core::transport::TransportChannel;
use nalgebra::{Quaternion, Vector3};
use std::io;

pub type NetworkMessageHandlerFunc = Box<dyn Fn(&mut NetworkConnection, &mut UnBatch, TransportChannel) + Send + Sync>;

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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TimeSnapshotMessage {}
impl TimeSnapshotMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.TimeSnapshotMessage";
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }
}
impl NetworkMessageReader for TimeSnapshotMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {
        let _ = reader;
        Ok(TimeSnapshotMessage {})
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}
impl NetworkMessageWriter for TimeSnapshotMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        writer.compress_var_uint(2);
        // 57097
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ReadyMessage {}
impl ReadyMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.ReadyMessage";
}
impl NetworkMessageReader for ReadyMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {
        let _ = reader;
        Ok(ReadyMessage {})
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}
impl NetworkMessageWriter for ReadyMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        writer.compress_var_uint(2);
        // 43708
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct NotReadyMessage {}
impl NotReadyMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.NotReadyMessage";
}
impl NetworkMessageReader for NotReadyMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {
        let _ = reader;
        Ok(NotReadyMessage {})
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}
impl NetworkMessageWriter for NotReadyMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        writer.compress_var_uint(2);
        // 43378
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AddPlayerMessage {}
impl AddPlayerMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.AddPlayerMessage";
}
impl NetworkMessageReader for AddPlayerMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {
        let _ = reader;
        Ok(AddPlayerMessage {})
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}
impl NetworkMessageWriter for AddPlayerMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        writer.compress_var_uint(2);
        // 49414
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum SceneOperation {
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
#[derive(Debug, PartialEq, Clone)]
pub struct SceneMessage {
    pub scene_name: String,
    pub operation: SceneOperation,
    pub custom_handling: bool,
}
impl SceneMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.SceneMessage";
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
impl NetworkMessageReader for SceneMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {
        let scene_name = reader.read_string_le()?;
        let operation = SceneOperation::from(reader.read_u8()?);
        let custom_handling = reader.read_bool()?;
        Ok(SceneMessage {
            scene_name,
            operation,
            custom_handling,
        })
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}
impl NetworkMessageWriter for SceneMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        let str_bytes = self.scene_name.as_bytes();
        let total_len = 6 + str_bytes.len() as u64;
        writer.compress_var_uint(total_len);
        // 3552
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
        writer.write_str(self.scene_name.as_str());
        writer.write_byte(self.operation.to_u8());
        writer.write_bool(self.custom_handling);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CommandMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Vec<u8>,
}
impl CommandMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.CommandMessage";
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
impl NetworkMessageReader for CommandMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<CommandMessage> {
        let net_id = reader.read_u32_le()?;
        let component_index = reader.read_u8()?;
        let function_hash = reader.read_u16_le()?;
        let payload = reader.read_remaining()?;
        Ok(CommandMessage {
            net_id,
            component_index,
            function_hash,
            payload: payload.to_vec(),
        })
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}
impl NetworkMessageWriter for CommandMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 2 + 4 + 1 + 2 + 4 + self.payload.len()
        let total_len = 13 + self.payload.len() as u64;
        writer.compress_var_uint(total_len);
        // 39124
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
        writer.write_uint(self.net_id);
        writer.write_byte(self.component_index);
        writer.write_ushort(self.function_hash);
        writer.write_uint(1 + self.payload.len() as u32);
        writer.write_bytes_all(self.payload.as_slice());
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RpcMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Vec<u8>,
}
impl RpcMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.RpcMessage";
    #[allow(dead_code)]
    pub fn new(net_id: u32, component_index: u8, function_hash: u16, payload: Vec<u8>) -> RpcMessage {
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
impl NetworkMessageReader for RpcMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {
        let net_id = reader.read_u32_le()?;
        let component_index = reader.read_u8()?;
        let function_hash = reader.read_u16_le()?;
        let payload = reader.read_remaining()?;
        Ok(RpcMessage {
            net_id,
            component_index,
            function_hash,
            payload: payload.to_vec(),
        })
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}
impl NetworkMessageWriter for RpcMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 2 + 4 + 1 + 2 + 4 + self.payload.len()
        let total_len = 13 + self.payload.len() as u64;
        writer.compress_var_uint(total_len);
        // 40238
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
        writer.write_uint(self.net_id);
        writer.write_byte(self.component_index);
        writer.write_ushort(self.function_hash);
        writer.write_uint(1 + self.payload.len() as u32);
        writer.write_bytes_all(self.payload.as_slice());
    }
}

#[derive(Debug, PartialEq, Clone)]
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
    pub const FULL_NAME: &'static str = "Mirror.SpawnMessage";
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
impl NetworkMessageReader for SpawnMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {
        let net_id = reader.read_u32_le()?;
        let is_local_player = reader.read_bool()?;
        let is_owner = reader.read_bool()?;
        let scene_id = reader.read_u64_le()?;
        let asset_id = reader.read_u32_le()?;
        let position = reader.read_vector3_f32_le()?;
        let rotation = reader.read_quaternion_f32_le()?;
        let scale = reader.read_vector3_f32_le()?;
        let payload = reader.read_remaining()?;
        Ok(SpawnMessage {
            net_id,
            is_local_player,
            is_owner,
            scene_id,
            asset_id,
            position,
            rotation,
            scale,
            payload: payload.to_vec(),
        })
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}

impl NetworkMessageWriter for SpawnMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 2 + 4 + 1 + 1 + 8 + 12 * 4 + self.payload.len()
        let total_len = 64 + self.payload.len() as u64;
        writer.compress_var_uint(total_len);
        // 12504
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
        writer.write_uint(self.net_id);
        writer.write_bool(self.is_local_player);
        writer.write_bool(self.is_owner);
        writer.write_ulong(self.scene_id);
        writer.write_uint(self.asset_id);
        writer.write_vector3(self.position);
        writer.write_quaternion(self.rotation);
        writer.write_vector3(self.scale);
        writer.write_uint(1 + self.payload.len() as u32);
        writer.write_bytes_all(self.payload.as_slice());
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChangeOwnerMessage {
    pub net_id: u32,
    pub is_owner: bool,
    pub is_local_player: bool,
}
impl ChangeOwnerMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.ChangeOwnerMessage";
    #[allow(dead_code)]
    pub fn new(net_id: u32, is_owner: bool, is_local_player: bool) -> Self {
        Self {
            net_id,
            is_owner,
            is_local_player,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ObjectSpawnStartedMessage {}
impl ObjectSpawnStartedMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.ObjectSpawnStartedMessage";
    pub fn new() -> Self {
        Self {}
    }
}
impl NetworkMessageReader for ObjectSpawnStartedMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {
        let _ = reader;
        Ok(ObjectSpawnStartedMessage {})
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}
impl NetworkMessageWriter for ObjectSpawnStartedMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        writer.compress_var_uint(2);
        // 12504
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectSpawnFinishedMessage {}
impl ObjectSpawnFinishedMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.ObjectSpawnFinishedMessage";
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }
}
impl NetworkMessageReader for ObjectSpawnFinishedMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {
        let _ = reader;
        Ok(ObjectSpawnFinishedMessage {})
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}
impl NetworkMessageWriter for ObjectSpawnFinishedMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        writer.compress_var_uint(2);
        // 43444
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectDestroyMessage {
    pub net_id: u32,
}
impl ObjectDestroyMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.ObjectDestroyMessage";
    #[allow(dead_code)]
    pub fn new(net_id: u32) -> ObjectDestroyMessage {
        ObjectDestroyMessage { net_id }
    }
}
impl NetworkMessageReader for ObjectDestroyMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {
        let net_id = reader.read_u32_le()?;
        Ok(ObjectDestroyMessage { net_id })
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}
impl NetworkMessageWriter for ObjectDestroyMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        writer.compress_var_uint(6);
        // 12504
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
        writer.write_uint(self.net_id);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectHideMessage {
    pub net_id: u32,
}
impl ObjectHideMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.ObjectHideMessage";
    #[allow(dead_code)]
    pub fn new(net_id: u32) -> Self {
        Self { net_id }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct EntityStateMessage {
    pub net_id: u32,
    pub payload: Vec<u8>,
}
impl EntityStateMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.EntityStateMessage";
    #[allow(dead_code)]
    pub fn new(net_id: u32, payload: Vec<u8>) -> EntityStateMessage {
        EntityStateMessage { net_id, payload }
    }

    #[allow(dead_code)]
    pub fn get_payload_no_len(&self) -> Vec<u8> {
        self.payload[4..].to_vec()
    }
}
impl NetworkMessageReader for EntityStateMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {
        let net_id = reader.read_u32_le()?;
        let payload = reader.read_remaining()?;
        Ok(EntityStateMessage { net_id, payload: payload.to_vec() })
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}
impl NetworkMessageWriter for EntityStateMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        // 2 + 4 + 4 + self.payload.len()
        let total_len = 10 + self.payload.len() as u64;
        writer.compress_var_uint(total_len);
        // 12504
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
        writer.write_uint(self.net_id);
        writer.write_uint(1 + self.payload.len() as u32);
        writer.write_bytes_all(self.payload.as_slice());
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NetworkPingMessage {
    pub local_time: f64,
    pub predicted_time_adjusted: f64,
}
impl NetworkPingMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.NetworkPingMessage";
    #[allow(dead_code)]
    pub fn new(local_time: f64, predicted_time_adjusted: f64) -> Self {
        Self {
            local_time,
            predicted_time_adjusted,
        }
    }
}
impl NetworkMessageReader for NetworkPingMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {
        let local_time = reader.read_f64_le()?;
        let predicted_time_adjusted = reader.read_f64_le()?;
        Ok(NetworkPingMessage {
            local_time,
            predicted_time_adjusted,
        })
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}
impl NetworkMessageWriter for NetworkPingMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        writer.compress_var_uint(18);
        // 17487
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
        writer.write_double(self.local_time);
        writer.write_double(self.predicted_time_adjusted);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NetworkPongMessage {
    pub local_time: f64,
    pub prediction_error_unadjusted: f64,
    pub prediction_error_adjusted: f64,
}
impl NetworkPongMessage {
    #[allow(dead_code)]
    pub const FULL_NAME: &'static str = "Mirror.NetworkPongMessage";
    #[allow(dead_code)]
    pub fn new(
        local_time: f64,
        prediction_error_unadjusted: f64,
        prediction_error_adjusted: f64,
    ) -> NetworkPongMessage {
        NetworkPongMessage {
            local_time,
            prediction_error_unadjusted,
            prediction_error_adjusted,
        }
    }
}
impl NetworkMessageReader for NetworkPongMessage {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {
        let local_time = reader.read_f64_le()?;
        let prediction_error_unadjusted = reader.read_f64_le()?;
        let prediction_error_adjusted = reader.read_f64_le()?;
        Ok(NetworkPongMessage {
            local_time,
            prediction_error_unadjusted,
            prediction_error_adjusted,
        })
    }

    fn get_hash_code() -> u16 {
        Self::FULL_NAME.get_stable_hash_code16()
    }
}
impl NetworkMessageWriter for NetworkPongMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter) {
        writer.compress_var_uint(26);
        // 27095
        writer.write_ushort(Self::FULL_NAME.get_stable_hash_code16());
        writer.write_double(self.local_time);
        writer.write_double(self.prediction_error_unadjusted);
        writer.write_double(self.prediction_error_adjusted);
    }
}
