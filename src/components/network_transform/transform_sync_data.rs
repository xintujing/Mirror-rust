use crate::core::batcher::{Batch, DataReader, DataWriter, UnBatch};
use nalgebra::{Quaternion, Vector3, Vector4};
use std::fmt::Debug;
use std::io;

#[derive(Clone, Copy, PartialEq)]
pub struct SyncData {
    // 改变的数据
    pub changed_data_bytes: u8,
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
    const TEN_BITS_MAX: u32 = 0x3FF; // 10 bits max value: 1023
    const QUATERNION_MIN_RANGE: f32 = -1.0;
    const QUATERNION_MAX_RANGE: f32 = 1.0;

    #[allow(dead_code)]
    pub fn new(changed_data_bytes: u8, position: Vector3<f32>, quat_rotation: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        Self {
            changed_data_bytes,
            position,
            quat_rotation,
            vec_rotation: Self::quaternion_to_euler_angles(quat_rotation),
            scale,
        }
    }

    pub fn get_changes(&self) -> Vec<Changed> {
        Changed::changes_from_u8(self.changed_data_bytes)
    }

    /// 安全地规范化四元数，即使输入包含无效值（如 NaN）
    fn quaternion_normalize_safe(q: Quaternion<f32>) -> Quaternion<f32> {
        let norm = q.norm();
        if norm.is_finite() && norm > 0.0 {
            q / norm
        } else {
            Quaternion::identity()
        }
    }

    /// 将 `u16` 值缩放到指定的浮点范围
    fn scale_u16_to_float(
        value: u16,
        input_min: u32,
        input_max: u32,
        output_min: f32,
        output_max: f32,
    ) -> f32 {
        let normalized = (value as f32 - input_min as f32) / (input_max - input_min) as f32;
        output_min + normalized * (output_max - output_min)
    }

