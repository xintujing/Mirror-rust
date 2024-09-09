struct Transform {
    position: Vec3,
}

impl Transform {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Transform {
            position: Vec3 { x, y, z },
        }
    }
}

#[derive(Clone)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

struct NetworkManager;

impl NetworkManager {
    // Assuming we store a list of positions globally
    static mut START_POSITIONS: Vec<Transform> = Vec::new();

    fn register_start_position(position: &Transform) {
        unsafe {
            Self::START_POSITIONS.push(position.clone());
            println!("Registered start position at {:?}", position.position);
        }
    }

    fn unregister_start_position(position: &Transform) {
        unsafe {
            let index = Self::START_POSITIONS.iter().position(|p| p.position == position.position);
            if let Some(index) = index {
                Self::START_POSITIONS.remove(index);
                println!("Unregistered start position at {:?}", position.position);
            }
        }
    }
}

struct NetworkStartPosition {
    transform: Transform,
}

impl NetworkStartPosition {
    fn new(x: f32, y: f32, z: f32) -> Self {
        NetworkStartPosition {
            transform: Transform::new(x, y, z),
        }
    }

    fn awake(&self) {
        NetworkManager::register_start_position(&self.transform);
    }

    fn on_destroy(&self) {
        NetworkManager::unregister_start_position(&self.transform);
    }
}

fn main() {
    let start_position = NetworkStartPosition::new(0.0, 0.0, 0.0);
    start_position.awake();

    // Simulate object destruction
    start_position.on_destroy();
}
