use crate::core::network_writer_pooled::NetworkWriterPooled;
use crate::core::tools::pool::Pool;
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref NETWORK_WRITER_POOL: Arc<Mutex<Pool<NetworkWriterPooled>>> = {
        Arc::new(Mutex::new(Pool::new(|| NetworkWriterPooled::new(), 1000)))
    };
}


pub struct NetworkWriterPool;

impl NetworkWriterPool {
    pub fn count() -> usize {
        NETWORK_WRITER_POOL.lock().unwrap().count()
    }

    pub fn get() -> NetworkWriterPooled {
        let mut pool = NETWORK_WRITER_POOL.lock().unwrap();
        let mut writer = pool.get();
        writer.reset();
        writer
    }

    #[inline(always)]
    pub fn return_(writer: NetworkWriterPooled) {
        NETWORK_WRITER_POOL.lock().unwrap().return_(writer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_writer_pool() {
        // Initial count should be 1000
        assert_eq!(NetworkWriterPool::count(), 1000);

        // Get 5 writers from the pool
        let mut writers = Vec::new();
        for _ in 0..5 {
            let writer = NetworkWriterPool::get();
            writers.push(writer);
        }

        // Count should be 995 now
        assert_eq!(NetworkWriterPool::count(), 995);

        // Return all writers to the pool
        for writer in writers {
            writer.dispose();
        }

        // Count should be back to 1000
        assert_eq!(NetworkWriterPool::count(), 1000);
    }
}