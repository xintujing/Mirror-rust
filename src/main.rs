use crate::core::network_loop::NetworkLoop;
use crate::core::network_manager::{NetworkManager, NetworkManagerStatic, NetworkManagerTrait};
use crate::core::network_server::{NetworkServer, NetworkServerStatic};
use crate::transports::kcp2k::kcp2k_transport::{Kcp2kTransport, Kcp2kTransportTrait};

mod transports;
mod tools;
mod components;
mod core;
mod quick_start;

fn main() {
    Kcp2kTransport::awake();

    NetworkManager::awake();

    let network_manager_singleton = NetworkManagerStatic::get_network_manager_singleton();


    NetworkServerStatic::for_each_network_message_handler(|mut item| {
        println!("message hash: {} require_authentication: {}", item.key(), item.require_authentication);
    });

    NetworkServerStatic::for_each_network_connection(|mut item| {
        println!("connection hash: {} address: {}", item.key(), item.address);
    });

    network_manager_singleton.start();

    NetworkLoop::frame_loop(|| {
        NetworkServer::network_early_update();
        NetworkServer::network_late_update();
    });
}
