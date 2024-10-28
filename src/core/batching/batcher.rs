use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::core::network_writer_pool::NetworkWriterPooledPool;
use crate::core::network_writer_pooled::NetworkWriterPooled;
use crate::core::tools::compress;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Batcher {
    threshold: usize,
    batches: VecDeque<NetworkWriterPooled>,
    batch: Option<NetworkWriterPooled>,
}

impl Batcher {
    pub const TIMESTAMP_SIZE: usize = size_of::<f64>();

    pub fn new(threshold: usize) -> Self {
        Self {
            threshold,
            batches: VecDeque::new(),
            batch: None,
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

        if let Some(ref mut batch) = self.batch {
            if batch.get_network_writer().get_position() + needed_size > self.threshold {
                self.batches.push_back(batch.clone());
                self.batch = None;
            }
        }

        if self.batch.is_none() {
            let mut new_batch = NetworkWriterPooledPool::get();
            new_batch.get_network_writer().write_double(timestamp);
            self.batch = Some(new_batch);
        }

        if let Some(ref mut batch) = self.batch {
            batch.get_network_writer().compress_var_uint(message.len() as u64);
            batch.get_network_writer().write_bytes(message, 0, message.len());
        }
    }

    fn copy_and_return(mut batch: NetworkWriterPooled, writer: &mut NetworkWriter) {
        if writer.get_position() != 0 {
            panic!("GetBatch needs a fresh writer!");
        }

        let segment = batch.get_network_writer().to_array_segment();
        writer.write_bytes(segment, 0, segment.len());

        NetworkWriterPooledPool::return_(batch);
    }

    pub fn get_batch(&mut self, writer: &mut NetworkWriter) -> bool {
        if let Some(first) = self.batches.pop_front() {
            Self::copy_and_return(first, writer);
            return true;
        }

        if let Some(batch) = self.batch.take() {
            Self::copy_and_return(batch, writer);
            return true;
        }

        false
    }

    pub fn clear(&mut self) {
        if let Some(batch) = self.batch.take() {
            NetworkWriterPooledPool::return_(batch);
        }

        for queued in self.batches.drain(..) {
            NetworkWriterPooledPool::return_(queued);
        }
    }
}