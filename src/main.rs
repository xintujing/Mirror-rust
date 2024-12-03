use mirror::core::network_loop::NetworkLoop;
use signal_hook::iterator::Signals;
use std::thread;

mod mirror;
mod quick_start;

pub fn stop_signal() -> &'static mut bool {
    static mut STOP: bool = false;
    unsafe { &mut STOP }
}

fn main() {
    // 注册信号处理函数
    let mut signals_info =
        Signals::new(&[signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM])
            .expect("Failed to register signal handler");

    // 启动一个线程来监听终止信号
    thread::spawn(move || {
        for sig in signals_info.forever() {
            println!("\nSignal: {:?}", sig);
            *stop_signal() = true;
            break;
        }
    });

    // NetworkLoop
    NetworkLoop::run();
}
