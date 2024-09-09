// Convenience module for managing remote calls for network behaviors

use std::any::TypeId;
use std::collections::HashMap;

pub enum RemoteCallType {
    Command,
    ClientRpc,
}

// Remote call function trait (similar to delegate in C#)
pub trait RemoteCall {
    fn invoke(&self, obj: &mut dyn NetworkBehaviour, reader: &NetworkReader, sender_connection: Option<&NetworkConnectionToClient>);
}

// Implementing specific functions as types for static dispatch
pub struct MyRemoteCall;

impl RemoteCall for MyRemoteCall {
    fn invoke(&self, obj: &mut dyn NetworkBehaviour, reader: &NetworkReader, sender_connection: Option<&NetworkConnectionToClient>) {
        // Implementation here
    }
}

pub struct Invoker {
    pub component_type_id: TypeId,
    pub call_type: RemoteCallType,
    pub function: Box<dyn RemoteCall>,
    pub cmd_requires_authority: bool,
}

impl Invoker {
    pub fn are_equal(&self, other_type_id: TypeId, other_call_type: RemoteCallType, other_function: &dyn RemoteCall) -> bool {
        self.component_type_id == other_type_id && self.call_type == other_call_type && TypeId::of::<dyn RemoteCall>() == TypeId::of_val(other_function)
    }
}

pub struct RemoteProcedureCalls {
    pub remote_call_delegates: HashMap<u16, Invoker>,
}

impl RemoteProcedureCalls {
    pub fn new() -> Self {
        Self {
            remote_call_delegates: HashMap::new(),
        }
    }

    pub fn register_delegate<T: 'static + RemoteCall>(&mut self, component_type_id: TypeId, function_name: &str, remote_call_type: RemoteCallType, function: T, cmd_requires_authority: bool) -> u16 {
        let hash = Self::hash_function_name(function_name);
        if let Some(invoker) = self.remote_call_delegates.get(&hash) {
            if invoker.are_equal(component_type_id, remote_call_type, &function) {
                return hash;
            }
            panic!("Function hash collision detected. Please rename the function to avoid collision.");
        }

        self.remote_call_delegates.insert(hash, Invoker {
            component_type_id,
            call_type: remote_call_type,
            function: Box::new(function),
            cmd_requires_authority,
        });
        hash
    }

    pub fn hash_function_name(name: &str) -> u16 {
        // Simple hashing, should be replaced with a better hash function
        name.bytes().fold(0, |acc, b| acc.wrapping_add(b as u16) % 65535)
    }

    // Additional methods for invoking, checking commands, etc., go here
}

// Example usage
fn main() {
    let mut rpcs = RemoteProcedureCalls::new();
    let component_type_id = TypeId::of::<dyn NetworkBehaviour>();
    rpcs.register_delegate(component_type_id, "MyFuncName", RemoteCallType::Command, MyRemoteCall, true);
}
