use crate::rwder::{DataReader, DataWriter, Reader, Writer};
use crate::stable_hash::StableHash;
use bytes::Bytes;
use nalgebra::{Quaternion, Vector3};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TimeSnapshotMessage {}
impl DataReader<TimeSnapshotMessage> for TimeSnapshotMessage {
    fn deserialization(reader: &mut Reader) -> TimeSnapshotMessage {
        let _ = reader;
        TimeSnapshotMessage {}
    }
}
impl DataWriter<TimeSnapshotMessage> for TimeSnapshotMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        writer.compress_var(2);
        // 57097
        writer.write_u16("Mirror.TimeSnapshotMessage".get_stable_hash_code16());
    }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ReadyMessage {}
impl DataReader<ReadyMessage> for ReadyMessage {
    fn deserialization(reader: &mut Reader) -> ReadyMessage {
        let _ = reader;
        ReadyMessage {}
    }
}
impl DataWriter<ReadyMessage> for ReadyMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        writer.compress_var(2);
        // 43708
        writer.write_u16("Mirror.ReadyMessage".get_stable_hash_code16());
    }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub struct NotReadyMessage {}
impl DataReader<NotReadyMessage> for NotReadyMessage {
    fn deserialization(reader: &mut Reader) -> NotReadyMessage {
        let _ = reader;
        NotReadyMessage {}
    }
}
impl DataWriter<NotReadyMessage> for NotReadyMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        writer.compress_var(2);
        // 43378
        writer.write_u16("Mirror.NotReadyMessage".get_stable_hash_code16());
    }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AddPlayerMessage {}
impl DataReader<AddPlayerMessage> for AddPlayerMessage {
    fn deserialization(reader: &mut Reader) -> AddPlayerMessage {
        let _ = reader;
        AddPlayerMessage {}
    }
}
impl DataWriter<AddPlayerMessage> for AddPlayerMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        writer.compress_var(2);
        // 49414
        writer.write_u16("Mirror.AddPlayerMessage".get_stable_hash_code16());
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
    pub fn new(scene_name: String, operation: SceneOperation, custom_handling: bool) -> SceneMessage {
        SceneMessage {
            scene_name,
            operation,
            custom_handling,
        }
    }
}
impl DataReader<SceneMessage> for SceneMessage {
    fn deserialization(reader: &mut Reader) -> SceneMessage {
        let scene_name = reader.read_string();
        let operation = SceneOperation::from(reader.read_u8());
        let custom_handling = reader.read_bool();
        SceneMessage {
            scene_name,
            operation,
            custom_handling,
        }
    }
}
impl DataWriter<SceneMessage> for SceneMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        let str_bytes = self.scene_name.as_bytes();
        let total_len = 6 + str_bytes.len();
        writer.compress_var_uz(total_len);
        // 3552
        writer.write_u16("Mirror.SceneMessage".get_stable_hash_code16());
        writer.write_string(str_bytes);
        writer.write_u8(self.operation.to_u8());
        writer.write_bool(self.custom_handling);
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct CommandMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Bytes,
}
impl CommandMessage {
    #[allow(dead_code)]
    pub fn new(net_id: u32, component_index: u8, function_hash: u16, payload: Bytes) -> CommandMessage {
        CommandMessage {
            net_id,
            component_index,
            function_hash,
            payload,
        }
    }

    pub fn get_payload_no_len(&self) -> Bytes {
        self.payload.slice(4..)
    }
}
impl DataReader<CommandMessage> for CommandMessage {
    fn deserialization(reader: &mut Reader) -> CommandMessage {
        let net_id = reader.read_u32();
        let component_index = reader.read_u8();
        let function_hash = reader.read_u16();
        let payload = reader.read_remaining();
        CommandMessage {
            net_id,
            component_index,
            function_hash,
            payload,
        }
    }
}
impl DataWriter<CommandMessage> for CommandMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        // 2 + 4 + 1 + 2 + 4 + self.payload.len()
        let total_len = 13 + self.payload.len();
        writer.compress_var_uz(total_len);
        // 39124
        writer.write_u16("Mirror.CommandMessage".get_stable_hash_code16());
        writer.write_u32(self.net_id);
        writer.write_u8(self.component_index);
        writer.write_u16(self.function_hash);
        writer.write_u32(1 + self.payload.len() as u32);
        writer.write(self.payload.as_ref());
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct RpcMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Bytes,
}
impl RpcMessage {
    pub fn new(net_id: u32, component_index: u8, function_hash: u16, payload: Bytes) -> RpcMessage {
        RpcMessage {
            net_id,
            component_index,
            function_hash,
            payload,
        }
    }

