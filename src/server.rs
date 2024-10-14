use crate::backend_data::{import, BackendData, MethodType};
use crate::batcher::{Batch, DataReader, DataWriter, UnBatch};
use crate::connect::Connect;
use crate::messages::{AddPlayerMessage, CommandMessage, EntityStateMessage, NetworkPingMessage, NetworkPongMessage, ObjectSpawnFinishedMessage, ObjectSpawnStartedMessage, ReadyMessage, RpcMessage, SceneMessage, SceneOperation, SpawnMessage, TimeSnapshotMessage};
use crate::stable_hash::StableHash;
use crate::tools::{generate_id, get_s_e_t, to_hex_string};
use bytes::Bytes;
use dashmap::mapref::one::{Ref, RefMut};
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
            if let Err(_) = serv.s_send(connection_id, Bytes::copy_from_slice(writer.get_bytes()), channel) {
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

    pub fn switch_scene(&self, con_id: u64, scene_name: String, custom_handling: bool) {
        let mut writer = Batch::new();
        writer.write_f64_le(get_s_e_t());
        SceneMessage::new(scene_name, SceneOperation::Normal, custom_handling).serialization(&mut writer);
        self.send(con_id, &writer, Kcp2KChannel::Reliable);
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
        let _ = self.handel_connect(con_id, |mut connect| {
            connect.last_message_time = remote_time_stamp;
            Ok(())
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
            if msg_type_hash == 4296 {
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
        if let Some((_, cur_connection)) = self.cid_user_map.remove(&con_id) {
            // let mut writer = Batch::new();
            // writer.write_f64_le(get_s_e_t());
            // let mut object_destroy_message = ObjectDestroyMessage::new(cur_connection.identity);
            // object_destroy_message.serialization(&mut writer);
            // for connection in self.uid_con_map.iter() {
            //     self.send(connection.connect_id, &writer, Kcp2KChannel::Reliable);
            // }
        }
    }

    #[allow(dead_code)]
    pub fn handel_connect<F>(&self, con_id: u64, func: F) -> Result<(), String>
    where
        F: FnOnce(&mut Connect) -> Result<(), String>,
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

    #[allow(dead_code)]
    pub fn get_connects(&self) -> Vec<Connect> {
        let mut connects = Vec::new();
        for connection in self.uid_con_map.iter() {
            connects.push(connection.clone());
        }
        connects
    }

    // 处理 TimeSnapshotMessage 消息
    #[allow(dead_code)]
    pub fn handel_time_snapshot_message(&self, con_id: u64, reader: &mut UnBatch, channel: Kcp2KChannel) {
        let _ = reader;
        let _ = channel;
        // debug!("handel_time_snapshot_message");
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
            Ok(())
        });
    }

    #[allow(dead_code)]
    pub fn handel_network_auth_message(&self, con_id: u64, reader: &mut UnBatch, channel: Kcp2KChannel) {
        let username = reader.read_string_le().unwrap();
        let password = reader.read_string_le().unwrap();
        println!("username: {}, password: {}", username, password);

        let mut writer = Batch::new();
        writer.write_f64_le(get_s_e_t());
        writer.compress_var_u64_le(5);
        writer.write_u16_le(26160);
        writer.write_u8(100);
        writer.write_u16_le(0);
        self.send(con_id, &writer, Kcp2KChannel::Reliable);

        // 认证成功
        self.cid_user_map.insert(con_id, username.clone());

        // 切换场景
        self.switch_scene(con_id, "Assets/QuickStart/Scenes/MyScene.scene".to_string(), false);

        let mut connection = Connect::new();

        connection.connect_id = con_id;
        connection.identity = generate_id();
        connection.is_authenticated = true;

        self.uid_con_map.insert(username.clone(), connection);
    }

    // 处理 NetworkPongMessage 消息
    #[allow(dead_code)]
    pub fn handel_network_pong_message(&self, con_id: u64, reader: &mut UnBatch, channel: Kcp2KChannel) {
        // 读取 NetworkPongMessage 数据
        let network_pong_message = NetworkPongMessage::deserialization(reader);

        let _ = self.handel_connect(con_id, |cur_connect| {
            let _ = cur_connect;
            let _ = network_pong_message;
            Ok(())
            // debug!("network_pong_message: {:?}", network_pong_message);
        });
    }

    // 处理 ReadyMessage 消息
    #[allow(dead_code)]
    pub fn handel_ready_message(&self, con_id: u64, reader: &mut UnBatch, channel: Kcp2KChannel) {
        let _ = reader;
        // 设置连接为准备状态
        let _ = self.handel_connect(con_id, |connection| {
            connection.is_ready = true;
            Ok(())
        });
    }

    // 处理 AddPlayerMessage 消息
    #[allow(dead_code)]
    pub fn handel_add_player_message(&self, con_id: u64, reader: &mut UnBatch, channel: Kcp2KChannel) {
        let _ = reader;

        let connects = self.get_connects();

        let _ = self.handel_connect(con_id, |cur_connection| {
            let mut cur_writer = Batch::new();
            cur_writer.write_f64_le(get_s_e_t());

            // 添加 ObjectSpawnStartedMessage 数据
            ObjectSpawnStartedMessage {}.serialization(&mut cur_writer);

            let scale = Vector3::new(1.0, 1.0, 1.0);

            //  通知当前玩家生成自己和生成已经连接的玩家
            for connection in connects.iter() {
                // 生成自己
                if cur_connection.connect_id == connection.connect_id {
                    let cur_payload = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
                    let mut cur_spawn_message = SpawnMessage::new(cur_connection.identity, true, true, Default::default(), 3541431626, Default::default(), Default::default(), scale, Bytes::from(cur_payload));
                    cur_spawn_message.serialization(&mut cur_writer);
                    continue;
                }
                // 生成其它玩家
                let other_payload = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
                let mut other_spawn_message = SpawnMessage::new(connection.identity, false, false, Default::default(), 3541431626, Default::default(), Default::default(), scale, Bytes::from(other_payload));
                other_spawn_message.serialization(&mut cur_writer);
            }

            // 添加 ObjectSpawnStartedMessage 数据
            ObjectSpawnFinishedMessage {}.serialization(&mut cur_writer);

            // 发送给当前玩家
            self.send(cur_connection.connect_id, &cur_writer, Kcp2KChannel::Reliable);

            //  *****************************************************************************

            let mut other_writer = Batch::new();
            other_writer.write_f64_le(get_s_e_t());

            // 添加通知其他客户端有新玩家加入消息
            let cur_payload = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
            let mut cur_spawn_message = SpawnMessage::new(cur_connection.identity, false, false, 0, 3541431626, Default::default(), Default::default(), scale, Bytes::from(cur_payload));
            cur_spawn_message.serialization(&mut other_writer);


            // 通知其他玩家生成新加入的玩家
            for connection in connects.iter() {
                if cur_connection.connect_id == connection.connect_id {
                    continue;
                }
                let mut other_writer = Batch::new();
                other_writer.write_f64_le(get_s_e_t());
                let other_payload = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
                let mut other_spawn_message = SpawnMessage::new(cur_connection.identity, false, false, 0, 3541431626, Default::default(), Default::default(), scale, Bytes::from(other_payload));
                other_spawn_message.serialization(&mut other_writer);
                self.send(connection.connect_id, &other_writer, Kcp2KChannel::Reliable);
            }
            Ok(())
        });
    }

    // 处理 CommandMessage 消息
    #[allow(dead_code)]
    pub fn handel_command_message(&self, con_id: u64, batch: &mut UnBatch, channel: Kcp2KChannel) {
        // 读取 CommandMessage 数据
        let command_message = match CommandMessage::deserialization(batch) {
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
            // 创建 writer
            let mut writer = Batch::new();
            writer.write_f64_le(get_s_e_t());

            match method_data.method_type {
                // Command 类型
                MethodType::Command => {
                    // 如果有 rpc
                    if method_data.rpcs.len() > 0 {
                        // 遍历所有 rpc
                        for rpc in &method_data.rpcs {
                            debug!(format!("method_data: {} {} {} {}", method_data.name,method_data.name.get_fn_stable_hash_code(),component_idx,rpc.get_fn_stable_hash_code()));
                            let mut rpc_message = RpcMessage::new(net_id, component_idx, rpc.get_fn_stable_hash_code(), command_message.get_payload_no_len());
                            rpc_message.serialization(&mut writer);
                        }
                    }

                    // 如果 sync_vars 不为空
                    if method_data.sync_vars.len() > 0 {
                        debug!(format!("method_data: {} {} {} {}", method_data.name,method_data.name.get_fn_stable_hash_code(),component_idx,method_data.sync_vars.len()));

                        let mut sync_vars_reader = UnBatch::new(command_message.get_payload_no_len());
                        // println!("sync_vars_reader: {}", to_hex_string(sync_vars_reader.get_data()));
                        for i in 1..method_data.sync_vars.len() + 1 {
                            match i {
                                1 => {
                                    let name = sync_vars_reader.read_string_le().unwrap();
                                    println!("name: {}", name);
                                }
                                2 => {
                                    let a = sync_vars_reader.read_f32_le().unwrap();
                                    let b = sync_vars_reader.read_f32_le().unwrap();
                                    let c = sync_vars_reader.read_f32_le().unwrap();
                                    let d = sync_vars_reader.read_f32_le().unwrap();
                                    println!("a: {}, b: {}, c: {}, d: {}", a, b, c, d);
                                }
                                _ => {}
                            }
                        }
                    }
                    // 遍历所有连接并发送消息
                    for component in self.get_connects() {
                        self.send(component.connect_id, &writer, Kcp2KChannel::Reliable);
                    }
                }
                MethodType::TargetRpc => {}
                MethodType::ClientRpc => {}
            }
        }
        if function_hash == "System.Void QuickStart.PlayerScript::CmdSetupPlayer(System.String,UnityEngine.Color)".get_fn_stable_hash_code() {
            debug!(format!("CmdSetupPlayer {} {}","System.Void QuickStart.PlayerScript::CmdSetupPlayer(System.String,UnityEngine.Color)".get_fn_stable_hash_code(), to_hex_string(command_message.payload.as_ref())));

            let connects = self.get_connects();

            let _ = self.handel_connect(con_id, |cur_connection| {
                let mut writer = Batch::new();
                writer.write_f64_le(get_s_e_t());

                // 名字 颜色
                let payload = hex::decode(format!("{}{}", "022C00000000000000000600000000000000", to_hex_string(&command_message.get_payload_no_len()))).unwrap();
                let mut entity_state_message = EntityStateMessage::new(cur_connection.identity, Bytes::from(payload));
                entity_state_message.serialization(&mut writer);


                // 2 截取
                let tmp = &to_hex_string(&command_message.get_payload_no_len())[4..22];
                // 场景右上角
                let cur_payload = hex::decode(format!("01131100{}6A6F696E65642E", tmp)).unwrap();
                let mut cur_spawn_message = SpawnMessage::new(Default::default(), false, false, 14585647484178997735, Default::default(), Default::default(), Default::default(), Default::default(), Bytes::from(cur_payload));
                cur_spawn_message.serialization(&mut writer);

                for connection in connects {
                    self.send(connection.connect_id, &writer, Kcp2KChannel::Reliable);
                }
                Ok(())
            });
        } else if function_hash == "System.Void QuickStart.PlayerScript::CmdChangeActiveWeapon(System.Int32)".get_fn_stable_hash_code() {
            debug!(format!("CmdChangeActiveWeapon {}", to_hex_string(command_message.payload.as_ref())));
            let connects = self.get_connects();
            let _ = self.handel_connect(con_id, |cur_connection| {
                let mut writer = Batch::new();
                writer.write_f64_le(get_s_e_t());

                let payload = hex::decode(format!("{}{}", "021400000000000000000100000000000000", to_hex_string(&command_message.get_payload_no_len()))).unwrap();

                let mut entity_state_message = EntityStateMessage::new(cur_connection.identity, Bytes::from(payload));
                entity_state_message.serialization(&mut writer);

                for connection in connects {
                    self.send(connection.connect_id, &writer, Kcp2KChannel::Reliable);
                }
                Ok(())
            });
        } else {
            // debug!(format!("Unknown function hash: {}\n", function_hash));
        }
    }
}