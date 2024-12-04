use crate::{log_error, log_warn};
use half::f16;
use nalgebra::{Quaternion, Vector2, Vector3, Vector4};
use rust_decimal::Decimal;
use std::{fmt, ptr};

#[derive(Debug)]
pub struct NetworkWriter {
    data: Vec<u8>,
    position: usize,
}

impl NetworkWriter {
    // the limit of ushort is so we can write string size prefix as only 2 bytes.
    // -1 so we can still encode 'null' into it too.
    pub const MAX_STRING_LENGTH: usize = u16::MAX as usize - 1;
    // create writer immediately with its own buffer so no one can mess with it and so that we can resize it.
    // note: BinaryWriter allocates too much, so we only use a MemoryStream
    // => 1500 bytes by default because on average, most packets will be <= MTU
    pub const DEFAULT_CAPACITY: usize = 1500;
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(Self::DEFAULT_CAPACITY),
            position: 0,
        }
    }
    pub fn capacity(&self) -> usize {
        self.data.len()
    }
    pub fn reset(&mut self) {
        self.position = 0;
    }
    pub fn ensure_capacity(&mut self, size: usize) {
        let current_capacity = self.capacity();
        if current_capacity < size {
            let new_capacity = size.max(current_capacity * 2);
            self.data.resize(new_capacity, 0);
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data[..self.position].to_vec()
    }
    pub fn to_array_segment(&self) -> &[u8] {
        &self.data[..self.position]
    }
    pub fn position_sub(&mut self, value: usize) {
        if value > self.position {
            log_warn!("Cannot seek before the beginning of the buffer");
            return;
        }
        self.position -= value;
    }
    pub fn position_add(&mut self, value: usize) {
        if self.position + value > self.capacity() {
            log_warn!("Cannot seek past the end of the buffer");
            return;
        }
        self.position += value;
    }
    pub fn get_position(&self) -> usize {
        self.position
    }
    pub fn set_position(&mut self, value: usize) {
        self.position = value;
    }
    pub fn write_blittable<T: Copy>(&mut self, value: T) {
        // Check if the type is blittable (i.e., it has a defined layout)
        // In Rust, this is generally true for all Copy types, but we can add
        // more specific checks if needed.

        // Calculate the size of the type
        let size = size_of::<T>();

        // Ensure capacity
        self.ensure_capacity(self.position + size);

        // Write the blittable value
        unsafe {
            // Get a raw pointer to the buffer at the current position
            let ptr = self.data.as_mut_ptr().add(self.position) as *mut T;

            // Write the value to the buffer
            ptr::write(ptr, value);
        }

        // update the position
        self.position += size;
    }
    pub fn write_blittable_nullable<T: Copy>(&mut self, value: Option<T>) {
        // Write a boolean indicating whether the value is null
        self.write_byte(value.is_none() as u8);

        // If the value is not null, write the value
        if let Some(value) = value {
            self.write_blittable(value);
        }
    }
    pub fn write_bytes(&mut self, value: Vec<u8>, offset: usize, count: usize) {
        self.ensure_capacity(self.position + count);
        self.data[self.position..self.position + count].copy_from_slice(&value[offset..offset + count]);
        self.position += count;
    }
    pub fn write_bytes_all(&mut self, value: Vec<u8>) {
        let count = value.len();
        self.ensure_capacity(self.position + count);
        self.data[self.position..self.position + count].copy_from_slice(&value[..count]);
        self.position += count;
    }
    pub fn write_array_segment(&mut self, value: &[u8], offset: usize, count: usize) {
        self.ensure_capacity(self.position + count);
        self.data[self.position..self.position + count].copy_from_slice(&value[offset..offset + count]);
        self.position += count;
    }
    pub fn write_array_segment_all(&mut self, value: &[u8]) {
        let count = value.len();
        self.ensure_capacity(self.position + count);
        self.data[self.position..self.position + count].copy_from_slice(&value[..count]);
        self.position += count;
    }
    pub fn write<T: Writeable>(&mut self, value: T) {
        if let Some(write_fn) = T::get_writer() {
            write_fn(self, value);
        } else {
            log_error!("No writer found for type {}", std::any::type_name::<T>());
        }
    }
}

pub trait NetworkWriterTrait {
    fn write_byte(&mut self, value: u8);
    fn write_byte_nullable(&mut self, value: Option<u8>);

    fn write_sbyte(&mut self, value: i8);
    fn write_sbyte_nullable(&mut self, value: Option<i8>);

    fn write_char(&mut self, value: char);
    fn write_char_nullable(&mut self, value: Option<char>);

    fn write_bool(&mut self, value: bool);
    fn write_bool_nullable(&mut self, value: Option<bool>);

    fn write_short(&mut self, value: i16);
    fn write_short_nullable(&mut self, value: Option<i16>);

    fn write_ushort(&mut self, value: u16);
    fn write_ushort_nullable(&mut self, value: Option<u16>);

    fn write_int(&mut self, value: i32);
    fn write_int_nullable(&mut self, value: Option<i32>);

    fn write_uint(&mut self, value: u32);
    fn write_uint_nullable(&mut self, value: Option<u32>);

    fn write_long(&mut self, value: i64);
    fn write_long_nullable(&mut self, value: Option<i64>);

    fn write_ulong(&mut self, value: u64);
    fn write_ulong_nullable(&mut self, value: Option<u64>);

    fn write_float(&mut self, value: f32);
    fn write_float_nullable(&mut self, value: Option<f32>);

    fn write_double(&mut self, value: f64);
    fn write_double_nullable(&mut self, value: Option<f64>);

    fn write_decimal(&mut self, value: Decimal);
    fn write_decimal_nullable(&mut self, value: Option<Decimal>);
    fn write_half(&mut self, value: f16);

    fn write_str(&mut self, value: &str);
    fn write_string(&mut self, value: String);
    fn write_bytes_and_size(&mut self, value: Vec<u8>, offset: usize, count: usize);
    fn write_array_segment_and_size(&mut self, value: &[u8], offset: usize, count: usize);
    fn write_vector2(&mut self, value: Vector2<f32>);
    fn write_vector2_nullable(&mut self, value: Option<Vector2<f32>>);

    fn write_vector3(&mut self, value: Vector3<f32>);
    fn write_vector3_nullable(&mut self, value: Option<Vector3<f32>>);

    fn write_vector4(&mut self, value: Vector4<f32>);
    fn write_vector4_nullable(&mut self, value: Option<Vector4<f32>>);

    fn write_quaternion(&mut self, value: Quaternion<f32>);
    fn write_quaternion_nullable(&mut self, value: Option<Quaternion<f32>>);
    fn compress_var_int(&mut self, value: i32);
    fn compress_var_uint(&mut self, value: u32);
    fn compress_var_long(&mut self, value: i64);
    fn compress_var_ulong(&mut self, value: u64);
}

pub trait Writeable {
    fn get_writer() -> Option<fn(&mut NetworkWriter, Self)>
    where
        Self: Sized;
}

impl fmt::Display for NetworkWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_string = self.to_array_segment()
            .iter()
            .map(|byte| format!("{:02X}", byte))
            .collect::<String>();
        write!(f, "[{} @ {}/{}]", hex_string, self.position, self.capacity())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_blittable() {}


    #[test]
    fn test_pos_write() {
        let mut writer = NetworkWriter::new();
        writer.write_byte(1);
        println!("{} {:?}", writer.get_position(), writer.to_bytes());

        writer.set_position(0);
        writer.write_byte(2);
        println!("{} {:?}", writer.get_position(), writer.to_bytes());
    }
}