extern crate kcp2k_rust;
mod transports;
mod server;
mod logger;
mod rwder;
mod stable_hash;
mod tools;
mod sync_data;
mod messages;
mod connection;

#[tokio::main]
async fn main() {
    server::MirrorServer::listen().await;
    while let mut m_server = server::MirrorServer::get_instance().lock().await {
        if let Some(kcp_serv) = m_server.kcp_serv.as_mut() {
            kcp_serv.tick();
        }
    }
}