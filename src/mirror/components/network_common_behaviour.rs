use crate::mirror::core::backend_data::{
    BackendDataStatic, NetworkBehaviourComponent, SyncVarData,
};
use crate::mirror::core::network_behaviour::{
    GameObject, NetworkBehaviour, NetworkBehaviourTrait, SyncDirection, SyncMode,
};
use crate::mirror::core::network_identity::NetworkIdentity;
use crate::mirror::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::mirror::core::network_server::NetworkServerStatic;
use crate::mirror::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::mirror::core::network_writer_pool::NetworkWriterPool;
use crate::mirror::core::remote_calls::RemoteProcedureCalls;
use crate::mirror::core::sync_object::SyncObject;
use crate::mirror::core::tools::stable_hash::StableHash;
use crate::mirror::core::transport::TransportChannel;
use crate::{log_debug, log_error};
use dashmap::DashMap;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Once;

#[derive(Debug)]
pub struct NetworkCommonBehaviour {
    network_behaviour: NetworkBehaviour,
    sub_class: String,
    sync_vars: DashMap<u8, SyncVarData>,
}

impl NetworkCommonBehaviour {
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkCommonBehaviour";
    pub const INVOKE_USER_CODE_CMD: &'static str = "invoke_user_code_cmd";
    fn __update_sync_var(&mut self, index: u8, value: Vec<u8>) {
        match self.sync_vars.get_mut(&index) {
            // 未找到同步变量
            None => {
                return;
            }
            // 找到同步变量
            Some(mut sync_var) => {
                match NetworkBehaviour::sync_var_equal(&sync_var.value, &value) {
                    // 值相等
                    true => {
                        return;
                    }
                    // 值不相等
                    false => {
                        sync_var.value = value;
                    }
                }
            }
        }
        // 设置同步变量脏位
        self.set_sync_var_dirty_bits(1 << index);
    }
    fn __get_sync_var_index(&self, full_name: &str) -> Option<u8> {
        for sync_var in self.sync_vars.iter() {
            if sync_var.full_name == full_name {
                return Some(*sync_var.key());
            }
        }
        None
    }
    fn get_sync_var_value_vec(&self, r#type: &str, reader: &mut NetworkReader) -> Vec<u8> {
        // 初始化数据
        let mut value = Vec::new();
        match r#type {
            // 非定长类型
            "System.String" => {
                let string = reader.read_string();
                NetworkWriterPool::get_return(|writer| {
                    writer.write_string(string);
                    value = writer.to_bytes();
                });
            }
            // 常规类型

            // 压缩类型
            // TODO fix
            "System.Int32" | "System.UInt32" | "System.Long" | "System.ULong" => {
                value = reader.decompress_var()
            }

            // 4 字节
            "System.Float" => {
                value = reader.read_bytes(4);
            }
            // 8 字节
            "System.Double" => {
                value = reader.read_bytes(8);
            }
            // 12 字节
            "UnityEngine.Vector3" => {
                value = reader.read_bytes(12);
            }
            // 16 字节
            "UnityEngine.Color" => {
                value = reader.read_bytes(16);
            }
            // 未知类型
            _ => {}
        };
        value
    }
    // 更新同步变量
    fn update_sync_var(&mut self, full_name: &str, r#type: &str, reader: &mut NetworkReader) {
        if let Some(index) = self.__get_sync_var_index(full_name) {
            let value = self.get_sync_var_value_vec(r#type, reader);
            self.__update_sync_var(index, value);
        }
    }
    // 通用更新同步变量
    fn invoke_user_code_cmd_common_update_sync_var(
        identity: &mut NetworkIdentity,
        component_index: u8,
        func_hash: u16,
        reader: &mut NetworkReader,
        conn_id: u64,
    ) {
        if !NetworkServerStatic::active() {
            log_error!("Command CmdClientToServerSync called on client.");
            return;
        }
        NetworkBehaviour::early_invoke(identity, component_index)
            .as_any_mut()
            .downcast_mut::<Self>()
            .unwrap()
            .user_code_cmd_common_update_sync_var(reader, func_hash, conn_id);
        NetworkBehaviour::late_invoke(identity, component_index);
    }
    // 通用更新同步变量
    fn user_code_cmd_common_update_sync_var(
        &mut self,
        reader: &mut NetworkReader,
        func_hash: u16,
        _conn_id: u64,
    ) {
        // 获取方法数据
        if let Some(method_data) =
            BackendDataStatic::get_backend_data().get_method_data_by_hash_code(func_hash)
        {
            log_debug!(format!(
                "sub_class: {}, fn: {} - {}",
                self.sub_class,
                func_hash,
                method_data.name.split("::").collect::<Vec<&str>>()[1]
            ));
            // 更新同步变量
            for (index, parameter) in method_data.parameters.iter().enumerate() {
                let r#type = parameter.value.as_str();
                let full_name = method_data.var_list[index].value.as_str();
                self.update_sync_var(full_name, r#type, reader);
            }

            // 发送RPCs
            NetworkWriterPool::get_return(|writer| {
                writer.write_array_segment_all(reader.to_array_segment());
                for rpc in method_data.rpc_list.iter() {
                    self.send_rpc_internal(
                        rpc.as_str(),
                        rpc.get_stable_hash_code(),
                        writer,
                        TransportChannel::Reliable,
                        true,
                    );
                }
            });
        } else {
            log_error!("Method not found by hash code: {}", func_hash);
        }
    }
}

impl NetworkBehaviourTrait for NetworkCommonBehaviour {
    fn new(game_object: GameObject, network_behaviour_component: &NetworkBehaviourComponent) -> Self
    where
        Self: Sized,
    {
        let sync_vars = DashMap::new();
        for (i, sync_var) in BackendDataStatic::get_backend_data()
            .get_sync_var_data_s_by_sub_class(network_behaviour_component.sub_class.as_ref())
            .iter()
            .enumerate()
        {
            sync_vars.insert(i as u8, (*sync_var).clone());
        }
        Self::call_register_delegate();
        Self {
            network_behaviour: NetworkBehaviour::new(
                game_object,
                network_behaviour_component
                    .network_behaviour_setting
                    .clone(),
                network_behaviour_component.index,
            ),
            sub_class: network_behaviour_component.sub_class.clone(),
            sync_vars,
        }
    }

    fn register_delegate()
    where
        Self: Sized,
    {
        log_debug!("Registering delegate for ", Self::COMPONENT_TAG);
        RemoteProcedureCalls::register_command_delegate::<Self>(
            Self::INVOKE_USER_CODE_CMD,
            Box::new(Self::invoke_user_code_cmd_common_update_sync_var),
            true,
        );
    }

    fn get_once() -> &'static Once
    where
        Self: Sized,
    {
        static ONCE: Once = Once::new();
        &ONCE
    }

    fn sync_interval(&self) -> f64 {
        self.network_behaviour.sync_interval
    }

    fn set_sync_interval(&mut self, value: f64) {
        self.network_behaviour.sync_interval = value
    }

    fn last_sync_time(&self) -> f64 {
        self.network_behaviour.last_sync_time
    }

    fn set_last_sync_time(&mut self, value: f64) {
        self.network_behaviour.last_sync_time = value
    }

    fn sync_direction(&mut self) -> &SyncDirection {
        &self.network_behaviour.sync_direction
    }

    fn set_sync_direction(&mut self, value: SyncDirection) {
        self.network_behaviour.sync_direction = value
    }

    fn sync_mode(&mut self) -> &SyncMode {
        &self.network_behaviour.sync_mode
    }

    fn set_sync_mode(&mut self, value: SyncMode) {
        self.network_behaviour.sync_mode = value
    }

    fn index(&self) -> u8 {
        self.network_behaviour.index
    }

    fn set_index(&mut self, value: u8) {
        self.network_behaviour.index = value
    }

    fn sync_var_dirty_bits(&self) -> u64 {
        self.network_behaviour.sync_var_dirty_bits
    }

    fn __set_sync_var_dirty_bits(&mut self, value: u64) {
        self.network_behaviour.sync_var_dirty_bits = value
    }

    fn sync_object_dirty_bits(&self) -> u64 {
        self.network_behaviour.sync_object_dirty_bits
    }

    fn __set_sync_object_dirty_bits(&mut self, value: u64) {
        self.network_behaviour.sync_object_dirty_bits = value
    }

    fn net_id(&self) -> u32 {
        self.network_behaviour.net_id
    }

    fn set_net_id(&mut self, value: u32) {
        self.network_behaviour.net_id = value
    }

    fn connection_to_client(&self) -> u64 {
        self.network_behaviour.connection_to_client
    }

    fn set_connection_to_client(&mut self, value: u64) {
        self.network_behaviour.connection_to_client = value
    }

    fn observers(&self) -> &Vec<u64> {
        &self.network_behaviour.observers
    }

    fn set_observers(&mut self, value: Vec<u64>) {
        self.network_behaviour.observers = value
    }

    fn game_object(&self) -> &GameObject {
        &self.network_behaviour.game_object
    }

    fn set_game_object(&mut self, value: GameObject) {
        self.network_behaviour.game_object = value
    }

    fn sync_objects(&mut self) -> &mut Vec<Box<dyn SyncObject>> {
        &mut self.network_behaviour.sync_objects
    }

    fn set_sync_objects(&mut self, value: Vec<Box<dyn SyncObject>>) {
        self.network_behaviour.sync_objects = value
    }

    fn sync_var_hook_guard(&self) -> u64 {
        self.network_behaviour.sync_var_hook_guard
    }

    fn __set_sync_var_hook_guard(&mut self, value: u64) {
        self.network_behaviour.sync_var_hook_guard = value
    }

    fn is_dirty(&self) -> bool {
        self.network_behaviour.is_dirty()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn serialize_sync_vars(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        match initial_state {
            // 初始状态
            true => {
                for i in 0..self.sync_vars.len() as u8 {
                    if let Some(sync_var) = self.sync_vars.get(&i) {
                        writer.write_array_segment_all(sync_var.value.as_slice());
                    }
                }
            }
            // 非初始状态
            false => {
                writer.write_ulong(self.sync_var_dirty_bits());
                for i in 0..self.sync_vars.len() as u8 {
                    if self.sync_var_dirty_bits() & (1 << i) != 0 {
                        if let Some(sync_var) = self.sync_vars.get(&i) {
                            writer.write_array_segment_all(sync_var.value.as_slice());
                        }
                    }
                }
            }
        }
    }

    fn deserialize_sync_vars(&mut self, _reader: &mut NetworkReader, _initial_state: bool) -> bool {
        true
    }
}
