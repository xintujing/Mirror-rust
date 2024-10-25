pub enum Channels {
    None = 0,
    Reliable = 1,
    Unreliable = 2,
}

pub trait Transport {
    fn available(&self) -> bool;
    fn is_encrypted(&self) -> bool {
        false
    }
    fn encryption_cipher(&self) -> &str {
        ""
    }
    fn server_active(&self) -> bool;
    fn server_start(&mut self);
    fn server_send(&mut self, connection_id: u64, data: Vec<u8>, channel: Channels);
    fn server_disconnect(&mut self, connection_id: u64);
    fn server_get_client_address(&self, connection_id: u64) -> String;
    fn server_early_update(&mut self);
    fn server_late_update(&mut self);
    fn server_stop(&mut self);
    fn shutdown(&mut self);
    fn get_max_packet_size(&self, channel: Channels) -> u32;
    fn get_batch_threshold(&self, channel: Channels) -> u32 {
        self.get_max_packet_size(channel)
    }
}