    #[allow(dead_code)]
    pub fn get_payload_no_len(&self) -> Bytes {
        self.payload.slice(4..)
    }
}
impl DataReader<RpcMessage> for RpcMessage {
    fn deserialization(reader: &mut Reader) -> RpcMessage {
        let net_id = reader.read_u32();
        let component_index = reader.read_u8();
        let function_hash = reader.read_u16();
        let payload = reader.read_remaining();
        RpcMessage {
            net_id,
            component_index,
            function_hash,
            payload,
        }
    }
}
impl DataWriter<RpcMessage> for RpcMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        // 2 + 4 + 1 + 2 + 4 + self.payload.len()
        let total_len = 13 + self.payload.len();
        writer.compress_var_uz(total_len);
        // 40238
        writer.write_u16("Mirror.RpcMessage".get_stable_hash_code16());
        writer.write_u32(self.net_id);
        writer.write_u8(self.component_index);
        writer.write_u16(self.function_hash);
        writer.write_u32(1 + self.payload.len() as u32);
        writer.write(self.payload.as_ref());
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
    pub payload: Bytes,
}
impl SpawnMessage {
    pub fn new(net_id: u32, is_local_player: bool, is_owner: bool, scene_id: u64, asset_id: u32, position: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>, payload: Bytes) -> SpawnMessage {
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
    pub fn get_payload_no_len(&self) -> Vec<u8> {
        self.payload[4..].to_vec()
    }
}
impl DataReader<SpawnMessage> for SpawnMessage {
    fn deserialization(reader: &mut Reader) -> SpawnMessage {
        let net_id = reader.read_u32();
        let is_local_player = reader.read_bool();
        let is_owner = reader.read_bool();
        let scene_id = reader.read_u64();
        let asset_id = reader.read_u32();
        let position = Vector3::new(reader.read_f32(), reader.read_f32(), reader.read_f32());
        let rotation = Quaternion::new(reader.read_f32(), reader.read_f32(), reader.read_f32(), reader.read_f32());
        let scale = Vector3::new(reader.read_f32(), reader.read_f32(), reader.read_f32());
        let payload = reader.read_remaining();
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
}

impl DataWriter<SpawnMessage> for SpawnMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        // 2 + 4 + 1 + 1 + 8 + 12 * 4 + self.payload.len()
        let total_len = 64 + self.payload.len();
        writer.compress_var_uz(total_len);
        // 12504
        writer.write_u16("Mirror.SpawnMessage".get_stable_hash_code16());
        writer.write_u32(self.net_id);
        writer.write_bool(self.is_local_player);
        writer.write_bool(self.is_owner);
        writer.write_u64(self.scene_id);
        writer.write_u32(self.asset_id);
        writer.write_f32(self.position.x);
        writer.write_f32(self.position.y);
        writer.write_f32(self.position.z);
        writer.write_f32(self.rotation.coords.x);
        writer.write_f32(self.rotation.coords.y);
        writer.write_f32(self.rotation.coords.z);
        writer.write_f32(self.rotation.coords.w);
        writer.write_f32(self.scale.x);
        writer.write_f32(self.scale.y);
        writer.write_f32(self.scale.z);
        writer.write_u32(1 + self.payload.len() as u32);
        writer.write(self.payload.as_ref());
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct ChangeOwnerMessage {
    pub net_id: u32,
    pub is_owner: bool,
    pub is_local_player: bool,
}


#[derive(Debug, PartialEq, Clone)]
pub struct ObjectSpawnStartedMessage {}
impl DataReader<ObjectSpawnStartedMessage> for ObjectSpawnStartedMessage {
    fn deserialization(reader: &mut Reader) -> ObjectSpawnStartedMessage {
        let _ = reader;
        ObjectSpawnStartedMessage {}
    }
}
impl DataWriter<ObjectSpawnStartedMessage> for ObjectSpawnStartedMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        writer.compress_var(2);
        // 12504
        writer.write_u16("Mirror.ObjectSpawnStartedMessage".get_stable_hash_code16());
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct ObjectSpawnFinishedMessage {}
impl DataReader<ObjectSpawnFinishedMessage> for ObjectSpawnFinishedMessage {
    fn deserialization(reader: &mut Reader) -> ObjectSpawnFinishedMessage {
        let _ = reader;
        ObjectSpawnFinishedMessage {}
    }
}
impl DataWriter<ObjectSpawnFinishedMessage> for ObjectSpawnFinishedMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        writer.compress_var(2);
        // 43444
        writer.write_u16("Mirror.ObjectSpawnFinishedMessage".get_stable_hash_code16());
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct ObjectDestroyMessage {
    pub net_id: u32,
}
impl ObjectDestroyMessage {
    pub fn new(net_id: u32) -> ObjectDestroyMessage {
        ObjectDestroyMessage {
            net_id,
        }
    }
}
impl DataReader<ObjectDestroyMessage> for ObjectDestroyMessage {
    fn deserialization(reader: &mut Reader) -> ObjectDestroyMessage {
        let net_id = reader.read_u32();
        ObjectDestroyMessage {
            net_id,
        }
    }
}
impl DataWriter<ObjectDestroyMessage> for ObjectDestroyMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        writer.compress_var(6);
        // 12504
        writer.write_u16("Mirror.ObjectDestroyMessage".get_stable_hash_code16());
        writer.write_u32(self.net_id);
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct ObjectHideMessage {
    pub net_id: u32,
}


#[derive(Debug, PartialEq, Clone)]
pub struct EntityStateMessage {
    pub net_id: u32,
    pub payload: Bytes,
}
impl EntityStateMessage {
    pub fn new(net_id: u32, payload: Bytes) -> EntityStateMessage {
        EntityStateMessage {
            net_id,
            payload,
        }
    }

