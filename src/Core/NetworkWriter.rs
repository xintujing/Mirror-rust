use byteorder::{BigEndian, LittleEndian, WriteBytesExt};
use std::io::Cursor;

/// NetworkWriter struct for simple types like floats, ints, etc.
pub struct NetworkWriter {
    buffer: Vec<u8>,
    position: usize,
    is_big_endian: bool,
}

impl NetworkWriter {
    const MAX_STRING_LENGTH: u16 = u16::MAX - 1;
    const DEFAULT_CAPACITY: usize = 1500;

    pub fn new_with_big_endian() -> Self {
        Self {
            buffer: vec![0; Self::DEFAULT_CAPACITY],
            position: 0,
            is_big_endian: true,
        }
    }

    pub fn new_with_little_endian() -> Self {
        Self {
            buffer: vec![0; Self::DEFAULT_CAPACITY],
            position: 0,
            is_big_endian: false,
        }
    }

    /// 重置位置，保持容量不变，以便重复使用。
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// 确保缓冲区有足够的容量来容纳额外的字节。
    fn ensure_capacity(&mut self, additional: usize) {
        let needed_capacity = self.position + additional;
        if self.buffer.len() < needed_capacity {
            self.buffer.resize(needed_capacity, 0);
        }
    }

    /// 将 blittable 类型 T 写入缓冲区。
    pub fn write_blittable<T: bytemuck::Pod>(&mut self, value: &T) {
        self.ensure_capacity(size_of::<T>());
        let bytes = bytemuck::bytes_of(value);
        self.buffer[self.position..self.position + bytes.len()].copy_from_slice(bytes);
        self.position += bytes.len();
    }


    /// Writes an optional blittable type T to the buffer.
    pub fn write_blittable_nullable<T: bytemuck::Pod>(&mut self, value: Option<&T>) {
        self.write_u8(value.is_some() as u8);
        if let Some(val) = value {
            self.write_blittable(val);
        }
    }

    /// 将单个字节写入缓冲区。
    pub fn write_u8(&mut self, value: u8) {
        self.ensure_capacity(1);
        self.buffer[self.position] = value;
        self.position += 1;
    }

    /// 将 16 位无符号整数写入缓冲区。
    pub fn write_u16(&mut self, value: u16) {
        self.ensure_capacity(2);
        let mut cursor = Cursor::new(&mut self.buffer[self.position..self.position + 2]);
        if self.is_big_endian {
            cursor.write_u16::<BigEndian>(value).unwrap();
        } else {
            cursor.write_u16::<LittleEndian>(value).unwrap();
        }
        self.position += 2; // update position after writing
    }

    /// 将 32 位无符号整数写入缓冲区。
    pub fn write_u32(&mut self, value: u32) {
        self.ensure_capacity(4);
        let mut cursor = Cursor::new(&mut self.buffer[self.position..self.position + 4]);
        if self.is_big_endian {
            cursor.write_u32::<BigEndian>(value).unwrap();
        } else {
            cursor.write_u32::<LittleEndian>(value).unwrap();
        }
        self.position += 4; // update position after writing
    }

    /// 将 64 位无符号整数写入缓冲区。
    pub fn write_u64(&mut self, value: u64) {
        self.ensure_capacity(8);
        let mut cursor = Cursor::new(&mut self.buffer[self.position..self.position + 8]);
        if self.is_big_endian {
            cursor.write_u64::<BigEndian>(value).unwrap();
        } else {
            cursor.write_u64::<LittleEndian>(value).unwrap();
        }
        self.position += 8; // update position after writing
    }

    /// 将 32 位浮点数写入缓冲区。
    pub fn write_f32(&mut self, value: f32) {
        self.ensure_capacity(4);
        let mut cursor = Cursor::new(&mut self.buffer[self.position..self.position + 4]);
        if self.is_big_endian {
            cursor.write_f32::<BigEndian>(value).unwrap();
        } else {
            cursor.write_f32::<LittleEndian>(value).unwrap();
        }
        self.position += 4; // update position after writing
    }

    /// 将 64 位浮点数写入缓冲区。
    pub fn write_f64(&mut self, value: f64) {
        self.ensure_capacity(8);
        let mut cursor = Cursor::new(&mut self.buffer[self.position..self.position + 8]);
        if self.is_big_endian {
            cursor.write_f64::<BigEndian>(value).unwrap();
        } else {
            cursor.write_f64::<LittleEndian>(value).unwrap();
        }
        self.position += 8; // update position after writing
    }

