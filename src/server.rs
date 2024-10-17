use crate::backend_data::{import, BackendData, MethodType};
use crate::batcher::{Batch, DataReader, DataWriter, UnBatch};
use crate::connect::Connect;
use crate::messages::{AddPlayerMessage, CommandMessage, EntityStateMessage, NetworkPingMessage, NetworkPongMessage, ObjectSpawnFinishedMessage, ObjectSpawnStartedMessage, ReadyMessage, RpcMessage, SceneMessage, SceneOperation, SpawnMessage, TimeSnapshotMessage};
use crate::network_identity::{NetworkIdentity, SyncVar};
use crate::stable_hash::StableHash;
use crate::sync_data::SyncData;
use crate::tools::{generate_id, get_s_e_t, to_hex_string};
use bytes::Bytes;
use dashmap::DashMap;
use kcp2k_rust::error_code::ErrorCode;
use kcp2k_rust::kcp2k::Kcp2K;
use kcp2k_rust::kcp2k_callback::{Callback, CallbackType};
use kcp2k_rust::kcp2k_channel::Kcp2KChannel;
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use nalgebra::Vector3;
use std::process::exit;
use std::sync::mpsc;
use tklog::{debug, error};

type MapBridge = String;

pub struct MirrorServer {
    pub kcp_serv: Option<Kcp2K>,
    pub kcp_serv_rx: Option<mpsc::Receiver<Callback>>,
    pub uid_con_map: DashMap<MapBridge, Connect>,
    pub cid_user_map: DashMap<u64, MapBridge>,
    pub backend_data: BackendData,
}

impl MirrorServer {
    pub fn new(addr: String) -> Self {
        // 创建 kcp 服务器配置
        let config = Kcp2KConfig::default();
        match Kcp2K::new_server(config, addr) {
            Ok((server, s_rx)) => {
                Self {
                    kcp_serv: Some(server),
                    kcp_serv_rx: Some(s_rx),
                    uid_con_map: Default::default(),
                    cid_user_map: Default::default(),
                    backend_data: import(),
                }
            }
            Err(err) => {
                error!(format!("MirrorServer: {:?}", err));
                exit(1)
            }
        }
    }

    pub fn start(&self) {
        while let Some(kcp_serv) = self.kcp_serv.as_ref() {
            kcp_serv.tick();
            // 服务器接收
            if let Some(rx) = self.kcp_serv_rx.as_ref() {
                if let Ok(cb) = rx.try_recv() {
                    match cb.callback_type {
                        CallbackType::OnConnected => {
                            self.on_connected(cb.connection_id);
                        }
                        CallbackType::OnData => {
                            self.on_data(cb.connection_id, cb.data, cb.channel);
                        }
                        CallbackType::OnDisconnected => {
                            self.on_disconnected(cb.connection_id);
                        }
                        CallbackType::OnError => {
                            self.on_error(cb.connection_id, cb.error_code, cb.error_message);
                        }
                    }
                }
            }
        }
    }

    pub fn send(&self, connection_id: u64, writer: &Batch, channel: Kcp2KChannel) {
        if let Some(serv) = self.kcp_serv.as_ref() {
            if let Err(_) = serv.s_send(connection_id, writer.get_bytes(), channel) {
                // TODO: 发送失败
            }
        }
    }

    #[allow(dead_code)]
    pub fn send_bytes(&self, connection_id: u64, data: Bytes, channel: Kcp2KChannel) {
        if let Some(serv) = self.kcp_serv.as_ref() {
            if let Err(_) = serv.s_send(connection_id, data, channel) {
                // TODO: 发送失败
            }
        }
    }
    #[allow(dead_code)]
    pub fn disconnect(&self, connection_id: u64) {
        debug!(format!("Disconnect {}", connection_id));
    }

    #[allow(dead_code)]
    pub fn get_client_address(connection_id: u64) -> String {
        let _ = connection_id;
        "".to_string()
    }


    #[allow(dead_code)]
    pub fn on_connected(&self, con_id: u64) {
        debug!(format!("OnConnected {}", con_id));

        if con_id == 0 {
            return;
        }
    }

