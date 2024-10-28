use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::core::network_writer_pool::NetworkWriterPool;
use crate::core::tools::compress;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Batcher {
    threshold: usize,
    batches: VecDeque<NetworkWriter>,
    batch: Option<NetworkWriter>,
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

        if let Some(batch) = self.batch.take() {
            if batch.get_position() + needed_size > self.threshold {
                self.batches.push_back(batch);
            } else {
                self.batch = Some(batch);
            }
        }

        if self.batch.is_none() {
            let mut new_batch = NetworkWriterPool::get();
            new_batch.write_double(timestamp);
            self.batch = Some(new_batch);
        }

        if let Some(ref mut batch) = self.batch {
            // batch.get_network_writer().compress_var_uint(message.len() as u64);
            println!("if let Some(ref mut batch) = self.batch: {} {}", message.len(), batch.get_data().len());
            batch.write_bytes(message, 0, message.len());
        }
    }

    fn copy_and_return(mut batch: NetworkWriter, writer: &mut NetworkWriter) {
        if writer.get_position() != 0 {
            panic!("GetBatch needs a fresh writer!");
        }

        let segment = batch.to_array_segment();
        writer.write_bytes(segment, 0, segment.len());

        NetworkWriterPool::return_(batch);
    }

    pub fn get_batch(&mut self, writer: &mut NetworkWriter) -> bool {
        if let Some(first) = self.batches.pop_front() {
            println!("xxxxxxxxxxx1");
            Self::copy_and_return(first, writer);
            return true;
        }

        println!("xxxxxxxxxxx3 {}", self.batch.is_some());


        if let Some(mut batch) = self.batch.take() {
            println!("xxxxxxxxxxx2");
            Self::copy_and_return(batch.clone(), writer);
            batch.reset();
            return true;
        }

        false
    }

    pub fn clear(&mut self) {
        if let Some(batch) = self.batch.take() {
            NetworkWriterPool::return_(batch);
        }

        for queued in self.batches.drain(..) {
            NetworkWriterPool::return_(queued);
        }
    }
}