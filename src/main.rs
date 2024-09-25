extern crate kcp2k_rust;
mod transports;
mod server;
mod logger;
mod rwder;
mod stable_hash;
mod tools;
mod sync_data;

fn main() {
    server::MirrorServer::listen();
    while let Ok(m_server) = server::MirrorServer::get_instance().lock() {
        if let Some(kcp_serv) = m_server.kcp_serv.as_ref() {
            if let Ok(mut kcp_serv) = kcp_serv.lock() {
                kcp_serv.tick();
            }
        }
    }
}