    #[allow(dead_code)]
    pub fn on_data(&self, con_id: u64, message: Bytes, channel: Kcp2KChannel) {
        // 读取消息
        let mut batch = UnBatch::new(message);
        // 读取时间戳
        let remote_time_stamp = match batch.read_f64_le() {
            Ok(rts) => rts,
            Err(err) => {
                error!(format!("on_data: {:?}", err));
                return;
            }
        };

        // 更新连接时间
        let _ = self.handel_connect(con_id, |connect| {
            connect.last_message_time = remote_time_stamp;
            Ok(connect.clone())
        });

        while let Ok(mut batch) = batch.read_next() {
            // 读取消息类型
            let msg_type_hash = match batch.read_u16_le() {
                Ok(hash) => hash,
                Err(_) => continue,
            };
            // 处理消息 start

            // TimeSnapshotMessage
            if msg_type_hash == TimeSnapshotMessage::FULL_NAME.get_stable_hash_code16() {
                self.handel_time_snapshot_message(con_id, &mut batch, channel);
                continue;
            }
            // NetworkPingMessage
            if msg_type_hash == NetworkPingMessage::FULL_NAME.get_stable_hash_code16() {
                self.handel_network_ping_message(con_id, &mut batch, channel);
                continue;
            }
            // NetworkPongMessage
            if msg_type_hash == NetworkPongMessage::FULL_NAME.get_stable_hash_code16() {
                self.handel_network_pong_message(con_id, &mut batch, channel);
                continue;
            }
            // ReadyMessage
            if msg_type_hash == ReadyMessage::FULL_NAME.get_stable_hash_code16() {
                self.handel_ready_message(con_id, &mut batch, channel);
                continue;
            }
            // AddPlayerMessage
            if msg_type_hash == AddPlayerMessage::FULL_NAME.get_stable_hash_code16() {
                self.handel_add_player_message(con_id, &mut batch, channel);
                continue;
            }
            // CommandMessage
            if msg_type_hash == CommandMessage::FULL_NAME.get_stable_hash_code16() {
                self.handel_command_message(con_id, &mut batch, channel);
                continue;
            }
            // NetworkAuthMessage
            if msg_type_hash == 38882 {
                self.handel_network_auth_message(con_id, &mut batch, channel);
                continue;
            }
            // 处理消息 end
            debug!(format!("Unknown message type: {}\n", msg_type_hash));
        }
    }

    #[allow(dead_code)]
    pub fn on_error(&self, con_id: u64, code: ErrorCode, message: String) {
        debug!(format!("OnError {} - {:?} {}", con_id, code, message));
    }

    #[allow(dead_code)]
    pub fn on_disconnected(&self, con_id: u64) {
        debug!(format!("OnDisconnected {}", con_id));
        self.cid_user_map.remove(&con_id);
        let _ = self.handel_connect(con_id, |connect| {
            connect.is_ready = false;
            connect.is_authenticated = false;
            Ok(connect.clone())
        });
    }

    #[allow(dead_code)]
    pub fn switch_scene(&self, con_id: u64, scene_name: String, custom_handling: bool) {
        let mut writer = Batch::new();
        writer.write_f64_le(get_s_e_t());
        SceneMessage::new(scene_name, SceneOperation::Normal, custom_handling).serialization(&mut writer);
        self.send(con_id, &writer, Kcp2KChannel::Reliable);
    }

    #[allow(dead_code)]
    pub fn handel_connect<F>(&self, con_id: u64, func: F) -> Result<Connect, String>
    where
        F: FnOnce(&mut Connect) -> Result<Connect, String>,
    {
        let user_name = match self.cid_user_map.get(&con_id) {
            None => { return Err("can't find in cid_user_map".to_string()) }
            Some(user) => user.clone()
        };

        match self.uid_con_map.get_mut(&user_name) {
            None => { Err("can't find in uid_con_map".to_string()) }
            Some(mut connect) => func(connect.value_mut())
        }
    }

    pub fn handel_network_identity<F>(&self, net_id: u32, func: F) -> Result<NetworkIdentity, String>
    where
        F: FnOnce(&mut NetworkIdentity) -> Result<NetworkIdentity, String>,
    {
        for mut connect in self.uid_con_map.iter_mut() {
            for identity in connect.owned_identity.iter_mut() {
                if identity.net_id == net_id {
                    return func(identity);
                }
            }
            if connect.identity.net_id == net_id {
                return func(&mut connect.identity);
            }
        }
        Err("can't find in uid_con_map".to_string())
    }

