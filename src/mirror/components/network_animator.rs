use crate::log_error;
use crate::mirror::core::backend_data::NetworkBehaviourComponent;
use crate::mirror::core::network_behaviour::{
    GameObject, NetworkBehaviour, NetworkBehaviourTrait, SyncDirection, SyncMode,
};
use crate::mirror::core::network_identity::NetworkIdentity;
use crate::mirror::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::mirror::core::network_reader_pool::NetworkReaderPool;
use crate::mirror::core::network_server::NetworkServerStatic;
use crate::mirror::core::network_time::NetworkTime;
use crate::mirror::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::mirror::core::network_writer_pool::NetworkWriterPool;
use crate::mirror::core::remote_calls::RemoteProcedureCalls;
use crate::mirror::core::sync_object::SyncObject;
use crate::mirror::core::transport::TransportChannel;
use std::any::Any;
use std::hash::{DefaultHasher, Hash, Hasher};
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
    parameters: Vec<AnimatorControllerParameter>,
    animation_hash: Vec<i32>,
    transition_hash: Vec<i32>,
    layer_weight: Vec<f32>,
    next_send_time: f64,
}

impl NetworkAnimator {
    pub const COMPONENT_TAG: &'static str = "Mirror.NetworkAnimator";

    fn send_messages_allowed(&self) -> bool {
        if !self.client_authority {
            return true;
        }
        if self.network_behaviour.index != 0 && self.network_behaviour.connection_to_client != 0 {
            return true;
        }
        self.client_authority
    }

    //  no use start
    fn check_send_rate(&mut self) {
        let now = NetworkTime::local_time();

        if self.send_messages_allowed()
            && self.network_behaviour.sync_interval >= 0.0
            && now >= self.next_send_time
        {
            self.next_send_time = now + self.network_behaviour.sync_interval;
        }

        NetworkWriterPool::get_return(|writer| {
            if self.write_parameters(writer, false) {
                // send_animation_parameters_message
            }
        })
    }

