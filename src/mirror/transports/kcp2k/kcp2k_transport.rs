use crate::log_error;
use crate::mirror::core::transport::{
    Transport, TransportCallback, TransportCallbackType, TransportChannel, TransportError,
    TransportFunc, TransportTrait,
};
use bytes::Bytes;
use kcp2k_rust::error_code::ErrorCode;
use kcp2k_rust::kcp2k::Kcp2K;
use kcp2k_rust::kcp2k_callback::{Callback, CallbackType};
use kcp2k_rust::kcp2k_channel::Kcp2KChannel;
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use kcp2k_rust::kcp2k_peer::Kcp2KPeer;
use std::process::exit;

pub struct Kcp2kTransport {
    pub transport: Transport,
    pub server_active: bool,
    pub config: Kcp2KConfig,
    pub port: u16,
    pub kcp_serv: Option<Kcp2K>,
    pub kcp_serv_rx: Option<crossbeam_channel::Receiver<Callback>>,
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
    fn recv_data(&mut self) {
        // 服务器接收数据
        if let Some(ref kcp_serv_rx) = self.kcp_serv_rx {
            if let Ok(cb) = kcp_serv_rx.try_recv() {
                let mut tcb = TransportCallback::default();
                tcb.r#type = Self::from_kcp2k_callback_type(cb.callback_type);
                tcb.connection_id = cb.connection_id;
                tcb.data = cb.data.to_vec();
                tcb.channel = Self::from_kcp2k_channel(cb.channel);
                tcb.error = Self::from_kcp2k_error_code(cb.error_code);
                match self.transport.transport_cb_fn.as_ref() {
                    None => {
                        log_error!("Kcp2kTransport recv_data error: transport_cb_fn is None");
                    }
                    Some(transport_cb_fn) => {
                        transport_cb_fn(tcb);
                    }
                }
            }
        }
    }
}

impl TransportTrait for Kcp2kTransport {
    fn awake()
    where
        Self: Sized,
    {
        let kcp2k_transport = Self {
            transport: Transport::default(),
            server_active: false,
            config: Default::default(),
            port: 7777,
            kcp_serv: None,
            kcp_serv_rx: None,
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
        match Kcp2K::new_server(self.config, format!("0.0.0.0:{}", self.port)) {
            Ok((server, s_rx)) => {
                self.kcp_serv = Some(server);
                self.kcp_serv_rx = Some(s_rx);
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
                tcb.connection_id = connection_id;
                tcb.data = data;
                tcb.channel = channel;
            }
            Err(e) => {
                tcb.r#type = TransportCallbackType::OnServerError;
                tcb.connection_id = connection_id;
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
        self.recv_data();
    }

    fn server_late_update(&mut self) {
        self.kcp_serv.as_ref().unwrap().tick_outgoing();
        self.recv_data();
    }

    fn server_stop(&mut self) {
        let _ = self.kcp_serv.as_ref().unwrap().stop();
    }

    fn shutdown(&mut self) {}

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

    fn get_batcher_threshold(&self, channel: TransportChannel) -> usize {
        let _ = channel;
        Kcp2KPeer::unreliable_max_message_size(self.config.mtu as u32)
    }
}
