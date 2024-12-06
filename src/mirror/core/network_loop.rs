use crate::mirror::core::network_manager::{
    NetworkManager, NetworkManagerStatic, NetworkManagerTrait,
};
use crate::mirror::core::network_server::{NetworkServer, NetworkServerStatic};
use crate::mirror::core::network_time::NetworkTime;
use crate::{log_debug, log_warn};
use lazy_static::lazy_static;
use signal_hook::iterator::Signals;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

lazy_static! {
    // 需要 添加的 awake 函数列表
    static ref AWAKE_FUNCTIONS: RwLock<Vec<fn()>> = RwLock::new(vec![]);
    // 需要 添加的 on_enable 函数列表
    static ref ON_ENABLE_FUNCTIONS: RwLock<Vec<fn()>> = RwLock::new(vec![]);
    // 需要 添加的 disable 函数列表
    static ref ON_DISABLE_FUNCTIONS: RwLock<Vec<fn()>> = RwLock::new(vec![]);
}

pub fn stop_signal() -> &'static mut bool {
    static mut STOP: bool = false;
    unsafe { &mut STOP }
}

pub struct NetworkLoop;

impl NetworkLoop {
    pub fn add_awake_function(func: fn()) {
        match AWAKE_FUNCTIONS.write() {
            Ok(mut awake_functions) => {
                awake_functions.push(func);
            }
            Err(e) => {
                log_warn!(format!("add_awake_function error: {}", e));
            }
        }
    }

    fn awake_functions() -> &'static RwLock<Vec<fn()>> {
        &AWAKE_FUNCTIONS
    }

    pub fn add_on_enable_function(func: fn()) {
        match ON_ENABLE_FUNCTIONS.write() {
            Ok(mut on_enable_functions) => {
                on_enable_functions.push(func);
            }
            Err(e) => {
                log_warn!(format!("add_on_enable_function error: {}", e));
            }
        }
    }

    fn on_enable_functions() -> &'static RwLock<Vec<fn()>> {
        &ON_ENABLE_FUNCTIONS
    }

    pub fn add_on_disable_function(func: fn()) {
        match ON_DISABLE_FUNCTIONS.write() {
            Ok(mut on_disable_functions) => {
                on_disable_functions.push(func);
            }
            Err(e) => {
                log_warn!(format!("add_on_disable_function error: {}", e));
            }
        }
    }

    fn on_disable_functions() -> &'static RwLock<Vec<fn()>> {
        &ON_DISABLE_FUNCTIONS
    }

    // 1
    fn awake() {
        NetworkManager::awake();
        match Self::awake_functions().try_read() {
            Ok(awake_functions) => {
                for func in awake_functions.iter() {
                    func();
                }
            }
            Err(e) => {
                log_warn!(format!("NetworkLoop.awake() error: {}", e));
            }
        }
    }

    // 2
    fn on_enable() {
        match Self::on_enable_functions().try_read() {
            Ok(on_enable_functions) => {
                for func in on_enable_functions.iter() {
                    func();
                }
            }
            Err(e) => {
                log_warn!(format!("NetworkLoop.on_enable() error: {}", e));
            }
        }
    }

    // 3
    fn start() {
        let network_manager_singleton = NetworkManagerStatic::get_network_manager_singleton();
        network_manager_singleton.start();

        NetworkServerStatic::for_each_network_message_handler(|item| {
            log_debug!(format!(
                "message hash: {} require_authentication: {}",
                item.key(),
                item.require_authentication
            ));
        });

        NetworkServerStatic::for_each_network_connection(|item| {
            log_debug!(format!(
                "connection hash: {} address: {}",
                item.key(),
                item.address
            ));
        });
    }

    // 4
    fn fixed_update() {
        // NetworkBehaviour fixed_update  模拟
        NetworkServerStatic::spawned_network_identities()
            .iter_mut()
            .for_each(|mut identity| {
                identity
                    .network_behaviours
                    .iter_mut()
                    .for_each(|behaviour| {
                        behaviour.fixed_update();
                    });
            });
    }

    // 5
    fn update() {
        // NetworkEarlyUpdate
        // AddToPlayerLoop(NetworkEarlyUpdate, typeof(NetworkLoop), ref playerLoop, typeof(EarlyUpdate), AddMode.End);
        NetworkServer::network_early_update();

        // NetworkManager update
        NetworkManagerStatic::get_network_manager_singleton().update();

        // NetworkBehaviour update  模拟
        NetworkServerStatic::spawned_network_identities()
            .iter_mut()
            .for_each(|mut identity| {
                identity
                    .network_behaviours
                    .iter_mut()
                    .for_each(|behaviour| {
                        behaviour.update();
                    });
            });
    }

    // 6
    fn late_update() {
        // NetworkLateUpdate
        // AddToPlayerLoop(NetworkLateUpdate, typeof(NetworkLoop), ref playerLoop, typeof(PreLateUpdate), AddMode.End);
        NetworkServer::network_late_update();

        // NetworkBehaviour late_update  模拟
        NetworkManagerStatic::get_network_manager_singleton().late_update();

        // NetworkBehaviour late_update
        NetworkServerStatic::spawned_network_identities()
            .iter_mut()
            .for_each(|mut identity| {
                identity
                    .network_behaviours
                    .iter_mut()
                    .for_each(|behaviour| behaviour.late_update());
            });
    }

    // 7
    fn on_disable() {
        match Self::on_disable_functions().try_read() {
            Ok(on_disable_functions) => {
                for func in on_disable_functions.iter() {
                    func();
                }
            }
            Err(e) => {
                log_warn!(format!("NetworkLoop.on_disable() error: {}", e));
            }
        }
    }

    // 8
    fn on_destroy() {
        NetworkManager::shutdown();
    }

    pub fn run() {
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


        // 1
        Self::awake();
        // 2
        Self::on_enable();
        // 3
        Self::start();

        // 目标帧率
        let target_frame_time = Duration::from_secs(1) / NetworkServerStatic::tick_rate();
        while !*stop_signal() {
            Self::fixed_update();
            Self::update();
            Self::late_update();
            NetworkTime::increment_frame_count();
            let mut sleep_time = Duration::from_secs(0);
            match NetworkServerStatic::full_update_duration().try_read() {
                Ok(full_update_duration) => {
                    // 计算平均耗费时间
                    let average_elapsed_time = Duration::from_secs_f64(full_update_duration.average());
                    // 如果平均耗费时间小于目标帧率
                    if average_elapsed_time < target_frame_time {
                        // 计算帧平均补偿睡眠时间
                        sleep_time =
                            (target_frame_time - average_elapsed_time) / NetworkTime::frame_count();
                    }
                }
                Err(e) => {
                    log_warn!(format!(
                        "Server.network_late_update() full_update_duration error: {}",
                        e
                    ));
                }
            }
            // 休眠
            thread::sleep(sleep_time);
        }

        Self::on_disable();
        Self::on_destroy();
    }
}