    /// 解压缩四元数
    pub fn decompress_quaternion(v: u32) -> Quaternion<f32> {
        // 获取 cScaled（位 0..10）
        let c_scaled = (v & SyncData::TEN_BITS_MAX) as u16;

        // 获取 bScaled（位 10..20）
        let b_scaled = ((v >> 10) & SyncData::TEN_BITS_MAX) as u16;

        // 获取 aScaled（位 20..30）
        let a_scaled = ((v >> 20) & SyncData::TEN_BITS_MAX) as u16;

        // 获取 largestIndex（位 30..32）
        let largest_index = (v >> 30) as usize;

        // 缩放回浮点数
        let a = SyncData::scale_u16_to_float(
            a_scaled,
            0,
            SyncData::TEN_BITS_MAX,
            SyncData::QUATERNION_MIN_RANGE,
            SyncData::QUATERNION_MAX_RANGE,
        );
        let b = SyncData::scale_u16_to_float(
            b_scaled,
            0,
            SyncData::TEN_BITS_MAX,
            SyncData::QUATERNION_MIN_RANGE,
            SyncData::QUATERNION_MAX_RANGE,
        );
        let c = SyncData::scale_u16_to_float(
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

        // 创建四元数并安全地规范化
        let quaternion = Quaternion::new(v4.w, v4.x, v4.y, v4.z); // nalgebra 的 Quaternion 顺序为 (w, x, y, z)
        SyncData::quaternion_normalize_safe(quaternion)
    }

    /// 压缩四元数
    pub fn compress_quaternion(q: Quaternion<f32>) -> u32 {
        // 获取四元数的分量
        let (w, x, y, z) = (q.w, q.i, q.j, q.k);

        // 计算省略的分量
        let largest_index = if w.abs() > x.abs() {
            if w.abs() > y.abs() {
                if w.abs() > z.abs() {
                    0
                } else {
                    3
                }
            } else if y.abs() > z.abs() {
                1
            } else {
                3
            }
        } else if x.abs() > y.abs() {
            if x.abs() > z.abs() {
                1
            } else {
                3
            }
        } else if y.abs() > z.abs() {
            2
        } else {
            3
        };

        // 根据 largestIndex 重建四元数
        let (a, b, c) = match largest_index {
            0 => (w, x, y),
            1 => (x, w, y),
            2 => (x, y, w),
            _ => (x, y, z),
        };

        // 缩放到 u16 范围
        let a_scaled = ((a - SyncData::QUATERNION_MIN_RANGE) / (SyncData::QUATERNION_MAX_RANGE - SyncData::QUATERNION_MIN_RANGE) * SyncData::TEN_BITS_MAX as f32) as u16;
        let b_scaled = ((b - SyncData::QUATERNION_MIN_RANGE) / (SyncData::QUATERNION_MAX_RANGE - SyncData::QUATERNION_MIN_RANGE) * SyncData::TEN_BITS_MAX as f32) as u16;
        let c_scaled = ((c - SyncData::QUATERNION_MIN_RANGE) / (SyncData::QUATERNION_MAX_RANGE - SyncData::QUATERNION_MIN_RANGE) * SyncData::TEN_BITS_MAX as f32) as u16;

        // 重建 u32 值
        (largest_index as u32) << 30 | (a_scaled as u32) << 20 | (b_scaled as u32) << 10 | c_scaled as u32
    }

    pub fn quaternion_to_euler_angles(q: Quaternion<f32>) -> Vector3<f32> {
        let (x, y, z, w) = (q.i, q.j, q.k, q.w);

        let sinr_cosp = 2.0 * (w * x + y * z);
        let cosr_cosp = 1.0 - 2.0 * (x * x + y * y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        let sinp = 2.0 * (w * y - z * x);
        let pitch = if sinp.abs() >= 1.0 {
            std::f32::consts::FRAC_PI_2.copysign(sinp)
        } else {
            sinp.asin()
        };

        let siny_cosp = 2.0 * (w * z + x * y);
        let cosy_cosp = 1.0 - 2.0 * (y * y + z * z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        Vector3::new(roll, pitch, yaw)
    }
}

impl Debug for SyncData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SyncData {{ changed: {}, changes: {:?}, position: {:?}, quaternion: {:?}, vec_rotation: {:?}, scale: {:?} }}", self.changed_data_bytes, self.get_changes(), self.position, self.quat_rotation, self.vec_rotation, self.scale)
    }
}

impl DataReader for SyncData {
    fn deserialize(reader: &mut UnBatch) -> io::Result<Self> {

        // 改变的数据
        let changed = reader.read_u8()?;

        // 位置
        let mut position = Vector3::new(0.0, 0.0, 0.0);
        if (changed & Changed::PosX.to_u8()) > 0 {
            position.x = reader.read_f32_le()?;
        }
        if changed & Changed::PosY.to_u8() > 0 {
            position.y = reader.read_f32_le()?;
        }
        if changed & Changed::PosZ.to_u8() > 0 {
            position.z = reader.read_f32_le()?;
        }

        // 四元数
        let mut quaternion = Quaternion::identity();
        // 欧拉角
        let mut vec_rotation = Vector3::new(0.0, 0.0, 0.0);
        if (changed & Changed::CompressRot.to_u8()) > 0 {
            if (changed & Changed::RotX.to_u8()) > 0 {
                quaternion = SyncData::decompress_quaternion(reader.read_u32_le()?);
            }
        } else {
            if changed & Changed::RotX.to_u8() > 0 {
                vec_rotation.x = reader.read_f32_le()?;
            }
            if changed & Changed::RotY.to_u8() > 0 {
                vec_rotation.y = reader.read_f32_le()?;
            }
            if changed & Changed::RotZ.to_u8() > 0 {
                vec_rotation.z = reader.read_f32_le()?;
            }
        }

        // 缩放
        let mut scale = Vector3::new(0.0, 0.0, 0.0);
        if changed & Changed::Scale.to_u8() == Changed::Scale.to_u8() {
            scale.x = reader.read_f32_le()?;
            scale.y = reader.read_f32_le()?;
            scale.z = reader.read_f32_le()?;
        }

        Ok(Self {
            changed_data_bytes: changed,
            position,
            quat_rotation: quaternion,
            vec_rotation,
            scale,
        })
    }

