use crate::mirror::authenticators::basic_authenticator::BasicAuthenticator;
use crate::mirror::core::network_manager::{
    NetworkManager, NetworkManagerStatic, NetworkManagerTrait,
};
use crate::mirror::core::network_server::{NetworkServer, NetworkServerStatic};
use crate::mirror::core::network_start_position::NetworkStartPosition;
use crate::mirror::core::network_time::NetworkTime;
use crate::mirror::core::transport::TransportTrait;
use crate::mirror::transports::kcp2k::kcp2k_transport::Kcp2kTransport;
use crate::{log_debug, log_info, stop_signal};
use signal_hook::iterator::Signals;
use std::thread;
use std::time::{Duration, Instant};

pub struct NetworkLoop;

impl NetworkLoop {
    // 1
    fn awake() {
        Kcp2kTransport::awake();
        NetworkStartPosition::awake();
        NetworkManager::awake();
    }

    // 2
    fn on_enable() {
        let authenticator = BasicAuthenticator::new("123".to_string(), "456".to_string());
        let network_manager_singleton = NetworkManagerStatic::get_network_manager_singleton();
        network_manager_singleton.set_authenticator(Box::new(authenticator));
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
    fn on_disable() {}

    // 8
    fn on_destroy() {}

    pub fn run() {
        // 1
        Self::awake();
        // 2
        Self::on_enable();
        // 3
        Self::start();

        // 目标帧率
        let target_frame_time = Duration::from_secs(1) / NetworkServerStatic::tick_rate();
        // 休眠时间
        let mut sleep_time: Duration;
        // 上一帧时间
        let mut previous_frame_time = Instant::now();
        while !*stop_signal() {
            Self::fixed_update();
            Self::update();
            Self::late_update();
            // 计算帧时间
            let elapsed_time = previous_frame_time.elapsed();
            // 更新上一帧时间
            previous_frame_time = Instant::now();
            // 计算休眠时间
            sleep_time = if elapsed_time < target_frame_time {
                target_frame_time - elapsed_time
            } else {
                Duration::from_secs(0)
            };
            NetworkTime::increment_frame_count();
            // 休眠
            thread::sleep(sleep_time);
        }

        Self::on_disable();
        Self::on_destroy();
    }
}
