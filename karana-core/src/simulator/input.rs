//! Virtual Input Devices
//!
//! Simulates camera and microphone for testing AI features:
//! - Camera: Load images, video files, or generate test patterns
//! - Microphone: Text input simulating voice, or load audio files

use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Camera resolution preset
#[derive(Debug, Clone, Copy)]
pub enum CameraResolution {
    Low,      // 640x480
    Medium,   // 1280x720
    High,     // 1920x1080
    UltraHD,  // 3840x2160
}

impl CameraResolution {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            CameraResolution::Low => (640, 480),
            CameraResolution::Medium => (1280, 720),
            CameraResolution::High => (1920, 1080),
            CameraResolution::UltraHD => (3840, 2160),
        }
    }
}

/// A simulated camera frame
#[derive(Debug, Clone)]
pub struct CameraFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,  // RGB or JPEG bytes
    pub format: FrameFormat,
    pub timestamp: Instant,
    pub frame_number: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameFormat {
    RGB,
    JPEG,
    TestPattern,
}

/// Predefined test scenes for AI testing
#[derive(Debug, Clone)]
pub enum TestScene {
    /// Empty room
    EmptyRoom,
    /// Office with desk, computer, etc.
    Office,
    /// Kitchen with appliances
    Kitchen,
    /// Street with cars, pedestrians
    Street,
    /// Face for recognition testing
    Face { name: String },
    /// Custom objects
    Objects { labels: Vec<String> },
    /// Load from file
    FromFile { path: PathBuf },
}

impl TestScene {
    /// Get a description that can be used for simulated AI responses
    pub fn description(&self) -> String {
        match self {
            TestScene::EmptyRoom => "An empty room with white walls and a wooden floor".to_string(),
            TestScene::Office => "An office desk with a computer monitor, keyboard, coffee mug, and some papers".to_string(),
            TestScene::Kitchen => "A kitchen counter with a coffee maker, toaster, cutting board, and some fruit".to_string(),
            TestScene::Street => "A city street with cars parked, pedestrians walking, and storefronts".to_string(),
            TestScene::Face { name } => format!("A person's face, possibly {}", name),
            TestScene::Objects { labels } => format!("Objects visible: {}", labels.join(", ")),
            TestScene::FromFile { path } => format!("Image from: {:?}", path),
        }
    }

    /// Get simulated detected objects
    pub fn detected_objects(&self) -> Vec<DetectedObject> {
        match self {
            TestScene::EmptyRoom => vec![],
            TestScene::Office => vec![
                DetectedObject { label: "monitor".to_string(), confidence: 0.95, bbox: (0.3, 0.2, 0.4, 0.5) },
                DetectedObject { label: "keyboard".to_string(), confidence: 0.92, bbox: (0.3, 0.7, 0.3, 0.1) },
                DetectedObject { label: "coffee mug".to_string(), confidence: 0.88, bbox: (0.7, 0.5, 0.1, 0.15) },
                DetectedObject { label: "desk".to_string(), confidence: 0.97, bbox: (0.1, 0.4, 0.8, 0.6) },
            ],
            TestScene::Kitchen => vec![
                DetectedObject { label: "coffee maker".to_string(), confidence: 0.94, bbox: (0.1, 0.3, 0.2, 0.3) },
                DetectedObject { label: "toaster".to_string(), confidence: 0.91, bbox: (0.4, 0.4, 0.15, 0.2) },
                DetectedObject { label: "apple".to_string(), confidence: 0.89, bbox: (0.7, 0.5, 0.08, 0.08) },
                DetectedObject { label: "banana".to_string(), confidence: 0.87, bbox: (0.75, 0.5, 0.1, 0.06) },
            ],
            TestScene::Street => vec![
                DetectedObject { label: "car".to_string(), confidence: 0.96, bbox: (0.1, 0.5, 0.25, 0.2) },
                DetectedObject { label: "person".to_string(), confidence: 0.93, bbox: (0.5, 0.3, 0.1, 0.4) },
                DetectedObject { label: "traffic light".to_string(), confidence: 0.90, bbox: (0.8, 0.1, 0.05, 0.15) },
            ],
            TestScene::Face { name } => vec![
                DetectedObject { label: format!("face:{}", name), confidence: 0.85, bbox: (0.3, 0.2, 0.4, 0.5) },
            ],
            TestScene::Objects { labels } => {
                labels.iter().enumerate().map(|(i, label)| {
                    let x = (i as f32 * 0.2) % 0.8;
                    DetectedObject {
                        label: label.clone(),
                        confidence: 0.85,
                        bbox: (x, 0.3, 0.15, 0.15),
                    }
                }).collect()
            },
            TestScene::FromFile { .. } => vec![],
        }
    }
}