    // 处理 TimeSnapshotMessage 消息
    #[allow(dead_code)]
    pub fn handel_time_snapshot_message(&self, con_id: u64, un_batch: &mut UnBatch, channel: Kcp2KChannel) {
        let mut batch = Batch::new();
        batch.write_f64_le(get_s_e_t());
        TimeSnapshotMessage {}.serialization(&mut batch);
        // println!("handel_time_snapshot_message: {}", get_s_e_t());
        self.send(con_id, &batch, channel);
    }

    // 处理 NetworkPingMessage 消息
    #[allow(dead_code)]
    pub fn handel_network_ping_message(&self, con_id: u64, reader: &mut UnBatch, channel: Kcp2KChannel) {
        // 读取 NetworkPingMessage 数据
        let network_ping_message = match NetworkPingMessage::deserialization(reader) {
            Ok(npm) => npm,
            Err(err) => {
                error!(format!("handel_network_ping_message: {:?}", err));
                return;
            }
        };

        let _ = self.handel_connect(con_id, |cur_connect| {
            let local_time = network_ping_message.local_time;
            let predicted_time_adjusted = network_ping_message.predicted_time_adjusted;

            let mut writer = Batch::new();
            writer.write_f64_le(get_s_e_t());
            // 准备 NetworkPongMessage 数据
            let s_e_t = get_s_e_t();
            let unadjusted_error = s_e_t - local_time;
            let adjusted_error = s_e_t - predicted_time_adjusted;

            // 写入 NetworkPongMessage 数据
            NetworkPongMessage::new(local_time, unadjusted_error, adjusted_error).serialization(&mut writer);

            // 发送 writer 数据
            self.send(cur_connect.connect_id, &writer, Kcp2KChannel::Reliable);
            Ok(cur_connect.clone())
        });
    }

    #[allow(dead_code)]
    pub fn handel_network_auth_message(&self, con_id: u64, un_batch: &mut UnBatch, channel: Kcp2KChannel) {
        let username = un_batch.read(8).unwrap();
        // let password = un_batch.read_string_le().unwrap();
        let username = String::from_utf8(Vec::from(username)).unwrap();

        // println!("username: {}, password: {}", username, password);
        println!("username: {}", username);

        let mut batch = Batch::new();
        batch.write_f64_le(get_s_e_t());
        batch.compress_var_u64_le(5);
        batch.write_u16_le(56082);
        batch.write_u8(100);
        batch.write_u16_le(0);
        self.send(con_id, &batch, channel);

        // 认证成功
        self.cid_user_map.insert(con_id, username.clone());

        if let Err(_) = self.handel_connect(con_id, |cur_connect| {
            cur_connect.connect_id = con_id;
            cur_connect.is_authenticated = true;

            /// TODO: 断线重连
            self.switch_scene(con_id, "Assets/QuickStart/Scenes/MyScene.scene".to_string(), false);

            Ok(cur_connect.clone())
        }) {
            // 切换场景
            self.switch_scene(con_id, "Assets/QuickStart/Scenes/MyScene.scene".to_string(), false);

            let mut connect = Connect::new();
            connect.identity.net_id = generate_id();
            connect.connect_id = con_id;
            connect.is_authenticated = true;

            self.uid_con_map.insert(username.clone(), connect);
        }
    }

    // 处理 NetworkPongMessage 消息
    #[allow(dead_code)]
    pub fn handel_network_pong_message(&self, con_id: u64, reader: &mut UnBatch, channel: Kcp2KChannel) {
        // 读取 NetworkPongMessage 数据
        let network_pong_message = NetworkPongMessage::deserialization(reader);

        let _ = self.handel_connect(con_id, |cur_connect| {
            let _ = cur_connect;
            let _ = network_pong_message;
            Ok(cur_connect.clone())
            // debug!("network_pong_message: {:?}", network_pong_message);
        });
    }

    // 处理 ReadyMessage 消息
    #[allow(dead_code)]
    pub fn handel_ready_message(&self, con_id: u64, un_batch: &mut UnBatch, channel: Kcp2KChannel) {
        // 设置连接为准备状态
        if let Err(e) = self.handel_connect(con_id, |connection| {
            connection.is_ready = true;
            Ok(connection.clone())
        }) {
            error!(format!("handel_ready_message: {:?}", e));
        }
    }

