//! Gesture Recognition System
//!
//! Provides hand tracking and gesture recognition for AR interactions.
//! Supports:
//! - Hand pose detection (skeletal tracking)
//! - Static gesture recognition (poses like pinch, point, thumbs up)
//! - Dynamic gesture recognition (swipe, draw, rotate)
//! - Two-hand gestures (zoom, scale)
//!
//! Privacy: All processing is local on-device via ORB cameras.

mod hand;
mod detector;
mod recognizer;
mod gestures;

pub use hand::*;
pub use detector::*;
pub use recognizer::*;
pub use gestures::*;

use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::spatial::world_coords::LocalCoord;

/// Maximum number of hands tracked simultaneously
pub const MAX_HANDS: usize = 2;

/// Hand tracking frame rate target
pub const TARGET_FRAME_RATE: u32 = 60;

/// Gesture confidence threshold
pub const DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.7;

/// Handedness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Handedness {
    Left,
    Right,
    Unknown,
}

impl Default for Handedness {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Hand landmark indices (MediaPipe compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandLandmark {
    Wrist = 0,
    ThumbCmc = 1,
    ThumbMcp = 2,
    ThumbIp = 3,
    ThumbTip = 4,
    IndexMcp = 5,
    IndexPip = 6,
    IndexDip = 7,
    IndexTip = 8,
    MiddleMcp = 9,
    MiddlePip = 10,
    MiddleDip = 11,
    MiddleTip = 12,
    RingMcp = 13,
    RingPip = 14,
    RingDip = 15,
    RingTip = 16,
    PinkyMcp = 17,
    PinkyPip = 18,
    PinkyDip = 19,
    PinkyTip = 20,
}

impl HandLandmark {
    pub fn count() -> usize {
        21
    }
    
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Wrist),
            1 => Some(Self::ThumbCmc),
            2 => Some(Self::ThumbMcp),
            3 => Some(Self::ThumbIp),
            4 => Some(Self::ThumbTip),
            5 => Some(Self::IndexMcp),
            6 => Some(Self::IndexPip),
            7 => Some(Self::IndexDip),
            8 => Some(Self::IndexTip),
            9 => Some(Self::MiddleMcp),
            10 => Some(Self::MiddlePip),
            11 => Some(Self::MiddleDip),
            12 => Some(Self::MiddleTip),
            13 => Some(Self::RingMcp),
            14 => Some(Self::RingPip),
            15 => Some(Self::RingDip),
            16 => Some(Self::RingTip),
            17 => Some(Self::PinkyMcp),
            18 => Some(Self::PinkyPip),
            19 => Some(Self::PinkyDip),
            20 => Some(Self::PinkyTip),
            _ => None,
        }
    }
    
    pub fn finger(&self) -> Option<Finger> {
        match self {
            Self::ThumbCmc | Self::ThumbMcp | Self::ThumbIp | Self::ThumbTip => Some(Finger::Thumb),
            Self::IndexMcp | Self::IndexPip | Self::IndexDip | Self::IndexTip => Some(Finger::Index),
            Self::MiddleMcp | Self::MiddlePip | Self::MiddleDip | Self::MiddleTip => Some(Finger::Middle),
            Self::RingMcp | Self::RingPip | Self::RingDip | Self::RingTip => Some(Finger::Ring),
            Self::PinkyMcp | Self::PinkyPip | Self::PinkyDip | Self::PinkyTip => Some(Finger::Pinky),
            Self::Wrist => None,
        }
    }
}

/// Finger identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Finger {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

impl Finger {
    pub fn tip(&self) -> HandLandmark {
        match self {
            Self::Thumb => HandLandmark::ThumbTip,
            Self::Index => HandLandmark::IndexTip,
            Self::Middle => HandLandmark::MiddleTip,
            Self::Ring => HandLandmark::RingTip,
            Self::Pinky => HandLandmark::PinkyTip,
        }
    }
    
