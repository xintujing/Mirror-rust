// use std::sync::{Mutex, MutexGuard};
// use lazy_static::lazy_static;
//
// //  玩家生成方式
// #[derive(Default)]
// pub enum PlayerSpawnMethod {
//     #[default]
//     Random,
//     RoundRobin,
// }
// // 网络管理模式
// pub enum NetworkManagerMode {
//     Offline,
//     ServerOnly,
//     ClientOnly,
//     Host,
// }
//
// #[derive(Default)]
// pub enum HeadlessStartOptions {
//     #[default]
//     DoNothing,
//     AutoStartServer,
//     AutoStartClient,
// }
//
// // 假设 Transform 是你定义的结构体
// struct Transform {
//     x: f32,
//     y: f32,
//     z: f32,
// }
//
// lazy_static! {
//     // 由 NetworkStartPositions 填充的转换列表，使用 Mutex 保证线程安全
//     static ref START_POSITIONS: Mutex<Vec<Transform>> = Mutex::new(Vec::new());
//
//     static ref START_POSITION_INDEX : Mutex<usize> = Mutex::new(0);
//
//     // The one and only NetworkManager
//     static ref NETWORK_MANAGER: Mutex<NetworkManager> = Mutex::new(NetworkManager::default());
// }
//
// pub struct GlobalInterface {}
//
// impl GlobalInterface {
//     // 获取全局 START_POSITIONS
//     fn get_start_positions() -> MutexGuard<'static, Vec<Transform>> {
//         START_POSITIONS.lock().unwrap()
//     }
//
//     // 获取全局 START_POSITION_INDEX
//     fn get_start_position_index() -> MutexGuard<'static, usize> {
//         START_POSITION_INDEX.lock().unwrap()
//     }
//
//     // 获取全局 NETWORK_MANAGER
//     fn get_network_manager() -> MutexGuard<'static, NetworkManager> {
//         NETWORK_MANAGER.lock().unwrap()
//     }
// }
//
//
// #[derive(Default)]
// struct NetworkManager {
//     // 是否应该通过场景更改来保留 Network Manager 对象？
//     // default true
//     pub dont_destroy_on_load: bool,
//
//     // 多人游戏应始终在后台运行，以便网络不会超时。
//     // default true
//     pub run_in_background: bool,
//
//     // 选择 Server 或 Client 是否应在无头构建中自动启动
//     // default HeadlessStartOptions::DoNothing
//     pub headless_start_mode: HeadlessStartOptions,
//
//     // 编辑器中的无头启动模式启用后，编辑器中也将使用无头启动模式。
//     pub editor_auto_start: bool,
//
//     // 服务器/客户端每秒发送速率。
//     // 对于像 Counter-Strike 这样的快节奏游戏，使用 60-100Hz 以最小化延迟。
//     // 对于像 WoW 这样的游戏，使用大约 30Hz 以最小化计算。
//     // 对于像 EVE 这样的慢节奏游戏，使用大约 1-10Hz。
//     // default 60
//     pub send_rate: u32,
//
//     // Mirror 在客户端或服务器停止时将切换到的场景
//     pub offline_scene: String,
//
//     // Mirror 在服务器启动时将切换到的场景。客户端将在连接时收到 Scene Message 以加载服务器的当前场景。
//     pub online_scene: String,
//
//     // 断开连接后可用于显示“连接丢失...”的可选延迟message 或类似信息，这在大型项目中可能需要很长时间。
//     // 0-60  default 0
//     pub offline_scene_load_delay: f32,
//
//     // 连接到服务器和客户端将用于连接的此对象的传输组件
//     pub transport: Transport,
//
//     // 客户端应连接到服务器的 Network Address。Server 不会将其用于任何用途。
//     // default localhost
//     pub network_address: String,
//
//     // 最大并发连接数。
//     // default 100
//     pub max_connections: u32,
//
//     // 启用后，服务器会在配置的超时后自动断开非活动连接。
//     pub disconnect_inactive_connections: bool,
//
//     // 如果启用了 'disconnectInactiveConnections'，则服务器自动断开非活动连接的超时时间（以秒为单位）。
//     // default 60
//     pub disconnect_inactive_timeout: f32,
//
//     // 附加到此对象的身份验证组件
//     pub network_authenticator: NetworkAuthenticator,
//
//     // 玩家对象的预制件。预制件必须具有 Network Identity 组件。可以是空游戏对象或完整头像。
//     pub player_prefab: GameObject,
//
//     // 场景改变后镜子是否应该自动生成玩家？
//     // default true
//     pub auto_create_player: bool,
//
//     // 开始位置选择的循环法或随机顺序
//     pub player_spawn_method: PlayerSpawnMethod,
//
//     // 可以通过网络生成的预制件需要在此处注册。
//     pub spawn_prefabs: Vec<GameObject>,
//
//     // 为了安全起见，如果网络操作触发异常，建议断开玩家连接
//     // 这可能会阻止在未定义状态下访问组件，这可能是漏洞利用的攻击媒介。
//     // 但是，某些游戏可能希望允许异常以免打扰玩家的体验。
//     // default true
//     pub exceptions_disconnect: bool,
//
//     pub snapshot_settings: SnapshotInterpolationSettings,
//
//     pub evaluation_method: ConnectionQualityMethod,
//
//     // 评估连接质量的时间间隔（以秒为单位）。
//     // 设置为 0 以禁用连接质量评估。
//     // default 3
//     pub evaluation_interval: f32,
//
//     // 插值 UI - 需要编辑器/开发版本
//     // default false
//     pub time_interpolation_gui: bool,
//
// }
//
// impl NetworkManager {}
//
// #[cfg(test)]
// mod test {
//     use super::*;
//
//     #[test]
//     fn test_network_manager() {
//
//         // 添加一个新的开始位置
//         GlobalInterface::get_start_positions().push(Transform {
//             x: 0.2,
//             y: 0.2,
//             z: 0.3,
//         });
//
//         // 添加另一个新的开始位置
//         GlobalInterface::get_start_positions().push(Transform {
//             x: 0.4,
//             y: 0.4,
//             z: 0.5,
//         });
//
//
//         // 获取开始位置
//         let mut start_positions = GlobalInterface::get_start_positions();
//         for position in start_positions.iter_mut() {
//             println!("Start Position: x: {}, y: {}, z: {}", position.x, position.y, position.z);
//             position.x += 1.0;
//             position.y += 1.0;
//             position.z += 1.0;
//         }
//
//
//         for position in start_positions.iter() {
//             println!("Start Position: x: {}, y: {}, z: {}", position.x, position.y, position.z);
//         }
//     }
// }