use crate::messages::{CommandMessage, EntityStateMessage, NetworkPingMessage, NetworkPongMessage, ObjectDestroyMessage, ObjectSpawnFinishedMessage, ObjectSpawnStartedMessage, RpcMessage, SceneMessage, SceneOperation, SpawnMessage, TimeSnapshotMessage};
use crate::rwder::{DataReader, DataWriter, Reader, Writer};
use crate::stable_hash::StableHash;
use crate::sync_data::SyncData;
use crate::tools::{generate_id, get_s_e_t, to_hex_string};
use kcp2k_rust::error_code::ErrorCode;
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use kcp2k_rust::kcp2k_server::Server;
use nalgebra::{Quaternion, Vector3};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex, Once};
use std::thread::spawn;

#[derive(Debug)]
pub struct Connection {
    pub net_id: u32,
    /// 以下为 Mirror.Connection 类的属性
    pub connection_id: u64,
    pub address: String,
    pub is_authenticated: bool,
    /// TODO: Auth Data
    pub is_ready: bool,
    pub last_message_time: f32,
    /// TODO netid
    /// TODO 附属 netid
    pub remote_time_stamp: f64,
}

impl Connection {
    pub fn new(connection_id: u64, net_id: u32, address: String) -> Self {
        Connection {
            net_id,
            connection_id,
            address,
            is_authenticated: false,
            is_ready: false,
            last_message_time: 0.0,
            remote_time_stamp: 0.0,
        }
    }
    pub fn set_ready(&mut self, ready: bool) {
        self.is_ready = ready;
    }
}

pub struct MirrorServer {
    pub kcp_serv: Option<Arc<Mutex<Server>>>,
    pub connections_map: HashMap<u64, Connection>,
}

impl MirrorServer {
    pub fn get_instance() -> &'static Mutex<MirrorServer> {
        static mut INSTANCE: MaybeUninit<Mutex<MirrorServer>> = MaybeUninit::uninit();
        static ONCE: Once = Once::new();

        ONCE.call_once(|| unsafe {
            INSTANCE.as_mut_ptr().write(Mutex::new(MirrorServer {
                kcp_serv: None,
                connections_map: Default::default(),
            }));
        });

