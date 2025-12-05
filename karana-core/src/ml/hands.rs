// Kāraṇa OS - Hand Tracking Module
// MediaPipe-based hand detection and gesture recognition

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::{MLError, MLConfig};
use super::inference::ImageInput;
use super::vision::BoundingBox;

/// Hand tracking system
#[derive(Debug)]
pub struct HandTracker {
    /// Configuration
    config: HandConfig,
    /// Previous frame's hands for tracking
    previous_hands: Vec<TrackedHand>,
    /// Gesture recognizer
    gesture_recognizer: GestureRecognizer,
    /// Frame count
    frame_count: u64,
    /// Last tracking time
    last_tracking: Instant,
}

/// Hand tracking configuration
#[derive(Debug, Clone)]
pub struct HandConfig {
    /// Maximum hands to detect
    pub max_hands: usize,
    /// Minimum detection confidence
    pub detection_confidence: f32,
    /// Minimum tracking confidence
    pub tracking_confidence: f32,
    /// Enable gesture recognition
    pub enable_gestures: bool,
    /// Model complexity (0, 1, or 2)
    pub model_complexity: u8,
    /// Enable smoothing
    pub smoothing: bool,
    /// Smoothing factor
    pub smoothing_factor: f32,
}

impl Default for HandConfig {
    fn default() -> Self {
        Self {
            max_hands: 2,
            detection_confidence: 0.5,
            tracking_confidence: 0.5,
            enable_gestures: true,
            model_complexity: 1,
            smoothing: true,
            smoothing_factor: 0.5,
        }
    }
}

/// 21 hand landmarks per hand
pub const NUM_LANDMARKS: usize = 21;

/// Hand landmark indices
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
    /// Get landmark by index
    pub fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(HandLandmark::Wrist),
            1 => Some(HandLandmark::ThumbCmc),
            2 => Some(HandLandmark::ThumbMcp),
            3 => Some(HandLandmark::ThumbIp),
            4 => Some(HandLandmark::ThumbTip),
            5 => Some(HandLandmark::IndexMcp),
            6 => Some(HandLandmark::IndexPip),
            7 => Some(HandLandmark::IndexDip),
            8 => Some(HandLandmark::IndexTip),
            9 => Some(HandLandmark::MiddleMcp),
            10 => Some(HandLandmark::MiddlePip),
            11 => Some(HandLandmark::MiddleDip),
            12 => Some(HandLandmark::MiddleTip),
            13 => Some(HandLandmark::RingMcp),
            14 => Some(HandLandmark::RingPip),
            15 => Some(HandLandmark::RingDip),
            16 => Some(HandLandmark::RingTip),
            17 => Some(HandLandmark::PinkyMcp),
            18 => Some(HandLandmark::PinkyPip),
            19 => Some(HandLandmark::PinkyDip),
            20 => Some(HandLandmark::PinkyTip),
            _ => None,
        }
    }

    /// Get fingertip landmarks
    pub fn fingertips() -> [HandLandmark; 5] {
        [
            HandLandmark::ThumbTip,
            HandLandmark::IndexTip,
            HandLandmark::MiddleTip,
            HandLandmark::RingTip,
            HandLandmark::PinkyTip,
        ]
    }
}

