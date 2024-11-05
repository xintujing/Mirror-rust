use crate::core::network_manager::{NetworkManager, NetworkManagerTrait};
use crate::core::network_server::NetworkServer;
use crate::transports::kcp2k::kcp2k_transport::{Kcp2kTransport, Kcp2kTransportTrait};

mod transports;
mod tools;
mod components;
mod core;

fn main() {
    Kcp2kTransport::awake();

    NetworkManager::awake();

    let network_manager_singleton = NetworkManager::get_network_manager_singleton();

    network_manager_singleton.start();

    NetworkServer::for_each_network_message_handler(|mut item| {
        println!("message hash: {} require_authentication: {}", item.key(), item.require_authentication);
    });

    NetworkServer::for_each_network_connection(|mut item| {
        println!("connection hash: {} address: {}", item.key(), item.address);
    });

    loop {
        NetworkServer::network_early_update();
        NetworkServer::network_late_update();
    }
}
