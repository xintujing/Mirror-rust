use crate::mirror::core::messages::NetworkMessageTrait;
use crate::mirror::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::mirror::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::mirror::core::tools::compress::CompressTrait;
use nalgebra::{Quaternion, UnitQuaternion, Vector3};
use std::fmt::Debug;
use std::ops::BitOrAssign;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SyncData {
    // 改变的数据
    pub changed_data_byte: u8,
    // 位置
    pub position: Vector3<f32>,
    // 四元数
    pub quat_rotation: Quaternion<f32>,
    // 欧拉角
    pub vec_rotation: Vector3<f32>,
    // 缩放
    pub scale: Vector3<f32>,
}

impl SyncData {
    pub fn new(changed: u8, position: Vector3<f32>, quat_rotation: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        let rotation = UnitQuaternion::from_quaternion(quat_rotation);
        Self {
            changed_data_byte: changed,
            position,
            quat_rotation,
            vec_rotation: Vector3::new(rotation.euler_angles().0, rotation.euler_angles().1, rotation.euler_angles().2),
            scale,
        }
    }
}

impl NetworkMessageTrait for SyncData {
    const FULL_NAME: &'static str = "";
    fn deserialize(reader: &mut NetworkReader) -> Self {
        // 改变的数据
        let changed = reader.read_byte();

        // 位置
        let mut position = Vector3::new(0.0, 0.0, 0.0);
        if (changed & Changed::PosX.to_u8()) > 0 {
            position.x = reader.read_float();
        }
        if changed & Changed::PosY.to_u8() > 0 {
            position.y = reader.read_float();
        }
        if changed & Changed::PosZ.to_u8() > 0 {
            position.z = reader.read_float();
        }

        // 四元数
        let mut quaternion = Quaternion::identity();
        // 欧拉角
        let mut vec_rotation = Vector3::new(0.0, 0.0, 0.0);

        if (changed & Changed::CompressRot.to_u8()) > 0 {
            if (changed & Changed::RotX.to_u8()) > 0 {
                quaternion = Quaternion::decompress(reader.read_uint());
            }
        } else {
            if changed & Changed::RotX.to_u8() > 0 {
                vec_rotation.x = reader.read_float();
            }
            if changed & Changed::RotY.to_u8() > 0 {
                vec_rotation.y = reader.read_float();
            }
            if changed & Changed::RotZ.to_u8() > 0 {
                vec_rotation.z = reader.read_float();
            }
        }

        // 缩放
        let mut scale = Vector3::new(1.0, 1.0, 1.0);
        if changed & Changed::Scale.to_u8() == Changed::Scale.to_u8() {
            scale.x = reader.read_float();
            scale.y = reader.read_float();
            scale.z = reader.read_float();
        }

        if changed & Changed::CompressRot.to_u8() > 0 {
            (vec_rotation.x, vec_rotation.y, vec_rotation.z) = UnitQuaternion::from_quaternion(quaternion).euler_angles();
        } else {
            quaternion = *UnitQuaternion::from_euler_angles(vec_rotation.x, vec_rotation.y, vec_rotation.z);
        }

        Self {
            changed_data_byte: changed,
            position,
            quat_rotation: quaternion,
            vec_rotation,
            scale,
        }
    }

    fn serialize(&mut self, write: &mut NetworkWriter) {
        write.write_byte(self.changed_data_byte);

        // 位置
        if (self.changed_data_byte & Changed::PosX.to_u8()) > 0 {
            write.write_float(self.position.x);
        }

        if (self.changed_data_byte & Changed::PosY.to_u8()) > 0 {
            write.write_float(self.position.y);
        }

        if (self.changed_data_byte & Changed::PosZ.to_u8()) > 0 {
            write.write_float(self.position.z);
        }

        // rotation
        if (self.changed_data_byte & Changed::CompressRot.to_u8()) > 0 {
            if (self.changed_data_byte & Changed::Rot.to_u8()) > 0 {
                write.write_uint(self.quat_rotation.compress());
            }
        } else {
            if (self.changed_data_byte & Changed::RotX.to_u8()) > 0 {
                write.write_float(self.vec_rotation.x);
            }

            if (self.changed_data_byte & Changed::RotY.to_u8()) > 0 {
                write.write_float(self.vec_rotation.y);
            }

            if (self.changed_data_byte & Changed::RotZ.to_u8()) > 0 {
                write.write_float(self.vec_rotation.z);
            }
        }

        // 缩放
        if (self.changed_data_byte & Changed::Scale.to_u8()) == Changed::Scale.to_u8() {
            write.write_float(self.scale.x);
            write.write_float(self.scale.y);
            write.write_float(self.scale.z);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Changed {
    None = 0,
    PosX = 1,
    PosY = 2,
    PosZ = 4,
    CompressRot = 8,
    RotX = 16,   // 0x10
    RotY = 32,   // 0x20
    RotZ = 64,   // 0x40
    Scale = 128, // 0x80

    Pos = 0x07, // 0x07
    Rot = 0x70, // 0x70
}

impl Changed {
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

// 为 Changed 实现 BitOrAssign
impl BitOrAssign for Changed {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Changed::from(*self as u8 | rhs as u8);
    }
}

// 将 u8 转换回 Changed 枚举
impl From<u8> for Changed {
    fn from(value: u8) -> Self {
        match value {
            1 => Changed::PosX,
            2 => Changed::PosY,
            4 => Changed::PosZ,
            8 => Changed::CompressRot,
            16 => Changed::RotX,
            32 => Changed::RotY,
            64 => Changed::RotZ,
            128 => Changed::Scale,
            0x07 => Changed::Pos,
            0x70 => Changed::Rot,
            _ => Changed::None, // 默认值
        }
    }
}

// 保留小数
pub trait Round {
    fn my_round(&self, digits: u32) -> f32;
}

impl Round for f32 {
    fn my_round(&self, digits: u32) -> f32 {
        let multiplier = 10u32.pow(digits);
        (self * multiplier as f32).round() / multiplier as f32
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_data() {
        // -0.00069123507 0.93129396 -0.00069123507 -0.36426723
        let quat = Quaternion::<f32>::new(-0.36426723, -0.00069123507, 0.93129396, -0.00069123507);
        println!("decompress_quaternion1: {:?}", quat);
        let compress_quaternion = quat.compress();
        println!("compress_quaternion: {}", compress_quaternion);
        let decompress_quaternion = Quaternion::decompress(compress_quaternion);
        println!("decompress_quaternion2: {:?}", decompress_quaternion);

        let compress_quaternion = decompress_quaternion.compress();
        println!("compress_quaternion: {}", compress_quaternion);
    }
}