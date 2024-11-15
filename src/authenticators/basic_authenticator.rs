use crate::authenticators::network_authenticator::NetworkAuthenticatorTrait;
use crate::core::messages::NetworkMessageTrait;
use crate::core::network_connection::NetworkConnectionTrait;
use crate::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::core::network_server::{NetworkServer, NetworkServerStatic};
use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::core::tools::stable_hash::StableHash;
use crate::core::transport::TransportChannel;

pub struct BasicAuthenticator {
    username: String,
    password: String,
}

impl BasicAuthenticator {
    pub fn new(username: String, password: String) -> Self {
        Self {
            username,
            password,
        }
    }
}

impl NetworkAuthenticatorTrait for BasicAuthenticator {
    fn on_auth_request_message(connection_id: u64, reader: &mut NetworkReader, channel: TransportChannel) {
        let message = AuthRequestMessage::deserialize(reader);
        println!("on_auth_request_message: {:?}", message);
        if let Some(mut conn) = NetworkServerStatic::get_static_network_connections().get_mut(&connection_id) {
            let mut response = AuthResponseMessage::new(100, "Success".to_string());
            conn.send_network_message(&mut response, channel);

            Self::server_accept(&mut conn);
        }
    }
    fn on_start_server(&mut self) {
        NetworkServer::register_handler::<AuthRequestMessage>(Box::new(Self::on_auth_request_message), false);
    }
    fn on_stop_server(&mut self) {
        NetworkServer::unregister_handler::<AuthRequestMessage>();
    }
}

// auth请求消息
#[derive(Debug, Default)]
pub struct AuthRequestMessage {
    pub username: String,
    pub password: String,
}
impl NetworkMessageTrait for AuthRequestMessage {
    const FULL_NAME: &'static str = "";

    fn deserialize(reader: &mut NetworkReader) -> Self {
        Self {
            username: reader.read_string(),
            password: reader.read_string(),
        }
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        writer.write_string(self.username.to_string());
        writer.write_string(self.password.to_string());
    }

    fn get_hash_code() -> u16 {
        4296
    }
}


// auth响应消息
#[derive(Debug, Default)]
pub struct AuthResponseMessage {
    pub code: u8,
    pub message: String,
}

impl AuthResponseMessage {
    pub fn new(code: u8, message: String) -> Self {
        Self {
            code,
            message,
        }
    }
}

impl NetworkMessageTrait for AuthResponseMessage {
    const FULL_NAME: &'static str = "";

    fn deserialize(reader: &mut NetworkReader) -> Self {
        Self {
            code: reader.read_byte(),
            message: reader.read_string(),
        }
    }

    fn serialize(&mut self, writer: &mut NetworkWriter) {
        writer.write_byte(self.code);
        writer.write_string(self.message.to_string());
    }

    fn get_hash_code() -> u16 {
        4296
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_response_message() {
        println!("{}", AuthRequestMessage::FULL_NAME.get_stable_hash_code16())
    }
}