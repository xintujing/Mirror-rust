use crate::tools::get_s_e_t;
use byteorder::ReadBytesExt;
use bytes::{Bytes, BytesMut};
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::io;
use std::io::{Cursor, Read};

#[derive(PartialEq, Copy, Clone)]
pub enum Endian {
    Big,
    Little,
}

impl Debug for Endian {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Endian::Big => write!(f, "Big"),
            Endian::Little => write!(f, "Little"),
        }
    }
}

#[derive(Clone)]
pub struct UnBatch {
    bytes: Cursor<Bytes>,
}

impl UnBatch {
    // 创建一个新的 UnBatch 实例
    #[allow(dead_code)]
    pub fn new(bytes: Bytes) -> Self {
        UnBatch {
            bytes: Cursor::new(bytes),
        }
    }

    // 获取总长度
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.bytes.get_ref().len()
    }

    // 获取剩余长度
    #[allow(dead_code)]
    pub fn remaining(&self) -> usize {
        self.bytes.get_ref().len() - self.bytes.position() as usize
    }

    // 获取当前读取位置
    #[allow(dead_code)]
    pub fn position(&self) -> u64 {
        self.bytes.position()
    }

    // 设置读取位置
    #[allow(dead_code)]
    pub fn set_position(&mut self, position: u64) {
        self.bytes.set_position(position);
    }

    // 读取指定长度的数据
    #[allow(dead_code)]
    pub fn read(&mut self, buffer: &mut [u8]) -> io::Result<()> {
        self.bytes.read_exact(buffer)
    }

    #[allow(dead_code)]
    pub fn read_next(&mut self) -> io::Result<Self> {
        let len = self.decompress_var()?;
        if len > self.remaining() as u64 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid data"));
        }
        let mut buffer = vec![0; len as usize];
        self.bytes.read_exact(&mut buffer).unwrap();
        Ok(UnBatch::new(Bytes::from(buffer)))
    }

    // 读取剩余的数据
    #[allow(dead_code)]
    pub fn read_remaining(&mut self) -> io::Result<Bytes> {
        let mut buffer = vec![0; self.remaining()];
        self.bytes.read_exact(&mut buffer)?;
        Ok(Bytes::from(buffer))
    }

    // 读取一个 bool 类型的数据
    #[allow(dead_code)]
    pub fn read_bool(&mut self) -> io::Result<bool> {
        self.bytes.read_u8().map(|v| v != 0)
    }

    // 读取一个 u8 类型的数据
    #[allow(dead_code)]
    pub fn read_u8(&mut self) -> io::Result<u8> {
        self.bytes.read_u8()
    }

    // 读取一个 i8 类型的数据
    #[allow(dead_code)]
    pub fn read_i8(&mut self) -> io::Result<i8> {
        self.bytes.read_i8()
    }

    // 大端序 读取一个 u16 类型的数据
    #[allow(dead_code)]
    pub fn read_u16_be(&mut self) -> io::Result<u16> {
        self.bytes.read_u16::<byteorder::BigEndian>()
    }

    // 大端序 读取一个 i16 类型的数据
    #[allow(dead_code)]
    pub fn read_i16_be(&mut self) -> io::Result<i16> {
        self.bytes.read_i16::<byteorder::BigEndian>()
    }

    // 小端序 读取一个 u16 类型的数据
    #[allow(dead_code)]
    pub fn read_u16_le(&mut self) -> io::Result<u16> {
        self.bytes.read_u16::<byteorder::LittleEndian>()
    }

    // 小端序 读取一个 i16 类型的数据
    #[allow(dead_code)]
    pub fn read_i16_le(&mut self) -> io::Result<i16> {
        self.bytes.read_i16::<byteorder::LittleEndian>()
    }

    // 大端序 读取一个 u32 类型的数据
    #[allow(dead_code)]
    pub fn read_u32_be(&mut self) -> io::Result<u32> {
        self.bytes.read_u32::<byteorder::BigEndian>()
    }

    // 大端序 读取一个 i32 类型的数据
    #[allow(dead_code)]
    pub fn read_i32_be(&mut self) -> io::Result<i32> {
        self.bytes.read_i32::<byteorder::BigEndian>()
    }

    // 小端序 读取一个 u32 类型的数据
    #[allow(dead_code)]
    pub fn read_u32_le(&mut self) -> io::Result<u32> {
        self.bytes.read_u32::<byteorder::LittleEndian>()
    }

    // 小端序 读取一个 i32 类型的数据
    #[allow(dead_code)]
    pub fn read_i32_le(&mut self) -> io::Result<i32> {
        self.bytes.read_i32::<byteorder::LittleEndian>()
    }

    // 大端序 读取一个 u64 类型的数据
    #[allow(dead_code)]
    pub fn read_u64_be(&mut self) -> io::Result<u64> {
        self.bytes.read_u64::<byteorder::BigEndian>()
    }

    // 大端序 读取一个 i64 类型的数据
    #[allow(dead_code)]
    pub fn read_i64_be(&mut self) -> io::Result<i64> {
        self.bytes.read_i64::<byteorder::BigEndian>()
    }

    // 小端序 读取一个 u64 类型的数据
    #[allow(dead_code)]
    pub fn read_u64_le(&mut self) -> io::Result<u64> {
        self.bytes.read_u64::<byteorder::LittleEndian>()
    }

    // 小端序 读取一个 i64 类型的数据
    #[allow(dead_code)]
    pub fn read_i64_le(&mut self) -> io::Result<i64> {
        self.bytes.read_i64::<byteorder::LittleEndian>()
    }

    // 大端序 读取一个 u128 类型的数据
    #[allow(dead_code)]
    pub fn read_u128_be(&mut self) -> io::Result<u128> {
        self.bytes.read_u128::<byteorder::BigEndian>()
    }

    // 大端序 读取一个 i128 类型的数据
    #[allow(dead_code)]
    pub fn read_i128_be(&mut self) -> io::Result<i128> {
        self.bytes.read_i128::<byteorder::BigEndian>()
    }

    // 小端序 读取一个 u128 类型的数据
    #[allow(dead_code)]
    pub fn read_u128_le(&mut self) -> io::Result<u128> {
        self.bytes.read_u128::<byteorder::LittleEndian>()
    }

    // 小端序 读取一个 i128 类型的数据
    #[allow(dead_code)]
    pub fn read_i128_le(&mut self) -> io::Result<i128> {
        self.bytes.read_i128::<byteorder::LittleEndian>()
    }

    // 大端序 读取一个 f32 类型的数据
    #[allow(dead_code)]
    pub fn read_f32_be(&mut self) -> io::Result<f32> {
        self.bytes.read_f32::<byteorder::BigEndian>()
    }

    // 小端序 读取一个 f32 类型的数据
    #[allow(dead_code)]
    pub fn read_f32_le(&mut self) -> io::Result<f32> {
        self.bytes.read_f32::<byteorder::LittleEndian>()
    }

    // 大端序 读取一个 f64 类型的数据
    #[allow(dead_code)]
    pub fn read_f64_be(&mut self) -> io::Result<f64> {
        self.bytes.read_f64::<byteorder::BigEndian>()
    }

    // 小端序 读取一个 f64 类型的数据
    #[allow(dead_code)]
    pub fn read_f64_le(&mut self) -> io::Result<f64> {
        self.bytes.read_f64::<byteorder::LittleEndian>()
    }

    // 读取一个压缩的 u64 类型的数据
    #[allow(dead_code)]
    pub fn decompress_var(&mut self) -> io::Result<u64> {
        let a0 = self.read_u8()?;
        if a0 < 241 {
            return Ok(u64::from(a0));
        }

        let a1 = self.read_u8()?;
        if a0 <= 248 {
            return Ok(240 + ((u64::from(a0) - 241) << 8) + u64::from(a1));
        }

        let a2 = self.read_u8()?;
        if a0 == 249 {
            return Ok(2288 + (u64::from(a1) << 8) + u64::from(a2));
        }

        let a3 = self.read_u8()?;
        if a0 == 250 {
            return Ok(u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16));
        }

        let a4 = self.read_u8()?;
        if a0 == 251 {
            return Ok(u64::from(a1)
                + (u64::from(a2) << 8)
                + (u64::from(a3) << 16)
                + (u64::from(a4) << 24));
        }

        let tmp = u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16) + (u64::from(a4) << 24);

        let a5 = self.read_u8()?;
        if a0 == 252 {
            return Ok(tmp + (u64::from(a5) << 32));
        }

        let a6 = self.read_u8()?;
        if a0 == 253 {
            return Ok(tmp + (u64::from(a5) << 32) + (u64::from(a6) << 40));
        }

        let a7 = self.read_u8()?;
        if a0 == 254 {
            return Ok(tmp + (u64::from(a5) << 32) + (u64::from(a6) << 40) + (u64::from(a7) << 48));
        }

        let a8 = self.read_u8()?;
        if a0 == 255 {
            return Ok(tmp
                + (u64::from(a5) << 32)
                + (u64::from(a6) << 40)
                + (u64::from(a7) << 48)
                + (u64::from(a8) << 56));
        }
        Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid data"))
    }

    // 大端序 读取一个 str 类型的数据
    #[allow(dead_code)]
    pub fn read_string_be(&mut self) -> io::Result<String> {
        let length = self.read_u16_be()? as usize - 1;
        let mut buffer = vec![0; length];
        self.bytes.read_exact(&mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    // 小端序 读取一个 String 类型的数据
    #[allow(dead_code)]
    pub fn read_string_le(&mut self) -> io::Result<String> {
        let length = self.read_u16_le()? as usize - 1;
        let mut buffer = vec![0; length];
        self.bytes.read_exact(&mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).to_string())
    }
}

