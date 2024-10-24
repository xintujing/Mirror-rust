extern crate atomic;
extern crate kcp2k_rust;

use crate::core::network_server::NetworkServer;

mod transports;
mod tools;
mod components;
mod core;

fn main() {
    NetworkServer::listen(99);
    NetworkServer::for_each_network_connection(|mut item| {
        println!("connection hash: {} address: {}", item.key(), item.address);
    });
    NetworkServer::for_each_network_message_handler(|mut item| {
        println!("message hash: {} require_authentication: {}", item.key(), item.require_authentication);
    });

    let m_server = core::test_server::MirrorServer::new("0.0.0.0:7777".to_string());
    m_server.start();
}
