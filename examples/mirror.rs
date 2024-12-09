use crate::quick_start::player_script::PlayerScript;
use Mirror_rust::log_debug;
use Mirror_rust::mirror::authenticators::basic_authenticator::BasicAuthenticator;
use Mirror_rust::mirror::authenticators::network_authenticator::NetworkAuthenticatorTrait;
use Mirror_rust::mirror::components::network_common_behaviour::NetworkCommonBehaviour;
use Mirror_rust::mirror::core::backend_data::NetworkBehaviourComponent;
use Mirror_rust::mirror::core::network_behaviour::{
    GameObject, NetworkBehaviourFactory, NetworkBehaviourTrait,
};
use Mirror_rust::mirror::core::network_loop::NetworkLoop;
use Mirror_rust::mirror::core::network_manager::NetworkManagerStatic;
use Mirror_rust::mirror::core::network_server::NetworkServerStatic;
use Mirror_rust::mirror::core::network_start_position::NetworkStartPosition;
use Mirror_rust::mirror::core::remote_calls::RemoteProcedureCalls;
use Mirror_rust::mirror::core::transport::TransportTrait;
use Mirror_rust::mirror::transports::kcp2k::kcp2k_transport::Kcp2kTransport;

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