impl Debug for UnBatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UnBatch {{ bytes: {:?}, position: {}, len: {} }}",
            self.bytes, self.position(), self.len()
        )
    }
}

#[derive(Clone)]
pub struct Writer {
    buffer: BytesMut,
    position: usize,
    length: usize,
    endian: Endian,
    elapsed_time: f64,
}

impl Writer {
    // 初始化一个 Writer 实例
    #[allow(dead_code)]
    pub fn new(endian: Endian, is_wet: bool) -> Self {
        match endian {
            Endian::Big => Self::new_with_ben(is_wet),
            Endian::Little => Self::new_with_len(is_wet),
        }
    }

    // 初始化一个 Writer 实例，指定是否为大端序
    #[allow(dead_code)]
    pub fn new_with_ben(is_wet: bool) -> Self {
        let mut writer = Self {
            buffer: BytesMut::new(),
            position: 0,
            length: 0,
            endian: Endian::Big,
            elapsed_time: get_s_e_t(),
        };
        if is_wet {
            writer.write_f64(writer.elapsed_time);
        }
        writer
    }

    // 初始化一个 Writer 实例，指定是否为小端序
    #[allow(dead_code)]
    pub fn new_with_len(is_wet: bool) -> Self {
        let mut writer = Self {
            buffer: BytesMut::new(),
            position: 0,
            length: 0,
            endian: Endian::Little,
            elapsed_time: get_s_e_t(),
        };
        if is_wet {
            writer.write_f64(writer.elapsed_time);
        }
        writer
    }

