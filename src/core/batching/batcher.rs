use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::core::network_writer_pool::NetworkWriterPool;
use crate::core::tools::compress;
use std::collections::VecDeque;

pub struct Batcher {
    threshold: usize,
    batches: VecDeque<NetworkWriter>,
    batcher: Option<NetworkWriter>,
}

impl Batcher {
    pub const TIMESTAMP_SIZE: usize = size_of::<f64>();

    pub fn new(threshold: usize) -> Self {
        Self {
            threshold,
            batches: VecDeque::new(),
            batcher: None,
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

        if let Some(batcher) = self.batcher.take() {
            if batcher.get_position() + needed_size > self.threshold {
                self.batches.push_back(batcher);
            } else {
                self.batcher = Some(batcher);
            }
        }

        if self.batcher.is_none() {
            let mut batcher = NetworkWriterPool::get();
            batcher.write_double(timestamp);
            self.batcher = Some(batcher);
        }

        if let Some(ref mut batcher) = self.batcher {
            // batcher.compress_var_uint(message.len() as u64);
            batcher.write_bytes(message, 0, message.len());
        }
    }

    pub fn get_batcher_writer(&mut self, writer: &mut NetworkWriter) -> bool {
        if let Some(batcher) = self.batches.pop_front() {
            Self::copy_and_return_batcher(batcher, writer);
            return true;
        }
        if let Some(batcher) = self.batcher.take() {
            Self::copy_and_return_batcher(batcher, writer);
            return true;
        }
        false
    }

    fn copy_and_return_batcher(batcher: NetworkWriter, writer: &mut NetworkWriter) {
        if writer.get_position() != 0 {
            panic!("Writer must be empty");
        }
        let segment = batcher.to_array_segment();
        writer.write_bytes(segment, 0, segment.len());
        NetworkWriterPool::return_(batcher);
    }

    pub fn clear(&mut self) {
        if let Some(batcher) = self.batcher.take() {
            NetworkWriterPool::return_(batcher);
        }
        for batcher in self.batches.drain(..) {
            NetworkWriterPool::return_(batcher);
        }
    }
}
