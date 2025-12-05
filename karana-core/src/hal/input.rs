// Kāraṇa OS - Input HAL
// Hardware abstraction for smart glasses input devices

use super::HalError;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Touch state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchState {
    /// No touch
    None,
    /// Touch started
    Down,
    /// Touch moving
    Move,
    /// Touch ended
    Up,
    /// Touch cancelled
    Cancel,
}

/// Touch point
#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    /// Unique touch ID
    pub id: u32,
    /// X position (0.0 - 1.0)
    pub x: f32,
    /// Y position (0.0 - 1.0)
    pub y: f32,
    /// Pressure (0.0 - 1.0)
    pub pressure: f32,
    /// Touch state
    pub state: TouchState,
    /// Timestamp (microseconds)
    pub timestamp: u64,
}

/// Touch event
#[derive(Debug, Clone)]
pub struct TouchEvent {
    /// All active touch points
    pub points: Vec<TouchPoint>,
    /// Primary touch point
    pub primary: Option<TouchPoint>,
    /// Timestamp
    pub timestamp: u64,
}

/// Gesture type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureType {
    /// Single tap
    Tap,
    /// Double tap
    DoubleTap,
    /// Long press
    LongPress,
    /// Swipe left
    SwipeLeft,
    /// Swipe right
    SwipeRight,
    /// Swipe up
    SwipeUp,
    /// Swipe down
    SwipeDown,
    /// Two finger tap
    TwoFingerTap,
    /// Pinch in
    PinchIn,
    /// Pinch out
    PinchOut,
    /// Rotate
    Rotate,
}

/// Gesture event
#[derive(Debug, Clone)]
pub struct GestureEvent {
    /// Gesture type
    pub gesture_type: GestureType,
    /// Gesture velocity (for swipes)
    pub velocity: f32,
    /// Gesture magnitude (for pinch)
    pub magnitude: f32,
    /// Gesture angle (for rotate, radians)
    pub angle: f32,
    /// Start position
    pub start_pos: (f32, f32),
    /// End position
    pub end_pos: (f32, f32),
    /// Timestamp
    pub timestamp: u64,
}

/// Button event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonEvent {
    /// Button pressed
    Press,
    /// Button released
    Release,
    /// Long press
    LongPress,
    /// Double press
    DoublePress,
}

/// Physical button
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    /// Power button
    Power,
    /// Volume up
    VolumeUp,
    /// Volume down
    VolumeDown,
    /// Action button (programmable)
    Action,
    /// Camera button
    Camera,
}

/// Head gesture type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadGesture {
    /// Nod up
    NodUp,
    /// Nod down
    NodDown,
    /// Shake left
    ShakeLeft,
    /// Shake right
    ShakeRight,
    /// Tilt left
    TiltLeft,
    /// Tilt right
    TiltRight,
    /// Look up
    LookUp,
    /// Look down
    LookDown,
}

/// Eye gesture (if eye tracking available)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EyeGesture {
    /// Blink
    Blink,
    /// Double blink
    DoubleBlink,
    /// Wink left
    WinkLeft,
    /// Wink right
    WinkRight,
    /// Look left
    LookLeft,
    /// Look right
    LookRight,
    /// Gaze dwell (stare at target)
    GazeDwell,
}

/// Voice command
#[derive(Debug, Clone)]
pub struct VoiceCommand {
    /// Transcribed text
    pub text: String,
    /// Confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Detected intent
    pub intent: Option<String>,
    /// Timestamp
    pub timestamp: u64,
}

/// Input HAL configuration
#[derive(Debug, Clone)]
pub struct InputConfig {
    /// Enable touch input
    pub touch_enabled: bool,
    /// Enable gesture recognition
    pub gesture_enabled: bool,
    /// Enable head gestures
    pub head_gesture_enabled: bool,
    /// Enable eye tracking/gestures
    pub eye_tracking_enabled: bool,
    /// Enable voice commands
    pub voice_enabled: bool,
    /// Touch sensitivity (0.0 - 1.0)
    pub touch_sensitivity: f32,
    /// Long press duration (ms)
    pub long_press_ms: u32,
    /// Double tap interval (ms)
    pub double_tap_ms: u32,
    /// Swipe threshold (normalized)
    pub swipe_threshold: f32,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            touch_enabled: true,
            gesture_enabled: true,
            head_gesture_enabled: true,
            eye_tracking_enabled: false,
            voice_enabled: true,
            touch_sensitivity: 0.5,
            long_press_ms: 500,
            double_tap_ms: 300,
            swipe_threshold: 0.1,
        }
    }
}

