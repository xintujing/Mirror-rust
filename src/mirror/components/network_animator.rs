use crate::mirror::core::backend_data::NetworkBehaviourComponent;
use crate::mirror::core::network_behaviour::{Animator, GameObject, NetworkBehaviour, NetworkBehaviourTrait, SyncDirection, SyncMode};
use crate::mirror::core::sync_object::SyncObject;
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

    fn fixed_update(&mut self) {}
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