    pub fn mcp(&self) -> HandLandmark {
        match self {
            Self::Thumb => HandLandmark::ThumbMcp,
            Self::Index => HandLandmark::IndexMcp,
            Self::Middle => HandLandmark::MiddleMcp,
            Self::Ring => HandLandmark::RingMcp,
            Self::Pinky => HandLandmark::PinkyMcp,
        }
    }
}

/// 3D landmark position with confidence
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Landmark3D {
    /// Position in world space
    pub position: LocalCoord,
    /// Detection confidence (0-1)
    pub confidence: f32,
    /// Visibility (0-1, for self-occlusion)
    pub visibility: f32,
}

impl Landmark3D {
    pub fn new(x: f32, y: f32, z: f32, confidence: f32) -> Self {
        Self {
            position: LocalCoord::new(x, y, z),
            confidence,
            visibility: 1.0,
        }
    }
    
    /// Distance to another landmark
    pub fn distance_to(&self, other: &Landmark3D) -> f32 {
        self.position.distance_to(&other.position)
    }
}

/// Gesture event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureEvent {
    /// Static gesture started
    GestureStart {
        gesture_type: GestureType,
        hand: Handedness,
        confidence: f32,
    },
    /// Static gesture ended
    GestureEnd {
        gesture_type: GestureType,
        hand: Handedness,
        duration: Duration,
    },
    /// Dynamic gesture progress
    GestureProgress {
        gesture_type: GestureType,
        hand: Handedness,
        progress: f32,
        direction: Option<LocalCoord>,
    },
    /// Pinch with magnitude
    PinchUpdate {
        hand: Handedness,
        pinch_strength: f32,
        position: LocalCoord,
    },
    /// Point ray update
    PointUpdate {
        hand: Handedness,
        origin: LocalCoord,
        direction: LocalCoord,
    },
    /// Two-hand gesture
    TwoHandGesture {
        gesture_type: TwoHandGestureType,
        left_position: LocalCoord,
        right_position: LocalCoord,
        scale: f32,
    },
}

/// Static gesture types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GestureType {
    /// Open palm facing forward
    OpenPalm,
    /// Closed fist
    Fist,
    /// Index finger pointing
    Point,
    /// Thumb and index touching
    Pinch,
    /// Thumbs up
    ThumbsUp,
    /// Thumbs down
    ThumbsDown,
    /// Peace sign (V)
    Peace,
    /// Rock gesture (horns)
    Rock,
    /// OK sign
    OkSign,
    /// Call me
    CallMe,
    /// Grab/grasp
    Grab,
    /// Custom gesture by name
    Custom(u32),
}

impl GestureType {
    pub fn name(&self) -> &str {
        match self {
            Self::OpenPalm => "open_palm",
            Self::Fist => "fist",
            Self::Point => "point",
            Self::Pinch => "pinch",
            Self::ThumbsUp => "thumbs_up",
            Self::ThumbsDown => "thumbs_down",
            Self::Peace => "peace",
            Self::Rock => "rock",
            Self::OkSign => "ok_sign",
            Self::CallMe => "call_me",
            Self::Grab => "grab",
            Self::Custom(_) => "custom",
        }
    }
}

/// Dynamic gesture types (motion-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DynamicGestureType {
    /// Swipe in direction
    Swipe { direction: SwipeDirection },
    /// Circle drawing
    Circle { clockwise: bool },
    /// Wave gesture
    Wave,
    /// Throw motion
    Throw,
    /// Grab and drag
    Drag,
    /// Rotate wrist
    Rotate,
    /// Draw shape
    Draw,
}

/// Swipe directions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
    Forward,
    Back,
}

/// Two-hand gesture types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TwoHandGestureType {
    /// Pinch-to-zoom
    Zoom,
    /// Rotate with two hands
    Rotate,
    /// Scale object
    Scale,
    /// Stretch
    Stretch,
    /// Clap
    Clap,
    /// Heart shape
    Heart,
}

