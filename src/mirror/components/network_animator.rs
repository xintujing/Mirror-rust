use crate::mirror::core::backend_data::NetworkBehaviourComponent;
use crate::mirror::core::network_behaviour::{Animator, GameObject, NetworkBehaviour, NetworkBehaviourTrait, SyncDirection, SyncMode};
use crate::mirror::core::sync_object::SyncObject;
use std::any::Any;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Once;
use crate::mirror::core::network_time::NetworkTime;
use crate::mirror::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::mirror::core::network_writer_pool::NetworkWriterPool;

#[derive(Debug)]
pub struct NetworkAnimator {
    network_behaviour: NetworkBehaviour,
    client_authority: bool,
    animator: Animator,
    animator_speed: f32,
    previous_speed: f32,
    last_int_parameters: Vec<(i32)>,
    last_float_parameters: Vec<(f32)>,
    last_bool_parameters: Vec<(bool)>,
    parameters: Vec<(AnimatorControllerParameter)>,
    animation_hash: Vec<i32>,
    transition_hash: Vec<i32>,
    layer_weight: Vec<f32>,
    next_send_time: f64,
}

impl NetworkBehaviourTrait for NetworkAnimator {
    fn new(game_object: GameObject, network_behaviour_component: &NetworkBehaviourComponent) -> Self
    where
        Self: Sized,
    {
        Self::call_register_delegate();
        // TODO  Initialize
        Self {
            network_behaviour: NetworkBehaviour::new(game_object, network_behaviour_component.network_behaviour_setting, network_behaviour_component.index),
            client_authority: false,
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
        }
    }

    fn register_delegate()
    where
        Self: Sized,
    {
        todo!()
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

    fn fixed_update(&mut self) {
        if !self.send_messages_allowed() {
            return;
        }
        // TODO if (!animator.enabled)
        //                 return;

    }
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

    fn check_send_rate(&mut self) {
        let now = NetworkTime::local_time();

        if self.send_messages_allowed() && self.network_behaviour.sync_interval >= 0.0 && now >= self.next_send_time {
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
        self.rpc_on_animation_parameters_client_message(parameters);
    }

    fn rpc_on_animation_parameters_client_message(&mut self, parameters: Vec<u8>) {
        NetworkWriterPool::get_return(|writer| {
            writer.write_bytes_all(parameters);
            // TODO send
        })
    }

    // void send_animation_message(int stateHash, float normalizedTime, int layerId, float weight, byte[] parameters)
    fn send_animation_message(&mut self, state_hash: i32, normalized_time: f32, layer_id: i32, weight: f32, parameters: Vec<u8>) {
        // RpcOnAnimationClientMessage
    }

    fn reset(&mut self) {
        self.network_behaviour.sync_direction = SyncDirection::ClientToServer;
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
            return self.m_name == controller_parameter.m_name && self.m_type == controller_parameter.m_type && self.m_default_float == controller_parameter.m_default_float && self.m_default_int == controller_parameter.m_default_int && self.m_default_bool == controller_parameter.m_default_bool;
        }
        false
    }

    pub fn hash_code(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.name().hash(&mut hasher);
        hasher.finish()
    }
}