use crate::messages::{CommandMessage, NetworkPingMessage, NetworkPongMessage, ObjectSpawnFinishedMessage, ObjectSpawnStartedMessage, SceneMessage, SceneOperation, SpawnMessage, TimeSnapshotMessage};
use crate::rwder::{DataReader, DataWriter, Reader, Writer};
use crate::stable_hash::StableHash;
use crate::sync_data::SyncData;
use crate::tools::{get_start_elapsed_time, to_hex_string};
use kcp2k_rust::error_code::ErrorCode;
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use kcp2k_rust::kcp2k_server::Server;
use nalgebra::Vector3;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex, Once};
use std::thread::spawn;

#[derive(Debug)]
pub struct Connection {
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

pub struct MirrorServer {
    pub kcp_serv: Option<Arc<Mutex<Server>>>,
    pub connections: HashMap<u64, Connection>,
}

impl MirrorServer {
    pub fn get_instance() -> &'static Mutex<MirrorServer> {
        static mut INSTANCE: MaybeUninit<Mutex<MirrorServer>> = MaybeUninit::uninit();
        static ONCE: Once = Once::new();

        ONCE.call_once(|| unsafe {
            INSTANCE.as_mut_ptr().write(Mutex::new(MirrorServer {
                kcp_serv: None,
                connections: Default::default(),
            }));
        });

        unsafe { &*INSTANCE.as_ptr() }
    }

