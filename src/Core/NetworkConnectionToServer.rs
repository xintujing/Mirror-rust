use std::runtime::CompilerServices;

use crate::NetworkConnection;
use crate::Transport;

pub struct NetworkConnectionToServer {
    connection_id: u32,
}

impl NetworkConnectionToServer {
    pub fn new(connection_id: u32) -> Self {
        Self {
            connection_id,
        }
    }
}

impl NetworkConnection for NetworkConnectionToServer {
    fn send_to_transport(&self, segment: ArraySegment<u8>, channel: u32) {
        Transport::client_send(segment, channel);
    }

    fn disconnect(&mut self) {
        self.is_ready = false;
        NetworkClient::set_ready(false);
        Transport::client_disconnect();
    }
}

mod NetworkClient {
    pub fn set_ready(ready: bool) {
        // Implement the set ready logic
    }
}

mod Transport {
    pub fn client_send(segment: ArraySegment<u8>, channel: u32) {
        // Implement the client send logic
    }

    pub fn client_disconnect() {
        // Implement the client disconnect logic
    }
}