    #[allow(dead_code)]
    pub fn get_payload_no_len(&self) -> Vec<u8> {
        self.payload[4..].to_vec()
    }
}
impl DataReader<EntityStateMessage> for EntityStateMessage {
    fn deserialization(reader: &mut Reader) -> EntityStateMessage {
        let net_id = reader.read_u32();
        let payload = reader.read_remaining();
        EntityStateMessage {
            net_id,
            payload,
        }
    }
}
impl DataWriter<EntityStateMessage> for EntityStateMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        // 2 + 4 + 4 + self.payload.len()
        let total_len = 10 + self.payload.len();
        writer.compress_var_uz(total_len);
        // 12504
        writer.write_u16("Mirror.EntityStateMessage".get_stable_hash_code16());
        writer.write_u32(self.net_id);
        writer.write_u32(1 + self.payload.len() as u32);
        writer.write(self.payload.as_ref());
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct NetworkPingMessage {
    pub local_time: f64,
    pub predicted_time_adjusted: f64,
}


impl NetworkPingMessage {
    #[allow(dead_code)]
    pub fn new(local_time: f64, predicted_time_adjusted: f64) -> NetworkPingMessage {
        NetworkPingMessage {
            local_time,
            predicted_time_adjusted,
        }
    }
}
impl DataReader<NetworkPingMessage> for NetworkPingMessage {
    fn deserialization(reader: &mut Reader) -> NetworkPingMessage {
        let local_time = reader.read_f64();
        let predicted_time_adjusted = reader.read_f64();
        NetworkPingMessage {
            local_time,
            predicted_time_adjusted,
        }
    }
}
impl DataWriter<NetworkPingMessage> for NetworkPingMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        writer.compress_var(18);
        // 17487
        writer.write_u16("Mirror.NetworkPingMessage".get_stable_hash_code16());
        writer.write_f64(self.local_time);
        writer.write_f64(self.predicted_time_adjusted);
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct NetworkPongMessage {
    pub local_time: f64,
    pub prediction_error_unadjusted: f64,
    pub prediction_error_adjusted: f64,
}
impl NetworkPongMessage {
    pub fn new(local_time: f64, prediction_error_unadjusted: f64, prediction_error_adjusted: f64) -> NetworkPongMessage {
        NetworkPongMessage {
            local_time,
            prediction_error_unadjusted,
            prediction_error_adjusted,
        }
    }
}
impl DataReader<NetworkPongMessage> for NetworkPongMessage {
    fn deserialization(reader: &mut Reader) -> NetworkPongMessage {
        let local_time = reader.read_f64();
        let prediction_error_unadjusted = reader.read_f64();
        let prediction_error_adjusted = reader.read_f64();
        NetworkPongMessage {
            local_time,
            prediction_error_unadjusted,
            prediction_error_adjusted,
        }
    }
}
impl DataWriter<NetworkPongMessage> for NetworkPongMessage {
    fn serialization(&mut self, writer: &mut Writer) {
        writer.compress_var(26);
        // 27095
        writer.write_u16("Mirror.NetworkPongMessage".get_stable_hash_code16());
        writer.write_f64(self.local_time);
        writer.write_f64(self.prediction_error_unadjusted);
        writer.write_f64(self.prediction_error_adjusted);
    }
}