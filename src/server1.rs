// use crate::connection::Connection;
// use crate::messages::{AddPlayerMessage, CommandMessage, EntityStateMessage, NetworkPingMessage, NetworkPongMessage, ObjectDestroyMessage, ObjectSpawnFinishedMessage, ObjectSpawnStartedMessage, ReadyMessage, RpcMessage, SceneMessage, SceneOperation, SpawnMessage, TimeSnapshotMessage};
// use crate::rwder::{DataReader, DataWriter, Reader, Writer};
// use crate::stable_hash::StableHash;
// use crate::sync_data::SyncData;
// use crate::tools::{get_s_e_t, to_hex_string};
// use bytes::Bytes;
// use dashmap::DashMap;
// use kcp2k_rust::error_code::ErrorCode;
// use kcp2k_rust::kcp2k::Kcp2K;
// use kcp2k_rust::kcp2k_callback::{Callback, CallbackType};
// use kcp2k_rust::kcp2k_channel::Kcp2KChannel;
// use kcp2k_rust::kcp2k_config::Kcp2KConfig;
// use nalgebra::{Quaternion, Vector3};
// use std::process::exit;
// use std::sync::mpsc;
// use tklog::{debug, error};
//
// pub type UserIdType = [u8; 12];
//
// pub struct MirrorServer {
//     pub kcp_serv: Option<Kcp2K>,
//     pub kcp_serv_rx: Option<mpsc::Receiver<Callback>>,
//     pub uid_con_map: DashMap<UserIdType, Connection>,
//     pub cid_user_map: DashMap<u64, UserIdType>,
// }
//
// impl MirrorServer {
//     pub fn new(addr: String) -> Self {
//         // 创建 kcp 服务器配置
//         let config = Kcp2KConfig::default();
//         match Kcp2K::new_server(config, addr) {
//             Ok((server, s_rx)) => {
//                 Self {
//                     kcp_serv: Some(server),
//                     kcp_serv_rx: Some(s_rx),
//                     uid_con_map: DashMap::new(),
//                     cid_user_map: DashMap::new(),
//                 }
//             }
//             Err(err) => {
//                 error!(format!("MirrorServer: {:?}", err));
//                 exit(1)
//             }
//         }
//     }
//
//     pub fn start(&self) {
//         while let Some(kcp_serv) = self.kcp_serv.as_ref() {
//             kcp_serv.tick();
//             // 服务器接收
//             if let Some(rx) = self.kcp_serv_rx.as_ref() {
//                 if let Ok(cb) = rx.try_recv() {
//                     match cb.callback_type {
//                         CallbackType::OnConnected => {
//                             self.on_connected(cb.connection_id);
//                         }
//                         CallbackType::OnData => {
//                             self.on_data(cb.connection_id, cb.data, cb.channel);
//                         }
//                         CallbackType::OnDisconnected => {
//                             self.on_disconnected(cb.connection_id);
//                         }
//                         CallbackType::OnError => {
//                             self.on_error(cb.connection_id, cb.error_code, cb.error_message);
//                         }
//                     }
//                 }
//             }
//         }
//     }
//
//     pub fn send(&self, connection_id: u64, writer: &Writer, channel: Kcp2KChannel) {
//         if let Some(serv) = self.kcp_serv.as_ref() {
//             if let Err(_) = serv.s_send(connection_id, Bytes::copy_from_slice(writer.get_data()), channel) {
//                 // TODO: 发送失败
//             }
//         }
//     }
//
//     #[allow(dead_code)]
//     pub fn send_bytes(&self, connection_id: u64, data: Bytes, channel: Kcp2KChannel) {
//         if let Some(serv) = self.kcp_serv.as_ref() {
//             if let Err(_) = serv.s_send(connection_id, data, channel) {
//                 // TODO: 发送失败
//             }
//         }
//     }
//     #[allow(dead_code)]
//     pub fn disconnect(&self, connection_id: u64) {
//         debug!(format!("Disconnect {}", connection_id));
//     }
//
//     #[allow(dead_code)]
//     pub fn get_client_address(connection_id: u64) -> String {
//         let _ = connection_id;
//         "".to_string()
//     }
//
//     pub fn switch_scene(&self, con_id: u64, scene_name: String, custom_handling: bool) {
//         let mut writer = Writer::new_with_len(true);
//         SceneMessage::new(scene_name, SceneOperation::Normal, custom_handling).serialization(&mut writer);
//         self.send(con_id, &writer, Kcp2KChannel::Reliable);
//     }
//
//     #[allow(dead_code)]
//     pub fn on_connected(&self, con_id: u64) {
//         debug!(format!("OnConnected {}", con_id));
//
//         if con_id == 0 || self.uid_con_map.contains_key(&con_id) {
//             return;
//         }
//
//         // 切换场景
//         self.switch_scene(con_id, "Assets/QuickStart/Scenes/MyScene.scene".to_string(), false);
//
//         let connection = Connection::new(con_id, MirrorServer::get_client_address(con_id));
//         self.uid_con_map.insert(connection.connection_id, connection);
//
//         // TODO network_manager#L1177 auth class
//
//     }
//
//     #[allow(dead_code)]
//     pub fn on_data(&self, con_id: u64, message: Bytes, channel: Kcp2KChannel) {
//         let _ = channel;
//         let mut t_reader = Reader::new_with_len(message, true);
//         if let Some(mut connection) = self.uid_con_map.get_mut(&con_id) {
//             connection.remote_time_stamp = t_reader.get_elapsed_time()
//         }
//
//         let mut output_first = String::new();
//         output_first.push_str(&format!("Client Started: {}\n", t_reader.get_elapsed_time()));
//
//         while t_reader.get_remaining() > 0 {
//             let mut reader = t_reader.read_next();
//             let msg_type_hash = reader.read_u16();
//             output_first.push_str(&format!("msg_len: {} - msg_type_hash: {} - msg_type: ", reader.get_length(), msg_type_hash));
//
//
//             if msg_type_hash == TimeSnapshotMessage::FULL_NAME.get_stable_hash_code16() {
//                 output_first.push_str(&format!("{}\n", TimeSnapshotMessage::FULL_NAME));
//                 self.handel_time_snapshot_message(con_id, &mut reader);
//             } else if msg_type_hash == NetworkPingMessage::FULL_NAME.get_stable_hash_code16() {
//                 output_first.push_str(&format!("{}\n", NetworkPingMessage::FULL_NAME));
//                 // debug!("{}", output_first);
//                 self.handel_network_ping_message(con_id, &mut reader)
//             } else if msg_type_hash == NetworkPongMessage::FULL_NAME.get_stable_hash_code16() {
//                 output_first.push_str(&format!("{}\n", NetworkPongMessage::FULL_NAME));
//                 // debug!("{}", output_first);
//                 self.handel_network_pong_message(con_id, &mut reader);
//             } else if msg_type_hash == ReadyMessage::FULL_NAME.get_stable_hash_code16() {
//                 output_first.push_str(&format!("{}\n", ReadyMessage::FULL_NAME));
//                 // debug!(format!("{}", output_first));
//                 self.handel_ready_message(con_id, &mut reader);
//             } else if msg_type_hash == AddPlayerMessage::FULL_NAME.get_stable_hash_code16() {
//                 output_first.push_str(&format!("{}\n", AddPlayerMessage::FULL_NAME));
//                 debug!(format!("{}", output_first));
//                 self.handel_add_player_message(con_id, &mut reader);
//             } else if msg_type_hash == CommandMessage::FULL_NAME.get_stable_hash_code16() {
//                 output_first.push_str(&format!("{}", CommandMessage::FULL_NAME));
//                 debug!(format!("{}", output_first));
//
//                 let command_message = CommandMessage::deserialization(&mut reader);
//
//                 let net_id = command_message.net_id;
//                 let component_index = command_message.component_index;
//                 let function_hash = command_message.function_hash;
//                 debug!(format!("netId: {} componentIndex: {} functionHash: {}",  net_id, component_index, function_hash));
//
//                 if function_hash == "System.Void Mirror.NetworkTransformUnreliable::CmdClientToServerSync(Mirror.SyncData)".get_fn_stable_hash_code() {
//                     let mut sync_writer = Reader::new_with_len(command_message.payload.clone(), false);
//                     let sync_data = SyncData::deserialization(&mut sync_writer);
//                     debug!(format!("sync_data: {:?}\n", sync_data));
//
//                     let mut rpc_writer = Writer::new_with_len(true);
//                     let mut rpc_message = RpcMessage::new(net_id, component_index, 28456, command_message.get_payload_no_len());
//                     rpc_message.serialization(&mut rpc_writer);
//
//
//                     // 遍历所有连接并发送消息
//                     for connection in self.uid_con_map.iter() {
//                         if connection.connection_id == con_id {
//                             continue;
//                         }
//                         let mut writer = Writer::new_with_len(true);
//                         let mut entity_state_message = EntityStateMessage::new(connection.net_id, command_message.payload.clone());
//                         entity_state_message.serialization(&mut writer);
//                         self.send(connection.connection_id, &writer, Kcp2KChannel::Reliable);
//                     }
//                 } else if function_hash == 20088 {
//                     // debug!(format!("CmdClientRpc 20088 {}", to_hex_string(command_message.payload.as_slice())));
//
//                     if let Some(cur_connection) = self.uid_con_map.get(&con_id) {
//                         let mut writer = Writer::new_with_len(true);
//                         let payload = hex::decode(format!("{}{}", "022b00000000000000000600000000000000", to_hex_string(&command_message.payload[4..]))).unwrap();
//                         debug!(format!("CmdClientRpc 20088 payload: {}", to_hex_string(&payload)));
//                         let mut entity_state_message = EntityStateMessage::new(cur_connection.net_id, Bytes::from(payload));
//                         entity_state_message.serialization(&mut writer);
//                         for connection in self.uid_con_map.iter() {
//                             self.send(connection.connection_id, &writer, Kcp2KChannel::Reliable);
//                         }
//                     }
//                 } else if function_hash == "System.Void QuickStart.PlayerScript::CmdShootRay()".get_fn_stable_hash_code() {
//                     debug!(format!("CmdShootRay {}", to_hex_string(command_message.payload.as_ref())));
//
//                     if let Some(cur_connection) = self.uid_con_map.get(&con_id) {
//                         let mut writer = Writer::new_with_len(true);
//                         let mut rpc_message = RpcMessage::new(cur_connection.net_id, 1, 10641, command_message.get_payload_no_len());
//                         rpc_message.serialization(&mut writer);
//                         for connection in self.uid_con_map.iter() {
//                             self.send(connection.connection_id, &writer, Kcp2KChannel::Reliable);
//                         }
//                     }
//                 } else if function_hash == "System.Void QuickStart.PlayerScript::CmdChangeActiveWeapon(System.Int32)".get_fn_stable_hash_code() {
//                     debug!(format!("CmdChangeActiveWeapon {}", to_hex_string(command_message.payload.as_ref())));
//
//                     if let Some(cur_connection) = self.uid_con_map.get(&con_id) {
//                         let mut writer = Writer::new_with_len(true);
//                         let payload = hex::decode(format!("{}{}", "021400000000000000000100000000000000", to_hex_string(&command_message.payload[4..]))).unwrap();
//                         let mut entity_state_message = EntityStateMessage::new(cur_connection.net_id, Bytes::from(payload));
//                         entity_state_message.serialization(&mut writer);
//                         for connection in self.uid_con_map.iter() {
//                             self.send(connection.connection_id, &writer, Kcp2KChannel::Reliable);
//                         }
//                     }
//                 } else {
//                     debug!(format!("Unknown function hash: {}\n", function_hash));
//                 }
//             } else {
//                 debug!(format!("Unknown message type: {}\n", msg_type_hash));
//             }
//         }
//     }
//
//     #[allow(dead_code)]
//     pub fn on_error(&self, con_id: u64, code: ErrorCode, message: String) {
//         debug!(format!("OnError {} - {:?} {}", con_id, code, message));
//     }
//
//     #[allow(dead_code)]
//     pub fn on_disconnected(&self, con_id: u64) {
//         debug!(format!("OnDisconnected {}", con_id));
//         if let Some(connection) = self.uid_con_map.get(&con_id) {
//             let mut writer = Writer::new_with_len(true);
//             let mut object_destroy_message = ObjectDestroyMessage::new(connection.net_id);
//             object_destroy_message.serialization(&mut writer);
//             for connection in self.uid_con_map.iter() {
//                 self.send(connection.connection_id, &writer, Kcp2KChannel::Reliable);
//             }
//             self.uid_con_map.remove(&con_id);
//         }
//     }
//
//     // 处理 TimeSnapshotMessage 消息
//     #[allow(dead_code)]
//     pub fn handel_time_snapshot_message(&self, con_id: u64, reader: &mut Reader) {
//         let _ = reader;
//         if let Some(cur_connection) = self.uid_con_map.get(&con_id) {
//             let mut writer = Writer::new_with_len(true);
//             // 写入 TimeSnapshotMessage 数据
//             TimeSnapshotMessage {}.serialization(&mut writer);
//             // 发送 writer 数据
//             self.send(cur_connection.connection_id, &writer, Kcp2KChannel::Reliable);
//         }
//     }
//
//     // 处理 NetworkPingMessage 消息
//     #[allow(dead_code)]
//     pub fn handel_network_ping_message(&self, con_id: u64, reader: &mut Reader) {
//         if let Some(cur_connection) = self.uid_con_map.get(&con_id) {
//             // 读取 NetworkPingMessage 数据
//             let network_ping_message = NetworkPingMessage::deserialization(reader);
//             let local_time = network_ping_message.local_time;
//             let predicted_time_adjusted = network_ping_message.predicted_time_adjusted;
//
//             let mut writer = Writer::new_with_len(true);
//             // 准备 NetworkPongMessage 数据
//             let s_e_t = get_s_e_t();
//             let unadjusted_error = s_e_t - local_time;
//             let adjusted_error = s_e_t - predicted_time_adjusted;
//
//             // 写入 NetworkPongMessage 数据
//             NetworkPongMessage::new(local_time, unadjusted_error, adjusted_error).serialization(&mut writer);
//
//             // 发送 writer 数据
//             self.send(cur_connection.connection_id, &writer, Kcp2KChannel::Reliable);
//         }
//     }
//
//     // 处理 NetworkPongMessage 消息
//     #[allow(dead_code)]
//     pub fn handel_network_pong_message(&self, con_id: u64, reader: &mut Reader) {
//         let _ = reader;
//         if let Some(cur_connection) = self.uid_con_map.get(&con_id) {
//             // 读取 NetworkPongMessage 数据
//             // let network_pong_message = NetworkPongMessage::read(reader);
//             // debug!("network_pong_message: {:?}", network_pong_message);
//         }
//     }
//
//     // 处理 ReadyMessage 消息
//     #[allow(dead_code)]
//     pub fn handel_ready_message(&self, con_id: u64, reader: &mut Reader) {
//         let _ = reader;
//         // 设置连接为准备状态
//         if let Some(mut cur_connection) = self.uid_con_map.get_mut(&con_id) {
//             cur_connection.is_ready = true;
//         }
//     }
//
//     // 处理 AddPlayerMessage 消息
//     #[allow(dead_code)]
//     pub fn handel_add_player_message(&self, con_id: u64, reader: &mut Reader) {
//         if let Some(cur_connection) = self.uid_con_map.get(&con_id) {
//             let mut cur_writer = Writer::new_with_len(true);
//
//             // 添加 ObjectSpawnStartedMessage 数据
//             ObjectSpawnStartedMessage {}.serialization(&mut cur_writer);
//
//             let position = Vector3::new(0.0, 0.0, 0.0);
//             let rotation = Quaternion::identity();
//             let scale = Vector3::new(1.0, 1.0, 1.0);
//
//             let cur_payload = hex::decode("01131200506C61796572363632206A6F696E65642E").unwrap();
//             let mut cur_spawn_message = SpawnMessage::new(1, false, false, 14585647484178997735, 0, position, rotation, scale, Bytes::from(cur_payload));
//             cur_spawn_message.serialization(&mut cur_writer);
//
//
//             //  通知当前客户自己和已经连接的客户端
//             for connection in self.uid_con_map.iter() {
//                 // 自己
//                 if cur_connection.connection_id == connection.connection_id {
//                     let cur_payload = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
//                     let mut cur_spawn_message = SpawnMessage::new(cur_connection.net_id, true, true, 0, 3541431626, position, rotation, scale, Bytes::from(cur_payload));
//                     cur_spawn_message.serialization(&mut cur_writer);
//                     continue;
//                 }
//                 // 其它玩家
//                 let other_payload = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
//                 let mut other_spawn_message = SpawnMessage::new(connection.net_id, false, false, 0, 3541431626, position, rotation, scale, Bytes::from(other_payload));
//                 other_spawn_message.serialization(&mut cur_writer);
//             }
//
//             // 添加 ObjectSpawnStartedMessage 数据
//             ObjectSpawnFinishedMessage {}.serialization(&mut cur_writer);
//
//             // 发送给当前连接
//             self.send(cur_connection.connection_id, &cur_writer, Kcp2KChannel::Reliable);
//
//             //  *****************************************************************************
//             let mut other_writer = Writer::new_with_len(true);
//
//             let position = Vector3::new(0.0, 0.0, 0.0);
//             let rotation = Quaternion::identity();
//             let scale = Vector3::new(1.0, 1.0, 1.0);
//
//             // 添加通知其他客户端有新玩家加入消息
//             let cur_payload = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
//             let mut cur_spawn_message = SpawnMessage::new(cur_connection.net_id, false, false, 0, 3541431626, position, rotation, scale, Bytes::from(cur_payload));
//             cur_spawn_message.serialization(&mut other_writer);
//
//
//             // 通知其他客户端有新玩家加入消息
//             for connection in self.uid_con_map.iter() {
//                 if cur_connection.connection_id == connection.connection_id {
//                     continue;
//                 }
//                 let mut other_writer = Writer::new_with_len(true);
//                 let other_payload = hex::decode("031CCDCCE44000000000C3F580C00000000000000000000000000000803F160000000001000000803F0000803F0000803F0000803F").unwrap();
//                 let mut other_spawn_message = SpawnMessage::new(cur_connection.net_id, false, false, 0, 3541431626, position, rotation, scale, Bytes::from(other_payload));
//                 other_spawn_message.serialization(&mut other_writer);
//                 self.send(connection.connection_id, &other_writer, Kcp2KChannel::Reliable);
//             }
//         }
//     }
// }