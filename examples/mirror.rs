use crate::quick_start::player_script::PlayerScript;
use mirror_rust::log_debug;
use mirror_rust::mirror::authenticators::basic_authenticator::BasicAuthenticator;
use mirror_rust::mirror::authenticators::network_authenticator::NetworkAuthenticatorTrait;
use mirror_rust::mirror::components::network_common_behaviour::NetworkCommonBehaviour;
use mirror_rust::mirror::components::network_room_manager::NetworkRoomManager;
use mirror_rust::mirror::core::network_loop::NetworkLoop;
use mirror_rust::mirror::core::network_manager::NetworkManagerTrait;
use mirror_rust::mirror::core::network_server::NetworkServerStatic;
use mirror_rust::mirror::core::network_start_position::NetworkStartPosition;
use mirror_rust::mirror::core::remote_calls::RemoteProcedureCalls;
use mirror_rust::mirror::core::transport::TransportTrait;
use mirror_rust::mirror::transports::kcp2k::kcp2k_transport::Kcp2kTransport;
use signal_hook::consts::SIGINT;
use signal_hook::flag::register;

mod quick_start;

fn network_behaviour_factory() {
    // 可以复用 NetworkCommonBehaviour 也可以全新实现 NetworkBehaviourTrait
    // NetworkBehaviourFactory::add_network_behaviour_factory(
    //     "QuickStart.PlayerScript".to_string(),
    //     |game_object: GameObject, component: &NetworkBehaviourComponent| {
    //         Box::new(NetworkCommonBehaviour::new(game_object, component))
    //     },
    // );
}

fn ext_network_common_behaviour_delegate() {
    RemoteProcedureCalls::register_command_delegate::<NetworkCommonBehaviour>(
        "System.Void QuickStart.PlayerScript::CmdSendPlayerMessage()",
        NetworkCommonBehaviour::invoke_user_code_cmd_send_player_message_string,
        true,
    );
}

fn awake() {
    // 传输层初始化
    Kcp2kTransport::awake();
    NetworkStartPosition::awake();
    NetworkRoomManager::awake();
}

fn on_enable() {
    // 启用基础认证
    BasicAuthenticator::new("123".to_string(), "456".to_string()).enable();
}

fn start() {
    NetworkServerStatic::for_each_network_message_handler(|item| {
        log_debug!(format!(
            "message hash: {} require_authentication: {}",
            item.key(),
            item.require_authentication
        ));
    });
}

fn early_update() {}

fn update() {}

fn late_update() {
    // NetworkServerStatic::for_each_network_connection(|item| {
    //     log_debug!(format!(
    //         "connection hash: {} address: {}",
    //         item.key(),
    //         item.address
    //     ));
    // });
}

fn on_disable() {}

fn on_destroy() {}

fn main() {
    // 注册信号
    match register(SIGINT, NetworkLoop::stop().clone()) {
        Ok(s) => {
            log_debug!(format!("register signal: {:?}", s));
        }
        Err(err) => {
            panic!("{}", err);
        }
    }

    // 添加网络行为工厂
    NetworkLoop::add_network_behaviour_factory(network_behaviour_factory);
    // 设置扩展网络公共行为委托函数
    NetworkLoop::set_ext_network_common_behaviour_delegate_func(
        ext_network_common_behaviour_delegate,
    );
    // 添加 awake 函数
    NetworkLoop::add_awake_function(awake);
    // 添加 on_enable 函数
    NetworkLoop::add_on_enable_function(on_enable);
    // 添加 start 函数
    NetworkLoop::add_start_function(start);
    // 添加 early_update 函数
    NetworkLoop::add_early_update_function(early_update);
    // 添加 update 函数
    NetworkLoop::add_update_function(update);
    // 添加 late_update 函数
    NetworkLoop::add_late_update_function(late_update);
    // 添加 on_disable 函数
    NetworkLoop::add_on_disable_function(on_disable);
    // 添加 on_destroy 函数
    NetworkLoop::add_on_destroy_function(on_destroy);
    // NetworkLoop
    NetworkLoop::run();
}
