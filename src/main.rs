extern crate kcp2k_rust;

mod tools;
mod transports;
mod components;
mod core;

fn main() {
    let mut network_server = core::network_server::NetworkServer::new();
    network_server.listen(32);

    for (hash_code, handler) in network_server.network_message_handlers {
        println!("hash_code: {}, require_authentication: {}", hash_code, handler.require_authentication);
    }

    let m_server = core::test_server::MirrorServer::new("0.0.0.0:7777".to_string());
    m_server.start();
}
