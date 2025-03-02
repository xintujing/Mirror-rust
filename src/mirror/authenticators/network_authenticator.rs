use crate::mirror::core::network_connection::NetworkConnectionTrait;
use crate::mirror::core::network_connection_to_client::NetworkConnectionToClient;
use crate::mirror::core::network_manager::NetworkManagerStatic;
use crate::mirror::core::network_reader::NetworkReader;
use crate::mirror::core::transport::TransportChannel;
use lazy_static::lazy_static;
use std::any::Any;
use std::sync::RwLock;

lazy_static! {
    static ref ON_SERVER_AUTHENTICATED: RwLock<fn(&mut NetworkConnectionToClient)> =
        RwLock::new(|_| {});
}

pub struct NetworkAuthenticatorTraitStatic;

impl NetworkAuthenticatorTraitStatic {
    pub fn set_on_server_authenticated(func: fn(&mut NetworkConnectionToClient)) {
        let mut on_server_authenticated = ON_SERVER_AUTHENTICATED.write().unwrap();
        *on_server_authenticated = func;
    }

    fn call_on_server_authenticated(connection: &mut NetworkConnectionToClient) {
        let on_server_authenticated = ON_SERVER_AUTHENTICATED.read().unwrap();
        on_server_authenticated(connection);
    }
}

pub trait NetworkAuthenticatorTrait: Send + Sync
where
    Self: 'static,
{
    fn enable(self)
    where
        Self: Sized,
    {
        let network_manager_singleton = NetworkManagerStatic::network_manager_singleton();
        network_manager_singleton.set_authenticator(Box::new(self));
    }
    fn on_auth_request_message(
        connection_id: u64,
        reader: &mut NetworkReader,
        channel: TransportChannel,
    ) where
        Self: Sized;

    // NetworkServer::register_handler::<AuthRequestMessage>(Box::new(Self::on_auth_request_message), false);
    fn on_start_server(&mut self);
    // NetworkServer::unregister_handler::<AuthRequestMessage>();
    fn on_stop_server(&mut self);
    fn on_server_authenticate(&mut self, conn: &mut NetworkConnectionToClient) {
        let _ = conn;
    }
    fn server_accept(conn: &mut NetworkConnectionToClient)
    where
        Self: Sized,
    {
        NetworkAuthenticatorTraitStatic::call_on_server_authenticated(conn);
    }
    fn server_reject(conn: &mut NetworkConnectionToClient)
    where
        Self: Sized,
    {
        conn.set_authenticated(false);
        conn.disconnect();
    }
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_mut_dyn_any() -> Option<&'static mut dyn Any>
    where
        Self: Sized,
    {
        let network_manager_singleton = NetworkManagerStatic::network_manager_singleton();

        if let Some(authenticator) = network_manager_singleton.authenticator() {
            return Some(authenticator.as_any_mut());
        }
        None
    }
    fn reset(&mut self) {}
}
