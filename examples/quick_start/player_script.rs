use nalgebra::Vector4;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Once;
use Mirror_rust::mirror::components::network_common_behaviour::NetworkCommonBehaviour;
use Mirror_rust::mirror::core::backend_data::NetworkBehaviourComponent;
use Mirror_rust::mirror::core::network_behaviour::{
    GameObject, NetworkBehaviour, NetworkBehaviourTrait, SyncDirection, SyncMode,
};
use Mirror_rust::mirror::core::network_identity::NetworkIdentity;
use Mirror_rust::mirror::core::network_reader::{NetworkReader, NetworkReaderTrait};
use Mirror_rust::mirror::core::network_server::NetworkServerStatic;
use Mirror_rust::mirror::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use Mirror_rust::mirror::core::network_writer_pool::NetworkWriterPool;
use Mirror_rust::mirror::core::remote_calls::RemoteProcedureCalls;
use Mirror_rust::mirror::core::sync_object::SyncObject;
use Mirror_rust::mirror::core::transport::TransportChannel;
use Mirror_rust::{log_debug, log_error};

pub trait PlayerScript {
    const COMPONENT_TAG: &'static str;
    fn invoke_user_code_cmd_send_player_message_string(
        identity: &mut NetworkIdentity,
        component_index: u8,
        _func_hash: u16,
        reader: &mut NetworkReader,
        _conn_id: u64,
    );
    fn user_code_cmd_send_player_message_string(
        network_common_behaviour: &mut NetworkCommonBehaviour,
        reader: &mut NetworkReader,
        func_hash: u16,
        conn_id: u64,
    );
}

impl PlayerScript for NetworkCommonBehaviour {
    const COMPONENT_TAG: &'static str = "QuickStart.PlayerScript";

    fn invoke_user_code_cmd_send_player_message_string(
        identity: &mut NetworkIdentity,
        component_index: u8,
        _func_hash: u16,
        reader: &mut NetworkReader,
        _conn_id: u64,
    ) {
        if !NetworkServerStatic::active() {
            log_error!("Command CmdClientToServerSync called on client.");
            return;
        }
        NetworkBehaviour::early_invoke(identity, component_index)
            .as_any_mut()
            .downcast_mut::<Self>()
            .unwrap()
            .user_code_cmd_common_update_func(
                reader,
                _func_hash,
                _conn_id,
                Self::user_code_cmd_send_player_message_string,
            );
        NetworkBehaviour::late_invoke(identity, component_index);
    }

    fn user_code_cmd_send_player_message_string(
        network_common_behaviour: &mut NetworkCommonBehaviour,
        reader: &mut NetworkReader,
        func_hash: u16,
        conn_id: u64,
    ) {
        println!("{}, {} {}", reader.to_string(), func_hash, conn_id);
        for x in network_common_behaviour.sync_vars.iter() {
            println!("key: {}, value: {:?}", x.key(), x.value());
        }
    }
}