    // 处理 AddPlayerMessage 消息
    #[allow(dead_code)]
    pub fn handel_add_player_message(&self, con_id: u64, reader: &mut UnBatch, channel: Kcp2KChannel) {
        let _ = reader;

        let set = get_s_e_t();

        let mut cur_batch = Batch::new();
        cur_batch.write_f64_le(set);

        let mut other_batch = Batch::new();
        other_batch.write_f64_le(set);

        let scale = Vector3::new(1.0, 1.0, 1.0);

        if let Ok(cur_connect) = self.handel_connect(con_id, |cur_connect| {
            let scale = Vector3::new(1.0, 1.0, 1.0);

            // 添加 ObjectSpawnStartedMessage 数据
            ObjectSpawnStartedMessage {}.serialization(&mut cur_batch);
            // 生成自己
            let cur_payload = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
            let mut cur_spawn_message = SpawnMessage::new(cur_connect.identity.net_id, true, true, Default::default(), 3541431626, Default::default(), Default::default(), scale, Bytes::copy_from_slice(cur_payload.as_slice()));
            cur_spawn_message.serialization(&mut cur_batch);

            //  *****************************************************************************

            // 添加 ObjectSpawnStartedMessage 数据
            ObjectSpawnStartedMessage {}.serialization(&mut other_batch);
            // 添加通知其他客户端有新玩家加入消息
            cur_spawn_message.is_owner = false;
            cur_spawn_message.is_local_player = false;
            cur_spawn_message.serialization(&mut other_batch);

            Ok(cur_connect.clone())
        }) {
            //  通知当前玩家生成已经连接的玩家
            for connect in self.uid_con_map.iter() {
                if connect.connect_id == cur_connect.connect_id {
                    continue;
                }
                // 添加已经连接的玩家信息
                let other_payload = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
                let mut other_spawn_message = SpawnMessage::new(connect.identity.net_id, false, false, Default::default(), 3541431626, Default::default(), Default::default(), scale, Bytes::from(other_payload));
                other_spawn_message.serialization(&mut cur_batch);
                // 发送给其它玩家
                ObjectSpawnFinishedMessage {}.serialization(&mut other_batch);
                self.send(connect.connect_id, &other_batch, Kcp2KChannel::Reliable);
            }
            // 发送给当前玩家
            ObjectSpawnFinishedMessage {}.serialization(&mut cur_batch);
            self.send(cur_connect.connect_id, &cur_batch, Kcp2KChannel::Reliable);
        }
    }

