use std::{fmt, ptr};
use tklog::error;

pub struct NetworkWriter {
    pub data: Vec<u8>,
    pub position: usize,
}

impl NetworkWriter {
    // the limit of ushort is so we can write string size prefix as only 2 bytes.
    // -1 so we can still encode 'null' into it too.
    pub const MAX_STRING_LENGTH: usize = u16::MAX as usize - 1;
    // create writer immediately with it's own buffer so no one can mess with it and so that we can resize it.
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
    pub fn get_data(&self) -> Vec<u8> {
        self.data[..self.position].to_vec()
    }
    pub fn get_position(&self) -> usize {
        self.position
    }
    pub fn set_position(&mut self, value: usize) {
        self.position = value;
    }
    pub fn to_array_segment(&self) -> &[u8] {
        &self.data[..self.position]
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

        // Update the position
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
    pub fn write_byte(&mut self, value: u8) {
        self.ensure_capacity(self.position + 1);
        self.data[self.position] = value;
        self.position += 1;
    }
    pub fn write_bytes(&mut self, value: &[u8], offset: usize, count: usize) {
        self.ensure_capacity(self.position + count);
        self.data[self.position..self.position + count].copy_from_slice(&value[offset..offset + count]);
        self.position += count;
    }
    pub fn write_bytes_all(&mut self, value: &[u8]) {
        self.write_bytes(value, 0, value.len());
    }
    pub fn write<T: Writeable>(&mut self, value: T) {
        if let Some(write_fn) = T::get_writer() {
            write_fn(self, value);
        } else {
            error!("No writer found for type {}", std::any::type_name::<T>());
        }
    }
    fn write_string<S: AsRef<str>>(writer: &mut Self, value: S) {
        let bytes = value.as_ref().as_bytes();
        let length = bytes.len();
        if length > Self::MAX_STRING_LENGTH - writer.position {
            error!("String length exceeds maximum length of {}", Self::MAX_STRING_LENGTH - writer.position);
        }
        writer.write_blittable(1 + length as u16);
        writer.write_bytes_all(bytes);
    }
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


impl Writeable for String {
    fn get_writer() -> Option<fn(&mut NetworkWriter, Self)>
    where
        Self: Sized,
    {
        Some(|writer, value| NetworkWriter::write_string(writer, value))
    }
}

impl Writeable for &str {
    fn get_writer() -> Option<fn(&mut NetworkWriter, Self)>
    where
        Self: Sized,
    {
        Some(|writer, value| NetworkWriter::write_string(writer, value))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::batcher::Batch;

    #[test]
    fn test_write_blittable() {
        let mut writer = NetworkWriter::new();
        writer.write_blittable(42u32);
        writer.write_blittable(3u8);
        writer.write_blittable(true);
        writer.write("Hello, world!");
        let data = writer.get_data();
        println!("{}", writer);

        let mut batch = Batch::new();
        batch.write_u32_le(42);
        batch.write_u8(3);
        batch.write_bool(true);
        batch.write_string_le("Hello, world!");

        println!("{:?}", data);
        println!("{:?}", batch.get_bytes().to_vec());
    }
}