    pub fn listen() {
        // 创建 KCP 服务器配置
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

        // 创建 KCP 服务器
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

    pub fn send(&self, connection_id: u64, writer: Writer, channel: kcp2k_rust::kcp2k_channel::Kcp2KChannel) {
        if let Some(serv) = &self.kcp_serv {
            let mut serv = serv.lock().unwrap();
            serv.send(connection_id, writer.get_data(), channel).expect("TODO: panic message");
        }
    }

    pub fn disconnect(&self, connection_id: u64) {
        if let Some(serv) = &self.kcp_serv {
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

        if con_id == 0 {
            self.disconnect(con_id);
        }

        if self.connections.contains_key(&con_id) {
            self.disconnect(con_id);
        }

        let connection = Connection {
            connection_id: con_id,
            address: self.get_client_address(con_id),
            is_authenticated: false,
            is_ready: false,
            last_message_time: 0.0,
            remote_time_stamp: 0.0,
        };

        let mut writer = Writer::new_with_len(true);
        SceneMessage::new("Assets/QuickStart/Scenes/MyScene.scene".to_string(), SceneOperation::Normal, false).write(&mut writer);
        println!("SceneMessage: {:?} {}", writer.get_data(), to_hex_string(writer.get_data().as_slice()));
        self.send(connection.connection_id, writer, kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);
        self.connections.insert(con_id, connection);


        // TODO network_manager#L1177 auth class

    }

    pub fn on_data(&self, con_id: u64, message: Vec<u8>, channel: kcp2k_rust::kcp2k_channel::Kcp2KChannel) {
        // println!("OnData {} {:?} {:?}", con_id, channel, message);

        if let Some(connection) = self.connections.get(&con_id) {
            let mut output_first = String::new();

            let mut t_reader = Reader::new_with_len(&message, true);
            let elapsed_time = t_reader.get_elapsed_time();
            output_first.push_str(&format!("Start: elapsed_time: {}\n", elapsed_time));

            while t_reader.get_remaining() > 0 {
                let mut reader = t_reader.read_one();

                output_first.push_str(&format!("msg_len: {}", reader.get_length()));

                let msg_type_hash = reader.read_u16();

                output_first.push_str(&format!(" - msg_type_hash: {} - msg_type: ", msg_type_hash));

                if msg_type_hash == "Mirror.TimeSnapshotMessage".get_stable_hash_code16() {
                    output_first.push_str(&"Mirror.TimeSnapshotMessage\n".to_string());
                    print!("{}", output_first);

                    let mut writer = Writer::new_with_len(true);
                    // 写入 TimeSnapshotMessage 数据
                    TimeSnapshotMessage {}.write(&mut writer);

                    // 发送 writer 数据
                    self.send(connection.connection_id, writer, kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);
                } else if msg_type_hash == "Mirror.NetworkPingMessage".get_stable_hash_code16() {
                    output_first.push_str(&"Mirror.NetworkPingMessage\n".to_string());
                    println!("{}", output_first);

                    // 读取 NetworkPingMessage 数据
                    let network_ping_message = NetworkPingMessage::read(&mut reader);
                    println!("network_ping_message: {:?}", network_ping_message);
                    let local_time = network_ping_message.local_time;
                    // println!("local_time: {}", local_time);
                    let predicted_time_adjusted = network_ping_message.predicted_time_adjusted;
                    // println!("predicted_time_adjusted: {}", predicted_time_adjusted);

                    let mut writer = Writer::new_with_len(true);

                    // 准备 NetworkPongMessage 数据
                    let s_e_t = get_start_elapsed_time();
                    let unadjusted_error = s_e_t - local_time;
                    let adjusted_error = s_e_t - predicted_time_adjusted;

                    // 写入 NetworkPongMessage 数据
                    NetworkPongMessage::new(local_time, unadjusted_error, adjusted_error).write(&mut writer);

                    // 写入 NetworkPingMessage 数据  test 客户端 回复 pong
                    // NetworkPingMessage::new(s_e_t, predicted_time_adjusted).write(&mut writer);

                    // 发送 writer 数据
                    self.send(connection.connection_id, writer, channel);
                } else if msg_type_hash == "Mirror.NetworkPongMessage".get_stable_hash_code16() {
                    output_first.push_str(&"Mirror.NetworkPongMessage\n".to_string());
                    println!("{}", output_first);

                    // 读取 NetworkPongMessage 数据
                    let network_pong_message = NetworkPongMessage::read(&mut reader);
                    println!("network_pong_message: {:?}", network_pong_message);
                } else if msg_type_hash == "Mirror.CommandMessage".get_stable_hash_code16() {
                    output_first.push_str(&"Mirror.CommandMessage\n".to_string());
                    println!("{}", output_first);

                    let command_message = CommandMessage::read(&mut reader);

                    let net_id = command_message.net_id;
                    let component_index = command_message.component_index;
                    let function_hash = command_message.function_hash;

                    println!("message_type: {} netId: {} componentIndex: {} functionHash: {}", msg_type_hash, net_id, component_index, function_hash);
                    if function_hash == "System.Void Mirror.NetworkTransformUnreliable::CmdClientToServerSync(Mirror.SyncData)".get_fn_stable_hash_code() {
                        let mut sync_writer = Reader::new_with_len(&command_message.payload, false);
                        let sync_data = SyncData::read(&mut sync_writer);
                        println!("sync_data: {:?}\n", sync_data);
                    } else if function_hash == "System.Void QuickStart.PlayerScript::CmdShootRay()".get_fn_stable_hash_code() {
                        // println!("CmdShootRay");
                    } else if function_hash == "System.Void QuickStart.PlayerScript::CmdChangeActiveWeapon(System.Int32)".get_fn_stable_hash_code() {
                        // let _ = reader.read_u32();
                        // let weapon_index = reader.read_i32();
                        // println!("CmdChangeActiveWeapon: {}", weapon_index);
                    } else {
                        println!("Unknown function hash: {}\n", function_hash);
                    }
                } else if msg_type_hash == "Mirror.ReadyMessage".get_stable_hash_code16() {
                    output_first.push_str(&"Mirror.ReadyMessage\n".to_string());
                    print!("{}", output_first);

                    let mut writer = Writer::new_with_len(true);

                    // 准备 ObjectSpawnStartedMessage 数据并写入 writer
                    ObjectSpawnStartedMessage {}.write(&mut writer);

                    // pub position: Vector3<f32>,
                    // pub rotation: Quaternion<f32>,
                    // pub scale: Vector3<f32>,

                    let position = Vector3::new(0.0, 0.0, 0.0);
                    let rotation = nalgebra::Quaternion::identity();
                    let scale = Vector3::new(0.0, 0.0, 0.0);

                    let play = hex::decode("01131200506C61796572363632206A6F696E65642E").unwrap();

                    let mut ss = SpawnMessage::new(1, false, false, 14585647484178997735, 0, position, rotation, scale, play);
                    ss.write(&mut writer);

                    // 发送 ObjectSpawnStartedMessage 数据并写入 writer
                    ObjectSpawnFinishedMessage {}.write(&mut writer);

                    let play0 = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
                    let mut ss0 = SpawnMessage::new(5, false, false, 0, 3541431626, position, rotation, scale, play0);
                    ss0.write(&mut writer);

                    println!("writer: {:?} {}", writer.get_data(), to_hex_string(writer.get_data().as_slice()));

                    // 发送 writer 数据
                    self.send(connection.connection_id, writer, kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);
                } else if msg_type_hash == "Mirror.AddPlayerMessage".get_stable_hash_code16() {
                    println!("{}", output_first);
                } else {
                    println!("Unknown message type: {}\n", msg_type_hash);
                }
            }
        }
    }

    pub fn on_error(&self, con_id: u64, code: ErrorCode, message: String) {
        println!("OnError {} - {:?} {}", con_id, code, message);
    }

    pub fn on_disconnected(&self, con_id: u64) {
        println!("OnDisconnected {}", con_id);
    }
}