    /// 将具有给定偏移量和计数的字节数组写入缓冲区。
    pub fn write_bytes(&mut self, array: &[u8], offset: usize, count: usize) {
        self.ensure_capacity(count);
        self.buffer[self.position..self.position + count].copy_from_slice(&array[offset..offset + count]);
        self.position += count;
    }

    /// 任何受支持类型的通用写入函数。实际实现的占位符。
    pub fn write<T>(&mut self, value: T)
    where
        T: WriteToNetwork,
    {
        value.write_to(self);
    }

    /// 将缓冲区的写入部分转换为字节片。
    pub fn to_slice(&self) -> &[u8] {
        &self.buffer[..self.position]
    }

    /// 将缓冲区的写入部分转换为 hex 字符串。
    pub fn to_hex_string(&self) -> String {
        let mut hex_string = String::new();
        for byte in self.to_slice() {
            hex_string.push_str(&format!("{:02X}", byte));
        }
        hex_string
    }

    /// Returns a string representation of the buffer content for debugging.
    pub fn to_string(&self) -> String {
        format!(
            "[{:02X?} @ {}/{}]",
            &self.buffer[..self.position],
            self.position,
            self.buffer.len()
        )
    }
}

/// Trait to be implemented by types that can be written to NetworkWriter.
pub trait WriteToNetwork {
    fn write_to(&self, writer: &mut NetworkWriter);
}

impl WriteToNetwork for u8 {
    fn write_to(&self, writer: &mut NetworkWriter) {
        writer.write_u8(*self);
    }
}

impl WriteToNetwork for u16 {
    fn write_to(&self, writer: &mut NetworkWriter) {
        writer.write_u16(*self);
    }
}

impl WriteToNetwork for u32 {
    fn write_to(&self, writer: &mut NetworkWriter) {
        writer.write_u32(*self);
    }
}

impl WriteToNetwork for u64 {
    fn write_to(&self, writer: &mut NetworkWriter) {
        writer.write_u64(*self);
    }
}

impl WriteToNetwork for f32 {
    fn write_to(&self, writer: &mut NetworkWriter) {
        writer.write_f32(*self);
    }
}

impl WriteToNetwork for f64 {
    fn write_to(&self, writer: &mut NetworkWriter) {
        writer.write_f64(*self);
    }
}

impl WriteToNetwork for &str {
    fn write_to(&self, writer: &mut NetworkWriter) {
        let bytes = self.as_bytes();
        let len = bytes.len();
        if len > NetworkWriter::MAX_STRING_LENGTH as usize {
            panic!("String is too long to write: {}", len);
        }
        writer.write_u16(len as u16);
        writer.write_bytes(bytes, 0, len);
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_network_writer() {
        let mut writer = NetworkWriter::new_with_little_endian();
        // writer.write_u8(12);
        // println!("writer: {:?}", writer.to_slice());
        // writer.write_u16(1234);
        // println!("writer: {:?}", writer.to_slice());
        // writer.write_u32(12345678);
        // println!("writer: {:?}", writer.to_slice());
        // writer.write_u64(1234567890);
        // println!("writer: {:?}", writer.to_slice());
        // writer.write_f32(1234.5678);
        // println!("writer: {:?}", writer.to_slice());
        // writer.write_f64(12345678.901234);
        // println!("writer: {:?}", writer.to_slice());

        // writer.write(12u8);
        // println!("writer: {:?}", writer.to_slice());
        writer.write(1234.45);
        println!("writer: {:?}", writer.to_slice());
        // writer.write(12345678u32);
        // println!("writer: {:?}", writer.to_slice());
        // writer.write(1234567890u64);
        // println!("writer: {:?}", writer.to_slice());
        // writer.write(1234.5678f32);
        // println!("writer: {:?}", writer.to_slice());
        // writer.write(12345678.901234f64);
        // println!("writer: {:?}", writer.to_slice());
        //
        // writer.write("Hello, World!");
        // println!("writer: {:?}", writer.to_slice());

        println!("writer: {}", writer.to_hex_string());
    }
}