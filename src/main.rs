extern crate kcp2k_rust;

use crate::server::MirrorServer;

mod transports;
mod server;
mod logger;
mod rwder;
mod stable_hash;
mod tools;
mod sync_data;
mod messages;
mod connection;
mod server1;

fn main() {
    let m_server = MirrorServer::new("0.0.0.0:7777".to_string());
    m_server.start();
}