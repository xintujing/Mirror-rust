use half::f16;
use nalgebra::{Quaternion, Vector2, Vector3, Vector4};
use rust_decimal::Decimal;
use std::any::Any;
use std::fmt;
use tklog::warn;

pub struct NetworkReader {
    data: Vec<u8>,
    position: usize,
}

impl NetworkReader {
    pub const ALLOCATION_LIMIT: usize = 1024 * 1024 * 16;

    pub fn new() -> Self {
        Self::new_with_bytes(Vec::new())
    }
    pub fn new_with_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            position: 0,
        }
    }
    pub fn reset(&mut self) {
        self.position = 0;
    }
    pub fn new_with_array_segment(data: &[u8]) -> Self {
        NetworkReader {
            data: data.to_vec(),
            position: 0,
        }
    }
    pub fn remaining(&self) -> usize {
        self.data.len() - self.position
    }
    pub fn capacity(&self) -> usize {
        self.data.len()
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data[..self.position].to_vec()
    }
    pub fn to_array_segment(&self) -> &[u8] {
        &self.data[..self.position]
    }
    pub fn get_position(&self) -> usize {
        self.position
    }
    pub fn set_position(&mut self, value: usize) {
        self.position = value;
    }
    pub fn set_bytes(&mut self, data: Vec<u8>) {
        self.data = data;
        self.position = 0;
    }
    pub fn set_array_segment(&mut self, data: &[u8]) {
        self.data = data.to_vec();
        self.position = 0;
    }
    pub fn read_blittable<T>(&mut self) -> T {
        let size = size_of::<T>();
        if self.remaining() < size {
            panic!("NetworkReader read_blittable not enough data to read");
        }
        let value = unsafe {
            let ptr = self.data.as_ptr().add(self.position) as *const T;
            ptr.read_unaligned()
        };
        self.position += size;
        value
    }
    pub fn read_blittable_nullable<T>(&mut self) -> Option<T> {
        let is_null = self.read_byte() == 0;
        if is_null {
            None
        } else {
            Some(self.read_blittable())
        }
    }
    pub fn read_bytes(&mut self, count: usize) -> Vec<u8> {
        if self.remaining() < count {
            warn!("Not enough data to read");
            return Vec::new();
        }
        let value = self.data[self.position..self.position + count].to_vec();
        self.position += count;
        value
    }
    pub fn read_remaining_bytes(&mut self) -> Vec<u8> {
        self.read_bytes(self.remaining())
    }
    pub fn read_array_segment(&mut self, count: usize) -> &[u8] {
        if self.remaining() < count {
            warn!("Not enough data to read");
            return &[];
        }
        let value = &self.data[self.position..self.position + count];
        self.position += count;
        value
    }
    pub fn read_remaining_array_segment(&mut self) -> &[u8] {
        self.read_array_segment(self.remaining())
    }
    pub fn read<T: Readable<TYPE=T>>(&mut self) -> T {
        if let Some(reader_fn) = T::get_reader() {
            reader_fn(self)
        } else {
            panic!("No reader found for type");
        }
    }
}

pub trait Readable {
    type TYPE;
    fn get_reader() -> Option<fn(&mut NetworkReader) -> Self::TYPE>
    where
        Self: Sized;
}

pub trait NetworkReaderTrait {
    fn read_byte(&mut self) -> u8;
    fn read_byte_nullable(&mut self) -> Option<u8>;

    fn read_sbyte(&mut self) -> i8;
    fn read_sbyte_nullable(&mut self) -> Option<i8>;

    fn read_char(&mut self) -> char;
    fn read_char_nullable(&mut self) -> Option<char>;

    fn read_bool(&mut self) -> bool;
    fn read_bool_nullable(&mut self) -> Option<bool>;

    fn read_short(&mut self) -> i16;
    fn read_short_nullable(&mut self) -> Option<i16>;

    fn read_ushort(&mut self) -> u16;
    fn read_ushort_nullable(&mut self) -> Option<u16>;

    fn read_int(&mut self) -> i32;
    fn read_int_nullable(&mut self) -> Option<i32>;

    fn read_uint(&mut self) -> u32;
    fn read_uint_nullable(&mut self) -> Option<u32>;

    fn read_long(&mut self) -> i64;
    fn read_long_nullable(&mut self) -> Option<i64>;

    fn read_ulong(&mut self) -> u64;
    fn read_ulong_nullable(&mut self) -> Option<u64>;

    fn read_float(&mut self) -> f32;
    fn read_float_nullable(&mut self) -> Option<f32>;

    fn read_double(&mut self) -> f64;
    fn read_double_nullable(&mut self) -> Option<f64>;

    fn read_string(&mut self) -> String;

    fn read_var_int(&mut self) -> i32;
    fn read_var_uint(&mut self) -> u32;
    fn read_var_long(&mut self) -> i64;
    fn read_var_ulong(&mut self) -> u64;

    fn read_decimal(&mut self) -> Decimal;
    fn read_decimal_nullable(&mut self) -> Option<Decimal>;

    fn read_half(&mut self) -> f16;

    fn read_bytes_and_size(&mut self) -> Vec<u8>;

    fn read_array_segment_and_size(&mut self) -> &[u8];

    fn read_vector2(&mut self) -> Vector2<f32>;
    fn read_vector2_nullable(&mut self) -> Option<Vector2<f32>>;

    fn read_vector3(&mut self) -> Vector3<f32>;
    fn read_vector3_nullable(&mut self) -> Option<Vector3<f32>>;

    fn read_vector4(&mut self) -> Vector4<f32>;
    fn read_vector4_nullable(&mut self) -> Option<Vector4<f32>>;

    fn read_quaternion(&mut self) -> Quaternion<f32>;
    fn read_quaternion_nullable(&mut self) -> Option<Quaternion<f32>>;

    fn decompress_var_int(&mut self) -> i64;
    fn decompress_var_uint(&mut self) -> u64;
}

impl fmt::Display for NetworkReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_string = self.data.iter().map(|byte| format!("{:02X}", byte)).collect::<String>();
        write!(f, "[{} @ {}/{}]", hex_string, self.position, self.capacity())
    }
}