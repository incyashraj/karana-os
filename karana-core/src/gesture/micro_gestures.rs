//! Micro-Gesture Recognition for Kāraṇa OS
//!
//! Detects subtle finger and hand movements for discreet control.
//! Ideal for public settings where large gestures are inappropriate.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::spatial::world_coords::LocalCoord;
use super::{Handedness, Finger, HandLandmark};

/// Micro-gesture recognizer
pub struct MicroGestureRecognizer {
    /// Finger position history per finger
    finger_history: [VecDeque<FingerState>; 5],
    /// Configuration
    config: MicroGestureConfig,
    /// Active micro-gestures
    active_gestures: Vec<ActiveMicroGesture>,
    /// Registered bindings
    bindings: Vec<MicroGestureBinding>,
    /// Statistics
    stats: MicroGestureStats,
    /// Last gesture time (for cooldown)
    last_gesture_time: Option<Instant>,
}

/// Single finger state
#[derive(Debug, Clone, Copy)]
pub struct FingerState {
    /// Finger identifier
    pub finger: Finger,
    /// Tip position in hand-relative space
    pub tip_position: LocalCoord,
    /// Curl amount (0=straight, 1=fully curled)
    pub curl: f32,
    /// Spread angle from middle finger axis
    pub spread: f32,
    /// Is finger extended
    pub extended: bool,
    /// Velocity of tip
    pub velocity: LocalCoord,
    /// Timestamp
    pub timestamp: Instant,
}

impl FingerState {
    /// Create new finger state
    pub fn new(finger: Finger, tip_position: LocalCoord, curl: f32) -> Self {
        Self {
            finger,
            tip_position,
            curl,
            spread: 0.0,
            extended: curl < 0.3,
            velocity: LocalCoord::new(0.0, 0.0, 0.0),
            timestamp: Instant::now(),
        }
    }
}

/// Micro-gesture configuration
#[derive(Debug, Clone)]
pub struct MicroGestureConfig {
    /// Tap detection threshold (meters)
    pub tap_threshold: f32,
    /// Double tap max interval (ms)
    pub double_tap_interval_ms: u64,
    /// Slide minimum distance (meters)
    pub slide_min_distance: f32,
    /// Slide maximum distance (meters)
    pub slide_max_distance: f32,
    /// Rotation detection threshold (radians)
    pub rotation_threshold: f32,
    /// Pinch detection threshold (meters)
    pub pinch_threshold: f32,
    /// Gesture cooldown (ms)
    pub cooldown_ms: u64,
    /// History size per finger
    pub history_size: usize,
    /// Enable thumb tap
    pub enable_thumb_tap: bool,
    /// Enable finger slides
    pub enable_finger_slides: bool,
    /// Enable finger rotations
    pub enable_finger_rotations: bool,
    /// Sensitivity multiplier
    pub sensitivity: f32,
}

impl Default for MicroGestureConfig {
    fn default() -> Self {
        Self {
            tap_threshold: 0.015,         // 15mm
            double_tap_interval_ms: 300,
            slide_min_distance: 0.01,     // 10mm
            slide_max_distance: 0.05,     // 50mm
            rotation_threshold: 0.3,      // ~17 degrees
            pinch_threshold: 0.02,        // 20mm
            cooldown_ms: 200,
            history_size: 30,
            enable_thumb_tap: true,
            enable_finger_slides: true,
            enable_finger_rotations: true,
            sensitivity: 1.0,
        }
    }
}

/// Micro-gesture types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MicroGesture {
    /// Thumb tap against index finger
    ThumbTapIndex,
    /// Thumb tap against middle finger
    ThumbTapMiddle,
    /// Thumb tap against ring finger
    ThumbTapRing,
    /// Thumb tap against pinky
    ThumbTapPinky,
    /// Double tap thumb-index
    DoubleTapIndex,
    /// Index finger slide up
    IndexSlideUp,
    /// Index finger slide down
    IndexSlideDown,
    /// Index finger slide left
    IndexSlideLeft,
    /// Index finger slide right
    IndexSlideRight,
    /// Thumb-index pinch (micro)
    MicroPinch,
    /// Thumb-index pinch release
    MicroPinchRelease,
    /// Index finger curl
    IndexCurl,
    /// Index finger extend
    IndexExtend,
    /// Finger wave (sequential taps)
    FingerWave,
    /// Twist thumb-index (rotation)
    TwistClockwise,
    /// Twist counter-clockwise
    TwistCounterClockwise,
    /// Squeeze all fingers
    Squeeze,
    /// Spread all fingers
    Spread,
    /// Thumb flick
    ThumbFlick,
}

