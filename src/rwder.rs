use crate::tools::get_start_elapsed_time;
use std::cmp::PartialEq;
use std::fmt::Debug;

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

pub struct Reader {
    data: Vec<u8>,
    position: usize,
    length: usize,
    endian: Endian,
}

impl Reader {
    // 初始化一个 Reader 实例
    #[allow(dead_code)]
    pub fn new(data: &[u8], endian: Endian) -> Self {
        match endian {
            Endian::Big => {
                Self::new_with_ben(data)
            }
            Endian::Little => {
                Self::new_with_len(data)
            }
        }
    }

    // 初始化一个 Reader 实例，指定是否为大端序
    #[allow(dead_code)]
    pub fn new_with_ben(data: &[u8]) -> Self {
        let length = data.len();
        Self {
            data: data.to_vec(),
            position: 0,
            length,
            endian: Endian::Big,
        }
    }

    // 初始化一个 Reader 实例，指定是否为小端序
    #[allow(dead_code)]
    pub fn new_with_len(data: &[u8]) -> Self {
        let length = data.len();
        Self {
            data: data.to_vec(),
            position: 0,
            length,
            endian: Endian::Little,
        }
    }

    // 读取指定长度的数据
    #[allow(dead_code)]
    pub fn read(&mut self, length: usize) -> &[u8] {
        if self.get_remaining() < length {
            return &[];
        }
        let start = self.position;
        self.position += length;
        &self.data[start..self.position]
    }

    // 读取剩余
    #[allow(dead_code)]
    pub fn read_remaining(&mut self) -> &[u8] {
        self.read(self.get_remaining())
    }

    // 写入数据
    #[allow(dead_code)]
    pub fn write(&mut self, data: &[u8]) {
        self.data.reserve(data.len());  // 预留空间，避免多次扩容
        self.length += data.len();
        self.data.extend(data);
    }

    // 清空数据
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.position = 0;
        self.length = 0;
        self.data.clear();
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

    // 获取剩余数据长度
    #[allow(dead_code)]
    pub fn get_remaining(&self) -> usize {
        self.length - self.position
    }

    // 设置端序
    #[allow(dead_code)]
    pub fn set_endian(&mut self, endian: Endian) {
        self.endian = endian;
    }

    /// read start

    // 读取 u8
    #[allow(dead_code)]
    pub fn read_u8(&mut self) -> u8 {
        let data = self.read(1);
        data[0]
    }

    // 读取 i8
    #[allow(dead_code)]
    pub fn read_i8(&mut self) -> i8 {
        let data = self.read(1);
        data[0] as i8
    }

    // 读取 u16
    #[allow(dead_code)]
    pub fn read_u16(&mut self) -> u16 {
        let endian = self.endian;
        let data = self.read(2);
        if endian == Endian::Big {
            u16::from_be_bytes([data[0], data[1]])
        } else {
            u16::from_le_bytes([data[0], data[1]])
        }
    }

    // 读取 i16
    #[allow(dead_code)]
    pub fn read_i16(&mut self) -> i16 {
        let endian = self.endian;
        let data = self.read(2);
        if endian == Endian::Big {
            i16::from_be_bytes([data[0], data[1]])
        } else {
            i16::from_le_bytes([data[0], data[1]])
        }
    }

    // 读取 u32
    #[allow(dead_code)]
    pub fn read_u32(&mut self) -> u32 {
        let endian = self.endian;
        let data = self.read(4);
        if endian == Endian::Big {
            u32::from_be_bytes([data[0], data[1], data[2], data[3]])
        } else {
            u32::from_le_bytes([data[0], data[1], data[2], data[3]])
        }
    }

    // 读取 i32
    #[allow(dead_code)]
    pub fn read_i32(&mut self) -> i32 {
        let endian = self.endian;
        let data = self.read(4);
        if endian == Endian::Big {
            i32::from_be_bytes([data[0], data[1], data[2], data[3]])
        } else {
            i32::from_le_bytes([data[0], data[1], data[2], data[3]])
        }
    }

