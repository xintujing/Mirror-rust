use std::fmt;

// Backing field for sync NetworkBehaviour
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NetworkBehaviourSyncVar {
    pub net_id: u32,
    // Limited to 255 behaviours per identity
    pub component_index: u8,
}

impl NetworkBehaviourSyncVar {
    pub fn new(net_id: u32, component_index: i32) -> Self {
        Self {
            net_id,
            component_index: component_index as u8,
        }
    }

    pub fn equals(&self, net_id: u32, component_index: i32) -> bool {
        self.net_id == net_id && self.component_index == component_index as u8
    }
}

impl fmt::Display for NetworkBehaviourSyncVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[netId:{} compIndex:{}]", self.net_id, self.component_index)
    }
}