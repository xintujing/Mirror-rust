use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// 全局 ID 计数器
static GLOBAL_ID_COUNTER: AtomicU32 = AtomicU32::new(1);
pub fn generate_id() -> u32 {
    GLOBAL_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut next_id = GLOBAL_ID_COUNTER.load(Ordering::Relaxed);
    if next_id > u32::MAX - 1 {
        next_id = 1;
        return next_id;
    }
    next_id
}

// // 获取启动时间
// #[allow(dead_code)]
// pub fn get_s_e_t() -> f64 {
//     get_e_t_f64()
// }
#[allow(dead_code)]
pub fn get_e_t_str(start: Instant) -> String {
    let secs = start.elapsed().as_secs();
    let millis = start.elapsed().subsec_millis();
    format!("{}.{:07}", secs, millis)
}

#[allow(dead_code)]
pub fn get_e_t_f64(start: Instant) -> f64 {
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
        f64_bytes[i] = u8::from_str_radix(
            &format!("{}{}", bytes[i * 2] as char, bytes[i * 2 + 1] as char),
            16,
        ).unwrap();
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

#[allow(dead_code)]
pub fn string_to_ascii(input: &str) -> Vec<u8> {
    input
        .chars()
        .filter(|&c| c.is_ascii()) // 过滤非 ASCII 字符
        .map(|c| c as u8) // 将字符转换为 ASCII 值
        .collect() // 收集到 Vec<u8> 中
}