    // 读取 u64
    #[allow(dead_code)]
    pub fn read_u64(&mut self) -> u64 {
        let endian = self.endian;
        let data = self.read(8);
        if endian == Endian::Big {
            u64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])
        } else {
            u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])
        }
    }

    // 读取 i64
    #[allow(dead_code)]
    pub fn read_i64(&mut self) -> i64 {
        let endian = self.endian;
        let data = self.read(8);
        if endian == Endian::Big {
            i64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])
        } else {
            i64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])
        }
    }

    // 读取 f32
    #[allow(dead_code)]
    pub fn read_f32(&mut self) -> f32 {
        let endian = self.endian;
        let data = self.read(4);
        if endian == Endian::Big {
            f32::from_be_bytes([data[0], data[1], data[2], data[3]])
        } else {
            f32::from_le_bytes([data[0], data[1], data[2], data[3]])
        }
    }

    // 读取 f64
    #[allow(dead_code)]
    pub fn read_f64(&mut self) -> f64 {
        let endian = self.endian;
        let data = self.read(8);
        if endian == Endian::Big {
            f64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])
        } else {
            f64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])
        }
    }

    // 读取一个 Message
    #[allow(dead_code)]
    pub fn read_one(&mut self) -> Self {
        let data_len = self.decompress_var_uint();
        match self.endian {
            Endian::Big => {
                Self::new_with_ben(self.read(data_len as usize))
            }
            Endian::Little => {
                Self::new_with_len(self.read(data_len as usize))
            }
        }
    }
    #[allow(dead_code)]
    pub fn decompress_var_uint(&mut self) -> u64 {
        let a0 = self.read_u8();
        if a0 < 241 {
            return u64::from(a0);
        }

        let a1 = self.read_u8();
        if a0 <= 248 {
            return 240 + ((u64::from(a0) - 241) << 8) + u64::from(a1);
        }

        let a2 = self.read_u8();
        if a0 == 249 {
            return 2288 + (u64::from(a1) << 8) + u64::from(a2);
        }

        let a3 = self.read_u8();
        if a0 == 250 {
            return u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16);
        }

        let a4 = self.read_u8();
        if a0 == 251 {
            return u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16) + (u64::from(a4) << 24);
        }

        let a5 = self.read_u8();
        if a0 == 252 {
            return u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16) + (u64::from(a4) << 24) + (u64::from(a5) << 32);
        }

        let a6 = self.read_u8();
        if a0 == 253 {
            return u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16) + (u64::from(a4) << 24) + (u64::from(a5) << 32) + (u64::from(a6) << 40);
        }

        let a7 = self.read_u8();
        if a0 == 254 {
            return u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16) + (u64::from(a4) << 24) + (u64::from(a5) << 32) + (u64::from(a6) << 40) + (u64::from(a7) << 48);
        }

        let a8 = self.read_u8();
        if a0 == 255 {
            return u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16) + (u64::from(a4) << 24) + (u64::from(a5) << 32) + (u64::from(a6) << 40) + (u64::from(a7) << 48) + (u64::from(a8) << 56);
        }
        0
    }

    // 读取字符串
    pub fn read_string(&mut self) -> String {
        let len = self.read_u16();
        let data = self.read(len as usize);
        String::from_utf8(data.to_vec()).unwrap()
    }

    // 读取 bool
    pub fn read_bool(&mut self) -> bool {
        self.read_u8() != 0
    }
}

impl Debug for Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Reader {{ data: {:?}, position: {}, length: {}, endian: {:?} }}", self.data, self.position, self.length, self.endian)
    }
}


pub struct Writer {
    data: Vec<u8>,
    position: usize,
    length: usize,
    endian: Endian,
}


impl Writer {
    // 初始化一个 Writer 实例
    #[allow(dead_code)]
    pub fn new(endian: Endian) -> Self {
        match endian {
            Endian::Big => {
                Self::new_with_ben()
            }
            Endian::Little => {
                Self::new_with_len()
            }
        }
    }

    // 初始化一个 Writer 实例，指定是否为大端序
    #[allow(dead_code)]
    pub fn new_with_ben() -> Self {
        let mut writer = Self {
            data: vec![],
            position: 0,
            length: 0,
            endian: Endian::Big,
        };
        writer.write_f64(get_start_elapsed_time());
        writer
    }

    // 初始化一个 Writer 实例，指定是否为小端序
    #[allow(dead_code)]
    pub fn new_with_len() -> Self {
        let mut writer = Self {
            data: vec![],
            position: 0,
            length: 0,
            endian: Endian::Little,
        };
        writer.write_f64(get_start_elapsed_time());
        writer
    }

    // 获取数据
    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    // 写入数据
    #[allow(dead_code)]
    pub fn write(&mut self, data: &[u8]) {
        let self_data_len = self.data.len();
        let need_len = self.position + data.len();
        if self_data_len < need_len {
            self.data.reserve(need_len - self_data_len);
        }

        // 扩展 self.data 以确保有足够的空间
        if self_data_len < need_len {
            self.data.resize(need_len, 0);
        }

        // 从 position 开始写入数据
        self.data[self.position..need_len].copy_from_slice(data);

        // 更新 position
        self.position += data.len();
    }

    // 清空数据
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.position = 0;
        self.length = 0;
        self.data.clear();
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
    pub fn compress_var_uint(&mut self, value: u64) {
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

    // 写入字符串
    pub fn write_string(&mut self, value: &str) {
        self.write_u16(value.len() as u16);
        self.write(value.as_bytes());
    }

    // 写入 bool
    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(if value { 1 } else { 0 });
    }
}

impl Debug for Writer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Writer {{ data: {:?}, position: {}, length: {}, endian: {:?} }}", self.data, self.position, self.length, self.endian)
    }
}


pub trait DataReader<T> {
    fn read(reader: &mut Reader) -> T;
}

pub trait DataWriter<T> {
    fn write(&mut self, writer: &mut Writer);
}