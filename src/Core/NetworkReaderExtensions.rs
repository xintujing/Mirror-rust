use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self};

pub struct NetworkReader<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> NetworkReader<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        NetworkReader { buffer, position: 0 }
    }

    pub fn read_blittable<T: Copy + Default>(&mut self) -> io::Result<T> {
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

    pub fn read_bytes(&mut self, count: usize) -> io::Result<&[u8]> {
        if self.remaining() < count {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data in buffer"));
        }
        let result = &self.buffer[self.position..self.position + count];
        self.position += count;
        Ok(result)
    }

    fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }
}

pub trait NetworkReaderExtensions {
    fn read_byte(&mut self) -> io::Result<u8>;
    fn read_int(&mut self) -> io::Result<i32>;
    fn read_uint(&mut self) -> io::Result<u32>;
    fn read_float(&mut self) -> io::Result<f32>;
    fn read_string(&mut self) -> io::Result<String>;
    // More functions as needed...
}

impl<'a> NetworkReaderExtensions for NetworkReader<'a> {
    fn read_byte(&mut self) -> io::Result<u8> {
        self.read_blittable()
    }

    fn read_int(&mut self) -> io::Result<i32> {
        self.read_blittable()
    }

    fn read_uint(&mut self) -> io::Result<u32> {
        self.read_blittable()
    }

    fn read_float(&mut self) -> io::Result<f32> {
        self.read_blittable()
    }

    fn read_string(&mut self) -> io::Result<String> {
        let length = self.read_uint()?;
        let bytes = self.read_bytes(length as usize)?;
        String::from_utf8(bytes.to_vec()).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
    // More functions as needed...
}

fn main() -> io::Result<()> {
    let data = [0, 1, 0, 0, 0, 4, 116, 101, 115, 116]; // Example data for "test"
    let mut reader = NetworkReader::new(&data);

    let int_value = reader.read_int()?;
    println!("Read int: {}", int_value);

    let string_value = reader.read_string()?;
    println!("Read string: {}", string_value);

    Ok(())
}