/// Input statistics
#[derive(Debug, Clone, Default)]
pub struct InputStats {
    /// Total touch events
    pub touch_events: u64,
    /// Total gestures recognized
    pub gestures_recognized: u64,
    /// Total button presses
    pub button_presses: u64,
    /// Total voice commands
    pub voice_commands: u64,
    /// Total head gestures
    pub head_gestures: u64,
}

/// Input HAL
#[derive(Debug)]
pub struct InputHal {
    /// Configuration
    config: InputConfig,
    /// Is initialized
    initialized: bool,
    /// Touch points
    touch_points: Vec<TouchPoint>,
    /// Event queue
    gesture_queue: VecDeque<GestureEvent>,
    /// Button states
    button_states: [bool; 5], // Power, VolUp, VolDown, Action, Camera
    /// Last tap time
    last_tap_time: Option<Instant>,
    /// Last tap position
    last_tap_pos: (f32, f32),
    /// Touch down time
    touch_down_time: Option<Instant>,
    /// Touch down position
    touch_down_pos: (f32, f32),
    /// Statistics
    stats: InputStats,
    /// Event counter
    event_counter: AtomicU64,
    /// Gesture recognition active
    gesture_active: AtomicBool,
}

impl InputHal {
    /// Create new input HAL
    pub fn new(config: InputConfig) -> Result<Self, HalError> {
        Ok(Self {
            config,
            initialized: false,
            touch_points: Vec::new(),
            gesture_queue: VecDeque::new(),
            button_states: [false; 5],
            last_tap_time: None,
            last_tap_pos: (0.0, 0.0),
            touch_down_time: None,
            touch_down_pos: (0.0, 0.0),
            stats: InputStats::default(),
            event_counter: AtomicU64::new(0),
            gesture_active: AtomicBool::new(false),
        })
    }

    /// Initialize input system
    pub fn initialize(&mut self) -> Result<(), HalError> {
        self.initialized = true;
        Ok(())
    }

