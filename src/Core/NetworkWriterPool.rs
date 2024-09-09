use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

struct NetworkWriter {
    buffer: Vec<u8>,
}

impl NetworkWriter {
    fn new() -> Self {
        NetworkWriter {
            buffer: Vec::with_capacity(1500), // Default capacity
        }
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }
}

struct NetworkWriterPool {
    pool: Arc<Mutex<VecDeque<NetworkWriter>>>,
    initial_capacity: usize,
}

impl NetworkWriterPool {
    fn new(initial_capacity: usize) -> Self {
        let pool = VecDeque::with_capacity(initial_capacity);
        let pool = Arc::new(Mutex::new(pool));

        let mut writer_pool = NetworkWriterPool { pool, initial_capacity };
        writer_pool.prepopulate();

        writer_pool
    }

    fn prepopulate(&mut self) {
        let mut pool = self.pool.lock().unwrap();
        for _ in 0..self.initial_capacity {
            pool.push_back(NetworkWriter::new());
        }
    }

    fn get(&self) -> NetworkWriter {
        let mut pool = self.pool.lock().unwrap();
        match pool.pop_front() {
            Some(mut writer) => {
                writer.reset();
                writer
            }
            None => NetworkWriter::new(), // If the pool is empty, create a new writer
        }
    }

    fn return_to_pool(&self, writer: NetworkWriter) {
        let mut pool = self.pool.lock().unwrap();
        pool.push_back(writer);
    }

    fn count(&self) -> usize {
        let pool = self.pool.lock().unwrap();
        pool.len()
    }
}

fn main() {
    let pool = NetworkWriterPool::new(1000);
    let writer = pool.get();
    println!("Pool count after getting one writer: {}", pool.count());

    pool.return_to_pool(writer);
    println!("Pool count after returning the writer: {}", pool.count());
}
