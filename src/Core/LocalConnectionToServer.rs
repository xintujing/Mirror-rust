use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

struct NetworkWriterPooled {
    data: Bytes,
}

impl NetworkWriterPooled {
    pub fn new() -> Self {
        Self { data: Bytes::new() }
    }

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
        NetworkWriterPooled::new()
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

struct LocalConnectionToClient {
    queue: Arc<Mutex<VecDeque<NetworkWriterPooled>>>,
}

struct LocalConnectionToServer {
    connection_to_client: Arc<LocalConnectionToClient>,
    queue: Arc<Mutex<VecDeque<NetworkWriterPooled>>>,
    connected_event_pending: Arc<Mutex<bool>>,
    disconnected_event_pending: Arc<Mutex<bool>>,
}

impl LocalConnectionToServer {
    pub fn new(client_connection: Arc<LocalConnectionToClient>) -> Self {
        Self {
            connection_to_client: client_connection,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            connected_event_pending: Arc::new(Mutex::new(false)),
            disconnected_event_pending: Arc::new(Mutex::new(false)),
        }
    }

    pub fn send(&self, segment: &[u8], channel_id: u32) {
        if segment.is_empty() {
            // Log error
            return;
        }
        let mut writer = NetworkWriterPool::get();
        writer.write_bytes(segment);
        let mut queue = self.connection_to_client.queue.lock().unwrap();
        queue.push_back(writer);
    }

    pub async fn update(&self) {
        // Update base if there is anything to do

        // Process connected event if pending
        {
            let mut connected = self.connected_event_pending.lock().unwrap();
            if *connected {
                *connected = false;
                // Trigger connected event
            }
        }

        // Process queued messages
        let mut queue = self.queue.lock().unwrap();
        while let Some(writer) = queue.pop_front() {
            let message = writer.to_array_segment();
            let mut batcher = Batcher::new();
            batcher.add_message(message, network_time::local_time());

            let mut batch_writer = NetworkWriterPool::get();
            if batcher.get_batch(&mut batch_writer) {
                // Simulate network transmission
            }
            NetworkWriterPool::return_writer(writer);
        }

        // Process disconnected event if pending
        {
            let mut disconnected = self.disconnected_event_pending.lock().unwrap();
            if *disconnected {
                *disconnected = false;
                // Trigger disconnected event
            }
        }
    }

    pub fn disconnect_internal(&self) {
        // Set ready state and handle disconnection
        // Simulate removal from server's active connections
    }

    pub fn disconnect(&self) {
        self.connection_to_client.disconnect_internal();
        self.disconnect_internal();
        // Simulate remote disconnection
    }
}
