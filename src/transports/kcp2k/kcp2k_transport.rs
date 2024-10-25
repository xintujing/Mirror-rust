use crate::core::transport::{Transport, TransportCallback, TransportCallbackType, TransportChannel, TransportError, TransportFunc, TransportTrait};
use bytes::Bytes;
use dashmap::DashMap;
use kcp2k_rust::error_code::ErrorCode;
use kcp2k_rust::kcp2k::Kcp2K;
use kcp2k_rust::kcp2k_callback::{Callback, CallbackType};
use kcp2k_rust::kcp2k_channel::Kcp2KChannel;
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use std::process::exit;
use std::sync::mpsc;
use tklog::error;

pub trait Kcp2kTransportTrait {
    fn awake(config: Kcp2KConfig, port: u16);
}

pub struct Kcp2kTransport {
    pub transport: Transport,
    pub server_active: bool,
    pub config: Kcp2KConfig,
    pub port: u16,
    pub kcp_serv: Kcp2K,
    pub kcp_serv_rx: mpsc::Receiver<Callback>,
}

impl Kcp2kTransport {
    const SCHEME: &'static str = "kcp2k";
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
            CallbackType::OnConnected => TransportCallbackType::OnConnected,
            CallbackType::OnData => TransportCallbackType::OnData,
            CallbackType::OnDisconnected => TransportCallbackType::OnDisconnected,
            CallbackType::OnError => TransportCallbackType::OnError,
        }
    }

    fn recv_data(&mut self) {
        // 服务器接收
        if let Ok(cb) = self.kcp_serv_rx.try_recv() {
            let mut tcb = TransportCallback::default();
            tcb.r#type = Self::from_kcp2k_callback_type(cb.callback_type);
            tcb.connection_id = cb.connection_id;
            tcb.data = cb.data.to_vec();
            tcb.channel = Self::from_kcp2k_channel(cb.channel);
            tcb.error = Self::from_kcp2k_error_code(cb.error_code);
            if let Ok(on_server_connected) = self.transport.server_cb_fn.lock() {
                if let Some(func) = on_server_connected.as_ref() {
                    func(tcb);
                }
            }
        }
    }
}

impl Kcp2kTransportTrait for Kcp2kTransport {
    fn awake(config: Kcp2KConfig, port: u16) {
        match Kcp2K::new_server(config, format!("0.0.0.0:{}", port)) {
            Ok((server, s_rx)) => {
                Self {
                    transport: Default::default(),
                    server_active: false,
                    config,
                    port,
                    kcp_serv: server,
                    kcp_serv_rx: s_rx,
                };
            }
            Err(err) => {
                error!(format!("Kcp2kTransport awake error: {:?}", err));
                exit(1)
            }
        }
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
        self.server_stop();
        Self::awake(self.config, self.port);
    }

    fn server_send(&mut self, connection_id: u64, data: Vec<u8>, channel: TransportChannel) {
        match self.kcp_serv.s_send(connection_id, Bytes::copy_from_slice(data.as_slice()), Self::two_kcp2k_channel(channel)) {
            Ok(_) => { todo!() }
            Err(_) => {}
        }
    }

    fn server_disconnect(&mut self, connection_id: u64) {
        todo!()
    }

    fn server_get_client_address(&self, connection_id: u64) -> String {
        todo!()
    }

    fn server_early_update(&mut self) {
        self.kcp_serv.tick_incoming();
        self.recv_data();
    }

    fn server_late_update(&mut self) {
        self.kcp_serv.tick_outgoing();
        self.recv_data();
    }

    fn server_stop(&mut self) {
        let _ = self.kcp_serv.stop();
    }

    fn shutdown(&mut self) {}

    fn set_server_cb_fn(&self, func: TransportFunc) {
        if let Ok(mut server_cb_fn) = self.transport.server_cb_fn.lock() {
            *server_cb_fn = Some(func);
        }
    }
}