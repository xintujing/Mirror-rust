use std::collections::HashSet;
use std::time::Instant;

pub trait NetworkMessage: Sized {
    fn pack(&self, writer: &mut NetworkWriter);
    fn max_message_size(channel: u32) -> usize;
}

pub struct NetworkConnection {
    pub connection_id: u32,
    pub is_authenticated: bool,
    pub authentication_data: Option<Box<dyn Any>>,
    pub is_ready: bool,
    pub last_message_time: Instant,
    pub identity: Option<NetworkIdentity>,
    pub owned: HashSet<NetworkIdentity>,
    batches: std::collections::HashMap<u32, Batcher>,
    pub remote_timestamp: f64,
}

impl NetworkConnection {
    pub fn new(connection_id: u32) -> Self {
        Self {
            connection_id,
            is_authenticated: false,
            authentication_data: None,
            is_ready: false,
            last_message_time: Instant::now(),
            identity: None,
            owned: HashSet::new(),
            batches: std::collections::HashMap::new(),
            remote_timestamp: 0.0,
        }
    }

    pub fn send<T: NetworkMessage>(&mut self, message: &T, channel: u32) {
        let mut writer = NetworkWriter::new();
        message.pack(&mut writer);

        let max_size = T::max_message_size(channel);
        if writer.len() > max_size {
            log::error!(
                "NetworkConnection.Send: message of type {} with a size of {} bytes is larger than the max allowed message size in one batch: {}.\nThe message was dropped, please make it smaller.",
                std::any::type_name::<T>(),
                writer.len(),
                max_size
            );
            return;
        }

        NetworkDiagnostics::on_send(message, channel, writer.len(), 1);
        self.send_bytes(writer.as_bytes(), channel);
    }

    fn send_bytes(&mut self, data: &[u8], channel: u32) {
        let batcher = self.get_batcher_for_channel(channel);
        batcher.add_message(data, Instant::now().as_secs_f64());
    }

    fn get_batcher_for_channel(&mut self, channel: u32) -> &mut Batcher {
        self.batches
            .entry(channel)
            .or_insert_with(|| Batcher::new(Transport::get_batch_threshold(channel)))
    }

    pub fn update(&mut self) {
        for (channel, batcher) in self.batches.iter_mut() {
            while let Some(batch) = batcher.get_batch(&mut NetworkWriter::new()) {
                Transport::send_to(*channel, batch);
            }
        }
    }

    pub fn is_alive(&self, timeout: f32) -> bool {
        self.last_message_time.elapsed().as_secs_f32() < timeout
    }

    pub fn disconnect(&mut self) {
        Transport::disconnect(self.connection_id);
    }

    pub fn cleanup(&mut self) {
        for batcher in self.batches.values_mut() {
            batcher.clear();
        }
    }
}

impl std::fmt::Display for NetworkConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "connection({})", self.connection_id)
    }
}

struct Batcher {
    threshold: usize,
    messages: Vec<(ArraySegment<u8>, f64)>,
}

impl Batcher {
    fn new(threshold: usize) -> Self {
        Self {
            threshold,
            messages: Vec::new(),
        }
    }

    fn add_message(&mut self, data: &[u8], timestamp: f64) {
        self.messages.push((data.into(), timestamp));
    }

    fn get_batch(&mut self, writer: &mut NetworkWriter) -> Option<ArraySegment<u8>> {
        if self.messages.is_empty() {
            return None;
        }

        writer.write_f64(self.messages[0].1);

        let mut total_size = 8;
        while let Some((message, timestamp)) = self.messages.first() {
            if total_size + message.len() > self.threshold {
                break;
            }
            writer.write_bytes(message);
            total_size += message.len();
            self.messages.remove(0);
        }

        Some(writer.as_bytes())
    }

    fn clear(&mut self) {
        self.messages.clear();
    }
}

struct Transport {}

impl Transport {
    fn get_batch_threshold(channel: u32) -> usize {
        // Implement your transport-specific logic to get the batch threshold
        64 * 1024
    }

    fn send_to(channel: u32, data: ArraySegment<u8>) {
        // Implement your transport-specific logic to send the data
    }

    fn disconnect(connection_id: u32) {
        // Implement your transport-specific logic to disconnect the connection
    }
}

struct NetworkDiagnostics {}

impl NetworkDiagnostics {
    fn on_send<T: NetworkMessage>(message: &T, channel: u32, size: usize, count: usize) {
        // Implement your network diagnostics logic
    }
}

struct NetworkWriter {
    // Implement your network writer logic
}

impl NetworkWriter {
    fn new() -> Self {
        // Implement your network writer constructor
    }

    fn write_f64(&mut self, value: f64) {
        // Implement your network writer logic to write a f64
    }

    fn write_bytes(&mut self, data: &[u8]) {
        // Implement your network writer logic to write bytes
    }

    fn as_bytes(&self) -> ArraySegment<u8> {
        // Implement your network writer logic to get the written bytes as an ArraySegment
    }

    fn len(&self) -> usize {
        // Implement your network writer logic to get the written bytes length
    }
}

struct NetworkIdentity {}