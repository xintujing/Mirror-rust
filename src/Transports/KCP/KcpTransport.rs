use kcp2k_rust::kcp2k_client::Client;
use kcp2k_rust::kcp2k_server::Server;
use std::sync::{Arc, Mutex};

// Constants and configurations
const SCHEME: &str = "KCP";
const MTU: usize = 1200; // Default MTU

// KcpTransport struct with configurations and state
pub struct KcpTransport {
    port: u16,
    dual_mode: bool,
    no_delay: bool,
    interval: u32,
    timeout: i32,
    recv_buffer_size: usize,
    send_buffer_size: usize,
    fast_resend: i32,
    congestion_window: bool,
    receive_window_size: u32,
    send_window_size: u32,
    max_retransmit: u32,
    maximize_socket_buffers: bool,
    reliable_max_message_size: usize,
    unreliable_max_message_size: usize,
    server: Option<Arc<Mutex<Kcp2K>>>,
    client: Option<Kcp2K>,
    debug_log: bool,
    statistics_gui: bool,
    statistics_log: bool,
}

// Implementation of KcpTransport
impl KcpTransport {
    // Initialization and configuration methods
    pub fn new(port: u16) -> Self {
        Self {
            port,
            dual_mode: true,
            no_delay: true,
            interval: 10,
            timeout: 10000,
            recv_buffer_size: 1024 * 1024 * 7,
            send_buffer_size: 1024 * 1024 * 7,
            fast_resend: 2,
            congestion_window: false,
            receive_window_size: 4096,
            send_window_size: 4096,
            max_retransmit: 20, // Example value, adjust based on context
            maximize_socket_buffers: true,
            reliable_max_message_size: 0,
            unreliable_max_message_size: 0,
            server: None,
            client: None,
            debug_log: false,
            statistics_gui: false,
            statistics_log: false,
        }
    }

    // Client methods
    pub async fn client_connect(&mut self, address: &str) {
        let (client, _) = Client::new().unwrap();
        self.client = Some(client);
    }

    pub async fn client_send(&self, data: &[u8], channel_id: kcp2k_rust::kcp2k_channel) {
        if let Some(mut client) = &self.client {
            client.send(data.to_vec(), channel_id).await;
        }
    }

    pub async fn client_disconnect(&mut self) {
        if let Some(client) = self.client.take() {
            client.disconnect();
        }
    }

    // Server methods
    pub async fn server_start(&mut self) {
        let (server, _) = Server::new().unwrap();
        self.server = Some(Arc::new(Mutex::new(server)));
    }

    pub async fn server_send(&self, connection_id: u64, data: &[u8], channel_id: kcp2k_rust::kcp2k_channel) {
        if let Some(server) = &self.server {
            let mut server = server.lock().unwrap();
            server.s_send(connection_id, data.to_vec(), channel_id).await;
        }
    }

    pub async fn server_stop(&mut self) {
        if let Some(server) = self.server.take() {
            let mut server = server.lock().unwrap();
            server.stop().await;
        }
    }
}
