use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
pub fn compress_var_uint(value: u64) -> Vec<u8> {
    let mut buffer = Vec::new();

    if value <= 240 {
        buffer.push(value as u8);
    } else if value <= 2287 {
        let a = ((value - 240) >> 8) as u8 + 241;
        let b = (value - 240) as u8;
        buffer.push(a);
        buffer.push(b);
    } else if value <= 67823 {
        let a = 249;
        let b = ((value - 2288) >> 8) as u8;
        let c = (value - 2288) as u8;
        buffer.push(a);
        buffer.push(b);
        buffer.push(c);
    } else if value <= 16_777_215 {
        let a = 250;
        let b = value as u32;
        let bytes = b.to_le_bytes();
        buffer.push(a);
        buffer.extend_from_slice(&bytes[0..3]);  // 只写入低 3 字节
    } else if value <= 4_294_967_295 {
        let a = 251;
        let b = value as u32;
        buffer.push(a);
        buffer.extend_from_slice(&b.to_le_bytes());
    } else if value <= 1_099_511_627_775 {
        let a = 252;
        let b = (value & 0xFF) as u8;
        let c = (value >> 8) as u32;
        buffer.push(a);
        buffer.push(b);
        buffer.extend_from_slice(&c.to_le_bytes());
    } else if value <= 281_474_976_710_655 {
        let a = 253;
        let b = (value & 0xFF) as u8;
        let c = ((value >> 8) & 0xFF) as u8;
        let d = (value >> 16) as u32;
        buffer.push(a);
        buffer.push(b);
        buffer.push(c);
        buffer.extend_from_slice(&d.to_le_bytes());
    } else if value <= 72_057_594_037_927_935 {
        let a = 254;
        let b = value;
        let bytes = b.to_le_bytes();
        buffer.push(a);
        buffer.extend_from_slice(&bytes[0..7]);  // 只写入低 7 字节
    } else {
        buffer.push(255);
        buffer.extend_from_slice(&value.to_le_bytes());
    }

    buffer
}

#[allow(dead_code)]
pub fn decompress_var_uint(reader: &[u8]) -> (u64, usize) {
    let a0 = reader[0];
    if a0 < 241 {
        return (u64::from(a0), 1);
    }

    let a1 = reader[1];
    if a0 <= 248 {
        return (240 + ((u64::from(a0) - 241) << 8) + u64::from(a1), 2);
    }

    let a2 = reader[2];
    if a0 == 249 {
        return (2288 + (u64::from(a1) << 8) + u64::from(a2), 3);
    }

    let a3 = reader[3];
    if a0 == 250 {
        return (u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16), 4);
    }

    let a4 = reader[4];
    if a0 == 251 {
        return (u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16) + (u64::from(a4) << 24), 5);
    }

    let a5 = reader[5];
    if a0 == 252 {
        return (u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16) + (u64::from(a4) << 24) + (u64::from(a5) << 32), 6);
    }

    let a6 = reader[6];
    if a0 == 253 {
        return (u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16) + (u64::from(a4) << 24) + (u64::from(a5) << 32) + (u64::from(a6) << 40), 7);
    }

    let a7 = reader[7];
    if a0 == 254 {
        return (u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16) + (u64::from(a4) << 24) + (u64::from(a5) << 32) + (u64::from(a6) << 40) + (u64::from(a7) << 48), 8);
    }

    let a8 = reader[8];
    if a0 == 255 {
        return (u64::from(a1) + (u64::from(a2) << 8) + (u64::from(a3) << 16) + (u64::from(a4) << 24) + (u64::from(a5) << 32) + (u64::from(a6) << 40) + (u64::from(a7) << 48) + (u64::from(a8) << 56), 9);
    }
    (0, 0)
}

#[allow(dead_code)]
pub fn get_elapsed_time(start: Instant) -> String {
    let secs = start.elapsed().as_secs();
    let millis = start.elapsed().subsec_millis();
    format!("{}.{:03}", secs, millis)
}

#[allow(dead_code)]
pub fn get_elapsed_time_f64(start: Instant) -> f64 {
    start.elapsed().as_micros() as f64 / 1_000_000.0
}

// 获取时间戳
#[allow(dead_code)]
pub fn get_timestamp() -> String {
    // 获取自 Unix 纪元以来的持续时间
    let now = SystemTime::now();
    match now.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            // 将持续时间分解为秒和毫秒
            let secs = duration.as_secs();
            let millis = duration.subsec_millis();
            format!("{}.{:03}", secs, millis)
        }
        Err(_) => String::from("Time before Unix epoch"),
    }
}

#[allow(dead_code)]
pub fn get_sec_timestamp_f64() -> f64 {
    // 获取自 Unix 纪元以来的持续时间
    let now = SystemTime::now();
    match now.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            // 将持续时间分解为秒和毫秒
            duration.as_micros() as f64 / 1_000_000.0
        }
        Err(_) => 0.0,
    }
}

#[allow(dead_code)]
pub fn sleep_ms(ms: u64) {
    sleep(Duration::from_millis(ms));
}

#[allow(dead_code)]
pub fn to_hex_string(data: &[u8]) -> String {
    let mut hex_string = String::new();
    for byte in data {
        hex_string.push_str(&format!("{:02X}", byte));
    }
    hex_string
}
#[allow(dead_code)]
pub fn hex_string_to_f64(hex_string: &str) -> f64 {
    let hex_string = hex_string.trim();
    let hex_string = if hex_string.starts_with("0x") {
        &hex_string[2..]
    } else {
        hex_string
    };
    let hex_string = if hex_string.len() % 2 == 1 {
        format!("0{}", hex_string)
    } else {
        hex_string.to_string()
    };
    let bytes = hex_string.as_bytes();
    let mut f64_bytes = [0u8; 8];
    for i in 0..8 {
        f64_bytes[i] = u8::from_str_radix(&format!("{}{}", bytes[i * 2] as char, bytes[i * 2 + 1] as char), 16).unwrap();
    }
    f64::from_be_bytes(f64_bytes)
}
#[allow(dead_code)]
pub fn bytes_to_f64(bytes: &[u8]) -> f64 {
    let mut f64_bytes = [0u8; 8];
    // 倒序
    for i in 0..8 {
        f64_bytes[i] = bytes[7 - i];
    }
    f64::from_be_bytes(f64_bytes)
}

#[allow(dead_code)]
pub fn bytes_to_f32(bytes: &[u8]) -> f32 {
    let mut f32_bytes = [0u8; 4];
    // 倒序
    for i in 0..4 {
        f32_bytes[i] = bytes[3 - i];
    }
    f32::from_be_bytes(f32_bytes)
}

#[allow(dead_code)]
pub fn bytes_to_u16(bytes: &[u8]) -> u16 {
    let mut u16_bytes = [0u8; 2];
    // 倒序
    for i in 0..2 {
        u16_bytes[i] = bytes[1 - i];
    }
    u16::from_be_bytes(u16_bytes)
}

#[allow(dead_code)]
pub fn bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut u32_bytes = [0u8; 4];
    // 倒序
    for i in 0..4 {
        u32_bytes[i] = bytes[3 - i];
    }
    u32::from_be_bytes(u32_bytes)
}