use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::core::network_writer_pool::NetworkWriterPool;
use crate::core::tools::compress;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Batcher {
    threshold: usize,
    batches: VecDeque<NetworkWriter>,
    batcher: NetworkWriter,
}

impl Batcher {
    pub const TIMESTAMP_SIZE: usize = size_of::<f64>();

    pub fn new(threshold: usize) -> Self {
        Self {
            threshold,
            batches: VecDeque::new(),
            batcher: NetworkWriter::new(),
        }
    }

    pub fn message_header_size(message_size: usize) -> usize {
        compress::var_uint_size(message_size as u64)
    }

    pub fn max_message_overhead(message_size: usize) -> usize {
        Self::TIMESTAMP_SIZE + Self::message_header_size(message_size)
    }

    pub fn add_message(&mut self, message: &[u8], timestamp: f64) {
        let header_size = compress::var_uint_size(message.len() as u64);
        let needed_size = header_size + message.len();

        if self.batcher.get_position() + needed_size > self.threshold {
            let mut batcher = NetworkWriterPool::get();
            self.copy_and_reset_batcher(&mut batcher);
            self.batches.push_back(batcher);
        }

        if self.batcher.get_position() == 0 {
            self.batcher.write_double(timestamp);
        }

        self.batcher.write_bytes(message, 0, message.len());
    }

    pub fn get_batcher_writer(&mut self, writer: &mut NetworkWriter) -> bool {
        if let Some(batcher) = self.batches.pop_front() {
            Self::copy_and_return_batcher(batcher, writer);
            return true;
        }

        if self.batcher.get_position() > Self::TIMESTAMP_SIZE {
            self.copy_and_reset_batcher(writer);
            return true;
        }
        false
    }

    fn copy_and_reset_batcher(&mut self, writer: &mut NetworkWriter) {
        writer.set_position(0);
        let segment = self.batcher.to_array_segment();
        writer.write_bytes(segment, 0, segment.len());
        self.batcher.set_position(0);
    }

    fn copy_and_return_batcher(batcher: NetworkWriter, writer: &mut NetworkWriter) {
        writer.set_position(0);
        let segment = batcher.to_array_segment();
        writer.write_bytes(segment, 0, segment.len());
        NetworkWriterPool::return_(batcher);
    }

    pub fn clear(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batcher() {
        let mut stack = VecDeque::new();
        stack.push_back(1);
        stack.push_back(2);
        stack.push_back(3);

        println!("{:?}", stack.pop_front());
        println!("{:?}", stack.pop_back());
    }
}