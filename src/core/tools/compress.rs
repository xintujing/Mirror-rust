use nalgebra::{Quaternion, Vector3, Vector4};

const QUATERNION_MIN_RANGE: f32 = -std::f32::consts::FRAC_1_SQRT_2;
const QUATERNION_MAX_RANGE: f32 = std::f32::consts::FRAC_1_SQRT_2;
const TEN_BITS_MAX: u16 = 1023;
pub trait CompressTrait {
    fn compress(&self) -> u32;
}

pub trait DecompressTrait {
    fn decompress(compressed: u32) -> Self;
}

impl CompressTrait for Quaternion<f32> {
    fn compress(&self) -> u32 {
        let (largest_index, _, mut without_largest) = largest_absolute_component_index(self);

        if self[largest_index] < 0.0 {
            without_largest = -without_largest;
        }

        let a_scaled = scale_float_to_ushort(without_largest.x, QUATERNION_MIN_RANGE, QUATERNION_MAX_RANGE, 0, TEN_BITS_MAX);
        let b_scaled = scale_float_to_ushort(without_largest.y, QUATERNION_MIN_RANGE, QUATERNION_MAX_RANGE, 0, TEN_BITS_MAX);
        let c_scaled = scale_float_to_ushort(without_largest.z, QUATERNION_MIN_RANGE, QUATERNION_MAX_RANGE, 0, TEN_BITS_MAX);

        // 将它们打包到一个整数中
        (largest_index as u32) << 30 | (a_scaled as u32) << 20 | (b_scaled as u32) << 10 | (c_scaled as u32)
    }
}

impl DecompressTrait for Quaternion<f32> {
    fn decompress(data: u32) -> Self {
        let c_scaled = ((data >> 00) & TEN_BITS_MAX as u32) as u16;
        let b_scaled = ((data >> 10) & TEN_BITS_MAX as u32) as u16;
        let a_scaled = ((data >> 20) & TEN_BITS_MAX as u32) as u16;
        let largest_index = data >> 30;

        let a = scale_ushort_to_float(a_scaled, 0, TEN_BITS_MAX, QUATERNION_MIN_RANGE, QUATERNION_MAX_RANGE);
        let b = scale_ushort_to_float(b_scaled, 0, TEN_BITS_MAX, QUATERNION_MIN_RANGE, QUATERNION_MAX_RANGE);
        let c = scale_ushort_to_float(c_scaled, 0, TEN_BITS_MAX, QUATERNION_MIN_RANGE, QUATERNION_MAX_RANGE);

        let d = (1.0 - a * a - b * b - c * c).max(0.0).sqrt();

        match largest_index {
            0 => quaternion_normalize_safe(Vector4::new(d, a, b, c)),
            1 => quaternion_normalize_safe(Vector4::new(a, d, b, c)),
            2 => quaternion_normalize_safe(Vector4::new(a, b, d, c)),
            _ => quaternion_normalize_safe(Vector4::new(a, b, c, d)),
        }
    }
}

fn largest_absolute_component_index(q: &Quaternion<f32>) -> (usize, f32, Vector3<f32>) {
    let abs = Vector4::new(q.i.abs(), q.j.abs(), q.k.abs(), q.w.abs());

    let mut largest_abs = abs.x;
    let mut without_largest = Vector3::new(q.j, q.k, q.w);
    let mut largest_index = 0;

    if abs.y > largest_abs {
        largest_index = 1;
        largest_abs = abs.y;
        without_largest = Vector3::new(q.i, q.k, q.w);
    }
    if abs.z > largest_abs {
        largest_index = 2;
        largest_abs = abs.z;
        without_largest = Vector3::new(q.i, q.j, q.w);
    }
    if abs.w > largest_abs {
        largest_index = 3;
        largest_abs = abs.w;
        without_largest = Vector3::new(q.i, q.j, q.k);
    }
    (largest_index, largest_abs, without_largest)
}

fn scale_float_to_ushort(value: f32, min_value: f32, max_value: f32, min_target: u16, max_target: u16) -> u16 {
    let target_range = (max_target - min_target) as f32;
    let value_range = max_value - min_value;
    let value_relative = value - min_value;
    min_target + ((value_relative / value_range) * target_range) as u16
}

fn scale_ushort_to_float(value: u16, min_value: u16, max_value: u16, min_target: f32, max_target: f32) -> f32 {
    let target_range = max_target - min_target;
    let value_range = max_value - min_value;
    let value_relative = value - min_value;
    min_target + ((value_relative / value_range) as f32 * target_range)
}

fn quaternion_normalize_safe(v4: Vector4<f32>) -> Quaternion<f32> {
    const FLT_MIN_NORMAL: f32 = 1.175494351e-38;
    let length_sq = v4.dot(&v4);
    if length_sq > FLT_MIN_NORMAL {
        let length = length_sq.sqrt();
        let normalized = v4 / length;
        Quaternion::new(normalized.w, normalized.x, normalized.y, normalized.z)
    } else {
        Quaternion::identity()
    }
}

pub fn scale_to_long_0(value: Vector3<f32>, precision: f32) -> (bool, Vector3<i64>) {
    let mut quantized = Vector3::new(0, 0, 0);
    let (result, x, y, z) = scale_to_long_1(value, precision);
    quantized.x = x;
    quantized.y = y;
    quantized.z = z;
    (result, quantized)
}

pub fn scale_to_long_1(value: Vector3<f32>, precision: f32) -> (bool, i64, i64, i64) {
    let mut result = true;
    let (res, x) = scale_to_long_2(value.x, precision);
    result &= res;
    let (res, y) = scale_to_long_2(value.y, precision);
    result &= res;
    let (res, z) = scale_to_long_2(value.z, precision);
    result &= res;
    (result, x, y, z)
}

pub fn scale_to_long_2(value: f32, precision: f32) -> (bool, i64) {
    if precision == 0.0 {
        panic!("precision cannot be 0");
    }
    let quantized = (value / precision).round() as i64;
    (true, quantized)
}

pub fn var_uint_size(value: u64) -> usize {
    if value <= 240 {
        return 1;
    }
    if value <= 2287 {
        return 2;
    }
    if value <= 67823 {
        return 3;
    }
    if value <= 16777215 {
        return 4;
    }
    if value <= 4294967295 {
        return 5;
    }
    if value <= 1099511627775 {
        return 6;
    }
    if value <= 281474976710655 {
        return 7;
    }
    if value <= 72057594037927935 {
        return 8;
    }
    9
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compress() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        println!("Original:     {:?}", q);
        let compressed = q.compress();
        println!("Compressed:   {}", compressed);
        let decompressed = Quaternion::decompress(compressed);
        println!("Decompressed: {:?}", decompressed);
    }
}