/// A detected object in the scene
#[derive(Debug, Clone)]
pub struct DetectedObject {
    pub label: String,
    pub confidence: f32,
    /// Bounding box: (x, y, width, height) normalized 0-1
    pub bbox: (f32, f32, f32, f32),
}

/// Virtual camera for testing
pub struct VirtualCamera {
    pub width: u32,
    pub height: u32,
    pub is_capturing: bool,
    pub current_scene: TestScene,
    pub frame_count: u64,
    pub fps: u32,
    last_frame_time: Instant,
    frame_queue: VecDeque<CameraFrame>,
}

impl VirtualCamera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            is_capturing: false,
            current_scene: TestScene::EmptyRoom,
            frame_count: 0,
            fps: 30,
            last_frame_time: Instant::now(),
            frame_queue: VecDeque::new(),
        }
    }

    pub fn with_resolution(resolution: CameraResolution) -> Self {
        let (w, h) = resolution.dimensions();
        Self::new(w, h)
    }

    /// Start capturing
    pub fn start_capture(&mut self) {
        self.is_capturing = true;
        self.last_frame_time = Instant::now();
    }

    /// Stop capturing
    pub fn stop_capture(&mut self) {
        self.is_capturing = false;
    }

    /// Set the test scene
    pub fn set_scene(&mut self, scene: TestScene) {
        self.current_scene = scene;
    }

    /// Generate a test pattern frame
    pub fn capture_test_pattern(&mut self) -> CameraFrame {
        self.frame_count += 1;
        
        // Generate simple gradient pattern
        let size = (self.width * self.height * 3) as usize;
        let mut data = vec![0u8; size];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = ((y * self.width + x) * 3) as usize;
                data[idx] = (x % 256) as u8;     // R
                data[idx + 1] = (y % 256) as u8; // G
                data[idx + 2] = ((self.frame_count % 256) as u8); // B (animated)
            }
        }

        CameraFrame {
            width: self.width,
            height: self.height,
            data,
            format: FrameFormat::TestPattern,
            timestamp: Instant::now(),
            frame_number: self.frame_count,
        }
    }

    /// Get simulated frame (just metadata for AI testing)
    pub fn capture_frame(&mut self) -> CameraFrame {
        self.frame_count += 1;
        self.last_frame_time = Instant::now();

        CameraFrame {
            width: self.width,
            height: self.height,
            data: vec![], // Empty - use scene description instead
            format: FrameFormat::TestPattern,
            timestamp: Instant::now(),
            frame_number: self.frame_count,
        }
    }

    /// Get AI-friendly description of current scene
    pub fn get_scene_description(&self) -> String {
        self.current_scene.description()
    }

    /// Get detected objects for current scene
    pub fn get_detected_objects(&self) -> Vec<DetectedObject> {
        self.current_scene.detected_objects()
    }

    /// Queue a frame for playback (simulating video)
    pub fn queue_frame(&mut self, frame: CameraFrame) {
        self.frame_queue.push_back(frame);
    }

    /// Get next queued frame
    pub fn next_queued_frame(&mut self) -> Option<CameraFrame> {
        self.frame_queue.pop_front()
    }
}

/// Input event types
#[derive(Debug, Clone)]
pub enum InputEvent {
    VoiceCommand(String),
    TextInput(String),
    Gesture(super::sensors::GestureType),
    Tap { x: f32, y: f32 },
    Wake,
    Sleep,
}

/// Virtual microphone for voice input simulation
pub struct VirtualMicrophone {
    pub is_listening: bool,
    pub sample_rate: u32,
    pub channels: u8,
    pending_text: Vec<String>,
    voice_activity: bool,
    voice_start: Option<Instant>,
}

