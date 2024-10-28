use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait, Writeable};
use nalgebra::{Quaternion, Vector2, Vector3, Vector4};
use tklog::error;

pub struct NetworkWriterExtensions;

impl NetworkWriterTrait for NetworkWriterExtensions {
    fn write_byte(writer: &mut NetworkWriter, value: u8) {
        writer.write_blittable(value);
    }
    fn write_byte_nullable(writer: &mut NetworkWriter, value: Option<u8>) {
        writer.write_blittable_nullable(value);
    }

    fn write_sbyte(writer: &mut NetworkWriter, value: i8) {
        writer.write_blittable(value);
    }
    fn write_sbyte_nullable(writer: &mut NetworkWriter, value: Option<i8>) {
        writer.write_blittable_nullable(value);
    }

    fn write_char(writer: &mut NetworkWriter, value: char) {
        writer.write_blittable(value as u16);
    }
    fn write_char_nullable(writer: &mut NetworkWriter, value: Option<char>) {
        match value {
            Some(v) => writer.write_blittable(v as u16),
            None => writer.write_blittable(0u16),
        }
    }

    fn write_bool(writer: &mut NetworkWriter, value: bool) {
        writer.write_blittable(value as u8);
    }
    fn write_bool_nullable(writer: &mut NetworkWriter, value: Option<bool>) {
        match value {
            Some(v) => writer.write_blittable(v as u8),
            None => writer.write_blittable(0u8),
        }
    }

    fn write_short(writer: &mut NetworkWriter, value: i16) {
        writer.write_blittable(value);
    }
    fn write_short_nullable(writer: &mut NetworkWriter, value: Option<i16>) {
        writer.write_blittable_nullable(value);
    }

    fn write_ushort(writer: &mut NetworkWriter, value: u16) {
        writer.write_blittable(value);
    }
    fn write_ushort_nullable(writer: &mut NetworkWriter, value: Option<u16>) {
        writer.write_blittable_nullable(value);
    }

    fn write_int(writer: &mut NetworkWriter, value: i32) {
        writer.write_blittable(value);
    }
    fn write_int_nullable(writer: &mut NetworkWriter, value: Option<i32>) {
        writer.write_blittable_nullable(value);
    }

    fn write_uint(writer: &mut NetworkWriter, value: u32) {
        writer.write_blittable(value);
    }
    fn write_uint_nullable(writer: &mut NetworkWriter, value: Option<u32>) {
        writer.write_blittable_nullable(value);
    }

    fn write_long(writer: &mut NetworkWriter, value: i64) {
        writer.write_blittable(value);
    }
    fn write_long_nullable(writer: &mut NetworkWriter, value: Option<i64>) {
        writer.write_blittable_nullable(value);
    }

    fn write_ulong(writer: &mut NetworkWriter, value: u64) {
        writer.write_blittable(value);
    }
    fn write_ulong_nullable(writer: &mut NetworkWriter, value: Option<u64>) {
        writer.write_blittable_nullable(value);
    }

    fn write_float(writer: &mut NetworkWriter, value: f32) {
        writer.write_blittable(value);
    }
    fn write_float_nullable(writer: &mut NetworkWriter, value: Option<f32>) {
        writer.write_blittable_nullable(value);
    }

    fn write_double(writer: &mut NetworkWriter, value: f64) {
        writer.write_blittable(value);
    }
    fn write_double_nullable(writer: &mut NetworkWriter, value: Option<f64>) {
        writer.write_blittable_nullable(value);
    }

    fn write_str(writer: &mut NetworkWriter, value: &str) {
        writer.write(value);
    }
    fn write_string(writer: &mut NetworkWriter, value: String) {
        writer.write(value);
    }

    fn write_bytes_and_size(writer: &mut NetworkWriter, value: &[u8], offset: usize, count: usize) {
        if value.len() == 0 {
            writer.write_blittable(0u16);
            return;
        }
        Self::write_uint(writer, 1 + count as u32);
        writer.write_bytes(value, offset, count);
    }

    fn write_vector2(writer: &mut NetworkWriter, value: Vector2<f32>) {
        writer.write_blittable(value.x);
        writer.write_blittable(value.y);
    }

    fn write_vector2_nullable(writer: &mut NetworkWriter, value: Option<Vector2<f32>>) {
        value.map(|v| {
            writer.write_blittable(v.x);
            writer.write_blittable(v.y);
        });
    }

    fn write_vector3(writer: &mut NetworkWriter, value: Vector3<f32>) {
        writer.write_blittable(value.x);
        writer.write_blittable(value.y);
        writer.write_blittable(value.z);
    }

    fn write_vector3_nullable(writer: &mut NetworkWriter, value: Option<Vector3<f32>>) {
        value.map(|v| {
            writer.write_blittable(v.x);
            writer.write_blittable(v.y);
            writer.write_blittable(v.z);
        });
    }

    fn write_vector4(writer: &mut NetworkWriter, value: Vector4<f32>) {
        writer.write_blittable(value.x);
        writer.write_blittable(value.y);
        writer.write_blittable(value.z);
        writer.write_blittable(value.w);
    }

    fn write_vector4_nullable(writer: &mut NetworkWriter, value: Option<Vector4<f32>>) {
        value.map(|v| {
            writer.write_blittable(v.x);
            writer.write_blittable(v.y);
            writer.write_blittable(v.z);
            writer.write_blittable(v.w);
        });
    }

    fn write_quaternion(writer: &mut NetworkWriter, value: Quaternion<f32>) {
        writer.write_blittable(value.coords.x);
        writer.write_blittable(value.coords.y);
        writer.write_blittable(value.coords.z);
        writer.write_blittable(value.coords.w);
    }

    fn write_quaternion_nullable(writer: &mut NetworkWriter, value: Option<Quaternion<f32>>) {
        value.map(|v| {
            writer.write_blittable(v.coords.x);
            writer.write_blittable(v.coords.y);
            writer.write_blittable(v.coords.z);
            writer.write_blittable(v.coords.w);
        });
    }
}

impl NetworkWriterExtensions {
    fn write_string<S: AsRef<str>>(writer: &mut NetworkWriter, value: S) {
        let bytes = value.as_ref().as_bytes();
        let length = bytes.len();
        if length > NetworkWriter::MAX_STRING_LENGTH - writer.position {
            error!("String length exceeds maximum length of {}", NetworkWriter::MAX_STRING_LENGTH - writer.position);
        }
        writer.write_blittable(1 + length as u16);
        writer.write_bytes_all(bytes);
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
        Some(|writer, value| NetworkWriterExtensions::write_string(writer, value))
    }
}