use crate::components::network_common_component::NetworkCommonComponent;
use crate::core::backend_data::BackendData;
use crate::core::batcher::{Batch, DataReader, DataWriter, UnBatch};
use crate::core::messages::{AddPlayerMessage, CommandMessage, EntityStateMessage, NetworkPingMessage, NetworkPongMessage, ObjectDestroyMessage, ObjectSpawnFinishedMessage, ObjectSpawnStartedMessage, ReadyMessage, RpcMessage, SceneMessage, SceneOperation, SpawnMessage, TimeSnapshotMessage};
use crate::core::network_connection::NetworkConnection;
use crate::core::network_time::NetworkTime;
use crate::core::tools::stable_hash::StableHash;
use crate::tools::utils::{generate_id, to_hex_string};
use bytes::Bytes;
use dashmap::DashMap;
use kcp2k_rust::error_code::ErrorCode;
use kcp2k_rust::kcp2k::Kcp2K;
use kcp2k_rust::kcp2k_callback::{Callback, CallbackType};
use kcp2k_rust::kcp2k_channel::Kcp2KChannel;
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use nalgebra::{Quaternion, Vector3};
use std::process::exit;
use std::sync::{mpsc, Arc};
use tklog::{debug, error};

type MapBridge = String;
#[derive(Debug)]
pub enum HandleConnectResult {
    Ok,                  // 处理成功但不需要返回值
    CID(u64),            // 处理成功并返回连接 ID
    CnId(u64, u32),      // 处理成功并返回连接 ID 和网络 ID
    Err(&'static str),   // 处理失败
}

pub struct MirrorServer {
    pub kcp_serv: Option<Kcp2K>,
    pub kcp_serv_rx: Option<mpsc::Receiver<Callback>>,
    pub uid_con_map: DashMap<MapBridge, NetworkConnection>,
    pub cid_user_map: DashMap<u64, MapBridge>,
    pub backend_data: Arc<BackendData>,
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
                    backend_data: Arc::new(BackendData::import("backend_data.json")),
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
            // sleep(std::time::Duration::from_millis(1));
        }
    }

    pub fn send(&self, connection_id: u64, batch: &Batch, channel: Kcp2KChannel) {
        if let Some(serv) = self.kcp_serv.as_ref() {
            if let Err(_) = serv.s_send(connection_id, batch.get_bytes(), channel) {
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
                error!(format!("on_data err: {:?}", err));
                return;
            }
        };

        // 更新连接时间
        self.handel_connect(con_id, |connect| {
            connect.last_message_time = remote_time_stamp;
            HandleConnectResult::Ok
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
        // 更改连接状态
        if let HandleConnectResult::CnId(c_id, net_id) = self.handel_connect(con_id, |connect| {
            connect.is_ready = false;
            connect.is_authenticated = false;
            HandleConnectResult::CnId(connect.connection_id, connect.identity.net_id)
        }) {
            // 删除连接
            if let Some((_, v)) = self.cid_user_map.remove(&c_id) {
                self.uid_con_map.remove(&v);
            }
            // 通知其它客户端
            let mut batch = Batch::new();
            batch.write_f64_le(NetworkTime::local_time());
            ObjectDestroyMessage::new(net_id).serialize(&mut batch);
            for connect in self.uid_con_map.iter() {
                self.send(connect.connection_id, &batch, Kcp2KChannel::Reliable);
            }
        }
    }

    #[allow(dead_code)]
    pub fn switch_scene(&self, con_id: u64, scene_name: String, custom_handling: bool) {
        let mut batch = Batch::new();
        batch.write_f64_le(NetworkTime::local_time());
        SceneMessage::new(scene_name, SceneOperation::Normal, custom_handling).serialize(&mut batch);
        self.send(con_id, &batch, Kcp2KChannel::Reliable);
    }


    #[allow(dead_code)]
    pub fn handel_connect<F>(&self, con_id: u64, func: F) -> HandleConnectResult
    where
        F: FnOnce(&mut NetworkConnection) -> HandleConnectResult,
    {
        let user_name = match self.cid_user_map.get(&con_id) {
            None => {
                return HandleConnectResult::Err("can't find user in cid_user_map")
            }
            Some(user_name) => user_name.to_string()
        };

        match self.uid_con_map.get_mut(user_name.as_str()) {
            None => {
                HandleConnectResult::Err("can't find connect in uid_con_map")
            }
            Some(mut connect) => func(connect.value_mut())
        }
    }

    // 处理 TimeSnapshotMessage 消息
    #[allow(dead_code)]
    pub fn handel_time_snapshot_message(&self, con_id: u64, un_batch: &mut UnBatch, channel: Kcp2KChannel) {
        let mut batch = Batch::new();
        batch.write_f64_le(NetworkTime::local_time());
        TimeSnapshotMessage {}.serialize(&mut batch);
        // println!("handel_time_snapshot_message: {}", get_s_e_t());
        self.send(con_id, &batch, channel);
    }

    // 处理 NetworkPingMessage 消息
    #[allow(dead_code)]
    pub fn handel_network_ping_message(&self, con_id: u64, reader: &mut UnBatch, channel: Kcp2KChannel) {
        // 读取 NetworkPingMessage 数据
        let network_ping_message = match NetworkPingMessage::deserialize(reader) {
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
            writer.write_f64_le(NetworkTime::local_time());
            // 准备 NetworkPongMessage 数据
            let s_e_t = NetworkTime::local_time();
            let unadjusted_error = s_e_t - local_time;
            let adjusted_error = s_e_t - predicted_time_adjusted;

            // 写入 NetworkPongMessage 数据
            NetworkPongMessage::new(local_time, unadjusted_error, adjusted_error).serialize(&mut writer);

            // 发送 writer 数据
            self.send(cur_connect.connection_id, &writer, Kcp2KChannel::Reliable);
            HandleConnectResult::Ok
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
        batch.write_f64_le(NetworkTime::local_time());
        batch.compress_var_u64_le(5);
        batch.write_u16_le(56082);
        batch.write_u8(100);
        batch.write_u16_le(0);

        // 认证成功
        self.cid_user_map.insert(con_id, username.clone());

        match self.handel_connect(con_id, |cur_connect| {
            cur_connect.connection_id = con_id;
            cur_connect.is_authenticated = true;
            self.switch_scene(con_id, "Assets/QuickStart/Scenes/MyScene.scene".to_string(), false);

            HandleConnectResult::Ok
        }) {
            HandleConnectResult::Err(_) => {
                // 切换场景
                self.switch_scene(con_id, "Assets/QuickStart/Scenes/MyScene.scene".to_string(), false);

                let mut connect = NetworkConnection::new(Arc::clone(&self.backend_data), 0, 3541431626);
                connect.connection_id = con_id;
                connect.is_authenticated = true;

                self.uid_con_map.insert(username.clone(), connect);
            }
            _ => {}
        }
        // 发送 NetworkAuthMessage 数据
        self.send(con_id, &batch, channel);
    }

    // 处理 NetworkPongMessage 消息
    #[allow(dead_code)]
    pub fn handel_network_pong_message(&self, con_id: u64, reader: &mut UnBatch, channel: Kcp2KChannel) {
        // 读取 NetworkPongMessage 数据
        let network_pong_message = NetworkPongMessage::deserialize(reader);

        let _ = self.handel_connect(con_id, |cur_connect| {
            let _ = cur_connect;
            let _ = network_pong_message;
            HandleConnectResult::Ok
            // debug!("network_pong_message: {:?}", network_pong_message);
        });
    }

    // 处理 ReadyMessage 消息
    #[allow(dead_code)]
    pub fn handel_ready_message(&self, con_id: u64, un_batch: &mut UnBatch, channel: Kcp2KChannel) {
        // 设置连接为准备状态
        if let HandleConnectResult::Ok = self.handel_connect(con_id, |connection| {
            connection.is_ready = true;
            HandleConnectResult::Ok
        }) {}
    }

    // 处理 AddPlayerMessage 消息
    #[allow(dead_code)]
    pub fn handel_add_player_message(&self, con_id: u64, reader: &mut UnBatch, channel: Kcp2KChannel) {
        let _ = reader;

        // 生成 net_id
        let set = NetworkTime::local_time();

        // 创建 cur_batch
        let mut cur_batch = Batch::new();
        cur_batch.write_f64_le(set);

        // 创建 other_batch
        let mut other_batch = Batch::new();
        other_batch.write_f64_le(set);

        // rotation
        let rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);

        // scale
        let scale = Vector3::new(1.0, 1.0, 1.0);

        if let HandleConnectResult::CID(c_id) = self.handel_connect(con_id, |cur_connect| {

            // If the object has not been spawned, then do a full spawn and update observers
            cur_connect.identity.net_id = generate_id();

            // 添加 ObjectSpawnStartedMessage 数据
            ObjectSpawnStartedMessage {}.serialize(&mut cur_batch);
            // payload
            let cur_payload = cur_connect.identity.new_spawn_message_payload();
            // 生成自己
            let mut cur_spawn_message = SpawnMessage::new(cur_connect.identity.net_id, true, true, cur_connect.identity.scene_id, cur_connect.identity.asset_id, Default::default(), rotation, scale, cur_payload);
            cur_spawn_message.serialize(&mut cur_batch);

            //  *****************************************************************************

            // 添加 ObjectSpawnStartedMessage 数据
            ObjectSpawnStartedMessage {}.serialize(&mut other_batch);
            // 添加通知其他客户端有新玩家加入消息
            cur_spawn_message.is_owner = false;
            cur_spawn_message.is_local_player = false;
            cur_spawn_message.serialize(&mut other_batch);

            HandleConnectResult::CID(cur_connect.connection_id)
        }) {
            //  通知当前玩家生成已经连接的玩家
            for mut connect in self.uid_con_map.iter() {
                if connect.connection_id == c_id {
                    continue;
                }
                // 添加已经连接的玩家信息
                let other_payload = connect.value().identity.new_spawn_message_payload();
                println!("other_payload1: {}", "031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F");
                println!("other_payload2: {}", to_hex_string(other_payload.as_ref()));
                let mut other_spawn_message = SpawnMessage::new(connect.identity.net_id, false, false, Default::default(), 3541431626, Default::default(), rotation, scale, Bytes::from(other_payload));
                other_spawn_message.serialize(&mut cur_batch);
                // 发送给其它玩家
                ObjectSpawnFinishedMessage {}.serialize(&mut other_batch);
                self.send(connect.connection_id, &other_batch, channel);
            }
            // 发送给当前玩家
            ObjectSpawnFinishedMessage {}.serialize(&mut cur_batch);
            self.send(c_id, &cur_batch, channel);
        }
    }

    // 处理 CommandMessage 消息
    #[allow(dead_code)]
    pub fn handel_command_message(&self, con_id: u64, un_batch: &mut UnBatch, channel: Kcp2KChannel) {
        // 读取 CommandMessage 数据
        let command_message = match CommandMessage::deserialize(un_batch) {
            Ok(cm) => cm,
            Err(err) => {
                error!(format!("handel_command_message: {:?}", err));
                return;
            }
        };

        let net_id = command_message.net_id;
        let component_idx = command_message.component_index;
        let function_hash = command_message.function_hash;
        let payload = command_message.get_payload();

        // self.handel_connect(con_id, |cur_connect| {
        //     let component = cur_connect.identity.components[component_idx as usize].as_ref();
        //     component.deserialize(&mut UnBatch::new(payload), false);
        //     HandleConnectResult::Ok
        // });


        // 获取 rpc_hash_code_s
        let rpc_hash_code_s = self.backend_data.get_rpc_hash_code_s(function_hash);

        // 创建 batch
        let mut batch = Batch::new_with_s_e_t();

        if function_hash == "System.Void QuickStart.PlayerScript::CmdSetupPlayer(System.String,UnityEngine.Color)".get_fn_stable_hash_code() {
            if let HandleConnectResult::Ok = self.handel_connect(con_id, |cur_connect| {
                debug!(format!("CmdSetupPlayer {} {}","System.Void QuickStart.PlayerScript::CmdSetupPlayer(System.String,UnityEngine.Color)".get_fn_stable_hash_code(), to_hex_string(command_message.get_payload().slice(4..).as_ref())));

                // 名字 颜色
                // let payload = hex::decode(format!("{}{}", "022C00000000000000000600000000000000", to_hex_string(&command_message.get_payload().slice(4..)))).unwrap();

                let mut un_batch = UnBatch::new(payload);
                let _ = un_batch.read_u32_le().unwrap();

                let name = un_batch.read_string_le().unwrap();

                let component = cur_connect.identity.components[component_idx as usize].as_any().downcast_ref::<NetworkCommonComponent>().unwrap();

                let mut batch01 = Batch::new();
                batch01.write_string_le(name.as_str());
                if let Some(mut var) = component.sync_vars.get_mut(&1) {
                    var.data = batch01.get_bytes();
                }

                let a = un_batch.read_f32_le().unwrap();
                let b = un_batch.read_f32_le().unwrap();
                let c = un_batch.read_f32_le().unwrap();
                let d = un_batch.read_f32_le().unwrap();

                let mut batch02 = Batch::new();
                batch02.write_f32_le(a);
                batch02.write_f32_le(b);
                batch02.write_f32_le(c);
                batch02.write_f32_le(d);
                if let Some(mut var) = component.sync_vars.get_mut(&2) {
                    var.data = batch02.get_bytes();
                }

                let mut batch03 = Batch::new();

                batch03.write_u8(0x02);
                batch03.write_u8(0x2c);

                batch03.write_u64_le(0);
                batch03.write_u64_le(6);

                batch03.write_string_le(name.as_str());
                batch03.write_f32_le(a);
                batch03.write_f32_le(b);
                batch03.write_f32_le(c);
                batch03.write_f32_le(d);

                let mut entity_state_message = EntityStateMessage::new(cur_connect.identity.net_id, batch03.get_bytes());
                entity_state_message.serialize(&mut batch);

                // 场景右上角
                let mut batch04 = Batch::new();
                batch04.write_u8(0x01);
                batch04.write_u8(0x13);
                batch04.write_string_le(format!("{} joined.", name).as_str());

                let mut cur_spawn_message = SpawnMessage::new(Default::default(), false, false, 14585647484178997735, Default::default(), Default::default(), Default::default(), Default::default(), batch04.get_bytes());
                cur_spawn_message.serialize(&mut batch);

                HandleConnectResult::Ok
            }) {
                for connect in self.uid_con_map.iter() {
                    self.send(connect.connection_id, &batch, Kcp2KChannel::Reliable);
                }
            }
        } else if function_hash == "System.Void QuickStart.PlayerScript::CmdChangeActiveWeapon(System.Int32)".get_fn_stable_hash_code() {
            if let HandleConnectResult::Ok = self.handel_connect(con_id, |cur_connection| {
                debug!(format!("CmdChangeActiveWeapon {}", to_hex_string(payload.slice(4..).as_ref())));

                // let tmp = cur_connection.identity.components[component_idx as usize].as_any().downcast_ref::<NetworkCommonComponent>().unwrap();
                // println!("{:?}", tmp);

                let mut un_batch = UnBatch::new(payload);

                // len
                let _ = un_batch.read_u32_le().unwrap();

                let weapon_index = un_batch.read_i32_le().unwrap();

                let mut batch01 = Batch::new();
                // mask
                batch01.write_u8(0x02);
                // safe
                batch01.write_u8(0x14);
                // sync obj dirty
                batch01.write_u64_le(0);
                // sync var dirty
                batch01.write_u64_le(1);
                // weapon index
                batch01.write_i32_le(weapon_index);

                let mut entity_state_message = EntityStateMessage::new(cur_connection.identity.net_id, batch01.get_bytes());
                entity_state_message.serialize(&mut batch);

                HandleConnectResult::Ok
            }) {
                for connection in self.uid_con_map.iter() {
                    self.send(connection.connection_id, &batch, channel);
                }
            }
        } else if rpc_hash_code_s.len() > 0 {
            println!("rpc_list: {:?}", self.backend_data.get_method_data_by_hash_code(function_hash).unwrap().rpc_list);
            for rpc_hash_code in rpc_hash_code_s {
                let mut rpc_message = RpcMessage::new(net_id, component_idx, rpc_hash_code, payload.slice(4..));
                rpc_message.serialize(&mut batch);
            }
            // 遍历所有连接并发送消息
            for connect in self.uid_con_map.iter() {
                self.send(connect.connection_id, &batch, channel);
            }
        } else {
            debug!(format!("Unknown function hash: {}\n", function_hash));
        }
    }
}