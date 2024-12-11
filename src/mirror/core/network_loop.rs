use crate::log_error;
use crate::mirror::core::network_behaviour::NetworkBehaviourFactory;
use crate::mirror::core::network_manager::{
    NetworkManager, NetworkManagerStatic, NetworkManagerTrait,
};
use crate::mirror::core::network_server::{NetworkServer, NetworkServerStatic};
use crate::mirror::core::network_time::NetworkTime;
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
    // 需要 添加的 start 函数列表
    static ref START_FUNCTIONS: RwLock<Vec<fn()>> = RwLock::new(vec![]);
    // 需要 添加的 early_update 函数列表
    static ref EARLY_UPDATE_FUNCTIONS: RwLock<Vec<fn()>> = RwLock::new(vec![]);
    // 需要 添加的 update 函数列表
    static ref UPDATE_FUNCTIONS: RwLock<Vec<fn()>> = RwLock::new(vec![]);
    // 需要 添加的 late_update 函数列表
    static ref LATE_UPDATE_FUNCTIONS: RwLock<Vec<fn()>> = RwLock::new(vec![]);
    // 需要 添加的 disable 函数列表
    static ref ON_DISABLE_FUNCTIONS: RwLock<Vec<fn()>> = RwLock::new(vec![]);
    // 需要 添加的 destroy 函数列表
    static ref ON_DESTROY_FUNCTIONS: RwLock<Vec<fn()>> = RwLock::new(vec![]);
    // 需要 添加的 network_behaviour_factory 函数列表
    static ref NETWORK_BEHAVIOUR_FACTORY_FUNCTIONS: RwLock<Vec<fn()>> = RwLock::new(vec![]);
    // 需要 添加的 network_common_behaviour_delegate 函数列表
    static ref NETWORK_COMMON_BEHAVIOUR_DELEGATE_FUNCTION: RwLock<fn()> = RwLock::new(||{});
}

