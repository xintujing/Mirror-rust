use crate::core::network_identity::NetworkIdentity;
use crate::core::network_reader::NetworkReader;
use crate::core::tools::stable_hash::StableHash;
use dashmap::mapref::one::RefMut;
use dashmap::DashMap;
use lazy_static::lazy_static;
use tklog::error;

#[derive(Debug, PartialEq, Eq)]
pub enum RemoteCallType {
    Command,
    ClientRpc,
}

pub type RemoteCallDelegateType = Box<dyn Fn(&mut NetworkIdentity, u8, &mut NetworkReader, u64) + Send + Sync>;

pub struct RemoteCallDelegate {
    pub method_name: &'static str,
    pub function: RemoteCallDelegateType,
}

impl RemoteCallDelegate {
    pub fn new(method_name: &'static str, function: RemoteCallDelegateType) -> Self {
        RemoteCallDelegate {
            method_name,
            function,
        }
    }
}

pub struct Invoker {
    pub call_type: RemoteCallType,
    pub remote_call_delegate: RemoteCallDelegate,
    pub cmd_requires_authority: bool,
}

impl Invoker {
    pub fn new(call_type: RemoteCallType, function: RemoteCallDelegate, cmd_requires_authority: bool) -> Self {
        Invoker {
            call_type,
            remote_call_delegate: function,
            cmd_requires_authority,
        }
    }
    pub fn are_equal(&self, remote_call_type: &RemoteCallType, invoke_function: &RemoteCallDelegate) -> bool {
        self.call_type == *remote_call_type && self.remote_call_delegate.method_name == invoke_function.method_name
    }
}

lazy_static! {
    static ref NETWORK_MESSAGE_HANDLERS: DashMap<u16, Invoker>=DashMap::new();
}

pub struct RemoteProcedureCalls;

impl RemoteProcedureCalls {
    pub fn check_if_delegate_exists(remote_call_type: &RemoteCallType, delegate: &RemoteCallDelegate, func_hash: u16) -> bool {
        if let Some(old_invoker) = NETWORK_MESSAGE_HANDLERS.get(&func_hash) {
            if old_invoker.are_equal(remote_call_type, delegate) {
                return true;
            }
            error!("Delegate already exists for hash: {}", func_hash);
        }
        false
    }
    pub fn register_delegate(function_full_name: &str, remote_call_type: RemoteCallType, delegate: RemoteCallDelegate, cmd_requires_authority: bool) -> u16 {
        let hash = function_full_name.get_fn_stable_hash_code();
        if Self::check_if_delegate_exists(&remote_call_type, &delegate, hash) {
            return hash;
        }
        let invoker = Invoker::new(remote_call_type, delegate, cmd_requires_authority);
        NETWORK_MESSAGE_HANDLERS.insert(hash, invoker);
        hash
    }

    pub fn register_command_delegate(function_full_name: &str, func: RemoteCallDelegate, cmd_requires_authority: bool) -> u16 {
        Self::register_delegate(function_full_name, RemoteCallType::Command, func, cmd_requires_authority)
    }

    pub fn register_rpc_delegate(function_full_name: &str, func: RemoteCallDelegate) -> u16 {
        Self::register_delegate(function_full_name, RemoteCallType::ClientRpc, func, true)
    }

    pub fn remove_delegate(func_hash: u16) {
        NETWORK_MESSAGE_HANDLERS.remove(&func_hash);
    }

    pub fn get_function_method_name(func_hash: u16) -> Option<String> {
        if let Some(invoker) = NETWORK_MESSAGE_HANDLERS.get(&func_hash) {
            return Some(invoker.remote_call_delegate.method_name.to_string());
        }
        None
    }

    fn get_invoker_for_hash(func_hash: u16, remote_call_type: RemoteCallType) -> (bool, Option<RefMut<'static, u16, Invoker>>) {
        if let Some(invoker) = NETWORK_MESSAGE_HANDLERS.get_mut(&func_hash) {
            if invoker.call_type == remote_call_type {
                return (true, Some(invoker));
            }
        }
        (false, None)
    }

    pub fn invoke(func_hash: u16, remote_call_type: RemoteCallType, identity: &mut NetworkIdentity, component_index: u8, reader: &mut NetworkReader, conn_id: u64) -> bool {
        let (has, invoker_option) = Self::get_invoker_for_hash(func_hash, remote_call_type);
        if has {
            if let Some(invoker) = invoker_option {
                (invoker.remote_call_delegate.function)(identity, component_index, reader, conn_id);
                return true;
            }
        }
        false
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
}