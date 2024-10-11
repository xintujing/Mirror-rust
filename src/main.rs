extern crate kcp2k_rust;

use crate::server::MirrorServer;

mod backend_data;
mod connection;
mod logger;
mod messages;
mod rwder;
mod server;
mod server1;
mod stable_hash;
mod sync_data;
mod tools;
mod transports;

fn main() {
    let m_server = MirrorServer::new("0.0.0.0:7777".to_string());
    m_server.start();
}