impl MicroGesture {
    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::ThumbTapIndex => "Tap thumb to index finger",
            Self::ThumbTapMiddle => "Tap thumb to middle finger",
            Self::ThumbTapRing => "Tap thumb to ring finger",
            Self::ThumbTapPinky => "Tap thumb to pinky finger",
            Self::DoubleTapIndex => "Double tap thumb to index",
            Self::IndexSlideUp => "Slide index finger up",
            Self::IndexSlideDown => "Slide index finger down",
            Self::IndexSlideLeft => "Slide index finger left",
            Self::IndexSlideRight => "Slide index finger right",
            Self::MicroPinch => "Small pinch gesture",
            Self::MicroPinchRelease => "Release pinch",
            Self::IndexCurl => "Curl index finger",
            Self::IndexExtend => "Extend index finger",
            Self::FingerWave => "Wave fingers sequentially",
            Self::TwistClockwise => "Twist clockwise",
            Self::TwistCounterClockwise => "Twist counter-clockwise",
            Self::Squeeze => "Squeeze all fingers",
            Self::Spread => "Spread all fingers",
            Self::ThumbFlick => "Quick thumb flick",
        }
    }

    /// Default action for this gesture
    pub fn default_action(&self) -> MicroGestureAction {
        match self {
            Self::ThumbTapIndex => MicroGestureAction::Select,
            Self::ThumbTapMiddle => MicroGestureAction::Back,
            Self::ThumbTapRing => MicroGestureAction::Menu,
            Self::ThumbTapPinky => MicroGestureAction::Dismiss,
            Self::DoubleTapIndex => MicroGestureAction::Confirm,
            Self::IndexSlideUp => MicroGestureAction::ScrollUp,
            Self::IndexSlideDown => MicroGestureAction::ScrollDown,
            Self::IndexSlideLeft => MicroGestureAction::NavigateBack,
            Self::IndexSlideRight => MicroGestureAction::NavigateForward,
            Self::MicroPinch => MicroGestureAction::Grab,
            Self::MicroPinchRelease => MicroGestureAction::Release,
            Self::IndexCurl => MicroGestureAction::Minimize,
            Self::IndexExtend => MicroGestureAction::Restore,
            Self::FingerWave => MicroGestureAction::NextItem,
            Self::TwistClockwise => MicroGestureAction::VolumeUp,
            Self::TwistCounterClockwise => MicroGestureAction::VolumeDown,
            Self::Squeeze => MicroGestureAction::QuickAction,
            Self::Spread => MicroGestureAction::QuickMenu,
            Self::ThumbFlick => MicroGestureAction::Dismiss,
        }
    }
}

/// Action triggered by micro-gesture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MicroGestureAction {
    /// Select item
    Select,
    /// Confirm selection
    Confirm,
    /// Go back
    Back,
    /// Open menu
    Menu,
    /// Dismiss current item
    Dismiss,
    /// Scroll up
    ScrollUp,
    /// Scroll down
    ScrollDown,
    /// Navigate back
    NavigateBack,
    /// Navigate forward
    NavigateForward,
    /// Grab/hold
    Grab,
    /// Release grabbed item
    Release,
    /// Minimize window
    Minimize,
    /// Restore window
    Restore,
    /// Next item
    NextItem,
    /// Volume up
    VolumeUp,
    /// Volume down
    VolumeDown,
    /// Quick action
    QuickAction,
    /// Quick menu
    QuickMenu,
    /// Custom action
    Custom(String),
    /// No action
    None,
}

/// Active micro-gesture being detected
#[derive(Debug, Clone)]
pub struct ActiveMicroGesture {
    /// Gesture type
    pub gesture: MicroGesture,
    /// Start time
    pub started_at: Instant,
    /// Progress (0-1)
    pub progress: f32,
    /// Confidence
    pub confidence: f32,
}

