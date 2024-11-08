use crate::core::batching::batcher::Batcher;
use crate::core::network_reader::{NetworkMessageReader, NetworkReader, NetworkReaderTrait};
use crate::core::network_writer::{NetworkMessageWriter, NetworkWriter};
use crate::core::network_writer_pool::NetworkWriterPool;
use crate::core::transport::{Transport, TransportChannel};
use tklog::{error, warn};

pub struct NetworkMessages;

impl NetworkMessages {
    pub const ID_SIZE: usize = size_of::<u16>();

    pub fn max_message_size(channel: TransportChannel) -> usize {
        Self::max_content_size(channel) + Self::ID_SIZE
    }

    pub fn max_content_size(channel: TransportChannel) -> usize {
        if let Some(transport) = Transport::get_active_transport() {
            let transport_max_size = transport.get_max_packet_size(channel);
            transport_max_size - NetworkMessages::ID_SIZE - Batcher::max_message_overhead(transport_max_size)
        } else {
            warn!("NetworkMessages::max_content_size() failed to get active transport");
            1500
        }
    }

    pub fn unpack_id(reader: &mut NetworkReader) -> u16 {
        reader.read_ushort()
    }

    pub fn pack<T>(message: &mut T, writer: &mut NetworkWriter)
    where
        T: NetworkMessageWriter + NetworkMessageReader + Send,
    {
        message.serialize(writer);
    }
}