    fn write_parameters(&mut self, writer: &mut NetworkWriter, force_all: bool) -> bool {
        let parameter_count = self.parameters.len() as u8;
        writer.write_byte(parameter_count);

        let mut dirty_bits = 0;
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

            let par = &self.parameters[i];

            if par.r#type() == &AnimatorControllerParameterType::Int {
                writer.write_int(self.last_int_parameters[i]);
            } else if par.r#type() == &AnimatorControllerParameterType::Float {
                writer.write_float(self.last_float_parameters[i]);
            } else if par.r#type() == &AnimatorControllerParameterType::Bool {
                writer.write_bool(self.last_bool_parameters[i]);
            }
        }

        dirty_bits != 0
    }

    fn next_dirty_bits(&mut self) -> u64 {
        let mut dirty_bits = 0;
        for i in 0..self.parameters.len() {
            let par = &self.parameters[i];
            let mut changed = false;
            if par.r#type() == &AnimatorControllerParameterType::Int {
                let new_int_value = self.animator.get_integer(par.name_hash());
                changed |= self.last_int_parameters[i] != new_int_value;
                if changed {
                    self.last_int_parameters[i] = new_int_value;
                }
            } else if par.r#type() == &AnimatorControllerParameterType::Float {
                let new_float_value = self.animator.get_float(par.name_hash());
                changed |= (new_float_value - self.last_float_parameters[i]).abs() > 0.001;
                if changed {
                    self.last_float_parameters[i] = new_float_value;
                }
            } else if par.r#type() == &AnimatorControllerParameterType::Bool {
                let new_bool_value = self.animator.get_bool(par.name_hash());
                changed |= self.last_bool_parameters[i] != new_bool_value;
                if changed {
                    self.last_bool_parameters[i] = new_bool_value;
                }
            }
            if changed {
                dirty_bits |= 1 << i;
            }
        }
        dirty_bits
    }

    fn send_animation_parameters_message(&mut self, parameters: Vec<u8>) {
        self.cmd_on_animation_parameters_server_message(parameters);
    }

    fn cmd_on_animation_parameters_server_message(&mut self, parameters: Vec<u8>) {
        NetworkWriterPool::get_return(|writer| {
            writer.write_bytes_all(parameters);
        })
    }

    fn send_animation_message(
        &mut self,
        state_hash: i32,
        normalized_time: f32,
        layer_id: i32,
        weight: f32,
        parameters: Vec<u8>,
    ) {}

    // no use end

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
                reader.read_int(),
                reader.read_float(),
                reader.read_int(),
                reader.read_float(),
                reader.read_remaining_bytes(),
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

        NetworkReaderPool::get_with_bytes_return(parameters.to_vec(), |reader| {
            self.handle_anim_msg(state_hash, normalized_time, layer_id, weight, reader);
            self.rpc_on_animation_client_message(
                state_hash,
                normalized_time,
                layer_id,
                weight,
                parameters,
            );
        });
    }
    // 1 HandleAnimMsg
    fn handle_anim_msg(
        &mut self,
        state_hash: i32,
        normalized_time: f32,
        layer_id: i32,
        weight: f32,
        reader: &mut NetworkReader,
    ) {
        if self.client_authority {
            return;
        }
        // TODO
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

        NetworkReaderPool::get_with_bytes_return(parameters.to_vec(), |reader| {
            self.handle_anim_params_msg(reader);
            self.rpc_on_animation_parameters_client_message(parameters);
        });
    }

    // 2 HandleAnimParamsMsg
    fn handle_anim_params_msg(&mut self, reader: &mut NetworkReader) {
        if self.client_authority {
            return;
        }
        // TODO
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
            .user_code_cmd_on_animation_trigger_server_message_int32(reader.read_int());
        NetworkBehaviour::late_invoke(identity, component_index);
    }

    // 3 UserCode_CmdOnAnimationTriggerServerMessage__Int32
    fn user_code_cmd_on_animation_trigger_server_message_int32(&mut self, state_hash: i32) {
        if !self.client_authority {
            return;
        }

        self.handle_anim_trigger_msg(state_hash);
        self.rpc_on_animation_trigger_client_message(state_hash);
    }

    // 3 HandleAnimTriggerMsg
    fn handle_anim_trigger_msg(&mut self, state_hash: i32) {
        if self.client_authority {
            return;
        }
        // TODO
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
            .user_code_cmd_on_animation_reset_trigger_server_message_int32(reader.read_int());
        NetworkBehaviour::late_invoke(identity, component_index);
    }

    // 4 UserCode_CmdOnAnimationResetTriggerServerMessage__Int32
    fn user_code_cmd_on_animation_reset_trigger_server_message_int32(&mut self, state_hash: i32) {
        if !self.client_authority {
            return;
        }

        self.handle_anim_reset_trigger_msg(state_hash);
        self.rpc_on_animation_trigger_client_message(state_hash);
    }

    // 4 HandleAnimResetTriggerMsg
    fn handle_anim_reset_trigger_msg(&mut self, state_hash: i32) {
        if self.client_authority {
            return;
        }
        // TODO
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
            writer.write_int(state_hash);
            writer.write_float(normalized_time);
            writer.write_int(layer_id);
            writer.write_float(weight);
            writer.write_bytes_all(parameters);
            self.send_rpc_internal("System.Void Mirror.NetworkAnimator::RpcOnAnimationClientMessage(System.Int32,System.Single,System.Int32,System.Single,System.Byte[])", -392669502, writer, TransportChannel::Reliable, true);
        });
    }

    // 2 RpcOnAnimationParametersClientMessage(byte[] parameters)
    fn rpc_on_animation_parameters_client_message(&mut self, parameters: Vec<u8>) {
        NetworkWriterPool::get_return(|writer| {
            writer.write_bytes_all(parameters);
            self.send_rpc_internal("System.Void Mirror.NetworkAnimator::RpcOnAnimationParametersClientMessage(System.Byte[])", -2095336766, writer, TransportChannel::Reliable, true);
        });
    }

    // 3 RpcOnAnimationTriggerClientMessage(int stateHash)
    fn rpc_on_animation_trigger_client_message(&mut self, state_hash: i32) {
        NetworkWriterPool::get_return(|writer| {
            writer.write_int(state_hash);
            self.send_rpc_internal("System.Void Mirror.NetworkAnimator::RpcOnAnimationTriggerClientMessage(System.Int32)", 1759094990, writer, TransportChannel::Reliable, true);
        });
    }

    // 4 RpcOnAnimationResetTriggerClientMessage(int stateHash)
    fn rpc_on_animation_reset_trigger_client_message(&mut self, state_hash: i32) {
        NetworkWriterPool::get_return(|writer| {
            writer.write_int(state_hash);
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
        // TODO  Initialize
        let animator = Self {
            network_behaviour: NetworkBehaviour::new(
                game_object,
                network_behaviour_component.network_behaviour_setting,
                network_behaviour_component.index,
            ),
            client_authority: true,
            animator: Animator::new(),
            animator_speed: 1.0,
            previous_speed: 1.0,
            last_int_parameters: Vec::default(),
            last_float_parameters: Vec::default(),
            last_bool_parameters: Vec::default(),
            parameters: Vec::default(),
            animation_hash: Vec::default(),
            transition_hash: Vec::default(),
            layer_weight: Vec::default(),
            next_send_time: 0.0,
        };
        animator
    }

    fn register_delegate()
    where
        Self: Sized,
    {
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
            let layer_count = self.animator.layer_count as u8;
            writer.write_byte(layer_count);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn fixed_update(&mut self) {
        if !self.send_messages_allowed() {
            return;
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Animator {
    pub layer_count: i32,
}
impl Animator {
    pub fn new() -> Self {
        Self { layer_count: 0 }
    }

    pub fn get_integer(&self, id: i32) -> i32 {
        id
    }
    pub fn get_float(&self, id: i32) -> f32 {
        id as f32
    }

    pub fn get_bool(&self, id: i32) -> bool {
        id > 0
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum AnimatorControllerParameterType {
    Float = 1,
    Int = 3,
    Bool = 4,
    Trigger = 9,
}
#[derive(Debug)]
pub struct AnimatorControllerParameter {
    m_name: String,
    m_type: AnimatorControllerParameterType,
    m_default_float: f32,
    m_default_int: i32,
    m_default_bool: bool,
}

impl AnimatorControllerParameter {
    pub fn name(&self) -> &str {
        &self.m_name
    }
    pub fn set_name(&mut self, value: String) {
        self.m_name = value
    }
    pub fn name_hash(&self) -> i32 {
        // TODO Animator.StringToHash(this.m_Name);
        let mut hasher = DefaultHasher::new();
        self.m_name.hash(&mut hasher);
        hasher.finish() as i32
    }
    pub fn r#type(&self) -> &AnimatorControllerParameterType {
        &self.m_type
    }
    pub fn set_type(&mut self, value: AnimatorControllerParameterType) {
        self.m_type = value
    }
    pub fn default_float(&self) -> f32 {
        self.m_default_float
    }
    pub fn set_default_float(&mut self, value: f32) {
        self.m_default_float = value
    }
    pub fn default_int(&self) -> i32 {
        self.m_default_int
    }
    pub fn set_default_int(&mut self, value: i32) {
        self.m_default_int = value
    }
    pub fn default_bool(&self) -> bool {
        self.m_default_bool
    }
    pub fn set_default_bool(&mut self, value: bool) {
        self.m_default_bool = value
    }

    pub fn equals(&self, o: &dyn Any) -> bool {
        if let Some(controller_parameter) = o.downcast_ref::<AnimatorControllerParameter>() {
            return self.m_name == controller_parameter.m_name
                && self.m_type == controller_parameter.m_type
                && self.m_default_float == controller_parameter.m_default_float
                && self.m_default_int == controller_parameter.m_default_int
                && self.m_default_bool == controller_parameter.m_default_bool;
        }
        false
    }

    pub fn hash_code(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.name().hash(&mut hasher);
        hasher.finish()
    }
}