/// Micro-gesture binding
#[derive(Debug, Clone)]
pub struct MicroGestureBinding {
    /// Gesture type
    pub gesture: MicroGesture,
    /// Required hand
    pub hand: Option<Handedness>,
    /// Action to trigger
    pub action: MicroGestureAction,
    /// Priority
    pub priority: u8,
}

/// Statistics
#[derive(Debug, Default)]
pub struct MicroGestureStats {
    /// Total micro-gestures detected
    pub total_detected: u64,
    /// Tap gestures
    pub tap_count: u64,
    /// Slide gestures
    pub slide_count: u64,
    /// Rotation gestures
    pub rotation_count: u64,
    /// Pinch gestures
    pub pinch_count: u64,
}

/// Detected micro-gesture event
#[derive(Debug, Clone)]
pub struct MicroGestureEvent {
    /// Gesture detected
    pub gesture: MicroGesture,
    /// Hand that performed gesture
    pub hand: Handedness,
    /// Confidence (0-1)
    pub confidence: f32,
    /// Timestamp
    pub timestamp: Instant,
}

impl MicroGestureRecognizer {
    /// Create new recognizer
    pub fn new() -> Self {
        Self {
            finger_history: [
                VecDeque::with_capacity(30),
                VecDeque::with_capacity(30),
                VecDeque::with_capacity(30),
                VecDeque::with_capacity(30),
                VecDeque::with_capacity(30),
            ],
            config: MicroGestureConfig::default(),
            active_gestures: Vec::new(),
            bindings: Vec::new(),
            stats: MicroGestureStats::default(),
            last_gesture_time: None,
        }
    }

    /// Create with configuration
    pub fn with_config(config: MicroGestureConfig) -> Self {
        let size = config.history_size;
        Self {
            finger_history: [
                VecDeque::with_capacity(size),
                VecDeque::with_capacity(size),
                VecDeque::with_capacity(size),
                VecDeque::with_capacity(size),
                VecDeque::with_capacity(size),
            ],
            config,
            active_gestures: Vec::new(),
            bindings: Vec::new(),
            stats: MicroGestureStats::default(),
            last_gesture_time: None,
        }
    }

    /// Update with new finger states
    pub fn update(&mut self, fingers: &[FingerState]) -> Vec<MicroGestureEvent> {
        // Check cooldown
        if let Some(last_time) = self.last_gesture_time {
            if last_time.elapsed().as_millis() < self.config.cooldown_ms as u128 {
                return Vec::new();
            }
        }

        // Update history
        for state in fingers {
            let idx = self.finger_index(state.finger);
            self.finger_history[idx].push_back(*state);
            while self.finger_history[idx].len() > self.config.history_size {
                self.finger_history[idx].pop_front();
            }
        }

        let mut events = Vec::new();

        // Detect thumb taps
        if self.config.enable_thumb_tap {
            if let Some(event) = self.detect_thumb_taps(fingers) {
                events.push(event);
            }
        }

        // Detect finger slides
        if self.config.enable_finger_slides {
            if let Some(event) = self.detect_finger_slides() {
                events.push(event);
            }
        }

        // Detect rotations
        if self.config.enable_finger_rotations {
            if let Some(event) = self.detect_rotations(fingers) {
                events.push(event);
            }
        }

        // Detect pinch
        if let Some(event) = self.detect_micro_pinch(fingers) {
            events.push(event);
        }

        // Update last gesture time
        if !events.is_empty() {
            self.last_gesture_time = Some(Instant::now());
            for event in &events {
                self.update_stats(&event.gesture);
            }
        }

        events
    }

    fn finger_index(&self, finger: Finger) -> usize {
        match finger {
            Finger::Thumb => 0,
            Finger::Index => 1,
            Finger::Middle => 2,
            Finger::Ring => 3,
            Finger::Pinky => 4,
        }
    }

