use bytes::{BufMut, BytesMut};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct Batcher {
    threshold: usize,
    batches: Mutex<VecDeque<Arc<BytesMut>>>,
    current_batch: Mutex<Option<Arc<BytesMut>>>,
}

impl Batcher {
    const TIMESTAMP_SIZE: usize = std::mem::size_of::<f64>();

    pub fn new(threshold: usize) -> Self {
        Batcher {
            threshold,
            batches: Mutex::new(VecDeque::new()),
            current_batch: Mutex::new(None),
        }
    }

    fn message_header_size(message_size: usize) -> usize {
        // Placeholder for actual varint size calculation.
        // Rust equivalent of C# Compression.VarUIntSize should be implemented.
        1 + message_size
    }

    pub fn max_message_overhead(message_size: usize) -> usize {
        Self::TIMESTAMP_SIZE + Self::message_header_size(message_size)
    }

    pub fn add_message(&self, message: &[u8], timestamp: f64) {
        let mut batch_lock = self.current_batch.lock().unwrap();
        let mut batches_lock = self.batches.lock().unwrap();

        let header_size = Self::message_header_size(message.len());
        let needed_size = header_size + message.len();

        if let Some(batch) = batch_lock.as_ref() {
            if batch.len() + needed_size > self.threshold {
                batches_lock.push_back(batch.clone());
                *batch_lock = None;
            }
        }

        if batch_lock.is_none() {
            let mut new_batch = BytesMut::with_capacity(self.threshold);
            new_batch.put_f64_le(timestamp); // Writing timestamp first.
            *batch_lock = Some(Arc::new(new_batch));
        }

        let batch = Arc::get_mut(batch_lock.as_mut().unwrap()).unwrap();
        batch.put_u64_le(message.len() as u64); // Include size prefix as varint.
        batch.put_slice(message);
    }

    pub fn get_batch(&self, writer: &mut Vec<u8>) -> bool {
        let mut batch_lock = self.current_batch.lock().unwrap();
        let mut batches_lock = self.batches.lock().unwrap();

        if let Some(first) = batches_lock.pop_front() {
            writer.extend_from_slice(&first);
            return true;
        }

        if let Some(batch) = batch_lock.take() {
            writer.extend_from_slice(&batch);
            return true;
        }

        false
    }

    pub fn clear(&self) {
        let mut batch_lock = self.current_batch.lock().unwrap();
        let mut batches_lock = self.batches.lock().unwrap();

        *batch_lock = None;
        batches_lock.clear();
    }
}
