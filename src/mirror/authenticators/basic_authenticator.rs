use crate::log_error;
use crate::mirror::authenticators::network_authenticator::NetworkAuthenticatorTrait;
use crate::mirror::core::messages::NetworkMessageTrait;
use crate::mirror::core::network_connection::NetworkConnectionTrait;
use crate::mirror::core::network_manager::NetworkManagerStatic;
use crate::mirror::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::mirror::core::network_server::{NetworkServer, NetworkServerStatic};
use crate::mirror::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::mirror::core::transport::TransportChannel;
use dashmap::try_result::TryResult;
use std::any::Any;

pub struct BasicAuthenticator {
    username: String,
    password: String,
}

impl BasicAuthenticator {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

impl NetworkAuthenticatorTrait for BasicAuthenticator {
    fn enable(self) {
        let network_manager_singleton = NetworkManagerStatic::network_manager_singleton();
        network_manager_singleton.set_authenticator(Box::new(self));
    }

    fn on_auth_request_message(
        connection_id: u64,
        reader: &mut NetworkReader,
        channel: TransportChannel,
    ) {
        // 获取认证器
        if let Some(authenticator) = Self::get_mut_dyn_any() {
            // 转为BasicAuthenticator
            let basic_authenticator = authenticator.downcast_mut::<Self>().unwrap();
            // 反序列化 auth请求消息
            let message = AuthRequestMessage::deserialize(reader);
            // 检查用户名和密码
            match message.username == basic_authenticator.username
                && message.password == basic_authenticator.password
            {
                // 认证成功
                true => {
                    match NetworkServerStatic::network_connections().try_get_mut(&connection_id) {
                        TryResult::Present(mut conn) => {
                            let mut response = AuthResponseMessage::new(100, "Success".to_string());

                            // 发送响应消息
                            conn.send_network_message(&mut response, channel);

                            // 设置连接已认证
                            Self::server_accept(&mut conn);
                        }
                        TryResult::Absent => {
                            log_error!(format!(
                                "Failed because connection {} is absent.",
                                connection_id
                            ));
                        }
                        TryResult::Locked => {
                            log_error!(format!(
                                "Failed because connection {} is locked.",
                                connection_id
                            ));
                        }
                    }
                }
                // 认证失败
                false => {
                    match NetworkServerStatic::network_connections().try_get_mut(&connection_id) {
                        TryResult::Present(mut conn) => {
                            let mut response =
                                AuthResponseMessage::new(200, "Invalid Credentials".to_string());

                            conn.send_network_message(&mut response, channel);

                            Self::server_reject(&mut conn);
                        }
                        TryResult::Absent => {
                            log_error!(format!(
                                "Failed to clear observers because connection {} is absent.",
                                connection_id
                            ));
                        }
                        TryResult::Locked => {
                            log_error!(format!(
                                "Failed to clear observers because connection {} is locked.",
                                connection_id
                            ));
                        }
                    }
                }
            }
        }
    }
    fn on_start_server(&mut self) {
        NetworkServer::register_handler::<AuthRequestMessage>(Self::on_auth_request_message, false);
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
        Self { code, message }
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
    use crate::mirror::core::tools::stable_hash::StableHash;

    #[test]
    fn test_auth_response_message() {
        println!("{}", AuthRequestMessage::FULL_NAME.get_stable_hash_code16())
    }
}
