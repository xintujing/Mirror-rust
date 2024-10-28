use crate::core::network_writer::NetworkWriter;
use crate::core::network_writer_pool::NetworkWriterPooledPool;


#[derive(Clone)]
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
        NetworkWriterPooledPool::return_(self);
    }

    pub fn get_network_writer(&mut self) -> &mut NetworkWriter {
        &mut self.writer
    }
}
