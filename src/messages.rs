use crate::rwder::{DataReader, DataWriter, Reader, Writer};
use crate::stable_hash::StableHash;
use nalgebra::{Quaternion, Vector3};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TimeSnapshotMessage {}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ReadyMessage {}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct NotReadyMessage {}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AddPlayerMessage {}
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
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SceneMessage {
    pub scene_name: &'static str,
    pub operation: SceneOperation,
    pub custom_handling: bool,
}
#[derive(Debug, PartialEq, Clone)]
pub struct CommandMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Vec<u8>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct RpcMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Vec<u8>,
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
#[derive(Debug, PartialEq, Clone)]
pub struct ChangeOwnerMessage {
    pub net_id: u32,
    pub is_owner: bool,
    pub is_local_player: bool,
}
#[derive(Debug, PartialEq, Clone)]
pub struct ObjectSpawnStartedMessage {}
impl DataReader<ObjectSpawnStartedMessage> for ObjectSpawnStartedMessage {
    fn read(reader: &mut Reader) -> ObjectSpawnStartedMessage {
        ObjectSpawnStartedMessage {}
    }
}
impl DataWriter<ObjectSpawnStartedMessage> for ObjectSpawnStartedMessage {
    fn write(&mut self, writer: &mut Writer) {
        writer.compress_var_uint(2);
        // 12504
        writer.write_u16("Mirror.ObjectSpawnStartedMessage".get_stable_hash_code16());
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct ObjectSpawnFinishedMessage {}
impl DataReader<ObjectSpawnFinishedMessage> for ObjectSpawnFinishedMessage {
    fn read(reader: &mut Reader) -> ObjectSpawnFinishedMessage {
        ObjectSpawnFinishedMessage {}
    }
}
impl DataWriter<ObjectSpawnFinishedMessage> for ObjectSpawnFinishedMessage {
    fn write(&mut self, writer: &mut Writer) {
        writer.compress_var_uint(2);
        // 43444
        writer.write_u16("Mirror.ObjectSpawnFinishedMessage".get_stable_hash_code16());
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct ObjectDestroyMessage {
    pub net_id: u32,
}
#[derive(Debug, PartialEq, Clone)]
pub struct ObjectHideMessage {
    pub net_id: u32,
}
#[derive(Debug, PartialEq, Clone)]
pub struct EntityStateMessage {
    pub net_id: u32,
    pub payload: Vec<u8>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct NetworkPingMessage {
    pub local_time: f64,
    pub predicted_time_adjusted: f64,
}
impl NetworkPingMessage {
    pub fn new(local_time: f64, predicted_time_adjusted: f64) -> NetworkPingMessage {
        NetworkPingMessage {
            local_time,
            predicted_time_adjusted,
        }
    }
}
impl DataReader<NetworkPingMessage> for NetworkPingMessage {
    fn read(reader: &mut Reader) -> NetworkPingMessage {
        let local_time = reader.read_f64();
        let predicted_time_adjusted = reader.read_f64();
        NetworkPingMessage {
            local_time,
            predicted_time_adjusted,
        }
    }
}
impl DataWriter<NetworkPingMessage> for NetworkPingMessage {
    fn write(&mut self, writer: &mut Writer) {
        writer.compress_var_uint(18);
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
    fn read(reader: &mut Reader) -> NetworkPongMessage {
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
    fn write(&mut self, writer: &mut Writer) {
        writer.compress_var_uint(26);
        // 27095
        writer.write_u16("Mirror.NetworkPongMessage".get_stable_hash_code16());
        writer.write_f64(self.local_time);
        writer.write_f64(self.prediction_error_unadjusted);
        writer.write_f64(self.prediction_error_adjusted);
    }
}