    fn get_hash_code() -> u16 {
        0x0001
    }
}

impl DataWriter for SyncData {
    fn serialize(&mut self, batch: &mut Batch) {
        batch.write_u8(self.changed_data_bytes);

        // 位置
        if (self.changed_data_bytes & Changed::PosX.to_u8()) > 0 {
            batch.write_f32_le(self.position.x);
        }

        if (self.changed_data_bytes & Changed::PosY.to_u8()) > 0 {
            batch.write_f32_le(self.position.y);
        }

        if (self.changed_data_bytes & Changed::PosZ.to_u8()) > 0 {
            batch.write_f32_le(self.position.z);
        }

        // rotation
        if (self.changed_data_bytes & Changed::CompressRot.to_u8()) > 0 {
            batch.write_u32_le(SyncData::compress_quaternion(self.quat_rotation));
        } else {
            if (self.changed_data_bytes & Changed::RotX.to_u8()) > 0 {
                batch.write_f32_le(self.vec_rotation.x);
            }

            if (self.changed_data_bytes & Changed::RotY.to_u8()) > 0 {
                batch.write_f32_le(self.vec_rotation.y);
            }

            if (self.changed_data_bytes & Changed::RotZ.to_u8()) > 0 {
                batch.write_f32_le(self.vec_rotation.z);
            }
        }

        // 缩放
        if (self.changed_data_bytes & Changed::Scale.to_u8()) == Changed::Scale.to_u8() {
            batch.write_f32_le(self.scale.x);
            batch.write_f32_le(self.scale.y);
            batch.write_f32_le(self.scale.z);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Changed {
    None = 0,
    PosX = 1 << 0,
    PosY = 1 << 1,
    PosZ = 1 << 2,
    CompressRot = 1 << 3,
    RotX = 1 << 4,
    RotY = 1 << 5,
    RotZ = 1 << 6,
    Scale = 1 << 7,

    Pos = Changed::PosX as u8 | Changed::PosX as u8 | Changed::PosZ as u8,
    Rot = Changed::RotX as u8 | Changed::RotY as u8 | Changed::RotZ as u8,
}

impl Changed {
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }

    pub fn changes_from_u8(byte: u8) -> Vec<Changed> {
        let mut changeds = Vec::new();
        // PosX
        if byte & Changed::PosX.to_u8() > 0 {
            changeds.push(Changed::PosX);
        }
        // PosY
        if byte & Changed::PosY.to_u8() > 0 {
            changeds.push(Changed::PosY);
        }
        // PosZ
        if byte & Changed::PosZ.to_u8() > 0 {
            changeds.push(Changed::PosZ);
        }
        // CompressRot
        if byte & Changed::CompressRot.to_u8() > 0 {
            changeds.push(Changed::CompressRot);
        }
        // RotX
        if byte & Changed::RotX.to_u8() > 0 {
            changeds.push(Changed::RotX);
        }
        // RotY
        if byte & Changed::RotY.to_u8() > 0 {
            changeds.push(Changed::RotY);
        }
        // RotZ
        if byte & Changed::RotZ.to_u8() > 0 {
            changeds.push(Changed::RotZ);
        }
        // Scale
        if byte & Changed::Scale.to_u8() > 0 {
            changeds.push(Changed::Scale);
        }
        // Pos
        if (byte & Changed::PosX.to_u8() > 0)
            && (byte & Changed::PosY.to_u8()) > 0
            && (byte & Changed::PosZ.to_u8()) > 0
        {
            changeds.push(Changed::Pos);
        }
        // Rot
        if (byte & Changed::RotX.to_u8() > 0)
            && (byte & Changed::RotY.to_u8() > 0)
            && (byte & Changed::RotZ.to_u8()) > 0
        {
            changeds.push(Changed::Rot);
        }
        changeds
    }
}
