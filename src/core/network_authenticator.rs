pub trait NetworkAuthenticatorTrait: Send + Sync {
    fn on_start_server(&mut self);
    fn on_stop_server(&mut self);
    fn on_server_authenticated(&mut self, connection_id: u64);
    fn server_accept(&mut self, connection_id: u64);
    fn server_reject(&mut self, connection_id: u64);
    fn reset(&mut self);
}