    fn detect_thumb_taps(&mut self, fingers: &[FingerState]) -> Option<MicroGestureEvent> {
        // Get thumb state
        let thumb = fingers.iter().find(|f| f.finger == Finger::Thumb)?;

        // Check distance to each finger
        for other in fingers.iter().filter(|f| f.finger != Finger::Thumb) {
            let distance = self.distance(&thumb.tip_position, &other.tip_position);

            if distance < self.config.tap_threshold * self.config.sensitivity {
                let gesture = match other.finger {
                    Finger::Index => MicroGesture::ThumbTapIndex,
                    Finger::Middle => MicroGesture::ThumbTapMiddle,
                    Finger::Ring => MicroGesture::ThumbTapRing,
                    Finger::Pinky => MicroGesture::ThumbTapPinky,
                    _ => continue,
                };

                return Some(MicroGestureEvent {
                    gesture,
                    hand: Handedness::Unknown,
                    confidence: 0.9,
                    timestamp: Instant::now(),
                });
            }
        }

        None
    }

    fn detect_finger_slides(&mut self) -> Option<MicroGestureEvent> {
        let history = &self.finger_history[1]; // Index finger

        if history.len() < 10 {
            return None;
        }

        let first = history.front()?;
        let last = history.back()?;

        let dx = last.tip_position.x - first.tip_position.x;
        let dy = last.tip_position.y - first.tip_position.y;
        let dz = last.tip_position.z - first.tip_position.z;

        let distance = (dx * dx + dy * dy + dz * dz).sqrt();

        if distance < self.config.slide_min_distance || distance > self.config.slide_max_distance {
            return None;
        }

        // Determine direction (in hand-relative space)
        let gesture = if dy.abs() > dx.abs() {
            if dy > 0.0 {
                MicroGesture::IndexSlideUp
            } else {
                MicroGesture::IndexSlideDown
            }
        } else {
            if dx > 0.0 {
                MicroGesture::IndexSlideRight
            } else {
                MicroGesture::IndexSlideLeft
            }
        };

        Some(MicroGestureEvent {
            gesture,
            hand: Handedness::Unknown,
            confidence: 0.8,
            timestamp: Instant::now(),
        })
    }

    fn detect_rotations(&mut self, fingers: &[FingerState]) -> Option<MicroGestureEvent> {
        let thumb = fingers.iter().find(|f| f.finger == Finger::Thumb)?;
        let index = fingers.iter().find(|f| f.finger == Finger::Index)?;

        // Check thumb-index relative position history
        let thumb_history = &self.finger_history[0];
        let index_history = &self.finger_history[1];

        if thumb_history.len() < 10 || index_history.len() < 10 {
            return None;
        }

        // Calculate angle change
        let first_thumb = thumb_history.front()?;
        let first_index = index_history.front()?;
        let first_angle = self.angle_between(first_thumb, first_index);

        let last_angle = self.angle_between(thumb, index);
        let angle_change = last_angle - first_angle;

        if angle_change.abs() > self.config.rotation_threshold {
            let gesture = if angle_change > 0.0 {
                MicroGesture::TwistClockwise
            } else {
                MicroGesture::TwistCounterClockwise
            };

            return Some(MicroGestureEvent {
                gesture,
                hand: Handedness::Unknown,
                confidence: 0.75,
                timestamp: Instant::now(),
            });
        }

        None
    }

    fn detect_micro_pinch(&mut self, fingers: &[FingerState]) -> Option<MicroGestureEvent> {
        let thumb = fingers.iter().find(|f| f.finger == Finger::Thumb)?;
        let index = fingers.iter().find(|f| f.finger == Finger::Index)?;

        let distance = self.distance(&thumb.tip_position, &index.tip_position);

        // Check history for transition
        let thumb_history = &self.finger_history[0];
        let index_history = &self.finger_history[1];

        if thumb_history.len() < 5 || index_history.len() < 5 {
            return None;
        }

        let first_thumb = thumb_history.front()?;
        let first_index = index_history.front()?;
        let first_distance = self.distance(&first_thumb.tip_position, &first_index.tip_position);

        let delta = first_distance - distance;

        if delta > self.config.pinch_threshold && distance < self.config.tap_threshold * 2.0 {
            return Some(MicroGestureEvent {
                gesture: MicroGesture::MicroPinch,
                hand: Handedness::Unknown,
                confidence: 0.85,
                timestamp: Instant::now(),
            });
        }

        if delta < -self.config.pinch_threshold && first_distance < self.config.tap_threshold * 2.0 {
            return Some(MicroGestureEvent {
                gesture: MicroGesture::MicroPinchRelease,
                hand: Handedness::Unknown,
                confidence: 0.85,
                timestamp: Instant::now(),
            });
        }

        None
    }