    /// Start input processing
    pub fn start(&mut self) -> Result<(), HalError> {
        if !self.initialized {
            return Err(HalError::ConfigError("Input HAL not initialized".into()));
        }
        self.gesture_active.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Stop input processing
    pub fn stop(&mut self) -> Result<(), HalError> {
        self.gesture_active.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Process touch event (from hardware)
    pub fn process_touch(&mut self, point: TouchPoint) -> Option<GestureEvent> {
        if !self.config.touch_enabled {
            return None;
        }

        self.stats.touch_events += 1;
        
        let gesture = match point.state {
            TouchState::Down => {
                self.touch_down_time = Some(Instant::now());
                self.touch_down_pos = (point.x, point.y);
                
                // Update touch points
                self.touch_points.retain(|p| p.id != point.id);
                self.touch_points.push(point);
                
                None
            }
            TouchState::Move => {
                // Update touch point
                if let Some(existing) = self.touch_points.iter_mut().find(|p| p.id == point.id) {
                    *existing = point;
                }
                None
            }
            TouchState::Up | TouchState::Cancel => {
                let gesture = self.recognize_gesture(&point);
                self.touch_points.retain(|p| p.id != point.id);
                gesture
            }
            TouchState::None => None,
        };

        if let Some(ref g) = gesture {
            self.stats.gestures_recognized += 1;
            self.gesture_queue.push_back(g.clone());
        }

        gesture
    }

    /// Recognize gesture from touch end
    fn recognize_gesture(&mut self, end_point: &TouchPoint) -> Option<GestureEvent> {
        let down_time = self.touch_down_time?;
        let duration = down_time.elapsed();
        
        let dx = end_point.x - self.touch_down_pos.0;
        let dy = end_point.y - self.touch_down_pos.1;
        let distance = (dx * dx + dy * dy).sqrt();
        
        let timestamp = self.event_counter.fetch_add(1, Ordering::Relaxed);
        let now = Instant::now();

        // Check for long press
        if duration >= Duration::from_millis(self.config.long_press_ms as u64) && distance < 0.05 {
            return Some(GestureEvent {
                gesture_type: GestureType::LongPress,
                velocity: 0.0,
                magnitude: 0.0,
                angle: 0.0,
                start_pos: self.touch_down_pos,
                end_pos: (end_point.x, end_point.y),
                timestamp,
            });
        }

        // Check for swipe
        if distance > self.config.swipe_threshold {
            let velocity = distance / duration.as_secs_f32();
            let angle = dy.atan2(dx);
            
            let gesture_type = if dx.abs() > dy.abs() {
                if dx > 0.0 { GestureType::SwipeRight } else { GestureType::SwipeLeft }
            } else {
                if dy > 0.0 { GestureType::SwipeDown } else { GestureType::SwipeUp }
            };

            return Some(GestureEvent {
                gesture_type,
                velocity,
                magnitude: distance,
                angle,
                start_pos: self.touch_down_pos,
                end_pos: (end_point.x, end_point.y),
                timestamp,
            });
        }

        // Check for tap/double tap
        if distance < 0.05 {
            // Check for double tap
            if let Some(last_tap) = self.last_tap_time {
                let tap_interval = now - last_tap;
                let tap_distance = ((end_point.x - self.last_tap_pos.0).powi(2) 
                    + (end_point.y - self.last_tap_pos.1).powi(2)).sqrt();
                
                if tap_interval < Duration::from_millis(self.config.double_tap_ms as u64) 
                    && tap_distance < 0.1 
                {
                    self.last_tap_time = None;
                    return Some(GestureEvent {
                        gesture_type: GestureType::DoubleTap,
                        velocity: 0.0,
                        magnitude: 0.0,
                        angle: 0.0,
                        start_pos: self.touch_down_pos,
                        end_pos: (end_point.x, end_point.y),
                        timestamp,
                    });
                }
            }

            self.last_tap_time = Some(now);
            self.last_tap_pos = (end_point.x, end_point.y);

            return Some(GestureEvent {
                gesture_type: GestureType::Tap,
                velocity: 0.0,
                magnitude: 0.0,
                angle: 0.0,
                start_pos: self.touch_down_pos,
                end_pos: (end_point.x, end_point.y),
                timestamp,
            });
        }

        None
    }

    /// Process button event
    pub fn process_button(&mut self, button: Button, event: ButtonEvent) {
        let idx = match button {
            Button::Power => 0,
            Button::VolumeUp => 1,
            Button::VolumeDown => 2,
            Button::Action => 3,
            Button::Camera => 4,
        };

        match event {
            ButtonEvent::Press => {
                self.button_states[idx] = true;
                self.stats.button_presses += 1;
            }
            ButtonEvent::Release => {
                self.button_states[idx] = false;
            }
            ButtonEvent::LongPress | ButtonEvent::DoublePress => {
                self.stats.button_presses += 1;
            }
        }
    }

    /// Is button pressed
    pub fn is_button_pressed(&self, button: Button) -> bool {
        let idx = match button {
            Button::Power => 0,
            Button::VolumeUp => 1,
            Button::VolumeDown => 2,
            Button::Action => 3,
            Button::Camera => 4,
        };
        self.button_states[idx]
    }

    /// Process head gesture (from IMU)
    pub fn process_head_gesture(&mut self, gesture: HeadGesture) -> GestureEvent {
        if !self.config.head_gesture_enabled {
            // Return empty event
            return GestureEvent {
                gesture_type: GestureType::Tap,
                velocity: 0.0,
                magnitude: 0.0,
                angle: 0.0,
                start_pos: (0.0, 0.0),
                end_pos: (0.0, 0.0),
                timestamp: 0,
            };
        }

        self.stats.head_gestures += 1;
        
        let gesture_type = match gesture {
            HeadGesture::NodUp => GestureType::SwipeUp,
            HeadGesture::NodDown => GestureType::SwipeDown,
            HeadGesture::ShakeLeft => GestureType::SwipeLeft,
            HeadGesture::ShakeRight => GestureType::SwipeRight,
            _ => GestureType::Tap,
        };

        GestureEvent {
            gesture_type,
            velocity: 0.0,
            magnitude: 0.0,
            angle: 0.0,
            start_pos: (0.5, 0.5),
            end_pos: (0.5, 0.5),
            timestamp: self.event_counter.fetch_add(1, Ordering::Relaxed),
        }
    }

    /// Process eye gesture
    pub fn process_eye_gesture(&mut self, gesture: EyeGesture) -> Option<GestureEvent> {
        if !self.config.eye_tracking_enabled {
            return None;
        }

        let gesture_type = match gesture {
            EyeGesture::Blink => GestureType::Tap,
            EyeGesture::DoubleBlink => GestureType::DoubleTap,
            EyeGesture::WinkLeft | EyeGesture::LookLeft => GestureType::SwipeLeft,
            EyeGesture::WinkRight | EyeGesture::LookRight => GestureType::SwipeRight,
            EyeGesture::GazeDwell => GestureType::LongPress,
        };

        Some(GestureEvent {
            gesture_type,
            velocity: 0.0,
            magnitude: 0.0,
            angle: 0.0,
            start_pos: (0.5, 0.5),
            end_pos: (0.5, 0.5),
            timestamp: self.event_counter.fetch_add(1, Ordering::Relaxed),
        })
    }

    /// Process voice command
    pub fn process_voice_command(&mut self, command: VoiceCommand) {
        if !self.config.voice_enabled {
            return;
        }
        self.stats.voice_commands += 1;
    }

    /// Get pending gesture
    pub fn poll_gesture(&mut self) -> Option<GestureEvent> {
        self.gesture_queue.pop_front()
    }

    /// Get current touch points
    pub fn touch_points(&self) -> &[TouchPoint] {
        &self.touch_points
    }

    /// Get configuration
    pub fn config(&self) -> &InputConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: InputConfig) {
        self.config = config;
    }

    /// Get statistics
    pub fn stats(&self) -> &InputStats {
        &self.stats
    }

    /// Self-test
    pub fn test(&self) -> Result<(), HalError> {
        if !self.initialized {
            return Err(HalError::ConfigError("Not initialized".into()));
        }
        Ok(())
    }
}

impl Default for InputHal {
    fn default() -> Self {
        Self::new(InputConfig::default()).expect("Default InputHal creation should not fail")
    }
}

/// Gesture recognizer for complex patterns
#[derive(Debug)]
pub struct GestureRecognizer {
    /// Pattern history
    pattern_history: VecDeque<GestureType>,
    /// Max pattern length
    max_pattern_len: usize,
    /// Registered patterns
    patterns: Vec<(Vec<GestureType>, String)>,
}

impl GestureRecognizer {
    /// Create new gesture recognizer
    pub fn new() -> Self {
        Self {
            pattern_history: VecDeque::new(),
            max_pattern_len: 5,
            patterns: Vec::new(),
        }
    }

