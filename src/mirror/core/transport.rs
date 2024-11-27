use std::fmt::Debug;
use std::sync::{Arc, RwLock};

static mut ACTIVE_TRANSPORT: Option<Box<dyn TransportTrait>> = None;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum TransportChannel {
    Reliable = 1,
    Unreliable = 2,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum TransportCallbackType {
    OnServerConnected,
    OnServerDataReceived,
    OnServerDisconnected,
    OnServerError,
    OnServerDataSent,
    OnServerTransportException,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum TransportError {
    None,
    DnsResolve,         // failed to resolve a host name
    Refused,            // connection refused by other end. server full etc.
    Timeout,            // ping timeout or dead link
    Congestion,         // more messages than transport / network can process
    InvalidReceive,     // recv invalid packet (possibly intentional attack)
    InvalidSend,        // user tried to send invalid data
    ConnectionClosed,   // connection closed voluntarily or lost involuntarily
    Unexpected,         // unexpected error / exception, requires fix.
    SendError,          // failed to send data
    ConnectionNotFound, // connection not found
}

#[derive(Debug, Clone)]
pub struct TransportCallback {
    pub r#type: TransportCallbackType,
    pub connection_id: u64,
    pub data: Vec<u8>,
    pub channel: TransportChannel,
    pub error: TransportError,
}
impl Default for TransportCallback {
    fn default() -> Self {
        Self {
            r#type: TransportCallbackType::OnServerError,
            data: Vec::new(),
            connection_id: 0,
            channel: TransportChannel::Reliable,
            error: TransportError::None,
        }
    }
}
pub type TransportFunc = Box<dyn Fn(TransportCallback)>;
#[derive(Clone, Default)]
pub struct Transport {
    pub transport_cb_fn: Arc<RwLock<Option<TransportFunc>>>,
}
impl Transport {
    pub fn get_active_transport() -> Option<&'static mut Box<dyn TransportTrait>> {
        unsafe { ACTIVE_TRANSPORT.as_mut() }
    }
    pub fn active_transport_exists() -> bool {
        unsafe { ACTIVE_TRANSPORT.is_some() }
    }
    pub fn set_active_transport(transport: Box<dyn TransportTrait>) {
        unsafe {
            ACTIVE_TRANSPORT.replace(transport);
        }
    }
}
pub trait TransportTrait {
    fn awake()
    where
        Self: Sized;
    fn available(&self) -> bool;
    fn is_encrypted(&self) -> bool {
        false
    }
    fn encryption_cipher(&self) -> &str {
        ""
    }
    fn server_active(&self) -> bool;
    fn server_start(&mut self);
    fn server_send(&mut self, connection_id: u64, data: Vec<u8>, channel: TransportChannel);
    fn server_disconnect(&mut self, connection_id: u64);
    fn server_get_client_address(&self, connection_id: u64) -> String;
    fn server_early_update(&mut self);
    fn server_late_update(&mut self);
    fn server_stop(&mut self);
    fn shutdown(&mut self);
    fn set_transport_cb_fn(&self, func: TransportFunc);
    fn get_max_packet_size(&self, channel: TransportChannel) -> usize;
    fn get_batcher_threshold(&self, channel: TransportChannel) -> usize {
        self.get_max_packet_size(channel)
    }
}