    // 获取数据
    pub fn get_data(&self) -> &[u8] {
        &self.buffer
    }

    // 获取时间戳
    #[allow(dead_code)]
    pub fn get_elapsed_time(&self) -> f64 {
        self.elapsed_time
    }

    // 写入数据
    #[allow(dead_code)]
    pub fn write(&mut self, data: &[u8]) {
        let self_data_len = self.buffer.len();
        let need_len = self.position + data.len();
        if self_data_len < need_len {
            self.buffer.reserve(need_len - self_data_len);
        }

        // 扩展 self.data 以确保有足够的空间
        if self_data_len < need_len {
            self.buffer.resize(need_len, 0);
        }

        // 从 position 开始写入数据
        self.buffer[self.position..need_len].copy_from_slice(data);

        // 更新 position
        self.position += data.len();
    }

    // 清空数据
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.position = 0;
        self.length = 0;
        self.buffer.clear();
    }

    // 获取当前位置
    #[allow(dead_code)]
    pub fn get_position(&self) -> usize {
        self.position
    }

    // 设置当前位置
    #[allow(dead_code)]
    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }

    // 获取数据长度
    #[allow(dead_code)]
    pub fn get_length(&self) -> usize {
        self.length
    }

    // 设置端序
    #[allow(dead_code)]
    pub fn set_endian(&mut self, endian: Endian) {
        self.endian = endian;
    }

    // 写入 u8
    #[allow(dead_code)]
    pub fn write_u8(&mut self, value: u8) {
        self.write(&value.to_be_bytes());
    }

    // 写入 i8
    #[allow(dead_code)]
    pub fn write_i8(&mut self, value: i8) {
        self.write(&value.to_be_bytes());
    }

    // 写入 u16
    #[allow(dead_code)]
    pub fn write_u16(&mut self, value: u16) {
        if self.endian == Endian::Big {
            self.write(&value.to_be_bytes());
        } else {
            self.write(&value.to_le_bytes());
        }
    }

    // 写入 i16
    #[allow(dead_code)]
    pub fn write_i16(&mut self, value: i16) {
        if self.endian == Endian::Big {
            self.write(&value.to_be_bytes());
        } else {
            self.write(&value.to_le_bytes());
        }
    }

    // 写入 u32
    #[allow(dead_code)]
    pub fn write_u32(&mut self, value: u32) {
        if self.endian == Endian::Big {
            self.write(&value.to_be_bytes());
        } else {
            self.write(&value.to_le_bytes());
        }
    }

    // 写入 i32
    #[allow(dead_code)]
    pub fn write_i32(&mut self, value: i32) {
        if self.endian == Endian::Big {
            self.write(&value.to_be_bytes());
        } else {
            self.write(&value.to_le_bytes());
        }
    }

    // 写入 u64
    #[allow(dead_code)]
    pub fn write_u64(&mut self, value: u64) {
        if self.endian == Endian::Big {
            self.write(&value.to_be_bytes());
        } else {
            self.write(&value.to_le_bytes());
        }
    }

    // 写入 i64
    #[allow(dead_code)]
    pub fn write_i64(&mut self, value: i64) {
        if self.endian == Endian::Big {
            self.write(&value.to_be_bytes());
        } else {
            self.write(&value.to_le_bytes());
        }
    }

    // 写入 f32
    #[allow(dead_code)]
    pub fn write_f32(&mut self, value: f32) {
        if self.endian == Endian::Big {
            self.write(&value.to_be_bytes());
        } else {
            self.write(&value.to_le_bytes());
        }
    }

    // 写入 f64
    #[allow(dead_code)]
    pub fn write_f64(&mut self, value: f64) {
        if self.endian == Endian::Big {
            self.write(&value.to_be_bytes());
        } else {
            self.write(&value.to_le_bytes());
        }
    }

    #[allow(dead_code)]
    pub fn compress_var(&mut self, value: u64) {
        if value <= 240 {
            self.write_u8(value as u8);
        } else if value <= 2287 {
            let a = ((value - 240) >> 8) as u8 + 241;
            let b = (value - 240) as u8;
            self.write_u8(a);
            self.write_u8(b);
        } else if value <= 67823 {
            let a = 249;
            let b = ((value - 2288) >> 8) as u8;
            let c = (value - 2288) as u8;
            self.write_u8(a);
            self.write_u8(b);
            self.write_u8(c);
        } else if value <= 16_777_215 {
            let a = 250;
            let b = value as u32;
            let bytes = b.to_le_bytes();
            self.write_u8(a);
            self.write(&bytes[0..3]); // 只写入低 3 字节
        } else if value <= 4_294_967_295 {
            let a = 251;
            let b = value as u32;
            self.write_u8(a);
            self.write(&b.to_le_bytes());
        } else if value <= 1_099_511_627_775 {
            let a = 252;
            let b = (value & 0xFF) as u8;
            let c = (value >> 8) as u32;
            self.write_u8(a);
            self.write_u8(b);
            self.write(&c.to_le_bytes());
        } else if value <= 281_474_976_710_655 {
            let a = 253;
            let b = (value & 0xFF) as u8;
            let c = ((value >> 8) & 0xFF) as u8;
            let d = (value >> 16) as u32;
            self.write_u8(a);
            self.write_u8(b);
            self.write_u8(c);
            self.write(&d.to_le_bytes());
        } else if value <= 72_057_594_037_927_935 {
            let a = 254;
            let b = value;
            let bytes = b.to_le_bytes();
            self.write_u8(a);
            self.write(&bytes[0..7]); // 只写入低 7 字节
        } else {
            self.write_u8(255);
            self.write(&value.to_le_bytes());
        }
    }

    #[allow(dead_code)]
    pub fn compress_var_uz(&mut self, value: usize) {
        self.compress_var(value as u64);
    }

    // 写入字符串
    pub fn write_string(&mut self, value: &[u8]) {
        self.write_u16(1 + value.len() as u16);
        self.write(value);
    }

    // 写入 bool
    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(if value { 1 } else { 0 });
    }
}

impl Debug for Writer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Writer {{ data: {:?}, position: {}, length: {}, endian: {:?} }}",
            self.buffer, self.position, self.length, self.endian
        )
    }
}

pub trait DataReader<T> {
    fn deserialization(batch: &mut UnBatch) -> T;
}

pub trait DataWriter<T> {
    fn serialization(&mut self, writer: &mut Writer);
}
