use crate::mirror::core::backend_data::NetworkBehaviourComponent;
use crate::mirror::core::network_behaviour::{
    GameObject, NetworkBehaviour, NetworkBehaviourTrait, SyncDirection, SyncMode,
};
use crate::mirror::core::network_identity::NetworkIdentity;
use crate::mirror::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::mirror::core::network_reader_pool::NetworkReaderPool;
use crate::mirror::core::network_server::NetworkServerStatic;
use crate::mirror::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::mirror::core::network_writer_pool::NetworkWriterPool;
use crate::mirror::core::remote_calls::RemoteProcedureCalls;
use crate::mirror::core::sync_object::SyncObject;
use crate::mirror::core::transport::TransportChannel;
use crate::{log_debug, log_error};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::any::Any;
use std::sync::Once;

#[derive(Debug)]
pub struct NetworkAnimator {
    network_behaviour: NetworkBehaviour,
    client_authority: bool,
    animator: Animator,
    animator_speed: f32,
    previous_speed: f32,
    last_int_parameters: Vec<i32>,
    last_float_parameters: Vec<f32>,
    last_bool_parameters: Vec<bool>,
}

impl NetworkAnimator {
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkAnimator";

    fn write_parameters(&mut self, writer: &mut NetworkWriter, force_all: bool) -> bool {
        let parameter_count = self.animator.parameters.len() as u8;
        writer.write_byte(parameter_count);

        let dirty_bits: u64;
        if force_all {
            dirty_bits = u64::MAX;
        } else {
            dirty_bits = self.next_dirty_bits();
        }
        writer.write_ulong(dirty_bits);

        for i in 0..parameter_count as usize {
            if dirty_bits & (1 << i) == 0 {
                continue;
            }

            let par = &self.animator.parameters[i];

            if par.r#type == AnimatorParameterType::Int {
                NetworkReaderPool::get_with_bytes_return(par.value.to_vec(), |reader| {
                    let int_value = reader.read_int();
                    writer.write_int(int_value);
                });
            } else if par.r#type == AnimatorParameterType::Float {
                NetworkReaderPool::get_with_bytes_return(par.value.to_vec(), |reader| {
                    let float_value = reader.read_float();
                    writer.write_float(float_value);
                });
            } else if par.r#type == AnimatorParameterType::Bool {
                NetworkReaderPool::get_with_bytes_return(par.value.to_vec(), |reader| {
                    let bool_value = reader.read_bool();
                    writer.write_bool(bool_value);
                });
            }
        }

        dirty_bits != 0
    }

    fn next_dirty_bits(&mut self) -> u64 {
        let mut dirty_bits = 0;
        for i in 0..self.animator.parameters.len() {
            let par = &self.animator.parameters[i];
            let mut changed = false;
            if par.r#type == AnimatorParameterType::Int {
                NetworkReaderPool::get_with_bytes_return(par.value.to_vec(), |reader| {
                    let new_int_value = reader.read_int();
                    changed |= self.last_int_parameters[i] != new_int_value;
                    if changed {
                        self.last_int_parameters[i] = new_int_value;
                    }
                });
            } else if par.r#type == AnimatorParameterType::Float {
                NetworkReaderPool::get_with_bytes_return(par.value.to_vec(), |reader| {
                    let new_float_value = reader.read_float();
                    changed |= (new_float_value - self.last_float_parameters[i]).abs() > 0.001;
                    if changed {
                        self.last_float_parameters[i] = new_float_value;
                    }
                });
            } else if par.r#type == AnimatorParameterType::Bool {
                NetworkReaderPool::get_with_bytes_return(par.value.to_vec(), |reader| {
                    let new_bool_value = reader.read_bool();
                    changed |= self.last_bool_parameters[i] != new_bool_value;
                    if changed {
                        self.last_bool_parameters[i] = new_bool_value;
                    }
                });
            }
            if changed {
                dirty_bits |= 1 << i;
            }
        }
        dirty_bits
    }

    // 1 CmdOnAnimationServerMessage(int stateHash, float normalizedTime, int layerId, float weight, byte[] parameters)
    fn invoke_user_code_cmd_on_animation_server_message_int32_single_int32_single_byte(
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
            .user_code_cmd_on_animation_server_message_int32_single_int32_single_byte(
                reader.decompress_var_int(),
                reader.read_float(),
                reader.decompress_var_int(),
                reader.read_float(),
                reader.read_bytes_and_size(),
            );
        NetworkBehaviour::late_invoke(identity, component_index);
    }

    // 1 UserCode_CmdOnAnimationServerMessage__Int32__Single__Int32__Single__Byte\u005B\u005D
    fn user_code_cmd_on_animation_server_message_int32_single_int32_single_byte(
        &mut self,
        state_hash: i32,
        normalized_time: f32,
        layer_id: i32,
        weight: f32,
        parameters: Vec<u8>,
    ) {
        if !self.client_authority {
            return;
        }
        self.rpc_on_animation_client_message(
            state_hash,
            normalized_time,
            layer_id,
            weight,
            parameters,
        );
    }

    // 2 RpcOnAnimationClientMessage(int stateHash, float normalizedTime, int layerId, float weight, byte[] parameters)
    fn invoke_user_code_cmd_on_animation_parameters_server_message_byte(
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
            .user_code_cmd_on_animation_parameters_server_message_byte(
                reader.read_bytes_and_size(),
            );
        NetworkBehaviour::late_invoke(identity, component_index);
    }

    // 2 UserCode_CmdOnAnimationParametersServerMessage__Byte\u005B\u005D
    fn user_code_cmd_on_animation_parameters_server_message_byte(&mut self, parameters: Vec<u8>) {
        if !self.client_authority {
            return;
        }
        self.rpc_on_animation_parameters_client_message(parameters);
    }

    // 3 CmdOnAnimationTriggerServerMessage(int stateHash)
    fn invoke_user_code_cmd_on_animation_trigger_server_message_int32(
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
            .user_code_cmd_on_animation_trigger_server_message_int32(reader.decompress_var_int());
        NetworkBehaviour::late_invoke(identity, component_index);
    }

    // 3 UserCode_CmdOnAnimationTriggerServerMessage__Int32
    fn user_code_cmd_on_animation_trigger_server_message_int32(&mut self, state_hash: i32) {
        if !self.client_authority {
            return;
        }
        self.rpc_on_animation_trigger_client_message(state_hash);
    }

    // 4 invoke_user_code_cmd_on_animation_reset_trigger_server_message_int32
    fn invoke_user_code_cmd_on_animation_reset_trigger_server_message_int32(
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
            .user_code_cmd_on_animation_reset_trigger_server_message_int32(
                reader.decompress_var_int(),
            );
        NetworkBehaviour::late_invoke(identity, component_index);
    }

    // 4 UserCode_CmdOnAnimationResetTriggerServerMessage__Int32
    fn user_code_cmd_on_animation_reset_trigger_server_message_int32(&mut self, state_hash: i32) {
        if !self.client_authority {
            return;
        }
        self.rpc_on_animation_reset_trigger_client_message(state_hash);
    }

    // 5 invoke_user_code_cmd_set_animator_speed_single
    fn invoke_user_code_cmd_set_animator_speed_single(
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
            .user_code_cmd_set_animator_speed_single(reader.read_float());
        NetworkBehaviour::late_invoke(identity, component_index);
    }

    // 5 UserCode_CmdSetAnimatorSpeed__Single
    fn user_code_cmd_set_animator_speed_single(&mut self, speed: f32) {
        if !self.client_authority {
            return;
        }
        // TODO this.animator.speed = newSpeed;
        self.animator_speed = speed;
        self.set_sync_object_dirty_bits(1 << 0);
    }

    // 1 RpcOnAnimationClientMessage(int stateHash, float normalizedTime, int layerId, float weight, byte[] parameters)
    fn rpc_on_animation_client_message(
        &mut self,
        state_hash: i32,
        normalized_time: f32,
        layer_id: i32,
        weight: f32,
        parameters: Vec<u8>,
    ) {
        NetworkWriterPool::get_return(|writer| {
            writer.compress_var_int(state_hash);
            writer.write_float(normalized_time);
            writer.compress_var_int(layer_id);
            writer.write_float(weight);
            writer.write_bytes_and_size(parameters);
            self.send_rpc_internal("System.Void Mirror.NetworkAnimator::RpcOnAnimationClientMessage(System.Int32,System.Single,System.Int32,System.Single,System.Byte[])", -392669502, writer, TransportChannel::Reliable, true);
        });
    }

    // 2 RpcOnAnimationParametersClientMessage(byte[] parameters)
    fn rpc_on_animation_parameters_client_message(&mut self, parameters: Vec<u8>) {
        NetworkWriterPool::get_return(|writer| {
            writer.write_bytes_and_size(parameters);
            self.send_rpc_internal("System.Void Mirror.NetworkAnimator::RpcOnAnimationParametersClientMessage(System.Byte[])", -2095336766, writer, TransportChannel::Reliable, true);
        });
    }

    // 3 RpcOnAnimationTriggerClientMessage(int stateHash)
    fn rpc_on_animation_trigger_client_message(&mut self, state_hash: i32) {
        NetworkWriterPool::get_return(|writer| {
            writer.compress_var_int(state_hash);
            self.send_rpc_internal("System.Void Mirror.NetworkAnimator::RpcOnAnimationTriggerClientMessage(System.Int32)", 1759094990, writer, TransportChannel::Reliable, true);
        });
    }

    // 4 RpcOnAnimationResetTriggerClientMessage(int stateHash)
    fn rpc_on_animation_reset_trigger_client_message(&mut self, state_hash: i32) {
        NetworkWriterPool::get_return(|writer| {
            writer.compress_var_int(state_hash);
            self.send_rpc_internal("System.Void Mirror.NetworkAnimator::RpcOnAnimationResetTriggerClientMessage(System.Int32)", 1545278305, writer, TransportChannel::Reliable, true);
        });
    }

    fn reset(&mut self) {
        self.network_behaviour.sync_direction = SyncDirection::ClientToServer;
    }
}

