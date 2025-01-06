use mirror_rust::log_error;
use mirror_rust::mirror::components::network_common_behaviour::NetworkCommonBehaviour;
use mirror_rust::mirror::core::network_behaviour::NetworkBehaviour;
use mirror_rust::mirror::core::network_identity::NetworkIdentity;
use mirror_rust::mirror::core::network_reader::NetworkReader;
use mirror_rust::mirror::core::network_server::NetworkServerStatic;

pub trait PlayerScript {
    fn invoke_user_code_cmd_send_player_message_string(
        identity: &mut NetworkIdentity,
        component_index: u8,
        _func_hash: u16,
        reader: &mut NetworkReader,
        _conn_id: u64,
    );
    fn user_code_cmd_send_player_message_string(
        &mut self,
        reader: &mut NetworkReader,
        func_hash: u16,
        conn_id: u64,
    );
}

impl PlayerScript for NetworkCommonBehaviour {
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
            .user_code_cmd_send_player_message_string(reader, _func_hash, _conn_id);
        NetworkBehaviour::late_invoke(identity, component_index);
    }

    fn user_code_cmd_send_player_message_string(
        &mut self,
        reader: &mut NetworkReader,
        func_hash: u16,
        conn_id: u64,
    ) {
        println!("{}, {} {}", reader.to_string(), func_hash, conn_id);
        for x in self.sync_vars.iter() {
            println!("key: {}, value: {:?}", x.key(), x.value());
        }
    }
}
