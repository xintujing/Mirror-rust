use crate::log_error;
use crate::mirror::components::network_common_behaviour::NetworkCommonBehaviour;
use crate::mirror::core::network_reader::NetworkReader;
use crate::mirror::core::tools::stable_hash::StableHash;
use dashmap::mapref::one::RefMut;
use dashmap::DashMap;
use lazy_static::lazy_static;
use std::any::TypeId;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum RemoteCallType {
    Command,
    ClientRpc,
}

pub type RemoteCallDelegate = fn(u64, u32, u8, u16, &mut NetworkReader);

pub struct Invoker {
    pub type_id: TypeId,
    pub call_type: RemoteCallType,
    pub function: RemoteCallDelegate,
    pub cmd_requires_authority: bool,
}

impl Invoker {
    pub fn new(
        type_id: TypeId,
        call_type: RemoteCallType,
        function: RemoteCallDelegate,
        cmd_requires_authority: bool,
    ) -> Self {
        Invoker {
            type_id,
            call_type,
            function,
            cmd_requires_authority,
        }
    }
    pub fn are_equal(
        &self,
        type_id: TypeId,
        remote_call_type: RemoteCallType,
        invoke_function: &RemoteCallDelegate,
    ) -> bool {
        self.type_id == type_id
            && self.call_type == remote_call_type
            && self.function.eq(invoke_function)
    }
}

lazy_static! {
    static ref NETWORK_MESSAGE_HANDLERS: DashMap<u16, Invoker> = DashMap::new();
}

pub struct RemoteProcedureCalls;

impl RemoteProcedureCalls {
    pub fn check_if_delegate_exists(
        type_id: TypeId,
        remote_call_type: RemoteCallType,
        func: &RemoteCallDelegate,
        func_hash: u16,
    ) -> bool {
        if let Some(old_invoker) = NETWORK_MESSAGE_HANDLERS.get(&func_hash) {
            if old_invoker.are_equal(type_id, remote_call_type, func) {
                return true;
            }
            log_error!("Delegate already exists for hash: {}", func_hash);
        }
        false
    }
    pub fn register_delegate<T: 'static>(
        function_full_name: &str,
        remote_call_type: RemoteCallType,
        func: RemoteCallDelegate,
        cmd_requires_authority: bool,
    ) -> u16 {
        let hash = function_full_name.get_fn_stable_hash_code();
        let type_id = Self::generate_type_id::<T>();
        if Self::check_if_delegate_exists(type_id, remote_call_type, &func, hash) {
            return hash;
        }
        let invoker = Invoker::new(type_id, remote_call_type, func, cmd_requires_authority);
        NETWORK_MESSAGE_HANDLERS.insert(hash, invoker);
        hash
    }

    pub fn register_command_delegate<T: 'static>(
        function_full_name: &str,
        func: RemoteCallDelegate,
        cmd_requires_authority: bool,
    ) -> u16 {
        Self::register_delegate::<T>(
            function_full_name,
            RemoteCallType::Command,
            func,
            cmd_requires_authority,
        )
    }

    pub fn register_rpc_delegate<T: 'static>(
        function_full_name: &str,
        func: RemoteCallDelegate,
    ) -> u16 {
        Self::register_delegate::<T>(function_full_name, RemoteCallType::ClientRpc, func, true)
    }

    pub fn remove_delegate(func_hash: u16) {
        NETWORK_MESSAGE_HANDLERS.remove(&func_hash);
    }

    pub fn get_function_method_name(func_hash: u16) -> Option<String> {
        if let Some(invoker) = NETWORK_MESSAGE_HANDLERS.get(&func_hash) {
            return Some(format!("{:?}-{:?}", invoker.type_id, invoker.call_type));
        }
        None
    }

    fn get_invoker_for_hash(
        func_hash: u16,
        remote_call_type: RemoteCallType,
    ) -> (bool, Option<RefMut<'static, u16, Invoker>>) {
        if let Some(invoker) = NETWORK_MESSAGE_HANDLERS.get_mut(&func_hash) {
            if invoker.call_type == remote_call_type {
                return (true, Some(invoker));
            }
        }
        (false, None)
    }

    pub fn invoke(
        conn_id: u64,
        net_id: u32,
        component_index: u8,
        func_hash: u16,
        reader: &mut NetworkReader,
        remote_call_type: RemoteCallType,
    ) -> bool {
        // 找到对应的委托
        let (has, invoker_option) = Self::get_invoker_for_hash(func_hash, remote_call_type);
        if has {
            if let Some(invoker) = invoker_option {
                (invoker.function)(conn_id, net_id, component_index, func_hash, reader);
                return has;
            }
        }

        // 找不到对应的委托，尝试调用INVOKE_USER_CODE_CMD
        let (has, invoker_option) = Self::get_invoker_for_hash(
            NetworkCommonBehaviour::INVOKE_USER_CODE_CMD.get_fn_stable_hash_code(),
            remote_call_type,
        );
        if has {
            if let Some(invoker) = invoker_option {
                (invoker.function)(conn_id, net_id, component_index, func_hash, reader);
                return has;
            }
        }
        has
    }

    pub fn command_requires_authority(func_hash: u16) -> bool {
        if let Some(invoker) = NETWORK_MESSAGE_HANDLERS.get(&func_hash) {
            return invoker.cmd_requires_authority;
        }
        false
    }

    pub fn get_delegate(func_hash: u16) -> Option<RefMut<'static, u16, Invoker>> {
        NETWORK_MESSAGE_HANDLERS.get_mut(&func_hash)
    }

    pub fn generate_type_id<T: 'static>() -> TypeId {
        TypeId::of::<T>()
    }
}