impl VirtualMicrophone {
    pub fn new() -> Self {
        Self {
            is_listening: false,
            sample_rate: 16000,
            channels: 1,
            pending_text: Vec::new(),
            voice_activity: false,
            voice_start: None,
        }
    }

    /// Start listening
    pub fn start_listening(&mut self) {
        self.is_listening = true;
    }

    /// Stop listening
    pub fn stop_listening(&mut self) {
        self.is_listening = false;
        self.voice_activity = false;
    }

    /// Simulate voice activity detection
    pub fn set_voice_activity(&mut self, active: bool) {
        if active && !self.voice_activity {
            self.voice_start = Some(Instant::now());
        }
        self.voice_activity = active;
    }

    /// Check if voice is currently active
    pub fn is_voice_active(&self) -> bool {
        self.voice_activity
    }

    /// Get duration of current voice activity
    pub fn voice_duration(&self) -> Duration {
        self.voice_start
            .map(|s| s.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    /// Inject text as simulated voice input
    pub fn inject_text(&mut self, text: &str) {
        self.pending_text.push(text.to_string());
    }

    /// Get pending transcriptions
    pub fn get_transcription(&mut self) -> Option<String> {
        self.pending_text.pop()
    }

    /// Simulate common voice commands
    pub fn simulate_command(&mut self, command: VoiceCommand) {
        let text = match command {
            VoiceCommand::SetTimer { minutes } => format!("set a timer for {} minutes", minutes),
            VoiceCommand::WhatAmILookingAt => "what am I looking at".to_string(),
            VoiceCommand::Navigate { destination } => format!("navigate to {}", destination),
            VoiceCommand::TakePhoto => "take a photo".to_string(),
            VoiceCommand::ReadNotifications => "read my notifications".to_string(),
            VoiceCommand::Call { contact } => format!("call {}", contact),
            VoiceCommand::SendMessage { to, message } => format!("send a message to {} saying {}", to, message),
            VoiceCommand::Search { query } => format!("search for {}", query),
            VoiceCommand::Reminder { text, when } => format!("remind me to {} {}", text, when),
            VoiceCommand::Custom(text) => text,
        };
        self.inject_text(&text);
    }

    /// Check if there are pending transcriptions
    pub fn has_pending(&self) -> bool {
        !self.pending_text.is_empty()
    }
}

impl Default for VirtualMicrophone {
    fn default() -> Self {
        Self::new()
    }
}

/// Common voice commands for testing
#[derive(Debug, Clone)]
pub enum VoiceCommand {
    SetTimer { minutes: u32 },
    WhatAmILookingAt,
    Navigate { destination: String },
    TakePhoto,
    ReadNotifications,
    Call { contact: String },
    SendMessage { to: String, message: String },
    Search { query: String },
    Reminder { text: String, when: String },
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_creation() {
        let camera = VirtualCamera::new(1280, 720);
        assert_eq!(camera.width, 1280);
        assert!(!camera.is_capturing);
    }

    #[test]
    fn test_scene_detection() {
        let mut camera = VirtualCamera::new(1280, 720);
        camera.set_scene(TestScene::Kitchen);
        
        let objects = camera.get_detected_objects();
        assert!(!objects.is_empty());
        assert!(objects.iter().any(|o| o.label == "coffee maker"));
    }

    #[test]
    fn test_microphone() {
        let mut mic = VirtualMicrophone::new();
        mic.inject_text("set a timer for 5 minutes");
        
        assert!(mic.has_pending());
        let text = mic.get_transcription().unwrap();
        assert!(text.contains("timer"));
    }

    #[test]
    fn test_voice_commands() {
        let mut mic = VirtualMicrophone::new();
        mic.simulate_command(VoiceCommand::SetTimer { minutes: 10 });
        
        let text = mic.get_transcription().unwrap();
        assert!(text.contains("10 minutes"));
    }

    #[test]
    fn test_resolution_presets() {
        let (w, h) = CameraResolution::High.dimensions();
        assert_eq!((w, h), (1920, 1080));
    }
}