    fn distance(&self, a: &LocalCoord, b: &LocalCoord) -> f32 {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        let dz = a.z - b.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    fn angle_between(&self, a: &FingerState, b: &FingerState) -> f32 {
        let dx = b.tip_position.x - a.tip_position.x;
        let dy = b.tip_position.y - a.tip_position.y;
        dy.atan2(dx)
    }

    fn update_stats(&mut self, gesture: &MicroGesture) {
        self.stats.total_detected += 1;
        match gesture {
            MicroGesture::ThumbTapIndex | MicroGesture::ThumbTapMiddle |
            MicroGesture::ThumbTapRing | MicroGesture::ThumbTapPinky |
            MicroGesture::DoubleTapIndex => {
                self.stats.tap_count += 1;
            }
            MicroGesture::IndexSlideUp | MicroGesture::IndexSlideDown |
            MicroGesture::IndexSlideLeft | MicroGesture::IndexSlideRight => {
                self.stats.slide_count += 1;
            }
            MicroGesture::TwistClockwise | MicroGesture::TwistCounterClockwise => {
                self.stats.rotation_count += 1;
            }
            MicroGesture::MicroPinch | MicroGesture::MicroPinchRelease => {
                self.stats.pinch_count += 1;
            }
            _ => {}
        }
    }

    /// Add gesture binding
    pub fn add_binding(&mut self, binding: MicroGestureBinding) {
        self.bindings.push(binding);
        self.bindings.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Get action for gesture
    pub fn get_action(&self, gesture: MicroGesture) -> MicroGestureAction {
        for binding in &self.bindings {
            if binding.gesture == gesture {
                return binding.action.clone();
            }
        }
        gesture.default_action()
    }

    /// Get statistics
    pub fn stats(&self) -> &MicroGestureStats {
        &self.stats
    }

    /// Reset recognizer state
    pub fn reset(&mut self) {
        for history in &mut self.finger_history {
            history.clear();
        }
        self.active_gestures.clear();
        self.last_gesture_time = None;
    }
}

impl Default for MicroGestureRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finger_state_creation() {
        let state = FingerState::new(
            Finger::Index,
            LocalCoord::new(0.0, 0.0, 0.0),
            0.0,
        );
        assert_eq!(state.finger, Finger::Index);
        assert!(state.extended);
    }

    #[test]
    fn test_micro_gesture_description() {
        assert!(!MicroGesture::ThumbTapIndex.description().is_empty());
        assert!(!MicroGesture::IndexSlideUp.description().is_empty());
    }

    #[test]
    fn test_default_actions() {
        let action = MicroGesture::ThumbTapIndex.default_action();
        assert!(matches!(action, MicroGestureAction::Select));
    }

    #[test]
    fn test_recognizer_creation() {
        let recognizer = MicroGestureRecognizer::new();
        assert_eq!(recognizer.stats.total_detected, 0);
    }

    #[test]
    fn test_config_defaults() {
        let config = MicroGestureConfig::default();
        assert!(config.enable_thumb_tap);
        assert!(config.sensitivity > 0.0);
    }

    #[test]
    fn test_binding_add() {
        let mut recognizer = MicroGestureRecognizer::new();
        recognizer.add_binding(MicroGestureBinding {
            gesture: MicroGesture::ThumbTapIndex,
            hand: None,
            action: MicroGestureAction::Custom("test".to_string()),
            priority: 1,
        });

        let action = recognizer.get_action(MicroGesture::ThumbTapIndex);
        assert!(matches!(action, MicroGestureAction::Custom(_)));
    }

    #[test]
    fn test_reset() {
        let mut recognizer = MicroGestureRecognizer::new();
        
        let state = FingerState::new(Finger::Index, LocalCoord::new(0.0, 0.0, 0.0), 0.0);
        recognizer.update(&[state]);
        
        recognizer.reset();
        
        assert!(recognizer.finger_history[1].is_empty());
    }
}
