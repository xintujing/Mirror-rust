use crate::core::batching::batcher::Batcher;
use crate::core::transport::{Transport, TransportChannel};

pub struct NetworkMessages;

impl NetworkMessages {
    pub const ID_SIZE: usize = size_of::<u16>();

    pub fn max_message_size(channel: TransportChannel)-> usize {
       Self::max_content_size(channel) + Self::ID_SIZE
    }

    pub fn max_content_size(channel: TransportChannel) -> usize {
        let transport_max_size = Transport::get_active_transport().unwrap().get_max_packet_size(channel);
        transport_max_size - NetworkMessages::ID_SIZE - Batcher::max_message_overhead(transport_max_size)
    }
}