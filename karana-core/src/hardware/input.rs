use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputSource {
    Keyboard,
    Gaze(f32, f32), // x, y coordinates (0.0 - 1.0)
    Voice(String),
    Gesture(String), // "SwipeLeft", "Nod"
}

pub struct MultimodalInput {
    pub last_gaze: (f32, f32),
    pub last_voice_command: Option<String>,
}

impl MultimodalInput {
    pub fn new() -> Self {
        Self {
            last_gaze: (0.5, 0.5), // Center
            last_voice_command: None,
        }
    }

    pub fn update_gaze(&mut self, x: f32, y: f32) {
        self.last_gaze = (x.max(0.0).min(1.0), y.max(0.0).min(1.0));
    }

    pub fn process_voice(&mut self, command: &str) {
        self.last_voice_command = Some(command.to_string());
        log::info!("Atom 3 (Input): Voice Command Recognized: '{}'", command);
    }

    pub fn simulate_random_gaze(&mut self) {
        // Simulate eye movement
        let dx = (rand::random::<f32>() - 0.5) * 0.1;
        let dy = (rand::random::<f32>() - 0.5) * 0.1;
        self.update_gaze(self.last_gaze.0 + dx, self.last_gaze.1 + dy);
    }
}
