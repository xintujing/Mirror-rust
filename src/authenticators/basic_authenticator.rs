use crate::authenticators::network_authenticator::NetworkAuthenticatorTrait;
use crate::core::messages::NetworkMessageTrait;
use crate::core::network_connection::NetworkConnectionTrait;
use crate::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::core::network_server::{NetworkServer, NetworkServerStatic};
use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::core::tools::stable_hash::StableHash;
use crate::core::transport::TransportChannel;
use std::any::Any;

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
        // 未认证标志
        let mut no_authed = true;
        // 获取认证器
        if let Some(authenticator) = Self::get_mut_dyn_any() {
            // 转为BasicAuthenticator
            let basic_authenticator = authenticator.downcast_mut::<Self>().unwrap();
            // 反序列化 auth请求消息
            let message = AuthRequestMessage::deserialize(reader);
            // 检查用户名和密码
            if message.username == basic_authenticator.username && message.password == basic_authenticator.password {
                if let Some(mut conn) = NetworkServerStatic::get_static_network_connections().get_mut(&connection_id) {
                    let mut response = AuthResponseMessage::new(100, "Success".to_string());

                    // 发送响应消息
                    conn.send_network_message(&mut response, channel);

                    // 设置连接已认证
                    Self::server_accept(&mut conn);
                }
                no_authed = false;
                return;
            }
        }

        // 拒绝连接
        if no_authed {
            if let Some(mut conn) = NetworkServerStatic::get_static_network_connections().get_mut(&connection_id) {
                let mut response = AuthResponseMessage::new(200, "Invalid Credentials".to_string());

                conn.send_network_message(&mut response, channel);

                conn.set_authenticated(false);

                Self::server_reject(&mut conn);
            }
        }
    }
    fn on_start_server(&mut self) {
        NetworkServer::register_handler::<AuthRequestMessage>(Box::new(Self::on_auth_request_message), false);
    }
    fn on_stop_server(&mut self) {
        NetworkServer::unregister_handler::<AuthRequestMessage>();
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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
        writer.write_ushort(Self::get_hash_code());
        writer.write_byte(self.code);
        writer.write_string(self.message.to_string());
    }

    fn get_hash_code() -> u16 {
        26160
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