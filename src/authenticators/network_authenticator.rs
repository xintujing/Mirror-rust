use crate::core::network_connection::NetworkConnectionTrait;
use crate::core::network_connection_to_client::NetworkConnectionToClient;
use crate::core::network_reader::NetworkReader;
use crate::core::transport::TransportChannel;
use lazy_static::lazy_static;
use std::sync::RwLock;

lazy_static! {
    static ref ON_SERVER_AUTHENTICATED: RwLock<Box<dyn Fn(&mut NetworkConnectionToClient) + Send + Sync>> =
        RwLock::new(Box::new(|_| {}));
}

pub struct NetworkAuthenticatorTraitStatic;

impl NetworkAuthenticatorTraitStatic {
    pub fn set_on_server_authenticated(func: Box<dyn Fn(&mut NetworkConnectionToClient) + Send + Sync>) {
        let mut on_server_authenticated = ON_SERVER_AUTHENTICATED.write().unwrap();
        *on_server_authenticated = func;
    }

    fn call_on_server_authenticated(connection: &mut NetworkConnectionToClient) {
        let on_server_authenticated = ON_SERVER_AUTHENTICATED.read().unwrap();
        on_server_authenticated(connection);
    }
}

pub trait NetworkAuthenticatorTrait: Send + Sync {
    fn on_auth_request_message(connection_id: u64, reader: &mut NetworkReader, channel: TransportChannel)
    where
        Self: Sized;
    fn on_start_server(&mut self) {
        // NetworkServer::register_handler::<AuthRequestMessage>(Box::new(Self::on_auth_request_message), false);
    }
    fn on_stop_server(&mut self) {
        // NetworkServer::unregister_handler::<AuthRequestMessage>();
    }
    fn on_server_authenticate(&mut self, conn: &mut NetworkConnectionToClient) {}
    fn server_accept(conn: &mut NetworkConnectionToClient)
    where
        Self: Sized,
    {
        NetworkAuthenticatorTraitStatic::call_on_server_authenticated(conn);
    }
    fn server_reject(&mut self, conn: &mut NetworkConnectionToClient) {
        conn.disconnect();
    }
    fn reset(&mut self) {}
}