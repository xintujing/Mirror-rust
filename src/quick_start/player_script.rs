use crate::components::network_behaviour::{NetworkBehaviour, NetworkBehaviourTrait, SyncDirection, SyncMode};
use crate::core::backend_data::NetworkBehaviourComponent;
use crate::core::network_identity::NetworkIdentity;
use crate::core::network_manager::GameObject;
use crate::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::core::network_server::NetworkServerStatic;
use crate::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::core::network_writer_pool::NetworkWriterPool;
use crate::core::remote_calls::{RemoteCallDelegate, RemoteProcedureCalls};
use crate::core::sync_object::SyncObject;
use crate::core::transport::TransportChannel;
use nalgebra::Vector4;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Once;
use tklog::error;

#[derive(Debug)]
pub struct PlayerScript {
    network_behaviour: NetworkBehaviour,
    pub active_weapon_synced: i32,
    pub player_name: String,
    pub player_color: Vector4<f32>,
}

impl PlayerScript {
    fn invoke_user_code_cmd_setup_player_string_color(identity: &mut NetworkIdentity, component_index: u8, reader: &mut NetworkReader, conn_id: u64) {
        if !NetworkServerStatic::get_static_active() {
            error!("Command CmdClientToServerSync called on client.");
            return;
        }
        NetworkBehaviour::early_invoke(identity, component_index).
            as_any_mut().
            downcast_mut::<Self>().
            unwrap().
            user_code_cmd_setup_player_string_color(reader.read_string(), reader.read_vector4());
        NetworkBehaviour::late_invoke(identity, component_index);
    }

    fn user_code_cmd_setup_player_string_color(&mut self, player_name: String, player_color: Vector4<f32>) {
        self.player_name = player_name;
        self.set_sync_var_dirty_bits(2);
        self.player_color = player_color;
        self.set_sync_var_dirty_bits(4);
        println!("PlayerScript::CmdSetupPlayer: player_name: {}, player_color: {:?}", self.player_name, self.player_color);
    }

    fn invoke_user_code_cmd_shoot_ray(identity: &mut NetworkIdentity, component_index: u8, reader: &mut NetworkReader, conn_id: u64) {
        if !NetworkServerStatic::get_static_active() {
            error!("Command CmdClientToServerSync called on client.");
            return;
        }
        NetworkBehaviour::early_invoke(identity, component_index).
            as_any_mut().
            downcast_mut::<Self>().
            unwrap().
            user_code_cmd_shoot_ray();
        NetworkBehaviour::late_invoke(identity, component_index);
    }

    fn user_code_cmd_shoot_ray(&mut self) {
        NetworkWriterPool::get_return(|writer| {
            self.serialize(writer, false);
            self.send_rpc_internal("System.Void QuickStart.PlayerScript::RpcFireWeapon()", 546187665, writer, TransportChannel::Reliable, true);
        });
    }

    fn invoke_user_code_cmd_change_active_weapon_int32(identity: &mut NetworkIdentity, component_index: u8, reader: &mut NetworkReader, conn_id: u64) {
        if !NetworkServerStatic::get_static_active() {
            error!("Command CmdClientToServerSync called on client.");
            return;
        }
        NetworkBehaviour::early_invoke(identity, component_index).
            as_any_mut().
            downcast_mut::<Self>().
            unwrap().
            user_code_cmd_change_active_weapon_int32(reader.read_int());
        NetworkBehaviour::late_invoke(identity, component_index);
    }

    fn user_code_cmd_change_active_weapon_int32(&mut self, active_weapon: i32) {
        self.active_weapon_synced = active_weapon;
        self.set_sync_var_dirty_bits(1);
    }
}


impl NetworkBehaviourTrait for PlayerScript {
    fn new(game_object: GameObject, network_behaviour_component: &NetworkBehaviourComponent) -> Self
    where
        Self: Sized,
    {
        Self::call_register_delegate();
        Self {
            network_behaviour: NetworkBehaviour::new(game_object, network_behaviour_component.network_behaviour_setting.clone(), network_behaviour_component.index),
            active_weapon_synced: 0,
            player_name: "".to_string(),
            player_color: Vector4::new(255.0, 255.0, 255.0, 255.0),
        }
    }

    fn register_delegate()
    where
        Self: Sized,
    {
        // System.Void QuickStart.PlayerScript::CmdSetupPlayer(System.String,UnityEngine.Color)
        RemoteProcedureCalls::register_command_delegate(
            "System.Void QuickStart.PlayerScript::CmdSetupPlayer(System.String,UnityEngine.Color)",
            RemoteCallDelegate::new("invoke_user_code_cmd_setup_player_string_color", Box::new(Self::invoke_user_code_cmd_setup_player_string_color)),
            true,
        );
        // System.Void QuickStart.PlayerScript::CmdShootRay()
        RemoteProcedureCalls::register_command_delegate(
            "System.Void QuickStart.PlayerScript::CmdShootRay()",
            RemoteCallDelegate::new("invoke_user_code_cmd_shoot_ray", Box::new(Self::invoke_user_code_cmd_shoot_ray)),
            true,
        );

        // System.Void QuickStart.PlayerScript::CmdChangeActiveWeapon(System.Int32)
        RemoteProcedureCalls::register_command_delegate(
            "System.Void QuickStart.PlayerScript::CmdChangeActiveWeapon(System.Int32)",
            RemoteCallDelegate::new("invoke_user_code_cmd_change_active_weapon_int32", Box::new(Self::invoke_user_code_cmd_change_active_weapon_int32)),
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
        if initial_state {
            writer.write_int(self.active_weapon_synced);
            writer.write_string(self.player_name.to_string());
            writer.write_vector4(self.player_color);
        } else {
            writer.write_ulong(self.sync_var_dirty_bits());
            if self.sync_var_dirty_bits() & 1 != 0 {
                writer.write_int(self.active_weapon_synced);
            }
            if self.sync_var_dirty_bits() & 2 != 0 {
                writer.write_string(self.player_name.to_string());
            }
            if self.sync_var_dirty_bits() & 4 != 0 {
                writer.write_vector4(self.player_color);
            }
        }
    }

    fn deserialize_sync_vars(&mut self, reader: &mut NetworkReader, initial_state: bool) -> bool {
        true
    }
}