    /// Register a gesture pattern
    pub fn register_pattern(&mut self, pattern: Vec<GestureType>, command: String) {
        self.patterns.push((pattern, command));
    }

    /// Process gesture and check for patterns
    pub fn process(&mut self, gesture: GestureType) -> Option<String> {
        self.pattern_history.push_back(gesture);
        
        if self.pattern_history.len() > self.max_pattern_len {
            self.pattern_history.pop_front();
        }

        // Check for matching patterns
        for (pattern, command) in &self.patterns {
            if self.pattern_history.len() >= pattern.len() {
                let history_slice: Vec<_> = self.pattern_history.iter()
                    .skip(self.pattern_history.len() - pattern.len())
                    .cloned()
                    .collect();
                
                if history_slice == *pattern {
                    self.pattern_history.clear();
                    return Some(command.clone());
                }
            }
        }

        None
    }

    /// Clear pattern history
    pub fn clear(&mut self) {
        self.pattern_history.clear();
    }
}

impl Default for GestureRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_hal_creation() {
        let hal = InputHal::new(InputConfig::default());
        assert!(hal.is_ok());
    }

    #[test]
    fn test_input_hal_initialization() {
        let mut hal = InputHal::new(InputConfig::default()).unwrap();
        hal.initialize().unwrap();
        assert!(hal.initialized);
    }

