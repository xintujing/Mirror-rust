use byteorder::{LittleEndian, ReadBytesExt};
use std::error::Error;
use std::fmt;
use std::io::{self, Read};

pub struct NetworkReader<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> NetworkReader<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        NetworkReader { buffer, position: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    pub fn set_buffer(&mut self, new_buffer: &'a [u8]) {
        self.buffer = new_buffer;
        self.position = 0;
    }

    // Read a type T that must be "Copy", meaning it doesn't require drop logic and is safe to copy around.
    pub fn read_blittable<T: Copy>(&mut self) -> io::Result<T> {
        let size_of_t = std::mem::size_of::<T>();
        if self.remaining() < size_of_t {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data in buffer"));
        }

        let value = unsafe {
            let ptr = self.buffer.as_ptr().add(self.position) as *const T;
            *ptr
        };
        self.position += size_of_t;
        Ok(value)
    }

    pub fn read_byte(&mut self) -> io::Result<u8> {
        self.read_blittable::<u8>()
    }

    pub fn read_bytes(&mut self, count: usize) -> io::Result<&[u8]> {
        if count > self.remaining() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data in buffer"));
        }
        let result = &self.buffer[self.position..self.position + count];
        self.position += count;
        Ok(result)
    }

    // Implement other read methods as needed, e.g., for integers, strings, etc.
}

impl fmt::Display for NetworkReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?} @ {}/{}]", self.buffer, self.position, self.capacity())
    }
}

// Example usage of read_blittable for a custom type that is blittable
#[derive(Debug, Copy, Clone)]
struct ExampleBlittable {
    data: i32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let data = [0x78, 0x56, 0x34, 0x12]; // Little-endian for 0x12345678
    let mut reader = NetworkReader::new(&data);
    let value: i32 = reader.read_blittable()?;
    println!("Value: {}", value);

    Ok(())
}
