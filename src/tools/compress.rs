use nalgebra::{Quaternion, Vector3, Vector4};

pub trait Compress {
    const QUATERNION_MIN_RANGE: f32;
    const QUATERNION_MAX_RANGE: f32;
    const TEN_BITS_MAX: u16;
    fn compress(&self) -> u32;
    fn decompress(&self, data: u32) -> Self;
}

impl Compress for Quaternion<f32> {
    const QUATERNION_MIN_RANGE: f32 = -std::f32::consts::FRAC_1_SQRT_2;
    const QUATERNION_MAX_RANGE: f32 = std::f32::consts::FRAC_1_SQRT_2;
    const TEN_BITS_MAX: u16 = 0b11_1111_1111;
    fn compress(&self) -> u32 {
        let (largest_index, _, mut without_largest) = largest_absolute_component_index(self);

        if self.coords[largest_index] < 0.0 {
            without_largest = -without_largest;
        }

        let a_scaled = scale_float_to_ushort(without_largest.x, Self::QUATERNION_MIN_RANGE, Self::QUATERNION_MAX_RANGE, 0, Self::TEN_BITS_MAX);
        let b_scaled = scale_float_to_ushort(without_largest.y, Self::QUATERNION_MIN_RANGE, Self::QUATERNION_MAX_RANGE, 0, Self::TEN_BITS_MAX);
        let c_scaled = scale_float_to_ushort(without_largest.z, Self::QUATERNION_MIN_RANGE, Self::QUATERNION_MAX_RANGE, 0, Self::TEN_BITS_MAX);

        // 将它们打包到一个整数中
        (largest_index as u32) << 30 | (a_scaled as u32) << 20 | (b_scaled as u32) << 10 | (c_scaled as u32)
    }

    fn decompress(&self, data: u32) -> Self {
        let c_scaled = (data & Self::TEN_BITS_MAX as u32) as u16;
        let b_scaled = ((data >> 10) & Self::TEN_BITS_MAX as u32) as u16;
        let a_scaled = ((data >> 20) & Self::TEN_BITS_MAX as u32) as u16;
        let largest_index = (data >> 30) as usize;

        let a = scale_ushort_to_float(a_scaled, 0, Self::TEN_BITS_MAX, Self::QUATERNION_MIN_RANGE, Self::QUATERNION_MAX_RANGE);
        let b = scale_ushort_to_float(b_scaled, 0, Self::TEN_BITS_MAX, Self::QUATERNION_MIN_RANGE, Self::QUATERNION_MAX_RANGE);
        let c = scale_ushort_to_float(c_scaled, 0, Self::TEN_BITS_MAX, Self::QUATERNION_MIN_RANGE, Self::QUATERNION_MAX_RANGE);

        let d = (1.0 - a * a - b * b - c * c).sqrt();

        let v4 = match largest_index {
            0 => Vector4::new(d, a, b, c),
            1 => Vector4::new(a, d, b, c),
            2 => Vector4::new(a, b, d, c),
            _ => Vector4::new(a, b, c, d),
        };
        quaternion_normalize_safe(Quaternion::from(v4))
    }
}

fn largest_absolute_component_index(q: &Quaternion<f32>) -> (usize, f32, Vector3<f32>) {
    let abs = Quaternion::new(q.w.abs(), q.i.abs(), q.j.abs(), q.k.abs());

    let mut largest_abs = abs.coords.x;
    let mut without_largest = Vector3::new(abs.coords.y, abs.coords.z, abs.coords.w);
    let mut largest_index = 0;

    if abs.coords.y > largest_abs {
        largest_index = 1;
        largest_abs = abs.coords.y;
        without_largest = Vector3::new(abs.coords.x, abs.coords.z, abs.coords.w);
    }
    if abs.coords.z > largest_abs {
        largest_index = 2;
        largest_abs = abs.coords.z;
        without_largest = Vector3::new(abs.coords.x, abs.coords.y, abs.coords.w);
    }
    if abs.coords.w > largest_abs {
        largest_index = 3;
        largest_abs = abs.coords.w;
        without_largest = Vector3::new(abs.coords.x, abs.coords.y, abs.coords.z);
    }
    (largest_index, largest_abs, without_largest)
}

fn scale_float_to_ushort(value: f32, min_value: f32, max_value: f32, min_target: u16, max_target: u16) -> u16 {
    let target_range = max_target as f32 - min_target as f32;
    let value_range = max_value - min_value;
    let value_relative = value - min_value;
    min_target + ((value_relative / value_range) * target_range) as u16
}

fn scale_ushort_to_float(value: u16, min_value: u16, max_value: u16, min_target: f32, max_target: f32) -> f32 {
    let target_range = max_target - min_target;
    let value_range = max_value - min_value;
    let value_relative = value as f32 - min_target;
    min_value as f32 + (value_relative / target_range * value_range as f32)
}

fn quaternion_normalize_safe(q: Quaternion<f32>) -> Quaternion<f32> {
    const FLT_MIN_NORMAL: f32 = 1.175494351e-38;
    let v = Vector4::new(q.coords.x, q.coords.y, q.coords.z, q.coords.w);
    let length = v.dot(&v);
    if length > FLT_MIN_NORMAL {
        q.normalize()
    } else {
        Quaternion::identity()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compress() {
        let q = Quaternion::new(0.0, 0.0, 0.0, 1.0);
        println!("Original:     {:?}", q);
        let compressed = q.compress();
        println!("Compressed:   {}", compressed);
        let decompressed = q.decompress(compressed);
        println!("Decompressed: {:?}", decompressed);
    }
}