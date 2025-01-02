use crate::mirror::core::backend_data::BackendDataStatic;
use crate::mirror::core::network_manager::NetworkManagerStatic;
use crate::mirror::core::transport::{
    Transport, TransportCallback, TransportCallbackType, TransportChannel, TransportError,
    TransportFunc, TransportTrait,
};
use crate::log_error;
use bytes::Bytes;
use kcp2k_rust::error_code::ErrorCode;
use kcp2k_rust::kcp2k::Kcp2K;
use kcp2k_rust::kcp2k_callback::{Callback, CallbackType};
use kcp2k_rust::kcp2k_channel::Kcp2KChannel;
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use kcp2k_rust::kcp2k_connection::Kcp2KConnection;
use kcp2k_rust::kcp2k_peer::Kcp2KPeer;
use serde::{Deserialize, Serialize};
use std::process::exit;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Kcp2kTransportConfig {
    pub port: u16,
    pub dual_mode: bool,
    pub no_delay: bool,
    pub interval: i32,
    pub timeout: u64,
    pub recv_buffer_size: usize,
    pub send_buffer_size: usize,
    pub fast_resend: i32,
    pub recv_win_size: u16,
    pub send_win_size: u16,
    pub max_retransmits: u32,
    pub maximize_socket_buffer: bool,
}

impl Default for Kcp2kTransportConfig {
    fn default() -> Self {
        Kcp2kTransportConfig {
            port: 7777,
            dual_mode: false,
            no_delay: true,
            interval: 10,
            timeout: 10000,
            recv_buffer_size: 7361536,
            send_buffer_size: 7361536,
            fast_resend: 2,
            recv_win_size: 4096,
            send_win_size: 4096,
            max_retransmits: 40,
            maximize_socket_buffer: true,
        }
    }
}

pub struct Kcp2kTransport {
    pub transport: Transport,
    pub server_active: bool,
    pub config: Kcp2KConfig,
    pub port: u16,
    pub kcp_serv: Option<Kcp2K>,
}

impl Kcp2kTransport {
    #[allow(dead_code)]
    pub const SCHEME: &'static str = "kcp2k";
    pub fn from_kcp2k_channel(kcp2k_channel: Kcp2KChannel) -> TransportChannel {
        match kcp2k_channel {
            Kcp2KChannel::Unreliable => TransportChannel::Unreliable,
            _ => TransportChannel::Reliable,
        }
    }
    pub fn two_kcp2k_channel(transport_channel: TransportChannel) -> Kcp2KChannel {
        match transport_channel {
            TransportChannel::Unreliable => Kcp2KChannel::Unreliable,
            _ => Kcp2KChannel::Reliable,
        }
    }
    pub fn from_kcp2k_error_code(error_code: ErrorCode) -> TransportError {
        match error_code {
            ErrorCode::None => TransportError::None,
            ErrorCode::DnsResolve => TransportError::DnsResolve,
            ErrorCode::Timeout => TransportError::Timeout,
            ErrorCode::Congestion => TransportError::Congestion,
            ErrorCode::InvalidReceive => TransportError::InvalidReceive,
            ErrorCode::InvalidSend => TransportError::InvalidSend,
            ErrorCode::ConnectionClosed => TransportError::ConnectionClosed,
            ErrorCode::Unexpected => TransportError::Unexpected,
            ErrorCode::SendError => TransportError::SendError,
            ErrorCode::ConnectionNotFound => TransportError::ConnectionNotFound,
            ErrorCode::ConnectionLocked => TransportError::ConnectionLocked,
        }
    }
    pub fn from_kcp2k_callback_type(callback_type: CallbackType) -> TransportCallbackType {
        match callback_type {
            CallbackType::OnConnected => TransportCallbackType::OnServerConnected,
            CallbackType::OnData => TransportCallbackType::OnServerDataReceived,
            CallbackType::OnDisconnected => TransportCallbackType::OnServerDisconnected,
            CallbackType::OnError => TransportCallbackType::OnServerError,
        }
    }
    fn kcp2k_cb(_: &Kcp2KConnection, cb: Callback) {
        // 服务器接收数据
        let tcb = TransportCallback {
            r#type: Self::from_kcp2k_callback_type(cb.r#type),
            conn_id: cb.conn_id,
            data: cb.data.to_vec(),
            channel: Self::from_kcp2k_channel(cb.channel),
            error: Self::from_kcp2k_error_code(cb.error_code),
            ..TransportCallback::default()
        };
        match Transport::active_transport() {
            None => {
                log_error!("Kcp2kTransport kcp2k_cb error: active_transport is None");
            }
            Some(active_transport) => match active_transport.transport_cb_fn() {
                None => {
                    log_error!("Kcp2kTransport kcp2k_cb error: transport_cb_fn is None");
                }
                Some(transport_cb_fn) => {
                    transport_cb_fn(tcb);
                }
            },
        }
    }
}

