use kcp2k_rust::kcp2k_channel::Kcp2KChannel;
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use kcp2k_rust::kcp2k_server::Server;
use std::sync::{Arc, Mutex};

const MTU: usize = 1200; // Default MTU

// KcpTransport struct with configurations and state
pub struct KcpTransport {
    pub port: u16,
    pub config: Kcp2KConfig,
    pub server: Option<Arc<Mutex<Server>>>,
}

// Implementation of KcpTransport
impl KcpTransport {
    pub const SCHEME: &'static str = "KCP";

    // Initialization methods
    pub fn new(port: u16) -> Self {
        Self {
            port,
            config: Kcp2KConfig::default(),
            server: None,
        }
    }

    // Initialization and configuration methods
    pub fn with_config(port: u16, config: Kcp2KConfig) -> Self {
        Self {
            port,
            config,
            server: None,
        }
    }

    // Server methods
    pub async fn awake(&mut self) {
        let server = Server::new(self.config, format!("{}:{}", "0.0.0.0", self.port),
                                 Arc::new(|callback: kcp2k_rust::kcp2k_callback::Callback| {}),
        ).unwrap();
        self.server = Some(Arc::new(Mutex::new(server)));
    }

    pub async fn server_start(&mut self) {
        if let Some(server) = &self.server {
            let mut server = server.lock().unwrap();
            server.start().expect("TODO: panic message");
        }
    }

    pub async fn server_send(&self, connection_id: u64, data: &[u8], channel: Kcp2KChannel) {
        if let Some(server) = &self.server {
            let mut server = server.lock().unwrap();
            server.send(connection_id, data.to_vec(), channel).expect("TODO: panic message");
        }
    }

    pub async fn server_stop(&mut self) {
        if let Some(server) = &self.server {
            let mut server = server.lock().unwrap();
            server.stop();
        }
    }

    pub async fn server_early_update(&mut self) {
        if let Some(server) = &self.server {
            let mut server = server.lock().unwrap();
            server.tick_incoming();
        }
    }

    pub async fn server_late_update(&mut self) {
        if let Some(server) = &self.server {
            let mut server = server.lock().unwrap();
            server.tick_outgoing();
        }
    }

    pub async fn shutdown(&mut self) {}
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_kcp_transport() {
        let mut kcp_transport = KcpTransport::new(1234);
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            kcp_transport.server_start().await;
            kcp_transport.awake().await;
            loop {
                kcp_transport.server_early_update().await;
                kcp_transport.server_late_update().await;
            }
        });
        loop {}
    }
}