use crate::mirror::core::network_reader::{NetworkReader, NetworkReaderTrait, Readable};
use half::f16;
use nalgebra::{Quaternion, Vector2, Vector3, Vector4};
use rust_decimal::Decimal;
use tklog::warn;

pub struct NetworkReaderExtensions;
impl NetworkReaderExtensions {
    fn read_string(reader: &mut NetworkReader) -> String {
        let length = reader.read_ushort() as usize - 1;
        let bytes = reader.read_bytes(length);
        if let Ok(string) = String::from_utf8(bytes) {
            string
        } else {
            warn!("NetworkReaderExtensions::read_string() failed to convert bytes to string");
            String::new()
        }
    }
}
impl NetworkReaderTrait for NetworkReader {
    fn read_byte(&mut self) -> u8 {
        self.read_blittable::<u8>()
    }

    fn read_byte_nullable(&mut self) -> Option<u8> {
        self.read_blittable_nullable::<u8>()
    }

    fn read_sbyte(&mut self) -> i8 {
        self.read_blittable::<i8>()
    }

    fn read_sbyte_nullable(&mut self) -> Option<i8> {
        self.read_blittable_nullable::<i8>()
    }

    fn read_char(&mut self) -> char {
        self.read_blittable::<char>()
    }

    fn read_char_nullable(&mut self) -> Option<char> {
        self.read_blittable_nullable::<char>()
    }

    fn read_bool(&mut self) -> bool {
        self.read_blittable::<bool>()
    }

    fn read_bool_nullable(&mut self) -> Option<bool> {
        self.read_blittable_nullable::<bool>()
    }

    fn read_short(&mut self) -> i16 {
        self.read_blittable::<i16>()
    }

    fn read_short_nullable(&mut self) -> Option<i16> {
        self.read_blittable_nullable::<i16>()
    }

    fn read_ushort(&mut self) -> u16 {
        self.read_blittable::<u16>()
    }

    fn read_ushort_nullable(&mut self) -> Option<u16> {
        self.read_blittable_nullable::<u16>()
    }

    fn read_int(&mut self) -> i32 {
        self.read_blittable::<i32>()
    }

    fn read_int_nullable(&mut self) -> Option<i32> {
        self.read_blittable_nullable::<i32>()
    }

    fn read_uint(&mut self) -> u32 {
        self.read_blittable::<u32>()
    }

    fn read_uint_nullable(&mut self) -> Option<u32> {
        self.read_blittable_nullable::<u32>()
    }

    fn read_long(&mut self) -> i64 {
        self.read_blittable::<i64>()
    }

    fn read_long_nullable(&mut self) -> Option<i64> {
        self.read_blittable_nullable::<i64>()
    }

    fn read_ulong(&mut self) -> u64 {
        self.read_blittable::<u64>()
    }

    fn read_ulong_nullable(&mut self) -> Option<u64> {
        self.read_blittable_nullable::<u64>()
    }

    fn read_float(&mut self) -> f32 {
        self.read_blittable::<f32>()
    }

    fn read_float_nullable(&mut self) -> Option<f32> {
        self.read_blittable_nullable::<f32>()
    }

    fn read_double(&mut self) -> f64 {
        self.read_blittable::<f64>()
    }

    fn read_double_nullable(&mut self) -> Option<f64> {
        self.read_blittable_nullable::<f64>()
    }

    fn read_string(&mut self) -> String {
        self.read()
    }

    fn read_var_int(&mut self) -> i32 {
        self.decompress_var_int() as i32
    }

    fn read_var_uint(&mut self) -> u32 {
        self.decompress_var_uint() as u32
    }

    fn read_var_long(&mut self) -> i64 {
        self.decompress_var_int()
    }

    fn read_var_ulong(&mut self) -> u64 {
        self.decompress_var_uint()
    }

    fn read_decimal(&mut self) -> Decimal {
        self.read_blittable::<Decimal>()
    }

    fn read_decimal_nullable(&mut self) -> Option<Decimal> {
        self.read_blittable_nullable::<Decimal>()
    }

    fn read_half(&mut self) -> f16 {
        f16::from_bits(self.read_ushort())
    }

    fn read_bytes_and_size(&mut self) -> Vec<u8> {
        let count = self.read_uint() as usize;
        if count == 0 {
            return Vec::new();
        }
        self.read_bytes(count - 1)
    }