    // 处理 CommandMessage 消息
    #[allow(dead_code)]
    pub fn handel_command_message(&self, con_id: u64, un_batch: &mut UnBatch, channel: Kcp2KChannel) {
        // 读取 CommandMessage 数据
        let command_message = match CommandMessage::deserialization(un_batch) {
            Ok(cm) => cm,
            Err(err) => {
                error!(format!("handel_command_message: {:?}", err));
                return;
            }
        };

        let net_id = command_message.net_id;
        let component_idx = command_message.component_index;
        let function_hash = command_message.function_hash;

        // 找到 MethodData
        if let Some(method_data) = self.backend_data.get_method(function_hash) {
            match method_data.method_type {
                // Command 类型
                MethodType::Command => {
                    // 创建 writer
                    let mut batch = Batch::new();
                    batch.write_f64_le(get_s_e_t());
                    // 如果有 rpc
                    if method_data.rpcs.len() > 0 {
                        // 遍历所有 rpc
                        for rpc in &method_data.rpcs {
                            debug!(format!("method_data: {} {} {} {}", method_data.name,method_data.name.get_fn_stable_hash_code(),component_idx,rpc.get_fn_stable_hash_code()));

                            let mut rpc_message = RpcMessage::new(net_id, component_idx, rpc.get_fn_stable_hash_code(), command_message.get_payload().slice(4..));
                            rpc_message.serialization(&mut batch);
                        }
                    }
                    // 遍历所有连接并发送消息
                    for connect in self.uid_con_map.iter() {
                        self.send(connect.connect_id, &batch, Kcp2KChannel::Reliable);
                    }
                    // 如果 parameters 不为空
                    // if method_data.parameters.len() > 0 {
                    //     debug!(format!("method_data: {} {} {} {:?}", method_data.name,method_data.name.get_fn_stable_hash_code(),component_idx,method_data.parameters));
                    //     let mut sync_vars_reader = UnBatch::new(command_message.get_payload_no_len());
                    //     // 遍历所有 parameters
                    //     for parameter in method_data.parameters.iter() {
                    //         println!("parameter: {} {}", parameter.key(), parameter.value());
                    //     }
                    // }
                }
                MethodType::TargetRpc => {}
                MethodType::ClientRpc => {}
            }
        }

        if function_hash == "System.Void QuickStart.PlayerScript::CmdSetupPlayer(System.String,UnityEngine.Color)".get_fn_stable_hash_code() {
            let mut writer = Batch::new();
            writer.write_f64_le(get_s_e_t());

            if let Ok(_) = self.handel_connect(con_id, |cur_connect| {
                debug!(format!("CmdSetupPlayer {} {}","System.Void QuickStart.PlayerScript::CmdSetupPlayer(System.String,UnityEngine.Color)".get_fn_stable_hash_code(), to_hex_string(command_message.get_payload().slice(4..).as_ref())));

                // 名字 颜色
                // let payload = hex::decode(format!("{}{}", "022C00000000000000000600000000000000", to_hex_string(&command_message.get_payload().slice(4..)))).unwrap();

                let mut un_batch = UnBatch::new(command_message.get_payload());
                let _ = un_batch.read_u32_le().unwrap();

                let name = un_batch.read_string_le().unwrap();

                let a = un_batch.read_f32_le().unwrap();
                let b = un_batch.read_f32_le().unwrap();
                let c = un_batch.read_f32_le().unwrap();
                let d = un_batch.read_f32_le().unwrap();

                let mut batch = Batch::new();

                batch.write_u8(0x02);
                batch.write_u8(0x2c);

                batch.write_u64_le(0);
                batch.write_u64_le(6);

                batch.write_string_le(name.as_str());
                batch.write_f32_le(a);
                batch.write_f32_le(b);
                batch.write_f32_le(c);
                batch.write_f32_le(d);

                let mut entity_state_message = EntityStateMessage::new(cur_connect.identity.net_id, batch.get_bytes());
                entity_state_message.serialization(&mut writer);

                // 场景右上角
                let mut batch = Batch::new();
                batch.write_u8(0x01);
                batch.write_u8(0x13);
                batch.write_string_le(format!("{} joined.", name).as_str());

                let mut cur_spawn_message = SpawnMessage::new(Default::default(), false, false, 14585647484178997735, Default::default(), Default::default(), Default::default(), Default::default(), batch.get_bytes());
                cur_spawn_message.serialization(&mut writer);

                Ok(cur_connect.clone())
            }) {
                for connect in self.uid_con_map.iter() {
                    self.send(connect.connect_id, &writer, Kcp2KChannel::Reliable);
                }
            }
        } else if function_hash == "System.Void QuickStart.PlayerScript::CmdChangeActiveWeapon(System.Int32)".get_fn_stable_hash_code() {
            let mut writer = Batch::new();
            writer.write_f64_le(get_s_e_t());

            if let Ok(_) = self.handel_connect(con_id, |cur_connection| {
                debug!(format!("CmdChangeActiveWeapon {}", to_hex_string(command_message.get_payload().slice(4..).as_ref())));


                let mut un_batch = UnBatch::new(command_message.get_payload());
                let _ = un_batch.read_u32_le().unwrap();

                let weapon_index = un_batch.read_i32_le().unwrap();

                let mut batch = Batch::new();
                batch.write_u8(0x02);
                batch.write_u8(0x14);
                batch.write_u64_le(0);
                batch.write_u64_le(1);
                batch.write_i32_le(weapon_index);

                let mut entity_state_message = EntityStateMessage::new(cur_connection.identity.net_id, batch.get_bytes());
                entity_state_message.serialization(&mut writer);

                Ok(cur_connection.clone())
            }) {
                for connection in self.uid_con_map.iter() {
                    self.send(connection.connect_id, &writer, Kcp2KChannel::Reliable);
                }
            }
        } else {
            // debug!(format!("Unknown function hash: {}\n", function_hash));
        }
    }
}