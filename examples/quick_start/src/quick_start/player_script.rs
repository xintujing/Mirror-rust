use dashmap::try_result::TryResult;
use mirror_rust::log_error;
use mirror_rust::mirror::components::network_common_behaviour::NetworkCommonBehaviour;
use mirror_rust::mirror::core::network_reader::NetworkReader;
use mirror_rust::mirror::core::network_server::{NetworkServerStatic, NETWORK_BEHAVIOURS};

pub trait PlayerScript {
    fn invoke_user_code_cmd_send_player_message_string(
        conn_id: u64,
        net_id: u32,
        component_index: u8,
        func_hash: u16,
        reader: &mut NetworkReader,
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
        conn_id: u64,
        net_id: u32,
        component_index: u8,
        func_hash: u16,
        reader: &mut NetworkReader,
    ) {
        if !NetworkServerStatic::active() {
            log_error!("Command CmdClientToServerSync called on client.");
            return;
        }

        // 获取 NetworkBehaviour
        match NETWORK_BEHAVIOURS.try_get_mut(&format!("{}_{}", net_id, component_index)) {
            TryResult::Present(mut component) => {
                component
                    .as_any_mut()
                    .downcast_mut::<Self>()
                    .unwrap()
                    .user_code_cmd_send_player_message_string(reader, func_hash, conn_id);
            }
            TryResult::Absent => {
                log_error!(
                    "NetworkBehaviour not found by net_id: {}, component_index: {}",
                    net_id,
                    component_index
                );
            }
            TryResult::Locked => {
                log_error!(
                    "NetworkBehaviour locked by net_id: {}, component_index: {}",
                    net_id,
                    component_index
                );
            }
        }
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
