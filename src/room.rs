use crate::connect::Connect;

#[derive(Debug, Clone)]
pub struct Room {
    pub id: [u8; 3],
    pub r#type: u8,
    pub name: &'static str,
    pub connects: Vec<Connect>,
    pub scene_id: u64,
}