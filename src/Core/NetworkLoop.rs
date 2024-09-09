use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};

lazy_static! {
    static ref EARLY_UPDATE_ACTIONS: Mutex<Vec<Box<dyn Fn() + Send>>> = Mutex::new(Vec::new());
    static ref LATE_UPDATE_ACTIONS: Mutex<Vec<Box<dyn Fn() + Send>>> = Mutex::new(Vec::new());
}

fn main() {
    // Initialize the custom loop modifications at the start of the application
    init_custom_loop();
    // Game loop
    game_loop();
}

fn init_custom_loop() {
    // Adding network updates to the early update phase of the loop
    add_to_early_update(|| {
        if is_game_playing() {
            network_early_update();
            println!("Performed network early update.");
        }
    });

    // Adding network updates to the late update phase of the loop
    add_to_late_update(|| {
        if is_game_playing() {
            network_late_update();
            println!("Performed network late update.");
        }
    });
}

fn game_loop() {
    loop {
        // Simulate the early update phase
        run_early_updates();

        // Placeholder for fixed update and main update logic
        println!("Running main update logic...");

        // Simulate the late update phase
        run_late_updates();

        // Simulating end of the loop/frame
        std::thread::sleep(std::time::Duration::from_millis(16)); // Simulate a frame delay of ~60 FPS
    }
}

fn add_to_early_update(action: impl Fn() + 'static + Send) {
    EARLY_UPDATE_ACTIONS.lock().unwrap().push(Box::new(action));
}

fn add_to_late_update(action: impl Fn() + 'static + Send) {
    LATE_UPDATE_ACTIONS.lock().unwrap().push(Box::new(action));
}

fn run_early_updates() {
    for action in EARLY_UPDATE_ACTIONS.lock().unwrap().iter() {
        action();
    }
}

fn run_late_updates() {
    for action in LATE_UPDATE_ACTIONS.lock().unwrap().iter() {
        action();
    }
}

fn network_early_update() {
    // Network update logic before the main game update
    println!("Updating network early...");
    // Additional logic and updates specific to the network handling
}

fn network_late_update() {
    // Network update logic after the main game update
    println!("Updating network late...");
    // Additional cleanup and final network updates
}

fn is_game_playing() -> bool {
    // Logic to determine if the game is currently in the playing state
    true
}