/// 3D point
#[derive(Debug, Clone, Copy, Default)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3D {
    /// Create new point
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Distance to another point
    pub fn distance(&self, other: &Point3D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// 2D distance (ignoring Z)
    pub fn distance_2d(&self, other: &Point3D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Lerp towards another point
    pub fn lerp(&self, other: &Point3D, t: f32) -> Point3D {
        Point3D {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }
}

/// Handedness
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handedness {
    Left,
    Right,
}

/// Tracked hand
#[derive(Debug, Clone)]
pub struct TrackedHand {
    /// Hand ID for tracking
    pub id: u32,
    /// Left or right hand
    pub handedness: Handedness,
    /// Handedness confidence
    pub handedness_confidence: f32,
    /// 21 landmarks
    pub landmarks: [Point3D; NUM_LANDMARKS],
    /// Detection confidence
    pub confidence: f32,
    /// Bounding box
    pub bbox: BoundingBox,
    /// Detected gesture
    pub gesture: Option<Gesture>,
    /// Velocity of wrist (for motion tracking)
    pub velocity: Point3D,
}

impl TrackedHand {
    /// Get landmark by type
    pub fn landmark(&self, landmark: HandLandmark) -> &Point3D {
        &self.landmarks[landmark as usize]
    }

    /// Get all fingertip positions
    pub fn fingertips(&self) -> [&Point3D; 5] {
        [
            &self.landmarks[HandLandmark::ThumbTip as usize],
            &self.landmarks[HandLandmark::IndexTip as usize],
            &self.landmarks[HandLandmark::MiddleTip as usize],
            &self.landmarks[HandLandmark::RingTip as usize],
            &self.landmarks[HandLandmark::PinkyTip as usize],
        ]
    }

    /// Get palm center (average of MCP joints)
    pub fn palm_center(&self) -> Point3D {
        let indices = [
            HandLandmark::IndexMcp as usize,
            HandLandmark::MiddleMcp as usize,
            HandLandmark::RingMcp as usize,
            HandLandmark::PinkyMcp as usize,
        ];

        let mut center = Point3D::default();
        for idx in indices {
            center.x += self.landmarks[idx].x;
            center.y += self.landmarks[idx].y;
            center.z += self.landmarks[idx].z;
        }
        center.x /= 4.0;
        center.y /= 4.0;
        center.z /= 4.0;
        center
    }

    /// Check if finger is extended
    pub fn is_finger_extended(&self, finger: Finger) -> bool {
        let (mcp, pip, dip, tip) = match finger {
            Finger::Thumb => (
                HandLandmark::ThumbCmc,
                HandLandmark::ThumbMcp,
                HandLandmark::ThumbIp,
                HandLandmark::ThumbTip,
            ),
            Finger::Index => (
                HandLandmark::IndexMcp,
                HandLandmark::IndexPip,
                HandLandmark::IndexDip,
                HandLandmark::IndexTip,
            ),
            Finger::Middle => (
                HandLandmark::MiddleMcp,
                HandLandmark::MiddlePip,
                HandLandmark::MiddleDip,
                HandLandmark::MiddleTip,
            ),
            Finger::Ring => (
                HandLandmark::RingMcp,
                HandLandmark::RingPip,
                HandLandmark::RingDip,
                HandLandmark::RingTip,
            ),
            Finger::Pinky => (
                HandLandmark::PinkyMcp,
                HandLandmark::PinkyPip,
                HandLandmark::PinkyDip,
                HandLandmark::PinkyTip,
            ),
        };

        // Check if tip is further from wrist than PIP
        let wrist = &self.landmarks[HandLandmark::Wrist as usize];
        let tip_dist = wrist.distance_2d(&self.landmarks[tip as usize]);
        let pip_dist = wrist.distance_2d(&self.landmarks[pip as usize]);

        tip_dist > pip_dist * 1.1
    }

    /// Get extended fingers
    pub fn extended_fingers(&self) -> Vec<Finger> {
        let mut extended = Vec::new();
        for finger in [Finger::Thumb, Finger::Index, Finger::Middle, Finger::Ring, Finger::Pinky] {
            if self.is_finger_extended(finger) {
                extended.push(finger);
            }
        }
        extended
    }
}

/// Finger enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finger {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

/// Recognized gesture
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gesture {
    /// Closed fist
    Fist,
    /// Open palm
    OpenPalm,
    /// Pointing (index extended)
    Pointing,
    /// Peace sign (V)
    Peace,
    /// Thumbs up
    ThumbsUp,
    /// Thumbs down
    ThumbsDown,
    /// OK sign
    OkSign,
    /// Pinch (thumb and index together)
    Pinch,
    /// Grab (closing hand)
    Grab,
    /// Swipe (hand moving sideways)
    Swipe { direction: SwipeDirection },
    /// Custom gesture
    Custom(String),
}

/// Swipe direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

impl HandTracker {
    /// Create new hand tracker
    pub fn new(config: HandConfig) -> Self {
        Self {
            config,
            previous_hands: Vec::new(),
            gesture_recognizer: GestureRecognizer::new(),
            frame_count: 0,
            last_tracking: Instant::now(),
        }
    }

    /// Process frame and detect hands
    pub fn process_frame(&mut self, image: &ImageInput) -> Result<HandTrackingResult, MLError> {
        self.frame_count += 1;
        let start = Instant::now();

        // Simulated hand detection
        let mut hands = self.detect_hands(image)?;

        // Apply smoothing if enabled
        if self.config.smoothing && !self.previous_hands.is_empty() {
            self.apply_smoothing(&mut hands);
        }

        // Recognize gestures
        if self.config.enable_gestures {
            for hand in &mut hands {
                hand.gesture = self.gesture_recognizer.recognize(hand);
            }
        }

        // Update tracking state
        self.previous_hands = hands.clone();
        self.last_tracking = Instant::now();

        Ok(HandTrackingResult {
            hands,
            frame_number: self.frame_count,
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }

    /// Detect hands (simulated)
    fn detect_hands(&self, image: &ImageInput) -> Result<Vec<TrackedHand>, MLError> {
        // Generate simulated hand in center of image
        let cx = image.width as f32 / 2.0;
        let cy = image.height as f32 / 2.0;

        let mut landmarks = [Point3D::default(); NUM_LANDMARKS];

        // Position landmarks in a hand-like pattern
        landmarks[0] = Point3D::new(cx, cy + 80.0, 0.0); // Wrist

        // Thumb
        landmarks[1] = Point3D::new(cx - 30.0, cy + 50.0, 0.0);
        landmarks[2] = Point3D::new(cx - 50.0, cy + 30.0, 0.0);
        landmarks[3] = Point3D::new(cx - 60.0, cy + 10.0, 0.0);
        landmarks[4] = Point3D::new(cx - 70.0, cy - 10.0, 0.0);

        // Index
        landmarks[5] = Point3D::new(cx - 20.0, cy + 20.0, 0.0);
        landmarks[6] = Point3D::new(cx - 25.0, cy - 10.0, 0.0);
        landmarks[7] = Point3D::new(cx - 25.0, cy - 30.0, 0.0);
        landmarks[8] = Point3D::new(cx - 25.0, cy - 50.0, 0.0);

        // Middle
        landmarks[9] = Point3D::new(cx, cy + 15.0, 0.0);
        landmarks[10] = Point3D::new(cx, cy - 20.0, 0.0);
        landmarks[11] = Point3D::new(cx, cy - 45.0, 0.0);
        landmarks[12] = Point3D::new(cx, cy - 65.0, 0.0);

        // Ring
        landmarks[13] = Point3D::new(cx + 20.0, cy + 20.0, 0.0);
        landmarks[14] = Point3D::new(cx + 22.0, cy - 10.0, 0.0);
        landmarks[15] = Point3D::new(cx + 22.0, cy - 35.0, 0.0);
        landmarks[16] = Point3D::new(cx + 22.0, cy - 55.0, 0.0);

        // Pinky
        landmarks[17] = Point3D::new(cx + 40.0, cy + 30.0, 0.0);
        landmarks[18] = Point3D::new(cx + 45.0, cy + 5.0, 0.0);
        landmarks[19] = Point3D::new(cx + 45.0, cy - 15.0, 0.0);
        landmarks[20] = Point3D::new(cx + 45.0, cy - 30.0, 0.0);

        let hand = TrackedHand {
            id: 1,
            handedness: Handedness::Right,
            handedness_confidence: 0.95,
            landmarks,
            confidence: 0.92,
            bbox: BoundingBox::new(cx - 80.0, cy - 70.0, 160.0, 160.0),
            gesture: None,
            velocity: Point3D::default(),
        };

        Ok(vec![hand])
    }

    /// Apply temporal smoothing
    fn apply_smoothing(&self, hands: &mut Vec<TrackedHand>) {
        let factor = self.config.smoothing_factor;

        for hand in hands {
            // Find matching previous hand
            if let Some(prev) = self.previous_hands.iter().find(|h| h.id == hand.id) {
                // Smooth each landmark
                for i in 0..NUM_LANDMARKS {
                    hand.landmarks[i] = prev.landmarks[i].lerp(&hand.landmarks[i], factor);
                }

                // Calculate velocity
                let dt = self.last_tracking.elapsed().as_secs_f32().max(0.001);
                let prev_wrist = &prev.landmarks[0];
                let curr_wrist = &hand.landmarks[0];
                hand.velocity = Point3D {
                    x: (curr_wrist.x - prev_wrist.x) / dt,
                    y: (curr_wrist.y - prev_wrist.y) / dt,
                    z: (curr_wrist.z - prev_wrist.z) / dt,
                };
            }
        }
    }

    /// Get configuration
    pub fn config(&self) -> &HandConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: HandConfig) {
        self.config = config;
    }
}

/// Gesture recognizer
#[derive(Debug)]
pub struct GestureRecognizer {
    /// Gesture history for temporal analysis
    history: Vec<(Instant, Gesture)>,
    /// History duration
    history_duration: Duration,
}

impl GestureRecognizer {
    /// Create new recognizer
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            history_duration: Duration::from_millis(500),
        }
    }

    /// Recognize gesture from hand
    pub fn recognize(&mut self, hand: &TrackedHand) -> Option<Gesture> {
        let extended = hand.extended_fingers();

        let gesture = match extended.len() {
            0 => Some(Gesture::Fist),
            1 if extended.contains(&Finger::Index) => Some(Gesture::Pointing),
            1 if extended.contains(&Finger::Thumb) => {
                // Check thumb direction for up/down
                let thumb_tip = hand.landmark(HandLandmark::ThumbTip);
                let thumb_mcp = hand.landmark(HandLandmark::ThumbMcp);
                if thumb_tip.y < thumb_mcp.y {
                    Some(Gesture::ThumbsUp)
                } else {
                    Some(Gesture::ThumbsDown)
                }
            }
            2 if extended.contains(&Finger::Index) && extended.contains(&Finger::Middle) => {
                Some(Gesture::Peace)
            }
            5 => Some(Gesture::OpenPalm),
            _ => None,
        };

        // Check for pinch
        if let Some(ref g) = gesture {
            if *g != Gesture::Fist {
                let thumb_tip = hand.landmark(HandLandmark::ThumbTip);
                let index_tip = hand.landmark(HandLandmark::IndexTip);
                let distance = thumb_tip.distance_2d(index_tip);

                // If thumb and index are close, it's a pinch
                if distance < 30.0 {
                    return Some(Gesture::Pinch);
                }
            }
        }

        // Update history
        if let Some(ref g) = gesture {
            self.history.push((Instant::now(), g.clone()));
            // Clean old entries
            let cutoff = Instant::now() - self.history_duration;
            self.history.retain(|(t, _)| *t > cutoff);
        }

        gesture
    }

    /// Check for dynamic gestures (swipes)
    pub fn check_swipe(&self, current_velocity: &Point3D) -> Option<SwipeDirection> {
        let threshold = 500.0; // pixels per second

        if current_velocity.x.abs() > current_velocity.y.abs() {
            if current_velocity.x > threshold {
                Some(SwipeDirection::Right)
            } else if current_velocity.x < -threshold {
                Some(SwipeDirection::Left)
            } else {
                None
            }
        } else {
            if current_velocity.y > threshold {
                Some(SwipeDirection::Down)
            } else if current_velocity.y < -threshold {
                Some(SwipeDirection::Up)
            } else {
                None
            }
        }
    }
}

impl Default for GestureRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Hand tracking result
#[derive(Debug)]
pub struct HandTrackingResult {
    /// Detected hands
    pub hands: Vec<TrackedHand>,
    /// Frame number
    pub frame_number: u64,
    /// Processing latency
    pub latency_ms: f64,
}

impl HandTrackingResult {
    /// Get dominant hand (right if both present)
    pub fn dominant_hand(&self) -> Option<&TrackedHand> {
        self.hands.iter().find(|h| h.handedness == Handedness::Right)
            .or_else(|| self.hands.first())
    }

    /// Get hand by side
    pub fn hand(&self, handedness: Handedness) -> Option<&TrackedHand> {
        self.hands.iter().find(|h| h.handedness == handedness)
    }

    /// Get any pinch gesture
    pub fn pinch(&self) -> Option<(&TrackedHand, &Point3D, &Point3D)> {
        for hand in &self.hands {
            if hand.gesture == Some(Gesture::Pinch) {
                let thumb = hand.landmark(HandLandmark::ThumbTip);
                let index = hand.landmark(HandLandmark::IndexTip);
                return Some((hand, thumb, index));
            }
        }
        None
    }

    /// Get pointing direction
    pub fn pointing_direction(&self) -> Option<(f32, f32)> {
        for hand in &self.hands {
            if hand.gesture == Some(Gesture::Pointing) {
                let mcp = hand.landmark(HandLandmark::IndexMcp);
                let tip = hand.landmark(HandLandmark::IndexTip);
                let dx = tip.x - mcp.x;
                let dy = tip.y - mcp.y;
                let len = (dx * dx + dy * dy).sqrt();
                if len > 0.0 {
                    return Some((dx / len, dy / len));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::inference::ImageFormat;

    fn test_image() -> ImageInput {
        ImageInput {
            data: vec![128u8; 640 * 480 * 3],
            width: 640,
            height: 480,
            format: ImageFormat::RGB,
        }
    }

    #[test]
    fn test_hand_config_default() {
        let config = HandConfig::default();
        assert_eq!(config.max_hands, 2);
        assert!(config.enable_gestures);
    }

    #[test]
    fn test_hand_landmark_indices() {
        assert_eq!(HandLandmark::Wrist as usize, 0);
        assert_eq!(HandLandmark::ThumbTip as usize, 4);
        assert_eq!(HandLandmark::PinkyTip as usize, 20);
    }

    #[test]
    fn test_point3d() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(3.0, 4.0, 0.0);

        assert_eq!(p1.distance(&p2), 5.0);
        assert_eq!(p1.distance_2d(&p2), 5.0);

        let mid = p1.lerp(&p2, 0.5);
        assert_eq!(mid.x, 1.5);
        assert_eq!(mid.y, 2.0);
    }

    #[test]
    fn test_hand_tracker() {
        let config = HandConfig::default();
        let mut tracker = HandTracker::new(config);
        let image = test_image();

        let result = tracker.process_frame(&image).unwrap();
        assert!(!result.hands.is_empty());
        assert!(result.latency_ms > 0.0);
    }

    #[test]
    fn test_hand_landmarks() {
        let config = HandConfig::default();
        let mut tracker = HandTracker::new(config);
        let image = test_image();

        let result = tracker.process_frame(&image).unwrap();
        let hand = &result.hands[0];

        // All landmarks should be valid
        for i in 0..NUM_LANDMARKS {
            let landmark = &hand.landmarks[i];
            assert!(landmark.x >= 0.0);
            assert!(landmark.y >= 0.0);
        }
    }

    #[test]
    fn test_palm_center() {
        let config = HandConfig::default();
        let mut tracker = HandTracker::new(config);
        let image = test_image();

        let result = tracker.process_frame(&image).unwrap();
        let hand = &result.hands[0];
        let center = hand.palm_center();

        // Center should be near image center
        assert!(center.x > 0.0);
        assert!(center.y > 0.0);
    }

    #[test]
    fn test_gesture_recognition() {
        let mut recognizer = GestureRecognizer::new();

        // Create a mock hand with all fingers extended
        let config = HandConfig::default();
        let mut tracker = HandTracker::new(config);
        let image = test_image();

        let result = tracker.process_frame(&image).unwrap();
        let hand = &result.hands[0];

        let gesture = recognizer.recognize(hand);
        assert!(gesture.is_some());
    }

    #[test]
    fn test_dominant_hand() {
        let config = HandConfig::default();
        let mut tracker = HandTracker::new(config);
        let image = test_image();

        let result = tracker.process_frame(&image).unwrap();
        let dominant = result.dominant_hand();

        assert!(dominant.is_some());
    }

    #[test]
    fn test_swipe_detection() {
        let recognizer = GestureRecognizer::new();

        // Fast rightward motion
        let velocity = Point3D::new(600.0, 100.0, 0.0);
        let swipe = recognizer.check_swipe(&velocity);
        assert_eq!(swipe, Some(SwipeDirection::Right));

        // Fast leftward motion
        let velocity = Point3D::new(-600.0, 100.0, 0.0);
        let swipe = recognizer.check_swipe(&velocity);
        assert_eq!(swipe, Some(SwipeDirection::Left));

        // Slow motion
        let velocity = Point3D::new(100.0, 100.0, 0.0);
        let swipe = recognizer.check_swipe(&velocity);
        assert!(swipe.is_none());
    }

    #[test]
    fn test_fingertips() {
        let config = HandConfig::default();
        let mut tracker = HandTracker::new(config);
        let image = test_image();

        let result = tracker.process_frame(&image).unwrap();
        let hand = &result.hands[0];

        let tips = hand.fingertips();
        assert_eq!(tips.len(), 5);
    }

    #[test]
    fn test_smoothing() {
        let mut config = HandConfig::default();
        config.smoothing = true;
        config.smoothing_factor = 0.5;

        let mut tracker = HandTracker::new(config);
        let image = test_image();

        // Process multiple frames
        let result1 = tracker.process_frame(&image).unwrap();
        let result2 = tracker.process_frame(&image).unwrap();

        // Both should have hands
        assert!(!result1.hands.is_empty());
        assert!(!result2.hands.is_empty());
    }
}
