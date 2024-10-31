use crate::core::batching::batcher::Batcher;
use crate::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::core::network_writer::NetworkWriter;
use crate::core::network_writer_pool::NetworkWriterPool;
use std::collections::VecDeque;

pub struct UnBatcher {
    un_batches: VecDeque<NetworkWriter>,
    un_batcher: NetworkReader,
    un_batch_timestamp: f64,
}

impl UnBatcher {
    pub fn new() -> UnBatcher {
        UnBatcher {
            un_batches: VecDeque::new(),
            un_batcher: NetworkReader::new_with_bytes(Vec::new()),
            un_batch_timestamp: 0.0,
        }
    }
    pub fn batches_count(&self) -> usize {
        self.un_batches.len()
    }
    pub fn add_batch_with_array_segment(&mut self, un_batch: &[u8]) -> bool {
        if un_batch.len() < Batcher::TIMESTAMP_SIZE {
            return false;
        }
        let mut writer = NetworkWriterPool::get();
        writer.write_array_segment_all(un_batch);

        if self.un_batches.is_empty() {
            self.un_batcher.set_array_segment(writer.to_array_segment());
            self.un_batch_timestamp = self.un_batcher.read_double();
        }
        self.un_batches.push_back(writer);
        true
    }

    pub fn add_batch_with_bytes(&mut self, un_batch: Vec<u8>) -> bool {
        if un_batch.len() < Batcher::TIMESTAMP_SIZE {
            return false;
        }
        let mut writer = NetworkWriterPool::get();
        writer.write_bytes_all(un_batch);

        if self.un_batches.is_empty() {
            self.un_batcher.set_array_segment(writer.to_array_segment());
            self.un_batch_timestamp = self.un_batcher.read_double();
        }
        self.un_batches.push_back(writer);
        true
    }

    pub fn get_next_message(&mut self) -> Option<(&[u8], f64)> {
        let mut message: &[u8] = &[];
        let mut remote_time_stamp = 0.0;
        if self.un_batches.is_empty() {
            return None;
        }

        if self.un_batcher.capacity() == 0 {
            return None
        }

        if self.un_batcher.remaining() == 0 {
            if let Some(write) = self.un_batches.pop_front() {
                NetworkWriterPool::return_(write);
            }

            if let Some(next) = self.un_batches.front() {
                self.un_batcher.set_array_segment(next.to_array_segment());
                self.un_batch_timestamp = self.un_batcher.read_double();
            } else {
                return None
            }
        }

        remote_time_stamp = self.un_batch_timestamp;

        if self.un_batcher.remaining() == 0 {
            return None
        }

        let size = self.un_batcher.decompress_var_uint() as usize;

        if self.un_batcher.remaining() < size {
            return None
        }

        message = self.un_batcher.read_array_segment(size);

        Some((message, remote_time_stamp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::network_writer::NetworkWriterTrait;

    #[test]
    fn test_un_batcher() {
        let mut un_batcher = UnBatcher::new();
        let mut batch = Vec::new();
        let mut batch_writer = NetworkWriter::new();

        batch_writer.write_double(0.1);
        batch_writer.compress_var_uint(5);
        batch_writer.write_array_segment_all(&[1, 2, 3, 4, 5]);
        batch.extend_from_slice(&batch_writer.to_array_segment());

        assert_eq!(un_batcher.batches_count(), 0);
        assert_eq!(un_batcher.add_batch_with_array_segment(&batch), true);
        assert_eq!(un_batcher.add_batch_with_array_segment(&batch), true);
        assert_eq!(un_batcher.batches_count(), 2);

        while let Some((message, remote_time_stamp))= un_batcher.get_next_message() {
            println!("Message: {:?}, Remote Time Stamp: {}", message, remote_time_stamp);
        }
        println!("Batches Count: {}", un_batcher.batches_count());
    }
}