        unsafe { &*INSTANCE.as_ptr() }
    }

    pub fn listen() {
        // 创建 kcp 服务器配置
        let config = Kcp2KConfig::default();


        // kcp2k_rust::kcp2k_callback::Callback  管道
        let (sender, receiver) = std::sync::mpsc::channel::<kcp2k_rust::kcp2k_callback::Callback>();

        // 创建回调线程
        spawn(move || {
            while let Ok(callback) = receiver.recv() {
                match callback.callback_type {
                    kcp2k_rust::kcp2k_callback::CallbackType::OnConnected => {
                        MirrorServer::get_instance().lock().unwrap().on_connected(callback.connection_id)
                    }
                    kcp2k_rust::kcp2k_callback::CallbackType::OnData => {
                        MirrorServer::get_instance().lock().unwrap().on_data(callback.connection_id, callback.data.clone(), callback.channel.clone())
                    }
                    kcp2k_rust::kcp2k_callback::CallbackType::OnError => {
                        MirrorServer::get_instance().lock().unwrap().on_error(callback.connection_id, callback.error_code.clone(), callback.error_message.clone())
                    }
                    kcp2k_rust::kcp2k_callback::CallbackType::OnDisconnected => {
                        MirrorServer::get_instance().lock().unwrap().on_disconnected(callback.connection_id)
                    }
                }
            }
        });

        // 创建 kcp 服务器
        if let Ok(mut serv) = Server::new(config, "0.0.0.0:7777".to_string(), Arc::new(move |callback: kcp2k_rust::kcp2k_callback::Callback| {
            if let Err(_) = sender.send(callback) {
                println!("Send callback failed");
            }
        })) {
            // 启动服务器
            if let Err(_) = serv.start() {
                println!("Server start failed");
            }
            // 设置服务器
            if let Ok(mut instance) = MirrorServer::get_instance().lock() {
                instance.set_kcp_serv(serv);
            }
        }
    }

    pub fn set_kcp_serv(&mut self, serv: Server) {
        self.kcp_serv = Some(Arc::new(Mutex::new(serv)));
    }

    pub fn send(&self, connection_id: u64, writer: &Writer, channel: kcp2k_rust::kcp2k_channel::Kcp2KChannel) {
        if let Some(serv) = &self.kcp_serv {
            let mut serv = serv.lock().unwrap();
            if let Err(_) = serv.send(connection_id, writer.get_data().to_vec(), channel) {
                // TODO: 发送失败
            }
        }
    }

    pub fn send_bytes(&self, connection_id: u64, data: Vec<u8>, channel: kcp2k_rust::kcp2k_channel::Kcp2KChannel) {
        if let Some(serv) = &self.kcp_serv {
            let mut serv = serv.lock().unwrap();
            if let Err(_) = serv.send(connection_id, data, channel) {
                // TODO: 发送失败
            }
        }
    }

    pub fn disconnect(&mut self, connection_id: u64) {
        if let Some(serv) = self.kcp_serv.as_ref() {
            let mut serv = serv.lock().unwrap();
            // TODO: 服务器断开连接
            // serv.disconnect(connection_id);
        }
    }

    pub fn get_client_address(&self, connection_id: u64) -> String {
        if let Some(serv) = &self.kcp_serv {
            let serv = serv.lock().unwrap();
            // TODO: 获取客户端地址
            // if let Some(addr) = serv.get_client_address(connection_id) {
            //     return addr;
            // }
        }
        "".to_string()
    }


    pub fn on_connected(&mut self, con_id: u64) {
        println!("OnConnected {}", con_id);

        if con_id == 0 || self.connections_map.contains_key(&con_id) {
            self.disconnect(con_id);
        }

        let connection = Connection {
            net_id: generate_id(),
            connection_id: con_id,
            address: self.get_client_address(con_id),
            is_authenticated: false,
            is_ready: false,
            last_message_time: 0.0,
            remote_time_stamp: 0.0,
        };

        let mut writer = Writer::new_with_len(true);
        SceneMessage::new("Assets/QuickStart/Scenes/MyScene.scene".to_string(), SceneOperation::Normal, false).serialization(&mut writer);
        self.send(connection.connection_id, &writer, kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);
        self.connections_map.insert(con_id, connection);

        // TODO network_manager#L1177 auth class

    }

    pub fn on_data(&mut self, con_id: u64, message: Vec<u8>, channel: kcp2k_rust::kcp2k_channel::Kcp2KChannel) {
        let mut t_reader = Reader::new_with_len(&message, true);
        if let Some(connection) = self.connections_map.get_mut(&con_id) {
            connection.remote_time_stamp = t_reader.get_elapsed_time()
        }

        let mut output_first = String::new();
        output_first.push_str(&format!("Start: elapsed_time: {}\n", t_reader.get_elapsed_time()));

        while t_reader.get_remaining() > 0 {
            let mut reader = t_reader.read_one();

            output_first.push_str(&format!("msg_len: {}", reader.get_length()));

            let msg_type_hash = reader.read_u16();

            output_first.push_str(&format!(" - msg_type_hash: {} - msg_type: ", msg_type_hash));

            if msg_type_hash == "Mirror.TimeSnapshotMessage".get_stable_hash_code16() {
                output_first.push_str(&"Mirror.TimeSnapshotMessage\n".to_string());
                // print!("{}", output_first);
                if let Some(cur_connection) = self.connections_map.get(&con_id) {
                    let mut writer = Writer::new_with_len(true);
                    // 写入 TimeSnapshotMessage 数据
                    TimeSnapshotMessage {}.serialization(&mut writer);
                    // 发送 writer 数据
                    self.send(cur_connection.connection_id, &writer, kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);
                }
            } else if msg_type_hash == "Mirror.NetworkPingMessage".get_stable_hash_code16() {
                output_first.push_str(&"Mirror.NetworkPingMessage\n".to_string());
                // println!("{}", output_first);
                if let Some(cur_connection) = self.connections_map.get(&con_id) {
                    // 读取 NetworkPingMessage 数据
                    let network_ping_message = NetworkPingMessage::deserialization(&mut reader);
                    let local_time = network_ping_message.local_time;
                    let predicted_time_adjusted = network_ping_message.predicted_time_adjusted;

                    let mut writer = Writer::new_with_len(true);
                    // 准备 NetworkPongMessage 数据
                    let s_e_t = get_s_e_t();
                    let unadjusted_error = s_e_t - local_time;
                    let adjusted_error = s_e_t - predicted_time_adjusted;

                    // 写入 NetworkPongMessage 数据
                    NetworkPongMessage::new(local_time, unadjusted_error, adjusted_error).serialization(&mut writer);

                    // 写入 NetworkPingMessage 数据  test 客户端 回复 pong
                    // NetworkPingMessage::new(s_e_t, predicted_time_adjusted).write(&mut writer);

                    // 发送 writer 数据
                    self.send(cur_connection.connection_id, &writer, channel);
                }
            } else if msg_type_hash == "Mirror.NetworkPongMessage".get_stable_hash_code16() {
                output_first.push_str(&"Mirror.NetworkPongMessage\n".to_string());
                // println!("{}", output_first);
                // 读取 NetworkPongMessage 数据
                // let network_pong_message = NetworkPongMessage::read(&mut reader);
                // println!("network_pong_message: {:?}", network_pong_message);
            } else if msg_type_hash == "Mirror.ReadyMessage".get_stable_hash_code16() {
                output_first.push_str(&"Mirror.ReadyMessage\n".to_string());
                print!("{}", output_first);

                // 设置连接为准备状态
                if let Some(cur_connection) = self.connections_map.get_mut(&con_id) {
                    cur_connection.is_ready = true;
                }
            } else if msg_type_hash == "Mirror.AddPlayerMessage".get_stable_hash_code16() {
                output_first.push_str(&"Mirror.AddPlayerMessage\n".to_string());
                println!("{}", output_first);

                if let Some(cur_connection) = self.connections_map.get(&con_id) {
                    let mut cur_writer = Writer::new_with_len(true);

                    // 添加 ObjectSpawnStartedMessage 数据
                    ObjectSpawnStartedMessage {}.serialization(&mut cur_writer);

                    let position = Vector3::new(0.0, 0.0, 0.0);
                    let rotation = Quaternion::identity();
                    let scale = Vector3::new(1.0, 1.0, 1.0);

                    let cur_payload = hex::decode("01131200506C61796572363632206A6F696E65642E").unwrap();
                    let mut cur_spawn_message = SpawnMessage::new(1, false, false, 14585647484178997735, 0, position, rotation, scale, cur_payload);
                    cur_spawn_message.serialization(&mut cur_writer);

                    for (_, connection) in &self.connections_map {
                        // 自己
                        if cur_connection.connection_id == connection.connection_id {
                            let cur_payload = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
                            let mut cur_spawn_message = SpawnMessage::new(cur_connection.net_id, true, true, 0, 3541431626, position, rotation, scale, cur_payload);
                            cur_spawn_message.serialization(&mut cur_writer);


                            continue;
                        }
                        // 其它玩家
                        let other_payload = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
                        let mut other_spawn_message = SpawnMessage::new(connection.net_id, false, false, 0, 3541431626, position, rotation, scale, other_payload);
                        other_spawn_message.serialization(&mut cur_writer);
                    }

                    // 添加 ObjectSpawnStartedMessage 数据
                    ObjectSpawnFinishedMessage {}.serialization(&mut cur_writer);

                    // 发送给当前连接
                    self.send(cur_connection.connection_id, &cur_writer, kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);

                    //  *****************************************************************************
                    let mut other_writer = Writer::new_with_len(true);

                    let position = Vector3::new(0.0, 0.0, 0.0);
                    let rotation = Quaternion::identity();
                    let scale = Vector3::new(1.0, 1.0, 1.0);

                    // 添加通知其他客户端有新玩家加入消息
                    let cur_payload = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
                    let mut cur_spawn_message = SpawnMessage::new(cur_connection.net_id, false, false, 0, 3541431626, position, rotation, scale, cur_payload);
                    cur_spawn_message.serialization(&mut other_writer);


                    // 通知其他客户端有新玩家加入消息
                    for (_, connection) in &self.connections_map {
                        if cur_connection.connection_id == connection.connection_id {
                            continue;
                        }
                        self.send(connection.connection_id, &other_writer, kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);
                    }
                }
            } else if msg_type_hash == "Mirror.CommandMessage".get_stable_hash_code16() {
                output_first.push_str(&"Mirror.CommandMessage\n".to_string());
                println!("{}", output_first);

                let command_message = CommandMessage::deserialization(&mut reader);

                let net_id = command_message.net_id;
                let component_index = command_message.component_index;
                let function_hash = command_message.function_hash;
                println!("message_type: {} netId: {} componentIndex: {} functionHash: {}", msg_type_hash, net_id, component_index, function_hash);

                if function_hash == "System.Void Mirror.NetworkTransformUnreliable::CmdClientToServerSync(Mirror.SyncData)".get_fn_stable_hash_code() {
                    let mut sync_writer = Reader::new_with_len(&command_message.payload, false);
                    let sync_data = SyncData::deserialization(&mut sync_writer);
                    println!("sync_data: {:?}\n", sync_data);

                    let mut rpc_writer = Writer::new_with_len(true);
                    let mut rpc_message = RpcMessage::new(net_id, component_index, 28456, command_message.get_payload_no_len());
                    rpc_message.serialization(&mut rpc_writer);

                    // 遍历所有连接并发送消息
                    for (_, connection) in &self.connections_map {
                        self.send(connection.connection_id, &rpc_writer, kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);
                    }
                } else if function_hash == 20088 {
                    // println!("CmdClientRpc 20088 {}", to_hex_string(command_message.payload.as_slice()));

                    if let Some(cur_connection) = self.connections_map.get(&con_id) {
                        let mut writer = Writer::new_with_len(true);
                        let payload = hex::decode(format!("{}{}", "022b00000000000000000600000000000000", to_hex_string(&command_message.payload[4..]))).unwrap();
                        println!("CmdClientRpc 20088 payload: {}", to_hex_string(&payload));
                        let mut entity_state_message = EntityStateMessage::new(cur_connection.net_id, payload);
                        entity_state_message.serialization(&mut writer);
                        for (_, connection) in &self.connections_map {
                            self.send(connection.connection_id, &writer, kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);
                        }
                    }
                } else if function_hash == "System.Void QuickStart.PlayerScript::CmdShootRay()".get_fn_stable_hash_code() {
                    println!("CmdShootRay {}", to_hex_string(command_message.payload.as_slice()));

                    if let Some(cur_connection) = self.connections_map.get(&con_id) {
                        let mut writer = Writer::new_with_len(true);
                        let mut rpc_message = RpcMessage::new(cur_connection.net_id, 1, 10641, command_message.get_payload_no_len());
                        rpc_message.serialization(&mut writer);
                        for (_, connection) in &self.connections_map {
                            self.send(connection.connection_id, &writer, kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);
                        }
                    }
                } else if function_hash == "System.Void QuickStart.PlayerScript::CmdChangeActiveWeapon(System.Int32)".get_fn_stable_hash_code() {
                    println!("CmdChangeActiveWeapon {}", to_hex_string(command_message.payload.as_slice()));

                    if let Some(cur_connection) = self.connections_map.get(&con_id) {
                        let mut writer = Writer::new_with_len(true);
                        let payload = hex::decode(format!("{}{}", "021400000000000000000100000000000000", to_hex_string(&command_message.payload[4..]))).unwrap();
                        let mut entity_state_message = EntityStateMessage::new(cur_connection.net_id, payload);
                        entity_state_message.serialization(&mut writer);
                        for (_, connection) in &self.connections_map {
                            self.send(connection.connection_id, &writer, kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);
                        }
                    }
                } else {
                    println!("Unknown function hash: {}\n", function_hash);
                }
            } else {
                println!("Unknown message type: {}\n", msg_type_hash);
            }
        }
    }

    pub fn on_error(&self, con_id: u64, code: ErrorCode, message: String) {
        println!("OnError {} - {:?} {}", con_id, code, message);
    }

    pub fn on_disconnected(&mut self, con_id: u64) {
        if let Some(connection) = self.connections_map.get(&con_id) {
            let mut writer = Writer::new_with_len(true);
            let mut object_destroy_message = ObjectDestroyMessage::new(connection.net_id);
            object_destroy_message.serialization(&mut writer);
            for (_, connection) in &self.connections_map {
                if con_id == connection.connection_id {
                    continue;
                }
                self.send(connection.connection_id, &writer, kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);
            }
            self.connections_map.remove(&con_id);
        }
    }
}