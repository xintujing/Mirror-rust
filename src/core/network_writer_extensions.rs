use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait, Writeable};
use half::f16;
use nalgebra::{Quaternion, Vector2, Vector3, Vector4};
use rust_decimal::Decimal;
use tklog::error;

pub struct NetworkWriterExtensions;

impl NetworkWriterExtensions {
    fn write_string<S: AsRef<str>>(writer: &mut NetworkWriter, value: S) {
        let bytes = value.as_ref().as_bytes();
        let length = bytes.len();
        if length > NetworkWriter::MAX_STRING_LENGTH - writer.get_position() {
            error!("String length exceeds maximum length of {}", NetworkWriter::MAX_STRING_LENGTH - writer.get_position());
        }
        writer.write_blittable(1 + length as u16);
        writer.write_array_segment_all(bytes);
    }
}

impl NetworkWriterTrait for NetworkWriter {
    fn write_byte(&mut self, value: u8) {
        self.write_blittable(value);
    }
    fn write_byte_nullable(&mut self, value: Option<u8>) {
        self.write_blittable_nullable(value);
    }

    fn write_sbyte(&mut self, value: i8) {
        self.write_blittable(value);
    }
    fn write_sbyte_nullable(&mut self, value: Option<i8>) {
        self.write_blittable_nullable(value);
    }

    fn write_char(&mut self, value: char) {
        self.write_blittable(value as u16);
    }
    fn write_char_nullable(&mut self, value: Option<char>) {
        match value {
            Some(v) => self.write_blittable(v as u16),
            None => self.write_blittable(0u16),
        }
    }

    fn write_bool(&mut self, value: bool) {
        self.write_blittable(value as u8);
    }
    fn write_bool_nullable(&mut self, value: Option<bool>) {
        match value {
            Some(v) => self.write_blittable(v as u8),
            None => self.write_blittable(0u8),
        }
    }

    fn write_short(&mut self, value: i16) {
        self.write_blittable(value);
    }
    fn write_short_nullable(&mut self, value: Option<i16>) {
        self.write_blittable_nullable(value);
    }

    fn write_ushort(&mut self, value: u16) {
        self.write_blittable(value);
    }
    fn write_ushort_nullable(&mut self, value: Option<u16>) {
        self.write_blittable_nullable(value);
    }

    fn write_int(&mut self, value: i32) {
        self.write_blittable(value);
    }
    fn write_int_nullable(&mut self, value: Option<i32>) {
        self.write_blittable_nullable(value);
    }

    fn write_uint(&mut self, value: u32) {
        self.write_blittable(value);
    }
    fn write_uint_nullable(&mut self, value: Option<u32>) {
        self.write_blittable_nullable(value);
    }

    fn write_long(&mut self, value: i64) {
        self.write_blittable(value);
    }
    fn write_long_nullable(&mut self, value: Option<i64>) {
        self.write_blittable_nullable(value);
    }

    fn write_ulong(&mut self, value: u64) {
        self.write_blittable(value);
    }
    fn write_ulong_nullable(&mut self, value: Option<u64>) {
        self.write_blittable_nullable(value);
    }

    fn write_float(&mut self, value: f32) {
        self.write_blittable(value);
    }
    fn write_float_nullable(&mut self, value: Option<f32>) {
        self.write_blittable_nullable(value);
    }

    fn write_double(&mut self, value: f64) {
        self.write_blittable(value);
    }
    fn write_double_nullable(&mut self, value: Option<f64>) {
        self.write_blittable_nullable(value);
    }

    fn write_decimal(&mut self, value: Decimal) {
        self.write_blittable(value);
    }

    fn write_decimal_nullable(&mut self, value: Option<Decimal>) {
        self.write_blittable_nullable(value);
    }

    fn write_half(&mut self, value: f16) {
        self.write_ushort(value.to_bits());
    }

    fn write_str(&mut self, value: &str) {
        self.write(value);
    }
    fn write_string(&mut self, value: String) {
        self.write(value);
    }

