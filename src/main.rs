extern crate kcp2k_rust;

mod tools;
mod transports;
mod components;
mod core;

fn main() {
    let m_server = core::network_server::MirrorServer::new("0.0.0.0:7777".to_string());
    m_server.start();
}
