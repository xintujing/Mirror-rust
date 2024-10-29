use crate::core::network_reader::{NetworkReader, NetworkReaderTrait, Readable};
use nalgebra::{Quaternion, Vector2, Vector3, Vector4};

pub struct NetworkReaderExtensions;
impl NetworkReaderExtensions {
    fn read_string(reader: &mut NetworkReader) -> String {
        let length = reader.read_blittable::<u16>() as usize;
        let bytes = reader.read_bytes(length);
        String::from_utf8(bytes).unwrap()
    }
}
impl NetworkReaderTrait for NetworkReader {
    fn read_byte(&mut self) -> u8 {
        self.read_blittable::<u8>()
    }

    fn read_byte_nullable(&mut self) -> Option<u8> {
        self.read_blittable_nullable::<u8>()
    }

    fn read_sbyte(&mut self) -> i8 {
        self.read_blittable::<i8>()
    }

    fn read_sbyte_nullable(&mut self) -> Option<i8> {
        self.read_blittable_nullable::<i8>()
    }

    fn read_char(&mut self) -> char {
        self.read_blittable::<char>()
    }

    fn read_char_nullable(&mut self) -> Option<char> {
        self.read_blittable_nullable::<char>()
    }

    fn read_bool(&mut self) -> bool {
        self.read_blittable::<bool>()
    }

    fn read_bool_nullable(&mut self) -> Option<bool> {
        self.read_blittable_nullable::<bool>()
    }

    fn read_short(&mut self) -> i16 {
        self.read_blittable::<i16>()
    }

    fn read_short_nullable(&mut self) -> Option<i16> {
        self.read_blittable_nullable::<i16>()
    }

    fn read_ushort(&mut self) -> u16 {
        self.read_blittable::<u16>()
    }

    fn read_ushort_nullable(&mut self) -> Option<u16> {
        self.read_blittable_nullable::<u16>()
    }

    fn read_int(&mut self) -> i32 {
        self.read_blittable::<i32>()
    }

    fn read_int_nullable(&mut self) -> Option<i32> {
        self.read_blittable_nullable::<i32>()
    }

    fn read_uint(&mut self) -> u32 {
        self.read_blittable::<u32>()
    }

    fn read_uint_nullable(&mut self) -> Option<u32> {
        self.read_blittable_nullable::<u32>()
    }

    fn read_long(&mut self) -> i64 {
        self.read_blittable::<i64>()
    }

    fn read_long_nullable(&mut self) -> Option<i64> {
        self.read_blittable_nullable::<i64>()
    }

    fn read_ulong(&mut self) -> u64 {
        self.read_blittable::<u64>()
    }

    fn read_ulong_nullable(&mut self) -> Option<u64> {
        self.read_blittable_nullable::<u64>()
    }

    fn read_float(&mut self) -> f32 {
        self.read_blittable::<f32>()
    }

    fn read_float_nullable(&mut self) -> Option<f32> {
        self.read_blittable_nullable::<f32>()
    }

    fn read_double(&mut self) -> f64 {
        self.read_blittable::<f64>()
    }

    fn read_double_nullable(&mut self) -> Option<f64> {
        self.read_blittable_nullable::<f64>()
    }

    fn read_str(&mut self) -> String {
        todo!()
    }

    fn read_string(&mut self) -> String {
        todo!()
    }

    fn read_bytes_and_size(&mut self) -> Vec<u8> {
        todo!()
    }

    fn read_vector2(&mut self) -> Vector2<f32> {
        todo!()
    }

    fn read_vector2_nullable(&mut self) -> Option<Vector2<f32>> {
        todo!()
    }

    fn read_vector3(&mut self) -> Vector3<f32> {
        todo!()
    }

    fn read_vector3_nullable(&mut self) -> Option<Vector3<f32>> {
        todo!()
    }

    fn read_vector4(&mut self) -> Vector4<f32> {
        todo!()
    }

    fn read_vector4_nullable(&mut self) -> Option<Vector4<f32>> {
        todo!()
    }

    fn read_quaternion(&mut self) -> Quaternion<f32> {
        todo!()
    }

    fn read_quaternion_nullable(&mut self) -> Option<Quaternion<f32>> {
        todo!()
    }

    fn decompress_var_uint(&mut self) -> u64 {
        todo!()
    }
}

impl Readable for String {
    fn get_reader<T>() -> Option<fn(&mut NetworkReader) -> T>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Readable for &str {
    fn get_reader<T>() -> Option<fn(&mut NetworkReader) -> T>
    where
        Self: Sized,
    {
        todo!()
    }
}