use crate::mirror::core::network_reader::NetworkReader;
use crate::mirror::core::network_writer::NetworkWriter;
use std::any::Any;
use std::fmt::Debug;

pub trait SyncObject: Any + Send + Sync + Debug {
    fn sub_class_name() -> &'static str
    where
        Self: Sized;
    fn is_recording(&self) -> bool {
        true
    }
    fn is_writable(&self) -> bool {
        true
    }
    fn clear_changes(&mut self);
    fn on_serialize_all(&self, writer: &mut NetworkWriter);
    fn on_serialize_delta(&self, writer: &mut NetworkWriter);
    fn on_deserialize_all(&mut self, reader: &mut NetworkReader) -> bool;
    fn on_deserialize_delta(&mut self, reader: &mut NetworkReader) -> bool;
    fn reset(&mut self);
}
