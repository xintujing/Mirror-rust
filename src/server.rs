use crate::rwder::{Reader, Writer};
use crate::stable_hash::StableHash;
use crate::sync_data::{SyncData, SyncDataReaderReader};
use crate::tools::{get_elapsed_time_f64, to_hex_string};
use kcp2k_rust::error_code::ErrorCode;
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use kcp2k_rust::kcp2k_server::Server;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex, Once};
use std::thread::spawn;
use std::time::Instant;

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
    pub instant: Instant,
}

impl MirrorServer {
    pub fn get_instance() -> &'static Mutex<MirrorServer> {
        static mut INSTANCE: MaybeUninit<Mutex<MirrorServer>> = MaybeUninit::uninit();
        static ONCE: Once = Once::new();

        ONCE.call_once(|| unsafe {
            INSTANCE.as_mut_ptr().write(Mutex::new(MirrorServer {
                kcp_serv: None,
                connections: Default::default(),
                instant: Instant::now(),
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

    pub fn send(&self, connection_id: u64, data: Vec<u8>, channel: kcp2k_rust::kcp2k_channel::Kcp2KChannel) {
        if let Some(serv) = &self.kcp_serv {
            let mut serv = serv.lock().unwrap();
            let mut writer = Writer::new_with_len();
            writer.write_f64(get_elapsed_time_f64(self.instant));
            writer.write(data.as_slice());
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

        // let local_time = get_elapsed_time_f64(self.instant);
        //
        // let predicted_time_adjusted = local_time + 0.1;
        //
        // let mut writer = Writer::new_with_len();
        // writer.compress_var_uint(18);
        // writer.write_u16(17487);
        //
        //
        // writer.write_f64(local_time);
        // writer.write_f64(predicted_time_adjusted);
        //
        // self.send(connection.connection_id, writer.get_data(), kcp2k_rust::kcp2k_channel::Kcp2KChannel::Unreliable);

        let mut writer = Writer::new_with_len();
        let d = vec![44, 224, 13, 39, 0, 65, 115, 115, 101, 116, 115, 47, 81, 117, 105, 99, 107, 83, 116, 97, 114, 116, 47, 83, 99, 101, 110, 101, 115, 47, 77, 121, 83, 99, 101, 110, 101, 46, 115, 99, 101, 110, 101, 0, 0];
        writer.write(d.as_slice());
        self.send(connection.connection_id, writer.get_data(), kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);
        self.connections.insert(con_id, connection);


        // TODO network_manager#L1177 auth class

    }

    pub fn on_data(&self, con_id: u64, message: Vec<u8>, channel: kcp2k_rust::kcp2k_channel::Kcp2KChannel) {
        // println!("OnData {} {:?} {:?}", con_id, channel, message);

        if let Some(connection) = self.connections.get(&con_id) {
            let mut output_first = String::new();


            let mut t_message = Reader::new_with_len(&message);

            let time_t = t_message.read_f64();

            output_first.push_str(&format!("time_t: {}\n", time_t));

            while t_message.get_remaining() > 0 {
                let mut s_message = t_message.read_one();

                output_first.push_str(&format!("len: {}", s_message.get_length()));

                let type_ = s_message.read_u16();

                output_first.push_str(&format!(" - type: {}\n", type_));

                if type_ == "Mirror.TimeSnapshotMessage".get_stable_hash_code16() {
                    print!("{}", output_first);
                    // println!("u16: {}", u01);
                } else if type_ == "Mirror.NetworkPingMessage".get_stable_hash_code16() {
                    println!("{}", output_first);

                    let local_time = s_message.read_f64();
                    // println!("local_time: {}", local_time);

                    let predicted_time_adjusted = s_message.read_f64();
                    // println!("predicted_time_adjusted: {}", predicted_time_adjusted);

                    // 1 local_time
                    // 2 unadjustedError
                    // 3 adjustedError

                    let lt = get_elapsed_time_f64(self.instant);

                    let unadjusted_error = lt - local_time;
                    let adjusted_error = lt - predicted_time_adjusted;

                    let mut writer = Writer::new_with_len();
                    writer.compress_var_uint(26);
                    writer.write_u16(27095);
                    writer.write_f64(local_time);
                    writer.write_f64(unadjusted_error);
                    writer.write_f64(adjusted_error);
                    println!("data hex: {}", to_hex_string(writer.get_data().as_slice()));
                    self.send(connection.connection_id, writer.get_data(), channel);
                } else if type_ == "Mirror.NetworkPongMessage".get_stable_hash_code16() {
                    println!("{}", output_first);
                    // let data = &data[data_start..];
                    // let a = bytes_to_f64(&data[..8]);
                    // println!("u16: {}", a);
                    // let b = bytes_to_f64(&data[8..16]);
                    // println!("u16: {}", b);
                    // let c = bytes_to_f64(&data[16..24]);
                    // println!("u16: {}", c);
                } else if type_ == "Mirror.CommandMessage".get_stable_hash_code16() {
                    println!("{}", output_first);
                    let net_id = s_message.read_u32();
                    let component_index = s_message.read_u8();
                    let function_hash = s_message.read_u16();
                    println!("message_type: {} netId: {} componentIndex: {} functionHash: {}", type_, net_id, component_index, function_hash);
                    if function_hash == "System.Void Mirror.NetworkTransformUnreliable::CmdClientToServerSync(Mirror.SyncData)".get_fn_stable_hash_code() {
                        let sync = s_message.read_remaining();
                        println!("sync_data: {:?} {}", sync, to_hex_string(sync));
                        let sync_data = SyncData::read_sync_data(&sync);
                        println!("sync_data: {:?}\n", sync_data);
                    } else if function_hash == "System.Void QuickStart.PlayerScript::CmdShootRay()".get_fn_stable_hash_code() {
                        println!("CmdShootRay");
                    } else if function_hash == "System.Void QuickStart.PlayerScript::CmdChangeActiveWeapon(System.Int32)".get_fn_stable_hash_code() {
                        let _ = s_message.read_u32();
                        let weapon_index = s_message.read_i32();
                        println!("CmdChangeActiveWeapon: {}", weapon_index);
                    } else {
                        println!("Unknown function hash: {}\n", function_hash);
                    }
                } else if type_ == "Mirror.ReadyMessage".get_stable_hash_code16() {
                    print!("{}", output_first);

                    let mut writer = Writer::new_with_len();
                    writer.compress_var_uint(2);
                    writer.write_u16(12504);
                    self.send(connection.connection_id, writer.get_data(), kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);

                    let mut writer = Writer::new_with_len();
                    writer.compress_var_uint(2);
                    writer.write_u16(43444);
                    self.send(connection.connection_id, writer.get_data(), kcp2k_rust::kcp2k_channel::Kcp2KChannel::Reliable);
                } else if type_ == "Mirror.AddPlayerMessage".get_stable_hash_code16() {
                    println!("{}", output_first);
                } else {
                    println!("Unknown message type: {}\n", type_);
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