impl NetworkBehaviourTrait for NetworkAnimator {
    fn new(game_object: GameObject, network_behaviour_component: &NetworkBehaviourComponent) -> Self
    where
        Self: Sized,
    {
        Self::call_register_delegate();
        // last_parameters_count
        let last_parameters_count = network_behaviour_component
            .network_animator_setting
            .animator
            .parameters
            .len();
        let animator = Self {
            network_behaviour: NetworkBehaviour::new(
                game_object,
                network_behaviour_component.network_behaviour_setting,
                network_behaviour_component.index,
            ),
            client_authority: network_behaviour_component
                .network_animator_setting
                .client_authority,
            animator: network_behaviour_component
                .network_animator_setting
                .animator
                .clone(),
            animator_speed: network_behaviour_component
                .network_animator_setting
                .animator_speed,
            previous_speed: network_behaviour_component
                .network_animator_setting
                .previous_speed,
            last_int_parameters: Vec::with_capacity(last_parameters_count),
            last_float_parameters: Vec::with_capacity(last_parameters_count),
            last_bool_parameters: Vec::with_capacity(last_parameters_count),
        };
        animator
    }

    fn register_delegate()
    where
        Self: Sized,
    {
        log_debug!("Registering delegate for ", Self::COMPONENT_TAG);
        // 1 RemoteProcedureCalls.RegisterCommand(typeof (NetworkAnimator), "System.Void Mirror.NetworkAnimator::CmdOnAnimationServerMessage(System.Int32,System.Single,System.Int32,System.Single,System.Byte[])", new RemoteCallDelegate(NetworkAnimator.invoke_user_code_cmd_on_animation_server_message_int32_single_int32_single_byte\u005B\u005D), true);
        RemoteProcedureCalls::register_command_delegate::<Self>(
            "System.Void Mirror.NetworkAnimator::CmdOnAnimationServerMessage(System.Int32,System.Single,System.Int32,System.Single,System.Byte[])",
            Box::new(Self::invoke_user_code_cmd_on_animation_server_message_int32_single_int32_single_byte),
            true,
        );

        // 2 RemoteProcedureCalls.RegisterCommand(typeof (NetworkAnimator), "System.Void Mirror.NetworkAnimator::CmdOnAnimationParametersServerMessage(System.Byte[])", new RemoteCallDelegate(NetworkAnimator.InvokeUserCode_CmdOnAnimationParametersServerMessage__Byte\u005B\u005D), true);
        RemoteProcedureCalls::register_command_delegate::<Self>(
            "System.Void Mirror.NetworkAnimator::CmdOnAnimationParametersServerMessage(System.Byte[])",
            Box::new(Self::invoke_user_code_cmd_on_animation_parameters_server_message_byte),
            true,
        );

        // 3 RemoteProcedureCalls.RegisterCommand(typeof (NetworkAnimator), "System.Void Mirror.NetworkAnimator::CmdOnAnimationTriggerServerMessage(System.Int32)", new RemoteCallDelegate(NetworkAnimator.InvokeUserCode_CmdOnAnimationTriggerServerMessage__Int32), true);
        RemoteProcedureCalls::register_command_delegate::<Self>(
            "System.Void Mirror.NetworkAnimator::CmdOnAnimationTriggerServerMessage(System.Int32)",
            Box::new(Self::invoke_user_code_cmd_on_animation_trigger_server_message_int32),
            true,
        );

        // 4 RemoteProcedureCalls.RegisterCommand(typeof (NetworkAnimator), "System.Void Mirror.NetworkAnimator::CmdOnAnimationResetTriggerServerMessage(System.Int32)", new RemoteCallDelegate(NetworkAnimator.InvokeUserCode_CmdOnAnimationResetTriggerServerMessage__Int32), true);
        RemoteProcedureCalls::register_command_delegate::<Self>(
            "System.Void Mirror.NetworkAnimator::CmdOnAnimationResetTriggerServerMessage(System.Int32)",
            Box::new(Self::invoke_user_code_cmd_on_animation_reset_trigger_server_message_int32),
            true,
        );

        // 5 RemoteProcedureCalls.RegisterCommand(typeof (NetworkAnimator), "System.Void Mirror.NetworkAnimator::CmdSetAnimatorSpeed(System.Single)", new RemoteCallDelegate(NetworkAnimator.InvokeUserCode_CmdSetAnimatorSpeed__Single), true);
        RemoteProcedureCalls::register_command_delegate::<Self>(
            "System.Void Mirror.NetworkAnimator::CmdSetAnimatorSpeed(System.Single)",
            Box::new(Self::invoke_user_code_cmd_set_animator_speed_single),
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

    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        // 默认实现 start
        self.serialize_sync_objects(writer, initial_state);
        self.serialize_sync_vars(writer, initial_state);
        // 默认实现 end

        if initial_state {
            let layer_count = self.animator.layers.len() as u8;
            writer.write_byte(layer_count);

            for layer in self.animator.layers.iter() {
                writer.write_int(layer.full_path_hash);
                writer.write_float(layer.normalized_time);
                writer.write_float(layer.layer_weight);
            }

            self.write_parameters(writer, true);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn fixed_update(&mut self) {}

    fn serialize_sync_vars(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        if initial_state {
            writer.write_float(self.animator_speed);
        } else {
            writer.write_ulong(self.sync_var_dirty_bits());
            if self.sync_var_dirty_bits() & (1 << 0) != 0 {
                writer.write_float(self.animator_speed);
            }
        }
    }

    fn deserialize_sync_vars(&mut self, _reader: &mut NetworkReader, _initial_state: bool) -> bool {
        true
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AnimatorLayer {
    #[serde(rename = "fullPathHash")]
    pub full_path_hash: i32,
    #[serde(rename = "normalizedTime")]
    pub normalized_time: f32,
    #[serde(rename = "layerWeight")]
    pub layer_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatorParameter {
    #[serde(rename = "index")]
    pub index: i32,
    #[serde(rename = "type")]
    pub r#type: AnimatorParameterType,
    #[serde(rename = "value")]
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Animator {
    #[serde(rename = "layers")]
    pub layers: Vec<AnimatorLayer>,
    #[serde(rename = "parameters")]
    pub parameters: Vec<AnimatorParameter>,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum AnimatorParameterType {
    Float = 1,
    Int = 3,
    Bool = 4,
    Trigger = 9,
}

// 测试
#[cfg(test)]
mod tests {
    use crate::mirror::core::tools::stable_hash::StableHash;

    #[test]
    fn test_network_animator() {
        println!("{}", "System.Void Mirror.NetworkAnimator::CmdOnAnimationServerMessage(System.Int32,System.Single,System.Int32,System.Single,System.Byte[])".get_fn_stable_hash_code());
        println!("{}", "System.Void Mirror.NetworkAnimator::CmdOnAnimationParametersServerMessage(System.Byte[])".get_fn_stable_hash_code());
        println!(
            "{}",
            "System.Void Mirror.NetworkAnimator::CmdOnAnimationTriggerServerMessage(System.Int32)"
                .get_fn_stable_hash_code()
        );
        println!("{}", "System.Void Mirror.NetworkAnimator::CmdOnAnimationResetTriggerServerMessage(System.Int32)".get_fn_stable_hash_code());
        println!(
            "{}",
            "System.Void Mirror.NetworkAnimator::CmdSetAnimatorSpeed(System.Single)"
                .get_fn_stable_hash_code()
        );
    }
}
