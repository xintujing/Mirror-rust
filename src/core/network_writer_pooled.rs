use crate::core::network_writer::NetworkWriter;
use crate::core::network_writer_pool::NetworkWriterPool;


pub struct NetworkWriterPooled {
    writer: NetworkWriter,
}

impl NetworkWriterPooled {
    pub fn new() -> Self {
        Self {
            writer: NetworkWriter::new(),
        }
    }

    pub fn reset(&mut self) {
        self.writer.reset();
    }

    pub fn dispose(self) {
        NetworkWriterPool::return_(self);
    }
}
