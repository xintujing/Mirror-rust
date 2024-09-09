use std::sync::Mutex;

pub struct NetworkDiagnostics {
    pub out_message_event: Mutex<Option<Box<dyn Fn(MessageInfo) + Send + Sync>>>,
    pub in_message_event: Mutex<Option<Box<dyn Fn(MessageInfo) + Send + Sync>>>,
}

impl NetworkDiagnostics {
    pub fn new() -> Self {
        Self {
            out_message_event: Mutex::new(None),
            in_message_event: Mutex::new(None),
        }
    }

    pub fn on_send<T: NetworkMessage>(&self, message: &T, channel: u32, bytes: usize, count: usize) {
        if count > 0 {
            let message_info = MessageInfo::new(message, channel, bytes, count);
            if let Some(event) = self.out_message_event.lock().unwrap().as_ref() {
                event(&message_info);
            }
        }
    }

    pub fn on_receive<T: NetworkMessage>(&self, message: &T, channel: u32, bytes: usize) {
        let message_info = MessageInfo::new(message, channel, bytes, 1);
        if let Some(event) = self.in_message_event.lock().unwrap().as_ref() {
            event(&message_info);
        }
    }
}

pub struct MessageInfo<'a, T: NetworkMessage> {
    pub message: &'a T,
    pub channel: u32,
    pub bytes: usize,
    pub count: usize,
}

impl<'a, T: NetworkMessage> MessageInfo<'a, T> {
    fn new(message: &'a T, channel: u32, bytes: usize, count: usize) -> Self {
        Self {
            message,
            channel,
            bytes,
            count,
        }
    }
}

pub trait NetworkMessage: Sized {
    // Implement the NetworkMessage trait
}