#[allow(warnings)]
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
                log_error!(format!("add_awake_function error: {}", e));
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
                log_error!(format!("add_on_enable_function error: {}", e));
            }
        }
    }

    fn on_enable_functions() -> &'static RwLock<Vec<fn()>> {
        &ON_ENABLE_FUNCTIONS
    }

    pub fn add_start_function(func: fn()) {
        match START_FUNCTIONS.write() {
            Ok(mut start_functions) => {
                start_functions.push(func);
            }
            Err(e) => {
                log_error!(format!("add_start_function error: {}", e));
            }
        }
    }

    fn start_functions() -> &'static RwLock<Vec<fn()>> {
        &START_FUNCTIONS
    }

    // early_update
    pub fn add_early_update_function(func: fn()) {
        match EARLY_UPDATE_FUNCTIONS.write() {
            Ok(mut early_update_functions) => {
                early_update_functions.push(func);
            }
            Err(e) => {
                log_error!(format!("add_early_update_function error: {}", e));
            }
        }
    }

    // early_update
    pub fn early_update_functions() -> &'static RwLock<Vec<fn()>> {
        &EARLY_UPDATE_FUNCTIONS
    }

    // update
    pub fn add_update_function(func: fn()) {
        match UPDATE_FUNCTIONS.write() {
            Ok(mut update_functions) => {
                update_functions.push(func);
            }
            Err(e) => {
                log_error!(format!("add_update_function error: {}", e));
            }
        }
    }

    // update
    pub fn update_functions() -> &'static RwLock<Vec<fn()>> {
        &UPDATE_FUNCTIONS
    }

    // late_update
    pub fn add_late_update_function(func: fn()) {
        match LATE_UPDATE_FUNCTIONS.write() {
            Ok(mut late_update_functions) => {
                late_update_functions.push(func);
            }
            Err(e) => {
                log_error!(format!("add_late_update_function error: {}", e));
            }
        }
    }

    // late_update
    pub fn late_update_functions() -> &'static RwLock<Vec<fn()>> {
        &LATE_UPDATE_FUNCTIONS
    }

    pub fn add_on_disable_function(func: fn()) {
        match ON_DISABLE_FUNCTIONS.write() {
            Ok(mut on_disable_functions) => {
                on_disable_functions.push(func);
            }
            Err(e) => {
                log_error!(format!("add_on_disable_function error: {}", e));
            }
        }
    }

    fn on_disable_functions() -> &'static RwLock<Vec<fn()>> {
        &ON_DISABLE_FUNCTIONS
    }

    pub fn add_on_destroy_function(func: fn()) {
        match ON_DESTROY_FUNCTIONS.write() {
            Ok(mut on_destroy_functions) => {
                on_destroy_functions.push(func);
            }
            Err(e) => {
                log_error!(format!("add_on_destroy_function error: {}", e));
            }
        }
    }

    fn on_destroy_functions() -> &'static RwLock<Vec<fn()>> {
        &ON_DESTROY_FUNCTIONS
    }

    pub fn add_network_behaviour_factory(func: fn()) {
        match NETWORK_BEHAVIOUR_FACTORY_FUNCTIONS.write() {
            Ok(mut network_behaviour_factory_functions) => {
                network_behaviour_factory_functions.push(func);
            }
            Err(e) => {
                log_error!(format!("add_network_behaviour_factory error: {}", e));
            }
        }
    }

    fn network_behaviour_factory_functions() -> &'static RwLock<Vec<fn()>> {
        &NETWORK_BEHAVIOUR_FACTORY_FUNCTIONS
    }

    // network_common_behaviour_delegate
    pub fn set_ext_network_common_behaviour_delegate_func(func: fn()) {
        match NETWORK_COMMON_BEHAVIOUR_DELEGATE_FUNCTION.write() {
            Ok(mut network_common_behaviour_delegate_function) => {
                *network_common_behaviour_delegate_function = func;
            }
            Err(e) => {
                log_error!(format!(
                    "add_network_common_behaviour_delegate error: {}",
                    e
                ));
            }
        }
    }

    // network_common_behaviour_delegate
    pub fn ext_network_common_behaviour_delegate_func() -> &'static RwLock<fn()> {
        &NETWORK_COMMON_BEHAVIOUR_DELEGATE_FUNCTION
    }

    // NetworkBehaviourFactory::register_network_behaviour_factory();
    fn register_network_behaviour_factory() {
        NetworkBehaviourFactory::register_network_behaviour_factory();
        match Self::network_behaviour_factory_functions().try_read() {
            Ok(network_behaviour_factory_functions) => {
                for func in network_behaviour_factory_functions.iter() {
                    func();
                }
            }
            Err(e) => {
                log_error!(format!(
                    "NetworkLoop.register_network_behaviour_factory() error: {}",
                    e
                ));
            }
        }
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
                log_error!(format!("NetworkLoop.awake() error: {}", e));
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
                log_error!(format!("NetworkLoop.on_enable() error: {}", e));
            }
        }
    }

    // 3
    fn start() {
        let network_manager_singleton = NetworkManagerStatic::network_manager_singleton();
        network_manager_singleton.start();

        match Self::start_functions().try_read() {
            Ok(start_functions) => {
                for func in start_functions.iter() {
                    func();
                }
            }
            Err(e) => {
                log_error!(format!("NetworkLoop.start() error: {}", e));
            }
        }
    }

    // 4
    fn early_update() {
        // NetworkEarlyUpdate
        // AddToPlayerLoop(NetworkEarlyUpdate, typeof(NetworkLoop), ref playerLoop, typeof(EarlyUpdate), AddMode.End);
        NetworkServer::network_early_update();

        match Self::early_update_functions().try_read() {
            Ok(early_update_functions) => {
                for func in early_update_functions.iter() {
                    func();
                }
            }
            Err(e) => {
                log_error!(format!("NetworkLoop.early_update() error: {}", e));
            }
        }
    }

    // 5
    fn update() {
        // NetworkManager update
        NetworkManagerStatic::network_manager_singleton().update();

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

        match Self::update_functions().try_read() {
            Ok(update_functions) => {
                for func in update_functions.iter() {
                    func();
                }
            }
            Err(e) => {
                log_error!(format!("NetworkLoop.update() error: {}", e));
            }
        }
    }

    // 6
    fn late_update() {
        // NetworkLateUpdate
        // AddToPlayerLoop(NetworkLateUpdate, typeof(NetworkLoop), ref playerLoop, typeof(PreLateUpdate), AddMode.End);
        NetworkServer::network_late_update();

        // NetworkBehaviour late_update  模拟
        NetworkManagerStatic::network_manager_singleton().late_update();

        // NetworkBehaviour late_update
        NetworkServerStatic::spawned_network_identities()
            .iter_mut()
            .for_each(|mut identity| {
                identity
                    .network_behaviours
                    .iter_mut()
                    .for_each(|behaviour| behaviour.late_update());
            });

        match Self::late_update_functions().try_read() {
            Ok(late_update_functions) => {
                for func in late_update_functions.iter() {
                    func();
                }
            }
            Err(e) => {
                log_error!(format!("NetworkLoop.late_update() error: {}", e));
            }
        }
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
                log_error!(format!("NetworkLoop.on_disable() error: {}", e));
            }
        }
    }

    // 8
    fn on_destroy() {
        let network_manager_singleton = NetworkManagerStatic::network_manager_singleton();
        network_manager_singleton.on_destroy();

        match Self::on_destroy_functions().try_read() {
            Ok(on_destroy_functions) => {
                for func in on_destroy_functions.iter() {
                    func();
                }
            }
            Err(e) => {
                log_error!(format!("NetworkLoop.on_destroy() error: {}", e));
            }
        }
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

        // 注册 NetworkBehaviourFactory
        Self::register_network_behaviour_factory();

        // 1
        Self::awake();
        // 2
        Self::on_enable();
        // 3
        Self::start();

        // 每一帧的目标时间
        let target_frame_time = Duration::from_secs(1) / NetworkServerStatic::tick_rate();
        // 循环
        while !*stop_signal() {
            // 4
            Self::early_update();
            // 5
            Self::update();
            // 6
            Self::late_update();
            // 计算帧数
            NetworkTime::increment_frame_count();
            // 计算睡眠时间
            let sleep_time = match NetworkServerStatic::full_update_duration().try_read() {
                Ok(full_update_duration) => {
                    // 计算平均耗费时间
                    let average_elapsed_time =
                        Duration::from_secs_f64(full_update_duration.average());
                    // 如果平均耗费时间小于目标帧率
                    match average_elapsed_time < target_frame_time {
                        true => {
                            // 计算帧平均补偿睡眠时间
                            (target_frame_time - average_elapsed_time) / 2
                        }
                        false => {
                            // 如果平均耗费时间大于目标帧率
                            Duration::from_secs(0)
                        }
                    }
                }
                Err(e) => {
                    log_error!(format!(
                        "Server.network_late_update() full_update_duration error: {}",
                        e
                    ));
                    Duration::from_secs(0)
                }
            };
            // 休眠
            thread::sleep(sleep_time);
        }

        Self::on_disable();
        Self::on_destroy();
    }
}