    fn write_bytes_and_size(&mut self, value: Vec<u8>, offset: usize, count: usize) {
        if value.len() == 0 {
            self.compress_var_uint(0);
            return;
        }
        self.compress_var_uint(1 + count as u64);
        self.write_bytes(value, offset, count);
    }

    fn write_array_segment_and_size(&mut self, value: &[u8], offset: usize, count: usize) {
        self.compress_var_uint(1 + count as u64);
        self.write_array_segment(value, offset, count);
    }

    fn write_vector2(&mut self, value: Vector2<f32>) {
        self.write_blittable(value.data);
    }

    fn write_vector2_nullable(&mut self, value: Option<Vector2<f32>>) {
        if let Some(v) = value {
            self.write_blittable(v.data);
        } else {
            self.write_byte(0);
        }
    }

    fn write_vector3(&mut self, value: Vector3<f32>) {
        self.write_blittable(value.data);
    }

    fn write_vector3_nullable(&mut self, value: Option<Vector3<f32>>) {
        if let Some(v) = value {
            self.write_blittable(v.data);
        } else {
            self.write_byte(0);
        }
    }

    fn write_vector4(&mut self, value: Vector4<f32>) {
        self.write_blittable(value.data);
    }

    fn write_vector4_nullable(&mut self, value: Option<Vector4<f32>>) {
        if let Some(v) = value {
            self.write_blittable(v.data)
        } else {
            self.write_byte(0);
        }
    }

    fn write_quaternion(&mut self, value: Quaternion<f32>) {
        self.write_blittable(value.coords.data);
    }

    fn write_quaternion_nullable(&mut self, value: Option<Quaternion<f32>>) {
        if let Some(v) = value {
            self.write_blittable(v.coords.data);
        } else {
            self.write_byte(0);
        }
    }

    fn compress_var_int(&mut self, value: i64) {
        let zigzagged = ((value >> 63) ^ (value << 1)) as u64;
        self.compress_var_uint(zigzagged);
    }

    fn compress_var_uint(&mut self, value: u64) {
        if value <= 240 {
            self.write_byte(value as u8);
            return;
        }
        if value <= 2287 {
            let a = ((value - 240) >> 8) as u16 + 241;
            let b = (value - 240) as u16;
            self.write_ushort((b << 8u16) | a);
            return;
        }
        if value <= 67823 {
            let a = 249;
            let b = ((value - 2288) >> 8) as u16;
            let c = (value - 2288) as u16;
            self.write_byte(a);
            self.write_ushort((c << 8u16) | b);
            return;
        }
        if value <= 16777215 {
            let a = 250;
            let b = (value << 8) as u32;
            self.write_uint(b | a);
            return;
        }
        if value <= 4294967295 {
            let a = 251;
            let b = value as u32;
            self.write_byte(a);
            self.write_uint(b);
            return;
        }
        if value <= 1099511627775 {
            let a = 252;
            let b = (value & 0xFF) as u16;
            let c = (value >> 8) as u32;
            self.write_ushort(b << 8 | a);
            self.write_uint(c);
            return;
        }
        if value <= 281474976710655 {
            let a = 253;
            let b = (value & 0xFF) as u16;
            let c = ((value >> 8) & 0xFF) as u16;
            let d = (value >> 16) as u32;
            self.write_byte(a);
            self.write_ushort(c << 8 | b);
            self.write_uint(d);
            return;
        }
        if value <= 72057594037927935 {
            let a = 254u64;
            let b = value << 8;
            self.write_ulong(b | a);
            return;
        }

        // all others
        {
            self.write_byte(255);
            self.write_ulong(value);
        }
    }
}

impl Writeable for String {
    fn get_writer() -> Option<fn(&mut NetworkWriter, Self)>
    where
        Self: Sized,
    {
        Some(|writer, value| NetworkWriterExtensions::write_string(writer, value))
    }
}

impl Writeable for &str {
    fn get_writer() -> Option<fn(&mut NetworkWriter, Self)>
    where
        Self: Sized,
    {
        Some(|writer, value| NetworkWriterExtensions::write_string(writer, value.to_string()))
    }
}