    fn read_array_segment_and_size(&mut self) -> &[u8] {
        let count = self.read_uint() as usize;
        if count == 0 {
            return &[];
        }
        self.read_array_segment(count - 1)
    }

    fn read_vector2(&mut self) -> Vector2<f32> {
        let x = self.read_float();
        let y = self.read_float();
        Vector2::new(x, y)
    }

    fn read_vector2_nullable(&mut self) -> Option<Vector2<f32>> {
        let has_value = self.read_bool();
        if has_value {
            Some(self.read_vector2())
        } else {
            None
        }
    }

    fn read_vector3(&mut self) -> Vector3<f32> {
        let x = self.read_float();
        let y = self.read_float();
        let z = self.read_float();
        Vector3::new(x, y, z)
    }

    fn read_vector3_nullable(&mut self) -> Option<Vector3<f32>> {
        let has_value = self.read_bool();
        if has_value {
            Some(self.read_vector3())
        } else {
            None
        }
    }

    fn read_vector4(&mut self) -> Vector4<f32> {
        let x = self.read_float();
        let y = self.read_float();
        let z = self.read_float();
        let w = self.read_float();
        Vector4::new(x, y, z, w)
    }

    fn read_vector4_nullable(&mut self) -> Option<Vector4<f32>> {
        let has_value = self.read_bool();
        if has_value {
            Some(self.read_vector4())
        } else {
            None
        }
    }

    fn read_quaternion(&mut self) -> Quaternion<f32> {
        let x = self.read_float();
        let y = self.read_float();
        let z = self.read_float();
        let w = self.read_float();
        Quaternion::new(w, x, y, z)
    }

    fn read_quaternion_nullable(&mut self) -> Option<Quaternion<f32>> {
        let has_value = self.read_bool();
        if has_value {
            Some(self.read_quaternion())
        } else {
            None
        }
    }

    fn decompress_var_int(&mut self) -> i64 {
        let data = self.decompress_var_uint() as i64;
        (data >> 1) ^ -(data & 1)
    }

    fn decompress_var_uint(&mut self) -> u64 {
        let a0 = self.read_byte() as u64;
        if a0 < 241 {
            return a0;
        }

        let a1 = self.read_byte() as u64;
        if a0 <= 248 {
            return 240 + ((a0 - 241) << 8) + a1;
        }

        let a2 = self.read_byte() as u64;
        if a0 == 249 {
            return 2288 + (a1 << 8) + a2;
        }

        let a3 = self.read_byte() as u64;
        if a0 == 250 {
            return a1 + (a2 << 8) + (a3 << 16);
        }

        let a4 = self.read_byte() as u64;
        if a0 == 251 {
            return a1 + (a2 << 8) + (a3 << 16) + (a4 << 24);
        }

        let a5 = self.read_byte() as u64;
        if a0 == 252 {
            return a1 + (a2 << 8) + (a3 << 16) + (a4 << 24) + (a5 << 32);
        }

        let a6 = self.read_byte() as u64;
        if a0 == 253 {
            return a1 + (a2 << 8) + (a3 << 16) + (a4 << 24) + (a5 << 32) + (a6 << 40);
        }

        let a7 = self.read_byte() as u64;
        if a0 == 254 {
            return a1 + (a2 << 8) + (a3 << 16) + (a4 << 24) + (a5 << 32) + (a6 << 40) + (a7 << 48);
        }

        let a8 = self.read_byte() as u64;
        if a0 == 255 {
            return a1 + (a2 << 8) + (a3 << 16) + (a4 << 24) + (a5 << 32) + (a6 << 40) + (a7 << 48) + (a8 << 56);
        }
        panic!("DecompressVarUInt failure: {}", a0);
    }
}

impl Readable for String {
    type TYPE = String;

    fn get_reader() -> Option<fn(&mut NetworkReader) -> Self::TYPE>
    where
        Self: Sized,
    {
        Some(|reader: &mut NetworkReader| -> Self::TYPE {
            NetworkReaderExtensions::read_string(reader)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mirror::core::network_reader::NetworkReader;
    use crate::mirror::core::network_writer::{NetworkWriter, NetworkWriterTrait};

    #[test]
    fn read_string() {
        let mut writer = NetworkWriter::new();
        writer.write_string("Hello, World!".to_string());
        let mut reader = NetworkReader::new_with_bytes(writer.to_bytes());
        let value = reader.read_string();
        assert_eq!(value, "Hello, World!");
    }
}