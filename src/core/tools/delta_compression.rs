use crate::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use nalgebra::{Vector3, Vector4};

pub struct DeltaCompression;

impl DeltaCompression {
    pub fn compress_long(writer: &mut NetworkWriter, last: i64, current: i64) {
        writer.compress_var_int(current - last);
    }
    pub fn decompress_long(reader: &mut NetworkReader, last: i64) -> i64 {
        last + reader.decompress_var_int()
    }
    pub fn compress_vector3long(writer: &mut NetworkWriter, last: Vector3<i64>, current: Vector3<i64>) {
        Self::compress_long(writer, last.x, current.x);
        Self::compress_long(writer, last.y, current.y);
        Self::compress_long(writer, last.z, current.z);
    }
    pub fn decompress_vector3long(reader: &mut NetworkReader, last: Vector3<i64>) -> Vector3<i64> {
        Vector3::new(
            Self::decompress_long(reader, last.x),
            Self::decompress_long(reader, last.y),
            Self::decompress_long(reader, last.z),
        )
    }
    pub fn compress_vector4long(writer: &mut NetworkWriter, last: Vector4<i64>, current: Vector4<i64>) {
        Self::compress_long(writer, last.x, current.x);
        Self::compress_long(writer, last.y, current.y);
        Self::compress_long(writer, last.z, current.z);
        Self::compress_long(writer, last.w, current.w);
    }
    pub fn decompress_vector4long(reader: &mut NetworkReader, last: Vector4<i64>) -> Vector4<i64> {
        Vector4::new(
            Self::decompress_long(reader, last.x),
            Self::decompress_long(reader, last.y),
            Self::decompress_long(reader, last.z),
            Self::decompress_long(reader, last.w),
        )
    }
}