/// Gesture state machine state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureState {
    /// No gesture detected
    None,
    /// Gesture starting (need hold time)
    Starting,
    /// Gesture active and held
    Active,
    /// Gesture ending (released)
    Ending,
}

/// Gesture configuration
#[derive(Debug, Clone)]
pub struct GestureConfig {
    /// Minimum confidence for gesture detection
    pub confidence_threshold: f32,
    /// Minimum hold time for static gestures (ms)
    pub hold_time_ms: u64,
    /// Maximum time between dynamic gesture frames (ms)
    pub max_frame_gap_ms: u64,
    /// Smoothing factor for position (0-1)
    pub position_smoothing: f32,
    /// Enable two-hand gestures
    pub two_hand_enabled: bool,
    /// Pinch distance threshold (meters)
    pub pinch_threshold: f32,
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: DEFAULT_CONFIDENCE_THRESHOLD,
            hold_time_ms: 100,
            max_frame_gap_ms: 200,
            position_smoothing: 0.8,
            two_hand_enabled: true,
            pinch_threshold: 0.03, // 3cm
        }
    }
}

/// Gesture history entry
#[derive(Debug, Clone)]
pub struct GestureHistoryEntry {
    pub gesture_type: GestureType,
    pub hand: Handedness,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub confidence: f32,
}

/// Gesture recognition engine
pub struct GestureEngine {
    /// Configuration
    config: GestureConfig,
    /// Hand detector
    detector: HandDetector,
    /// Gesture recognizer
    recognizer: GestureRecognizer,
    /// Current hand states
    hands: [Option<HandState>; MAX_HANDS],
    /// Gesture history
    history: VecDeque<GestureHistoryEntry>,
    /// Event queue
    events: VecDeque<GestureEvent>,
    /// Frame timestamp
    last_frame_time: u64,
}

impl GestureEngine {
    /// Create new gesture engine
    pub fn new(config: GestureConfig) -> Self {
        Self {
            config: config.clone(),
            detector: HandDetector::new(),
            recognizer: GestureRecognizer::new(config),
            hands: [None, None],
            history: VecDeque::with_capacity(100),
            events: VecDeque::new(),
            last_frame_time: 0,
        }
    }
    
    /// Process camera frame for hand detection
    pub fn process_frame(&mut self, _frame_data: &[u8], _width: u32, _height: u32) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // In production, this would:
        // 1. Run hand detection model (MediaPipe-style)
        // 2. Extract 21 landmarks per hand
        // 3. Run gesture classification
        
