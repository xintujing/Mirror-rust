use crate::core::network_writer::NetworkWriter;
use crate::core::tools::pool::Pool;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref NETWORK_WRITER_POOL: Arc<Mutex<Pool<NetworkWriter>>> = Arc::new(Mutex::new(Pool::new(|| NetworkWriter::new(), 1000)));
}

#[derive(Clone)]
pub struct NetworkWriterPool;

impl NetworkWriterPool {
    pub fn count() -> usize {
        NETWORK_WRITER_POOL.lock().unwrap().count()
    }

    pub fn get() -> NetworkWriter {
        let mut writer = NETWORK_WRITER_POOL.lock().unwrap().get();
        writer.reset();
        writer
    }

    pub fn get_return<T>(func: T)
    where
        T: FnOnce(&mut NetworkWriter),
    {
        let mut writer = Self::get();
        func(&mut writer);
        Self::return_(writer);
    }

    pub fn return_(mut writer: NetworkWriter) {
        writer.reset();
        NETWORK_WRITER_POOL.lock().unwrap().return_(writer);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_network_writer_pool() {}
}