extern crate kcp2k_rust;

use crate::server::MirrorServer;

mod backend_data;
mod logger;
mod messages;
mod batcher;
mod server;
mod stable_hash;
mod sync_data;
mod tools;
mod transports;
mod room;
mod network_connection;
mod network_identity;
mod components;

fn main() {
    let m_server = MirrorServer::new("0.0.0.0:7777".to_string());
    m_server.start();
}