impl TransportTrait for Kcp2kTransport {
    fn awake()
    where
        Self: Sized,
    {
        let backend_data = BackendDataStatic::get_backend_data();
        let kcp2k_transport_config = backend_data.get_kcp2k_config();
        let config = Kcp2KConfig {
            dual_mode: kcp2k_transport_config.dual_mode,
            recv_buffer_size: kcp2k_transport_config.recv_buffer_size,
            send_buffer_size: kcp2k_transport_config.send_buffer_size,
            no_delay: kcp2k_transport_config.no_delay,
            interval: kcp2k_transport_config.interval,
            fast_resend: kcp2k_transport_config.fast_resend,
            send_window_size: kcp2k_transport_config.send_win_size,
            receive_window_size: kcp2k_transport_config.recv_win_size,
            timeout: kcp2k_transport_config.timeout,
            max_retransmits: kcp2k_transport_config.max_retransmits,
            ..Kcp2KConfig::default()
        };
        let kcp2k_transport = Self {
            transport: Transport::default(),
            server_active: false,
            config,
            port: kcp2k_transport_config.port,
            kcp_serv: None,
        };
        Transport::set_active_transport(Box::new(kcp2k_transport));
    }

    fn available(&self) -> bool {
        true
    }

    fn server_active(&self) -> bool {
        self.server_active
    }

    fn server_start(&mut self) {
        let mut network_address = NetworkManagerStatic::network_manager_singleton()
            .network_address()
            .to_string();
        if network_address == "localhost" {
            network_address = "0.0.0.0".to_string()
        }
        match Kcp2K::new_server(
            self.config,
            format!("{}:{}", network_address, self.port),
            Self::kcp2k_cb,
        ) {
            Ok(server) => {
                self.kcp_serv = Some(server);
                self.server_active = true;
            }
            Err(err) => {
                log_error!(format!("Kcp2kTransport awake error: {:?}", err));
                exit(1)
            }
        }
    }

    fn server_send(&mut self, connection_id: u64, data: Vec<u8>, channel: TransportChannel) {
        let mut tcb = TransportCallback::default();
        match self.kcp_serv.as_ref().unwrap().s_send(
            connection_id,
            Bytes::copy_from_slice(data.as_slice()),
            Self::two_kcp2k_channel(channel),
        ) {
            Ok(_) => {
                tcb.r#type = TransportCallbackType::OnServerDataSent;
                tcb.conn_id = connection_id;
                tcb.data = data;
                tcb.channel = channel;
            }
            Err(e) => {
                tcb.r#type = TransportCallbackType::OnServerError;
                tcb.conn_id = connection_id;
                tcb.error = Self::from_kcp2k_error_code(e);
            }
        }
        match self.transport.transport_cb_fn.as_ref() {
            None => {
                log_error!("Kcp2kTransport server_send error: transport_cb_fn is None");
            }
            Some(transport_cb_fn) => {
                transport_cb_fn(tcb);
            }
        }
    }

    fn server_disconnect(&mut self, connection_id: u64) {
        self.kcp_serv
            .as_ref()
            .unwrap()
            .close_connection(connection_id);
    }

    fn server_get_client_address(&self, connection_id: u64) -> String {
        self.kcp_serv
            .as_ref()
            .unwrap()
            .get_connection_address(connection_id)
    }

    fn server_early_update(&mut self) {
        self.kcp_serv.as_ref().unwrap().tick_incoming();
    }

    fn server_late_update(&mut self) {
        self.kcp_serv.as_ref().unwrap().tick_outgoing();
    }

    fn server_stop(&mut self) {
        let _ = self.kcp_serv.as_ref().unwrap().stop();
    }

    fn transport_cb_fn(&self) -> Option<TransportFunc> {
        self.transport.transport_cb_fn
    }

    fn set_transport_cb_fn(&mut self, func: TransportFunc) {
        self.transport.transport_cb_fn.replace(func);
    }

    fn get_max_packet_size(&self, channel: TransportChannel) -> usize {
        match channel {
            TransportChannel::Reliable => {
                Kcp2KPeer::unreliable_max_message_size(self.config.mtu as u32)
            }
            TransportChannel::Unreliable => Kcp2KPeer::reliable_max_message_size(
                self.config.mtu as u32,
                self.config.receive_window_size as u32,
            ),
        }
    }

    fn get_batcher_threshold(&self, _channel: TransportChannel) -> usize {
        Kcp2KPeer::unreliable_max_message_size(self.config.mtu as u32)
    }
}
