use byteorder::ReadBytesExt;
use bytes::{BufMut, Bytes, BytesMut};
use nalgebra::{Quaternion, Vector3};
use std::fmt::Debug;
use std::io;
use std::io::{Cursor, Read};

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
    pub fn read(&mut self, len: usize) -> io::Result<Bytes> {
        let mut buffer = vec![0; len];
        self.bytes.read_exact(&mut buffer)?;
        Ok(Bytes::from(buffer))
    }

    #[allow(dead_code)]
    pub fn read_next(&mut self) -> io::Result<Self> {
        let len = self.decompress_var_u64_le()?;
        if len > self.remaining() as u64 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid data"));
        }
        let mut buffer = vec![0; len as usize];
        self.bytes.read_exact(&mut buffer)?;
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
    pub fn decompress_var_u64_le(&mut self) -> io::Result<u64> {
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

    // 大端序 读取一个 Vector3 类型的数据
    #[allow(dead_code)]
    pub fn read_vector3_f32_be(&mut self) -> io::Result<Vector3<f32>>
    {
        let x = self.read_f32_be()?;
        let y = self.read_f32_be()?;
        let z = self.read_f32_be()?;
        Ok(Vector3::new(x, y, z))
    }

    // 小端序 读取一个 Vector3 类型的数据
    #[allow(dead_code)]
    pub fn read_vector3_f32_le(&mut self) -> io::Result<Vector3<f32>>
    {
        let x = self.read_f32_le()?;
        let y = self.read_f32_le()?;
        let z = self.read_f32_le()?;
        Ok(Vector3::new(x, y, z))
    }

    // 大端序 读取一个 Quaternion 类型的数据
    #[allow(dead_code)]
    pub fn read_quaternion_f32_be(&mut self) -> io::Result<Quaternion<f32>>
    {
        let i = self.read_f32_be()?;
        let j = self.read_f32_be()?;
        let k = self.read_f32_be()?;
        let w = self.read_f32_be()?;
        Ok(Quaternion::new(i, j, k, w))
    }

    // 小端序 读取一个 Quaternion 类型的数据
    #[allow(dead_code)]
    pub fn read_quaternion_f32_le(&mut self) -> io::Result<Quaternion<f32>>
    {
        let i = self.read_f32_le()?;
        let j = self.read_f32_le()?;
        let k = self.read_f32_le()?;
        let w = self.read_f32_le()?;
        Ok(Quaternion::new(i, j, k, w))
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
pub struct Batch {
    bytes: BytesMut,
}

impl Batch {
    // 初始化一个 Batch 实例
    #[allow(dead_code)]
    pub fn new() -> Self {
        Batch {
            bytes: BytesMut::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get_bytes(&self) -> Bytes {
        self.bytes.clone().freeze()
    }

    // 获取总长度
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    // 写入 [u8] 类型的数据
    #[allow(dead_code)]
    pub fn write(&mut self, buffer: &[u8]) {
        self.bytes.put(buffer);
    }

    // 写入一个 UnBatch 类型的数据
    #[allow(dead_code)]
    pub fn write_unbatch(&mut self, unbatch: &UnBatch) {
        self.write(&unbatch.bytes.get_ref());
    }

    // 写入一个 Batch 类型的数据
    #[allow(dead_code)]
    pub fn write_batch(&mut self, batch: &Batch) {
        self.write(&batch.bytes);
    }

    // 写入一个 bool 类型的数据
    #[allow(dead_code)]
    pub fn write_bool(&mut self, value: bool) {
        self.bytes.put_u8(if value { 1 } else { 0 });
    }

    // 写入一个 u8 类型的数据
    #[allow(dead_code)]
    pub fn write_u8(&mut self, value: u8) {
        self.bytes.put_u8(value);
    }

    // 写入一个 i8 类型的数据
    #[allow(dead_code)]
    pub fn write_i8(&mut self, value: i8) {
        self.bytes.put_i8(value);
    }

    // 大端序 写入一个 u16 类型的数据
    #[allow(dead_code)]
    pub fn write_u16_be(&mut self, value: u16) {
        self.write(&value.to_be_bytes());
    }

    // 大端序 写入一个 i16 类型的数据
    #[allow(dead_code)]
    pub fn write_i16_be(&mut self, value: i16) {
        self.write(&value.to_be_bytes());
    }

    // 小端序 写入一个 u16 类型的数据
    #[allow(dead_code)]
    pub fn write_u16_le(&mut self, value: u16) {
        self.write(&value.to_le_bytes());
    }

    // 小端序 写入一个 i16 类型的数据
    #[allow(dead_code)]
    pub fn write_i16_le(&mut self, value: i16) {
        self.write(&value.to_le_bytes());
    }

    // 大端序 写入一个 u32 类型的数据
    #[allow(dead_code)]
    pub fn write_u32_be(&mut self, value: u32) {
        self.write(&value.to_be_bytes());
    }

    // 大端序 写入一个 i32 类型的数据
    #[allow(dead_code)]
    pub fn write_i32_be(&mut self, value: i32) {
        self.write(&value.to_be_bytes());
    }

    // 小端序 写入一个 u32 类型的数据
    #[allow(dead_code)]
    pub fn write_u32_le(&mut self, value: u32) {
        self.write(&value.to_le_bytes());
    }

    // 小端序 写入一个 i32 类型的数据
    #[allow(dead_code)]
    pub fn write_i32_le(&mut self, value: i32) {
        self.write(&value.to_le_bytes());
    }

    // 大端序 写入一个 u64 类型的数据
    #[allow(dead_code)]
    pub fn write_u64_be(&mut self, value: u64) {
        self.write(&value.to_be_bytes());
    }

    // 大端序 写入一个 i64 类型的数据
    #[allow(dead_code)]
    pub fn write_i64_be(&mut self, value: i64) {
        self.write(&value.to_be_bytes());
    }

    // 小端序 写入一个 u64 类型的数据
    #[allow(dead_code)]
    pub fn write_u64_le(&mut self, value: u64) {
        self.write(&value.to_le_bytes());
    }

    // 小端序 写入一个 i64 类型的数据
    #[allow(dead_code)]
    pub fn write_i64_le(&mut self, value: i64) {
        self.write(&value.to_le_bytes());
    }

    // 大端序 写入一个 u128 类型的数据
    #[allow(dead_code)]
    pub fn write_u128_be(&mut self, value: u128) {
        self.write(&value.to_be_bytes());
    }

    // 大端序 写入一个 i128 类型的数据
    #[allow(dead_code)]
    pub fn write_i128_be(&mut self, value: i128) {
        self.write(&value.to_be_bytes());
    }

    // 小端序 写入一个 u128 类型的数据
    #[allow(dead_code)]
    pub fn write_u128_le(&mut self, value: u128) {
        self.write(&value.to_le_bytes());
    }

    // 小端序 写入一个 i128 类型的数据
    #[allow(dead_code)]
    pub fn write_i128_le(&mut self, value: i128) {
        self.write(&value.to_le_bytes());
    }

    // 大端序 写入一个 f32 类型的数据
    #[allow(dead_code)]
    pub fn write_f32_be(&mut self, value: f32) {
        self.write(&value.to_be_bytes());
    }

    // 小端序 写入一个 f32 类型的数据
    #[allow(dead_code)]
    pub fn write_f32_le(&mut self, value: f32) {
        self.write(&value.to_le_bytes());
    }

    // 大端序 写入一个 f64 类型的数据
    #[allow(dead_code)]
    pub fn write_f64_be(&mut self, value: f64) {
        self.write(&value.to_be_bytes());
    }

    // 小端序 写入一个 f64 类型的数据
    #[allow(dead_code)]
    pub fn write_f64_le(&mut self, value: f64) {
        self.write(&value.to_le_bytes());
    }

    // 写入一个压缩的 u64 类型的数据
    #[allow(dead_code)]
    pub fn compress_var_u64_le(&mut self, value: u64) {
        if value <= 240 {
            self.write_u8(value as u8);
            return;
        }
        if value <= 2287 {
            let a = ((value - 240) >> 8) as u8 + 241;
            let b = (value - 240) as u8;
            self.write_u8(a);
            self.write_u8(b);
            return;
        }
        if value <= 67823 {
            let a = 249;
            let b = ((value - 2288) >> 8) as u8;
            let c = (value - 2288) as u8;
            self.write_u8(a);
            self.write_u8(b);
            self.write_u8(c);
            return;
        }
        if value <= 16777215 {
            let a = 250;
            let b = (value << 8) as u32;
            self.write_u8(a);
            self.write_u32_le(b | a as u32);
            return;
        }
        if value <= 4294967295 {
            let a = 251;
            let b = value as u32;
            self.write_u8(a);
            self.write_u32_le(b);
            return;
        }
        if value <= 1099511627775 {
            let a = 252;
            let b = (value & 0xFF) as u16;
            let c = (value >> 8) as u32;
            self.write_u16_le(b << 8 | a);
            self.write_u32_le(c);
            return;
        }
        if value <= 281474976710655 {
            let a = 253;
            let b = (value & 0xFF) as u16;
            let c = ((value >> 8) & 0xFF) as u16;
            let d = (value >> 16) as u32;
            self.write_u8(a);
            self.write_u16_le(c << 8 | b);
            self.write_u32_le(d);
            return;
        }
        if value <= 72057594037927935 {
            let a = 254;
            let b = value << 8;
            self.write_u64_le(b | a);
            return;
        }

        // all others
        {
            self.write_u8(255);
            self.write(&value.to_le_bytes());
        }
    }

    // 大端序 写入一个 string 类型的数据
    #[allow(dead_code)]
    pub fn write_string_be(&mut self, value: &str) {
        let length = value.len() as u16 + 1;
        self.write_u16_be(length);
        self.write(value.as_bytes());
    }

    // 小端序 写入一个 string 类型的数据
    #[allow(dead_code)]
    pub fn write_string_le(&mut self, value: &str) {
        let length = value.len() as u16 + 1;
        self.write_u16_le(length);
        self.write(value.as_bytes());
    }

    // 大端序 写入一个 Vector3 类型的数据
    #[allow(dead_code)]
    pub fn write_vector3_f32_be(&mut self, value: Vector3<f32>) {
        self.write_f32_be(value.x);
        self.write_f32_be(value.y);
        self.write_f32_be(value.z);
    }

    // 小端序 写入一个 Vector3 类型的数据
    #[allow(dead_code)]
    pub fn write_vector3_f32_le(&mut self, value: Vector3<f32>) {
        self.write_f32_le(value.x);
        self.write_f32_le(value.y);
        self.write_f32_le(value.z);
    }

    // 大端序 写入一个 Quaternion 类型的数据
    #[allow(dead_code)]
    pub fn write_quaternion_f32_be(&mut self, value: Quaternion<f32>) {
        self.write_f32_be(value.i);
        self.write_f32_be(value.j);
        self.write_f32_be(value.k);
        self.write_f32_be(value.w);
    }

    // 小端序 写入一个 Quaternion 类型的数据
    #[allow(dead_code)]
    pub fn write_quaternion_f32_le(&mut self, value: Quaternion<f32>) {
        self.write_f32_le(value.i);
        self.write_f32_le(value.j);
        self.write_f32_le(value.k);
        self.write_f32_le(value.w);
    }
}

impl Debug for Batch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Batch {{ bytes: {:?},  length: {} }}",
            self.bytes, self.len()
        )
    }
}

pub trait DataReader<T> {
    fn deserialization(batch: &mut UnBatch) -> io::Result<T>;
}


pub trait DataWriter {
    fn serialization(&mut self, batch: &mut Batch);
}
