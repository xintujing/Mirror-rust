use crate::core::transport::{Transport, TransportCallback, TransportCallbackType, TransportChannel, TransportError, TransportFunc, TransportTrait};
use bytes::Bytes;
use kcp2k_rust::error_code::ErrorCode;
use kcp2k_rust::kcp2k::Kcp2K;
use kcp2k_rust::kcp2k_callback::{Callback, CallbackType};
use kcp2k_rust::kcp2k_channel::Kcp2KChannel;
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use std::process::exit;
use tklog::error;

pub trait Kcp2kTransportTrait {
    fn awake();
}

pub struct Kcp2kTransport {
    pub transport: Transport,
    pub server_active: bool,
    pub config: Kcp2KConfig,
    pub port: u16,
    pub kcp_serv: Option<Kcp2K>,
    pub kcp_serv_rx: Option<crossbeam_channel::Receiver<Callback>>,
}

impl Kcp2kTransport {
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
        // 服务器接收
        if let Ok(cb) = self.kcp_serv_rx.as_ref().unwrap().try_recv() {
            let mut tcb = TransportCallback::default();
            tcb.r#type = Self::from_kcp2k_callback_type(cb.callback_type);
            tcb.connection_id = cb.connection_id;
            tcb.data = cb.data.to_vec();
            tcb.channel = Self::from_kcp2k_channel(cb.channel);
            tcb.error = Self::from_kcp2k_error_code(cb.error_code);
            if let Ok(on_server_connected) = self.transport.transport_cb_fn.lock() {
                if let Some(func) = on_server_connected.as_ref() {
                    func(tcb);
                }
            }
        }
    }
}

impl Kcp2kTransportTrait for Kcp2kTransport {
    fn awake() {
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
}

impl TransportTrait for Kcp2kTransport {
    fn available(&self) -> bool {
        todo!()
    }

    fn server_active(&self) -> bool {
        todo!()
    }

    fn server_start(&mut self) {
        match Kcp2K::new_server(self.config, format!("0.0.0.0:{}", self.port)) {
            Ok((server, s_rx)) => {
                self.kcp_serv = Some(server);
                self.kcp_serv_rx = Some(s_rx);
                self.server_active = true;
            }
            Err(err) => {
                error!(format!("Kcp2kTransport awake error: {:?}", err));
                exit(1)
            }
        }
    }

    fn server_send(&mut self, connection_id: u64, data: Vec<u8>, channel: TransportChannel) {
        let mut tcb = TransportCallback::default();
        match self.kcp_serv.as_ref().unwrap().s_send(connection_id, Bytes::copy_from_slice(data.as_slice()), Self::two_kcp2k_channel(channel)) {
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
        if let Ok(mut transport_cb_fn) = self.transport.transport_cb_fn.lock() {
            if let Some(func) = transport_cb_fn.as_ref() {
                func(tcb);
            }
        }
    }

    fn server_disconnect(&mut self, connection_id: u64) {
        self.kcp_serv.as_ref().unwrap().close_connection(connection_id);
    }

    fn server_get_client_address(&self, connection_id: u64) -> String {
        self.kcp_serv.as_ref().unwrap().get_connection_address(connection_id)
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

    fn set_transport_cb_fn(&self, func: TransportFunc) {
        if let Ok(mut transport_cb_fn) = self.transport.transport_cb_fn.lock() {
            *transport_cb_fn = Some(func);
        }
    }
}