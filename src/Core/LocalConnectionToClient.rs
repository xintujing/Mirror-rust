use bytes::Bytes;
use std::collections::Queue;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc::{self, Receiver, Sender};

struct NetworkWriterPooled {
    data: Bytes,
}

impl NetworkWriterPooled {
    pub fn write_bytes(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    pub fn to_array_segment(&self) -> &[u8] {
        &self.data
    }
}

struct NetworkWriterPool;

impl NetworkWriterPool {
    pub fn get() -> NetworkWriterPooled {
        NetworkWriterPooled {
            data: Bytes::new(),
        }
    }

    pub fn return_writer(_writer: NetworkWriterPooled) {
        // Return writer to the pool or handle cleanup
    }
}

struct Batcher {
    messages: Vec<(Vec<u8>, f64)>,
}

impl Batcher {
    fn new() -> Self {
        Self { messages: vec![] }
    }

    fn add_message(&mut self, message: &[u8], timestamp: f64) {
        self.messages.push((message.to_vec(), timestamp));
    }

    fn get_batch(&self, writer: &mut NetworkWriterPooled) -> bool {
        if !self.messages.is_empty() {
            for (msg, _ts) in &self.messages {
                writer.write_bytes(msg);
            }
            true
        } else {
            false
        }
    }
}

struct LocalConnectionToServer {
    queue: Arc<Mutex<Queue<NetworkWriterPooled>>>,
}

struct LocalConnectionToClient {
    connection_to_server: Arc<LocalConnectionToServer>,
    queue: Arc<Mutex<Queue<NetworkWriterPooled>>>,
    is_alive: Arc<RwLock<bool>>,
    is_ready: Arc<RwLock<bool>>,
}

impl LocalConnectionToClient {
    pub fn new(connection_to_server: Arc<LocalConnectionToServer>) -> Self {
        Self {
            connection_to_server,
            queue: Arc::new(Mutex::new(Queue::new())),
            is_alive: Arc::new(RwLock::new(true)),
            is_ready: Arc::new(RwLock::new(false)),
        }
    }

    pub fn send(&self, segment: &[u8], channel_id: u32) {
        let writer = NetworkWriterPool::get();
        writer.write_bytes(segment);
        let mut queue = self.connection_to_server.queue.lock().unwrap();
        queue.push(writer);
    }

    pub fn update(&self) {
        let mut queue = self.queue.lock().unwrap();
        while let Some(writer) = queue.pop() {
            let message = writer.to_array_segment();
            let batcher = Batcher::new();
            batcher.add_message(message, network_time::local_time());

            let mut batch_writer = NetworkWriterPool::get();
            if batcher.get_batch(&mut batch_writer) {
                network_server::on_transport_data(self.connection_id, batch_writer.to_array_segment(), channel_id);
            }
            NetworkWriterPool::return_writer(writer);
        }
    }

    pub fn disconnect_internal(&self) {
        let mut is_ready = self.is_ready.write().unwrap();
        *is_ready = false;
        self.remove_from_observings_observers();
    }

    pub fn disconnect(&self) {
        self.disconnect_internal();
        self.connection_to_server.disconnect_internal();
    }
}
