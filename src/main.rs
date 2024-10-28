extern crate atomic;
extern crate kcp2k_rust;

use crate::core::network_server::NetworkServer;
use crate::transports::kcp2k::kcp2k_transport::{Kcp2kTransport, Kcp2kTransportTrait};

mod transports;
mod tools;
mod components;
mod core;

fn main() {

    let m_server = core::test_server::MirrorServer::new("0.0.0.0:7777".to_string());
    m_server.start();

    Kcp2kTransport::awake();
    NetworkServer::listen(99);

    NetworkServer::for_each_network_connection(|mut item| {
        println!("connection hash: {} address: {}", item.key(), item.address);
    });
    NetworkServer::for_each_network_message_handler(|mut item| {
        println!("message hash: {} require_authentication: {}", item.key(), item.require_authentication);
    });

    loop {
        NetworkServer::network_early_update();
        NetworkServer::network_late_update();
    }
}