    #[test]
    fn test_tap_gesture() {
        let mut hal = InputHal::new(InputConfig::default()).unwrap();
        hal.initialize().unwrap();
        hal.start().unwrap();

        // Touch down
        hal.process_touch(TouchPoint {
            id: 1,
            x: 0.5,
            y: 0.5,
            pressure: 1.0,
            state: TouchState::Down,
            timestamp: 0,
        });

        // Touch up (tap)
        let gesture = hal.process_touch(TouchPoint {
            id: 1,
            x: 0.5,
            y: 0.5,
            pressure: 0.0,
            state: TouchState::Up,
            timestamp: 100,
        });

        assert!(gesture.is_some());
        assert_eq!(gesture.unwrap().gesture_type, GestureType::Tap);
    }

    #[test]
    fn test_swipe_gesture() {
        let mut hal = InputHal::new(InputConfig::default()).unwrap();
        hal.initialize().unwrap();
        hal.start().unwrap();

        // Touch down
        hal.process_touch(TouchPoint {
            id: 1,
            x: 0.2,
            y: 0.5,
            pressure: 1.0,
            state: TouchState::Down,
            timestamp: 0,
        });

        // Touch up (swipe right)
        let gesture = hal.process_touch(TouchPoint {
            id: 1,
            x: 0.8,
            y: 0.5,
            pressure: 0.0,
            state: TouchState::Up,
            timestamp: 100,
        });

        assert!(gesture.is_some());
        assert_eq!(gesture.unwrap().gesture_type, GestureType::SwipeRight);
    }

    #[test]
    fn test_button_events() {
        let mut hal = InputHal::new(InputConfig::default()).unwrap();
        hal.initialize().unwrap();

        hal.process_button(Button::Power, ButtonEvent::Press);
        assert!(hal.is_button_pressed(Button::Power));

        hal.process_button(Button::Power, ButtonEvent::Release);
        assert!(!hal.is_button_pressed(Button::Power));
    }

    #[test]
    fn test_head_gesture() {
        let mut hal = InputHal::new(InputConfig::default()).unwrap();
        hal.initialize().unwrap();

        let gesture = hal.process_head_gesture(HeadGesture::NodDown);
        assert_eq!(gesture.gesture_type, GestureType::SwipeDown);
    }

    #[test]
    fn test_eye_gesture() {
        let mut hal = InputHal::new(InputConfig {
            eye_tracking_enabled: true,
            ..Default::default()
        }).unwrap();
        hal.initialize().unwrap();

        let gesture = hal.process_eye_gesture(EyeGesture::Blink);
        assert!(gesture.is_some());
        assert_eq!(gesture.unwrap().gesture_type, GestureType::Tap);
    }

    #[test]
    fn test_gesture_recognizer() {
        let mut recognizer = GestureRecognizer::new();
        
        recognizer.register_pattern(
            vec![GestureType::SwipeLeft, GestureType::SwipeRight, GestureType::SwipeLeft],
            "shake_cancel".into()
        );

        assert!(recognizer.process(GestureType::SwipeLeft).is_none());
        assert!(recognizer.process(GestureType::SwipeRight).is_none());
        
        let result = recognizer.process(GestureType::SwipeLeft);
        assert_eq!(result, Some("shake_cancel".into()));
    }

    #[test]
    fn test_poll_gesture() {
        let mut hal = InputHal::new(InputConfig::default()).unwrap();
        hal.initialize().unwrap();
        hal.start().unwrap();

        // Generate a tap
        hal.process_touch(TouchPoint {
            id: 1,
            x: 0.5,
            y: 0.5,
            pressure: 1.0,
            state: TouchState::Down,
            timestamp: 0,
        });
        hal.process_touch(TouchPoint {
            id: 1,
            x: 0.5,
            y: 0.5,
            pressure: 0.0,
            state: TouchState::Up,
            timestamp: 100,
        });

        let gesture = hal.poll_gesture();
        assert!(gesture.is_some());
        assert!(hal.poll_gesture().is_none()); // Queue should be empty
    }

    #[test]
    fn test_input_stats() {
        let mut hal = InputHal::new(InputConfig::default()).unwrap();
        hal.initialize().unwrap();
        hal.start().unwrap();

        hal.process_touch(TouchPoint {
            id: 1,
            x: 0.5,
            y: 0.5,
            pressure: 1.0,
            state: TouchState::Down,
            timestamp: 0,
        });

        assert_eq!(hal.stats().touch_events, 1);
    }
}
