use crate::mirror::core::network_reader::NetworkReader;
use crate::mirror::core::tools::pool::Pool;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use tklog::warn;

lazy_static! {
    static ref NETWORK_READER_POOL: Arc<Mutex<Pool<NetworkReader>>> = Arc::new(Mutex::new(Pool::new(|| NetworkReader::new_with_bytes(Vec::new()), 1000)));
}
pub struct NetworkReaderPool;

impl NetworkReaderPool {
    pub fn count() -> usize {
        if let Ok(pool) = NETWORK_READER_POOL.lock() {
            pool.count()
        } else {
            warn!("NetworkReaderPool::count() failed to lock NETWORK_READER_POOL");
            0
        }
    }

    pub fn get() -> NetworkReader {
        if let Ok(mut pool) = NETWORK_READER_POOL.lock() {
            let mut reader = pool.get();
            reader.reset();
            reader
        } else {
            warn!("NetworkReaderPool::get() failed to lock NETWORK_READER_POOL");
            NetworkReader::new_with_bytes(Vec::new())
        }
    }

    pub fn get_return<T>(func: T)
    where
        T: FnOnce(&mut NetworkReader),
    {
        let mut reader = Self::get();
        func(&mut reader);
        Self::return_(reader);
    }

    pub fn get_with_bytes(bytes: Vec<u8>) -> NetworkReader {
        if let Ok(mut pool) = NETWORK_READER_POOL.lock() {
            let mut reader = pool.get();
            reader.reset();
            reader.set_bytes(bytes);
            reader
        } else {
            warn!("NetworkReaderPool::get_with_bytes() failed to lock NETWORK_READER_POOL");
            NetworkReader::new_with_bytes(bytes)
        }
    }

    pub fn get_with_bytes_return<T>(bytes: Vec<u8>, func: T)
    where
        T: FnOnce(&mut NetworkReader),
    {
        let mut reader = Self::get_with_bytes(bytes);
        func(&mut reader);
        Self::return_(reader);
    }

    pub fn get_with_array_segment(array_segment: &[u8]) -> NetworkReader {
        if let Ok(mut pool) = NETWORK_READER_POOL.lock() {
            let mut reader = pool.get();
            reader.reset();
            reader.set_array_segment(array_segment);
            reader
        } else {
            warn!("NetworkReaderPool::get_with_array_segment() failed to lock NETWORK_READER_POOL");
            NetworkReader::new_with_bytes(array_segment.to_vec())
        }
    }

    pub fn get_with_array_segment_return<T>(array_segment: &[u8], func: T)
    where
        T: FnOnce(&mut NetworkReader),
    {
        let mut reader = Self::get_with_array_segment(array_segment);
        func(&mut reader);
        Self::return_(reader);
    }

    pub fn return_(mut reader: NetworkReader) {
        if let Ok(mut pool) = NETWORK_READER_POOL.lock() {
            reader.reset();
            pool.return_(reader);
        } else {
            warn!("NetworkReaderPool::return_() failed to lock NETWORK_READER_POOL");
        }
    }
}