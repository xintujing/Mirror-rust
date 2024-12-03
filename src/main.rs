use mirror::core::network_loop::NetworkLoop;
use signal_hook::iterator::Signals;
use std::thread;
use std::thread::sleep;

mod mirror;
mod quick_start;

pub fn stop_signal() -> &'static mut bool {
    static mut STOP: bool = false;
    unsafe { &mut STOP }
}

// 热重载配置
pub fn hot_reload_signal() -> &'static mut bool {
    static mut HOT_RELOAD: bool = false;
    unsafe { &mut HOT_RELOAD }
}

fn start() {
    // NetworkLoop
    NetworkLoop::run();
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

    // 启动
    start();
    // 循环检测是否需要重启
    loop {
        if *hot_reload_signal() && *stop_signal() {
            *stop_signal() = false;
            *hot_reload_signal() = false;
            start();
        }
        if *stop_signal() && !*hot_reload_signal() {
            break;
        }
        sleep(std::time::Duration::from_secs(3));
    }
}