        // For now, update timestamp
        self.last_frame_time = now;
    }
    
    /// Update with detected hand poses
    pub fn update_hands(&mut self, detected_hands: Vec<HandPose>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // Clear old hand states
        for hand in &mut self.hands {
            if let Some(state) = hand {
                if now - state.last_update > self.config.max_frame_gap_ms {
                    *hand = None;
                }
            }
        }
        
        // Update with new detections
        for pose in detected_hands {
            let slot = match pose.handedness {
                Handedness::Left => 0,
                Handedness::Right => 1,
                Handedness::Unknown => continue,
            };
            
            // Recognize gesture for this hand
            let gesture = self.recognizer.classify(&pose);
            
            // Update or create hand state
            if let Some(state) = &mut self.hands[slot] {
                state.update(pose, gesture, now);
            } else {
                self.hands[slot] = Some(HandState::new(pose, gesture, now));
            }
            
            // Generate events
            self.generate_events(slot, now);
        }
        
        // Check for two-hand gestures
        if self.config.two_hand_enabled {
            self.check_two_hand_gestures(now);
        }
    }
    
    fn generate_events(&mut self, slot: usize, _now: u64) {
        if let Some(state) = &self.hands[slot] {
            // Pinch update
            if let Some((strength, pos)) = state.pinch_info() {
                if strength > 0.5 {
                    self.events.push_back(GestureEvent::PinchUpdate {
                        hand: state.pose.handedness,
                        pinch_strength: strength,
                        position: pos,
                    });
                }
            }
            
            // Point ray update
            if state.current_gesture == Some(GestureType::Point) {
                if let Some((origin, direction)) = state.point_ray() {
                    self.events.push_back(GestureEvent::PointUpdate {
                        hand: state.pose.handedness,
                        origin,
                        direction,
                    });
                }
            }
        }
    }
    
    fn check_two_hand_gestures(&mut self, _now: u64) {
        if let (Some(left), Some(right)) = (&self.hands[0], &self.hands[1]) {
            // Check for zoom gesture (both pinching)
            if let (Some((l_str, l_pos)), Some((r_str, r_pos))) = 
                (left.pinch_info(), right.pinch_info()) 
            {
                if l_str > 0.5 && r_str > 0.5 {
                    let distance = l_pos.distance_to(&r_pos);
                    self.events.push_back(GestureEvent::TwoHandGesture {
                        gesture_type: TwoHandGestureType::Zoom,
                        left_position: l_pos,
                        right_position: r_pos,
                        scale: distance,
                    });
                }
            }
        }
    }
    
    /// Get pending events
    pub fn poll_events(&mut self) -> Vec<GestureEvent> {
        self.events.drain(..).collect()
    }
    
    /// Get current hand state
    pub fn get_hand(&self, handedness: Handedness) -> Option<&HandState> {
        let slot = match handedness {
            Handedness::Left => 0,
            Handedness::Right => 1,
            Handedness::Unknown => return None,
        };
        self.hands[slot].as_ref()
    }
    
    /// Get both hands
    pub fn get_hands(&self) -> [Option<&HandState>; 2] {
        [self.hands[0].as_ref(), self.hands[1].as_ref()]
    }
    
    /// Get gesture history
    pub fn history(&self) -> &VecDeque<GestureHistoryEntry> {
        &self.history
    }
    
    /// Check if specific gesture is active
    pub fn is_gesture_active(&self, gesture: GestureType, hand: Handedness) -> bool {
        if let Some(state) = self.get_hand(hand) {
            state.current_gesture == Some(gesture) && state.gesture_state == GestureState::Active
        } else {
            false
        }
    }
}

impl Default for GestureEngine {
    fn default() -> Self {
        Self::new(GestureConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_landmark_fingers() {
        assert_eq!(HandLandmark::ThumbTip.finger(), Some(Finger::Thumb));
        assert_eq!(HandLandmark::IndexTip.finger(), Some(Finger::Index));
        assert_eq!(HandLandmark::Wrist.finger(), None);
    }
    
    #[test]
    fn test_finger_landmarks() {
        assert!(matches!(Finger::Thumb.tip(), HandLandmark::ThumbTip));
        assert!(matches!(Finger::Index.mcp(), HandLandmark::IndexMcp));
    }
    
    #[test]
    fn test_gesture_names() {
        assert_eq!(GestureType::OpenPalm.name(), "open_palm");
        assert_eq!(GestureType::Pinch.name(), "pinch");
    }
    
    #[test]
    fn test_landmark_from_index() {
        assert_eq!(HandLandmark::from_index(0), Some(HandLandmark::Wrist));
        assert_eq!(HandLandmark::from_index(8), Some(HandLandmark::IndexTip));
        assert_eq!(HandLandmark::from_index(21), None);
    }
    
    #[test]
    fn test_gesture_config_default() {
        let config = GestureConfig::default();
        assert_eq!(config.confidence_threshold, DEFAULT_CONFIDENCE_THRESHOLD);
        assert!(config.two_hand_enabled);
    }
    
    #[test]
    fn test_gesture_engine_creation() {
        let engine = GestureEngine::default();
        assert!(engine.get_hand(Handedness::Left).is_none());
        assert!(engine.get_hand(Handedness::Right).is_none());
    }
}
