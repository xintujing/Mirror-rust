// Convenience trait for transports which use a port.
// Useful for cases where someone wants to 'just set the port' independent of
// which transport it is.
//
// Note that not all transports have ports, but most do.

mod mirror {
    pub trait PortTransport {
        // Getter for the port
        fn get_port(&self) -> u16;

        // Setter for the port
        fn set_port(&mut self, port: u16);
    }
}
