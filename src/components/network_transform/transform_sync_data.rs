use crate::core::messages::NetworkMessageTrait;
use crate::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use nalgebra::{Quaternion, UnitQuaternion, Vector3, Vector4};
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
    /// 常量定义
    const TEN_BITS_MAX: u32 = 0b11_1111_1111; // 10 bits max value: 1023
    const QUATERNION_MIN_RANGE: f32 = -0.707107f32;
    const QUATERNION_MAX_RANGE: f32 = 0.707107f32;

    #[allow(dead_code)]
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

    /// 安全地规范化四元数，即使输入包含无效值（如 NaN）
    fn quaternion_normalize_safe(v4: Vector4<f32>) -> Quaternion<f32> {
        const FLT_MIN_NORMAL: f64 = 1.175494351e-38f64;
        let len = v4.dot(&v4);
        if len > FLT_MIN_NORMAL as f32 {
            v4.normalize().into()
        } else {
            Quaternion::identity()
        }
    }

    /// 将 `u16` 值缩放到指定的浮点范围
    fn scale_ushort_to_float(
        value: u16,
        min_value: u32,
        max_value: u32,
        min_target: f32,
        max_target: f32,
    ) -> f32 {
        let target_range: f32 = max_target - min_target;
        let value_range = (max_value - min_value) as f32;
        let value_relative = (value as u32 - min_value) as f32;
        min_target + value_relative / value_range * target_range
    }

    fn scale_float_to_ushort(
        value: f32,
        min_value: f32,
        max_value: f32,
        min_target: u16,
        max_target: u16,
    ) -> u16 {
        let target_range = (max_target - min_target) as f32;
        let value_range = max_value - min_value;
        let value_relative = value - min_value;
        min_target + (value_relative / value_range * target_range) as u16
    }

    /// 解压缩四元数
    pub fn decompress_quaternion(data: u32) -> Quaternion<f32> {
        // 获取 cScaled（位 0..10）
        let c_scaled = (data & SyncData::TEN_BITS_MAX) as u16;

        // 获取 bScaled（位 10..20）
        let b_scaled = ((data >> 10) & SyncData::TEN_BITS_MAX) as u16;

        // 获取 aScaled（位 20..30）
        let a_scaled = ((data >> 20) & SyncData::TEN_BITS_MAX) as u16;

        // 获取 largestIndex（位 30..32）
        let largest_index = (data >> 30) as usize;

        // 缩放回浮点数
        let a = SyncData::scale_ushort_to_float(
            a_scaled,
            0,
            SyncData::TEN_BITS_MAX,
            SyncData::QUATERNION_MIN_RANGE,
            SyncData::QUATERNION_MAX_RANGE,
        );
        let b = SyncData::scale_ushort_to_float(
            b_scaled,
            0,
            SyncData::TEN_BITS_MAX,
            SyncData::QUATERNION_MIN_RANGE,
            SyncData::QUATERNION_MAX_RANGE,
        );
        let c = SyncData::scale_ushort_to_float(
            c_scaled,
            0,
            SyncData::TEN_BITS_MAX,
            SyncData::QUATERNION_MIN_RANGE,
            SyncData::QUATERNION_MAX_RANGE,
        );

        // 计算省略的分量 d，基于 a² + b² + c² + d² = 1
        let d_squared = 1.0 - a * a - b * b - c * c;

        let d = if d_squared > 0.0 {
            d_squared.sqrt()
        } else {
            0.0
        };

        // 根据 largestIndex 重建四元数
        let v4 = match largest_index {
            0 => Vector4::new(d, a, b, c),
            1 => Vector4::new(a, d, b, c),
            2 => Vector4::new(a, b, d, c),
            _ => Vector4::new(a, b, c, d),
        };

        SyncData::quaternion_normalize_safe(v4)
    }

    /// 压缩四元数
    pub fn compress_quaternion(q: Quaternion<f32>) -> u32 {
        let v4 = Vector4::new(q.i, q.j, q.k, q.w);
        let (largest_index, _, mut without_largest) = Self::largest_absolute_component_index(v4);


        if q[largest_index] < 0f32 {
            without_largest = -without_largest;
        }

        // 缩放到 u16 范围
        let a_scaled = Self::scale_float_to_ushort(without_largest.x, SyncData::QUATERNION_MIN_RANGE, SyncData::QUATERNION_MAX_RANGE, 0, SyncData::TEN_BITS_MAX as u16);
        let b_scaled = Self::scale_float_to_ushort(without_largest.y, SyncData::QUATERNION_MIN_RANGE, SyncData::QUATERNION_MAX_RANGE, 0, SyncData::TEN_BITS_MAX as u16);
        let c_scaled = Self::scale_float_to_ushort(without_largest.z, SyncData::QUATERNION_MIN_RANGE, SyncData::QUATERNION_MAX_RANGE, 0, SyncData::TEN_BITS_MAX as u16);

        // 重建 u32 值
        (largest_index as u32) << 30 | (a_scaled as u32) << 20 | (b_scaled as u32) << 10 | c_scaled as u32
    }

    fn largest_absolute_component_index(value: Vector4<f32>) -> (usize, f32, Vector3<f32>) {
        let abs = value.map(|x| x.abs());
        let mut largest_abs = abs.x;
        let mut without_largest = Vector3::new(abs.y, abs.z, abs.w);
        let mut largest_index = 0;

        if abs.y > largest_abs {
            largest_index = 1;
            largest_abs = abs.y;
            without_largest = Vector3::new(abs.x, abs.z, abs.w);
        }
        if abs.z > largest_abs {
            largest_index = 2;
            largest_abs = abs.z;
            without_largest = Vector3::new(abs.x, abs.y, abs.w);
        }
        if abs.w > largest_abs {
            largest_index = 3;
            largest_abs = abs.w;
            without_largest = Vector3::new(abs.x, abs.y, abs.z);
        }
        (largest_index, largest_abs, without_largest)
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
                quaternion = SyncData::decompress_quaternion(reader.read_uint());
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
                write.write_uint(SyncData::compress_quaternion(self.quat_rotation));
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_data() {
        let quat = Quaternion::new(4.0, 1.0, 2.0, 3.0);
        let compress_quaternion = SyncData::compress_quaternion(quat);
        println!("compress_quaternion: {}", compress_quaternion);
        let decompress_quaternion = SyncData::decompress_quaternion(compress_quaternion);
        println!("decompress_quaternion: {:?}", decompress_quaternion);
    }
}