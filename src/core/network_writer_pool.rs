use crate::core::network_writer::NetworkWriter;
use crate::core::tools::pool::Pool;
use std::sync::LazyLock;

static mut NETWORK_WRITER_POOL: LazyLock<Pool<NetworkWriter>> = LazyLock::new(|| {
    Pool::new(|| NetworkWriter::new(), 1000)
});


#[derive(Clone)]
pub struct NetworkWriterPool;

impl NetworkWriterPool {
    pub fn count() -> usize {
        unsafe {
            NETWORK_WRITER_POOL.count()
        }
    }

    pub fn get() -> NetworkWriter {
        unsafe {
            let mut writer = NETWORK_WRITER_POOL.get();
            writer.reset();
            writer
        }
    }

    #[inline(always)]
    pub fn return_(writer: NetworkWriter) {
        unsafe {
            NETWORK_WRITER_POOL.return_(writer);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_network_writer_pool() {}
}