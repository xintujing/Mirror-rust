use crate::core::network_loop::NetworkLoop;

mod transports;
mod tools;
mod components;
mod core;
mod quick_start;

fn main() {
    NetworkLoop::run();
}
