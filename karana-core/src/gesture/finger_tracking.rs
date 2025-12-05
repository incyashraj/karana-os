// Kāraṇa OS - Finger Position Tracking
// Precise finger tracking for AR UI cursor control

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::spatial::world_coords::LocalCoord;
use super::{Handedness, Finger, HandLandmark, Landmark3D};

/// Number of frames to keep in history for smoothing
const SMOOTHING_WINDOW: usize = 5;

/// Default dead zone for cursor movement (meters)
const DEFAULT_DEAD_ZONE: f32 = 0.005;

/// Finger tracking state for cursor control
#[derive(Debug, Clone)]
pub struct FingerCursor {
    /// Current position in normalized screen space (0-1)
    pub screen_position: ScreenPoint,
    /// 3D world position of tracking finger
    pub world_position: LocalCoord,
    /// Velocity for prediction
    pub velocity: ScreenPoint,
    /// Active finger being tracked
    pub tracking_finger: Finger,
    /// Which hand is active
    pub active_hand: Option<Handedness>,
    /// Is cursor engaged (finger extended)
    pub engaged: bool,
    /// Cursor visibility
    pub visible: bool,
    /// Confidence of tracking
    pub confidence: f32,
    /// Time since last update
    pub last_update: Instant,
}

/// 2D screen position (normalized 0-1)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct ScreenPoint {
    pub x: f32,
    pub y: f32,
}

impl ScreenPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: &ScreenPoint) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn lerp(&self, other: &ScreenPoint, t: f32) -> ScreenPoint {
        ScreenPoint {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    pub fn clamp(&self) -> ScreenPoint {
        ScreenPoint {
            x: self.x.clamp(0.0, 1.0),
            y: self.y.clamp(0.0, 1.0),
        }
    }
}

/// Finger tracking configuration
#[derive(Debug, Clone)]
pub struct FingerTrackingConfig {
    /// Primary finger to track for cursor
    pub primary_finger: Finger,
    /// Smoothing factor (0=no smooth, 1=max smooth)
    pub smoothing: f32,
    /// Dead zone radius for small movements
    pub dead_zone: f32,
    /// Sensitivity multiplier
    pub sensitivity: f32,
    /// Enable velocity prediction
    pub prediction_enabled: bool,
    /// Prediction lookahead (ms)
    pub prediction_ms: u32,
    /// Field of view for 3D to screen mapping
    pub fov_horizontal: f32,
    pub fov_vertical: f32,
    /// Whether to use ray casting from finger
    pub use_raycast: bool,
}

impl Default for FingerTrackingConfig {
    fn default() -> Self {
        Self {
            primary_finger: Finger::Index,
            smoothing: 0.6,
            dead_zone: DEFAULT_DEAD_ZONE,
            sensitivity: 1.0,
            prediction_enabled: true,
            prediction_ms: 16, // ~1 frame at 60fps
            fov_horizontal: 52.0,  // Typical AR glasses FOV
            fov_vertical: 30.0,
        use_raycast: true,
        }
    }
}

/// Finger tracking engine
pub struct FingerTracker {
    /// Configuration
    config: FingerTrackingConfig,
    /// Current cursor state
    cursor: FingerCursor,
    /// Position history for smoothing
    position_history: VecDeque<ScreenPoint>,
    /// Velocity history
    velocity_history: VecDeque<ScreenPoint>,
    /// Per-finger states
    finger_states: HashMap<(Handedness, Finger), FingerState>,
    /// Calibration data
    calibration: TrackingCalibration,
    /// Tracking statistics
    stats: TrackingStats,
}

/// State of individual finger
#[derive(Debug, Clone)]
pub struct FingerState {
    /// 3D tip position
    pub tip_position: LocalCoord,
    /// Is finger extended
    pub extended: bool,
    /// Curl amount (0=straight, 1=fully curled)
    pub curl: f32,
    /// Confidence
    pub confidence: f32,
    /// Last update time
    pub last_seen: Instant,
}

impl Default for FingerState {
    fn default() -> Self {
        Self {
            tip_position: LocalCoord::default(),
            extended: false,
            curl: 0.5,
            confidence: 0.0,
            last_seen: Instant::now(),
        }
    }
}

/// Calibration data for tracking
#[derive(Debug, Clone)]
pub struct TrackingCalibration {
    /// User's neutral hand position
    pub neutral_position: LocalCoord,
    /// Screen bounds in world space
    pub screen_min: LocalCoord,
    pub screen_max: LocalCoord,
    /// Personal scaling factors
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
    /// Is calibrated
    pub calibrated: bool,
}

impl Default for TrackingCalibration {
    fn default() -> Self {
        Self {
            neutral_position: LocalCoord::new(0.0, -0.2, -0.4),
            screen_min: LocalCoord::new(-0.3, -0.1, -0.5),
            screen_max: LocalCoord::new(0.3, 0.2, -0.3),
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
            calibrated: false,
        }
    }
}

/// Tracking statistics
#[derive(Debug, Clone, Default)]
pub struct TrackingStats {
    /// Total frames processed
    pub frames_processed: u64,
    /// Frames with valid tracking
    pub valid_frames: u64,
    /// Average confidence
    pub avg_confidence: f32,
    /// Average latency (ms)
    pub avg_latency_ms: f32,
    /// Tracking drops
    pub tracking_drops: u32,
}

impl FingerTracker {
    /// Create new finger tracker
    pub fn new() -> Self {
        Self::with_config(FingerTrackingConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: FingerTrackingConfig) -> Self {
        Self {
            config,
            cursor: FingerCursor::default(),
            position_history: VecDeque::with_capacity(SMOOTHING_WINDOW),
            velocity_history: VecDeque::with_capacity(SMOOTHING_WINDOW),
            finger_states: HashMap::new(),
            calibration: TrackingCalibration::default(),
            stats: TrackingStats::default(),
        }
    }

    /// Update tracking with new hand landmarks
    pub fn update(&mut self, landmarks: &[Landmark3D], hand: Handedness) -> Option<FingerCursor> {
        self.stats.frames_processed += 1;
        let start = Instant::now();

        // Extract finger states from landmarks
        self.update_finger_states(landmarks, hand);

        // Check if primary finger is extended and trackable
        let primary_key = (hand, self.config.primary_finger);
        let finger_state = self.finger_states.get(&primary_key)?.clone();

        if !finger_state.extended || finger_state.confidence < 0.5 {
            self.cursor.engaged = false;
            return Some(self.cursor.clone());
        }

        // Convert 3D position to screen position
        let raw_screen_pos = self.world_to_screen(&finger_state.tip_position);

        // Apply dead zone
        let screen_pos = self.apply_dead_zone(raw_screen_pos);

        // Apply smoothing
        let smoothed_pos = self.apply_smoothing(screen_pos);

        // Calculate velocity
        let velocity = self.calculate_velocity(smoothed_pos);

        // Apply prediction if enabled
        let final_pos = if self.config.prediction_enabled {
            self.predict_position(smoothed_pos, velocity)
        } else {
            smoothed_pos
        };

        // Update cursor state
        self.cursor.screen_position = final_pos.clamp();
        self.cursor.world_position = finger_state.tip_position.clone();
        self.cursor.velocity = velocity;
        self.cursor.active_hand = Some(hand);
        self.cursor.engaged = true;
        self.cursor.visible = true;
        self.cursor.confidence = finger_state.confidence;
        self.cursor.last_update = Instant::now();

        // Update stats
        self.stats.valid_frames += 1;
        let latency = start.elapsed().as_micros() as f32 / 1000.0;
        self.stats.avg_latency_ms = (self.stats.avg_latency_ms * 0.9) + (latency * 0.1);

        Some(self.cursor.clone())
    }

    /// Update individual finger states from landmarks
    fn update_finger_states(&mut self, landmarks: &[Landmark3D], hand: Handedness) {
        // Update each finger
        for finger in [Finger::Thumb, Finger::Index, Finger::Middle, Finger::Ring, Finger::Pinky] {
            let tip_idx = finger.tip() as usize;
            let mcp_idx = finger.mcp() as usize;

            if tip_idx < landmarks.len() && mcp_idx < landmarks.len() {
                let tip = &landmarks[tip_idx];
                let mcp = &landmarks[mcp_idx];

                // Calculate curl (simplified: based on tip-to-mcp distance)
                let distance = tip.distance_to(mcp);
                let curl = 1.0 - (distance / 0.1).clamp(0.0, 1.0);

                // Finger is extended if curl is low and tip is forward of mcp
                let extended = curl < 0.4 && tip.position.z < mcp.position.z;

                let state = FingerState {
                    tip_position: tip.position.clone(),
                    extended,
                    curl,
                    confidence: tip.confidence,
                    last_seen: Instant::now(),
                };

                self.finger_states.insert((hand, finger), state);
            }
        }
    }

    /// Convert world position to normalized screen position
    fn world_to_screen(&self, world_pos: &LocalCoord) -> ScreenPoint {
        let cal = &self.calibration;

        // Map world position to screen space based on calibration
        let range_x = cal.screen_max.x - cal.screen_min.x;
        let range_y = cal.screen_max.y - cal.screen_min.y;

        let x = if range_x > 0.0 {
            ((world_pos.x - cal.screen_min.x) / range_x) * cal.scale_x * self.config.sensitivity
        } else {
            0.5
        };

        let y = if range_y > 0.0 {
            ((world_pos.y - cal.screen_min.y) / range_y) * cal.scale_y * self.config.sensitivity
        } else {
            0.5
        };

        // Invert Y for screen coordinates (top = 0)
        ScreenPoint::new(x, 1.0 - y)
    }

    /// Apply dead zone to filter small movements
    fn apply_dead_zone(&self, new_pos: ScreenPoint) -> ScreenPoint {
        if self.position_history.is_empty() {
            return new_pos;
        }

        let last_pos = self.position_history.back().unwrap();
        let distance = new_pos.distance(last_pos);

        if distance < self.config.dead_zone {
            *last_pos
        } else {
            new_pos
        }
    }

    /// Apply exponential smoothing
    fn apply_smoothing(&mut self, new_pos: ScreenPoint) -> ScreenPoint {
        self.position_history.push_back(new_pos);
        if self.position_history.len() > SMOOTHING_WINDOW {
            self.position_history.pop_front();
        }

        if self.position_history.len() < 2 {
            return new_pos;
        }

        // Weighted average with exponential decay
        let mut weighted_sum = ScreenPoint::new(0.0, 0.0);
        let mut weight_sum = 0.0;

        for (i, pos) in self.position_history.iter().enumerate() {
            let weight = (1.0 - self.config.smoothing).powi(
                (self.position_history.len() - 1 - i) as i32
            );
            weighted_sum.x += pos.x * weight;
            weighted_sum.y += pos.y * weight;
            weight_sum += weight;
        }

        if weight_sum > 0.0 {
            ScreenPoint::new(
                weighted_sum.x / weight_sum,
                weighted_sum.y / weight_sum,
            )
        } else {
            new_pos
        }
    }

    /// Calculate cursor velocity
    fn calculate_velocity(&mut self, current_pos: ScreenPoint) -> ScreenPoint {
        if self.position_history.len() < 2 {
            return ScreenPoint::new(0.0, 0.0);
        }

        let prev_pos = &self.position_history[self.position_history.len() - 2];
        let velocity = ScreenPoint::new(
            current_pos.x - prev_pos.x,
            current_pos.y - prev_pos.y,
        );

        self.velocity_history.push_back(velocity);
        if self.velocity_history.len() > SMOOTHING_WINDOW {
            self.velocity_history.pop_front();
        }

        // Average velocity
        let mut avg = ScreenPoint::new(0.0, 0.0);
        for v in &self.velocity_history {
            avg.x += v.x;
            avg.y += v.y;
        }
        let count = self.velocity_history.len() as f32;
        ScreenPoint::new(avg.x / count, avg.y / count)
    }

    /// Predict future position based on velocity
    fn predict_position(&self, pos: ScreenPoint, velocity: ScreenPoint) -> ScreenPoint {
        let prediction_frames = self.config.prediction_ms as f32 / 16.67; // Assuming 60fps
        ScreenPoint::new(
            pos.x + velocity.x * prediction_frames,
            pos.y + velocity.y * prediction_frames,
        )
    }

    /// Start calibration process
    pub fn start_calibration(&mut self) -> CalibrationSession {
        CalibrationSession::new()
    }

    /// Apply calibration results
    pub fn apply_calibration(&mut self, calibration: TrackingCalibration) {
        self.calibration = calibration;
    }

    /// Get current cursor
    pub fn cursor(&self) -> &FingerCursor {
        &self.cursor
    }

    /// Get finger state
    pub fn get_finger_state(&self, hand: Handedness, finger: Finger) -> Option<&FingerState> {
        self.finger_states.get(&(hand, finger))
    }

    /// Get tracking stats
    pub fn stats(&self) -> &TrackingStats {
        &self.stats
    }

    /// Check if tracking is active
    pub fn is_tracking(&self) -> bool {
        self.cursor.engaged && self.cursor.confidence > 0.5
    }

    /// Get pinch distance between thumb and finger
    pub fn get_pinch_distance(&self, hand: Handedness, finger: Finger) -> Option<f32> {
        let thumb = self.finger_states.get(&(hand, Finger::Thumb))?;
        let other = self.finger_states.get(&(hand, finger))?;
        Some(thumb.tip_position.distance_to(&other.tip_position))
    }

    /// Check if performing pinch gesture
    pub fn is_pinching(&self, hand: Handedness, threshold: f32) -> bool {
        if let Some(distance) = self.get_pinch_distance(hand, Finger::Index) {
            distance < threshold
        } else {
            false
        }
    }
}

impl Default for FingerTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FingerCursor {
    fn default() -> Self {
        Self {
            screen_position: ScreenPoint::new(0.5, 0.5),
            world_position: LocalCoord::default(),
            velocity: ScreenPoint::new(0.0, 0.0),
            tracking_finger: Finger::Index,
            active_hand: None,
            engaged: false,
            visible: false,
            confidence: 0.0,
            last_update: Instant::now(),
        }
    }
}

/// Calibration session for finger tracking
pub struct CalibrationSession {
    /// Calibration state
    pub state: CalibrationState,
    /// Collected samples
    samples: Vec<CalibrationSample>,
    /// Required samples per point
    samples_per_point: usize,
    /// Current calibration point
    current_point: usize,
    /// Calibration points (normalized screen positions)
    calibration_points: Vec<ScreenPoint>,
}

/// Calibration states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibrationState {
    /// Not started
    NotStarted,
    /// Collecting samples for a point
    CollectingSamples,
    /// Moving to next point
    TransitioningPoint,
    /// Completed
    Completed,
    /// Failed
    Failed,
}

/// Single calibration sample
#[derive(Debug, Clone)]
struct CalibrationSample {
    /// Target screen point
    target: ScreenPoint,
    /// Actual finger position in world space
    finger_position: LocalCoord,
    /// Confidence
    confidence: f32,
}

impl CalibrationSession {
    pub fn new() -> Self {
        Self {
            state: CalibrationState::NotStarted,
            samples: Vec::new(),
            samples_per_point: 30,
            current_point: 0,
            calibration_points: vec![
                ScreenPoint::new(0.5, 0.5),  // Center
                ScreenPoint::new(0.1, 0.1),  // Top-left
                ScreenPoint::new(0.9, 0.1),  // Top-right
                ScreenPoint::new(0.1, 0.9),  // Bottom-left
                ScreenPoint::new(0.9, 0.9),  // Bottom-right
                ScreenPoint::new(0.5, 0.1),  // Top-center
                ScreenPoint::new(0.5, 0.9),  // Bottom-center
                ScreenPoint::new(0.1, 0.5),  // Left-center
                ScreenPoint::new(0.9, 0.5),  // Right-center
            ],
        }
    }

    /// Start calibration
    pub fn start(&mut self) {
        self.state = CalibrationState::CollectingSamples;
        self.current_point = 0;
        self.samples.clear();
    }

    /// Get current target point
    pub fn current_target(&self) -> Option<ScreenPoint> {
        if self.current_point < self.calibration_points.len() {
            Some(self.calibration_points[self.current_point])
        } else {
            None
        }
    }

    /// Add calibration sample
    pub fn add_sample(&mut self, finger_pos: LocalCoord, confidence: f32) {
        if self.state != CalibrationState::CollectingSamples {
            return;
        }

        if let Some(target) = self.current_target() {
            self.samples.push(CalibrationSample {
                target,
                finger_position: finger_pos,
                confidence,
            });

            // Check if we have enough samples for current point
            let point_samples: Vec<_> = self.samples.iter()
                .filter(|s| s.target.x == target.x && s.target.y == target.y)
                .collect();

            if point_samples.len() >= self.samples_per_point {
                self.advance_point();
            }
        }
    }

    /// Advance to next calibration point
    fn advance_point(&mut self) {
        self.current_point += 1;
        if self.current_point >= self.calibration_points.len() {
            self.state = CalibrationState::Completed;
        } else {
            self.state = CalibrationState::TransitioningPoint;
        }
    }

    /// Continue after transition
    pub fn continue_calibration(&mut self) {
        if self.state == CalibrationState::TransitioningPoint {
            self.state = CalibrationState::CollectingSamples;
        }
    }

    /// Calculate calibration results
    pub fn calculate_calibration(&self) -> Option<TrackingCalibration> {
        if self.state != CalibrationState::Completed {
            return None;
        }

        // Calculate bounds from samples
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;

        for sample in &self.samples {
            min_x = min_x.min(sample.finger_position.x);
            max_x = max_x.max(sample.finger_position.x);
            min_y = min_y.min(sample.finger_position.y);
            max_y = max_y.max(sample.finger_position.y);
            min_z = min_z.min(sample.finger_position.z);
            max_z = max_z.max(sample.finger_position.z);
        }

        // Calculate center point (neutral position)
        let neutral = LocalCoord::new(
            (min_x + max_x) / 2.0,
            (min_y + max_y) / 2.0,
            (min_z + max_z) / 2.0,
        );

        Some(TrackingCalibration {
            neutral_position: neutral,
            screen_min: LocalCoord::new(min_x, min_y, min_z),
            screen_max: LocalCoord::new(max_x, max_y, max_z),
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
            calibrated: true,
        })
    }

    /// Get progress (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        let total_points = self.calibration_points.len();
        if total_points == 0 {
            return 0.0;
        }
        self.current_point as f32 / total_points as f32
    }
}

impl Default for CalibrationSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_point_distance() {
        let p1 = ScreenPoint::new(0.0, 0.0);
        let p2 = ScreenPoint::new(1.0, 0.0);
        assert!((p1.distance(&p2) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_screen_point_lerp() {
        let p1 = ScreenPoint::new(0.0, 0.0);
        let p2 = ScreenPoint::new(1.0, 1.0);
        let mid = p1.lerp(&p2, 0.5);
        assert!((mid.x - 0.5).abs() < 0.001);
        assert!((mid.y - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_screen_point_clamp() {
        let p = ScreenPoint::new(-0.5, 1.5);
        let clamped = p.clamp();
        assert_eq!(clamped.x, 0.0);
        assert_eq!(clamped.y, 1.0);
    }

    #[test]
    fn test_finger_tracker_creation() {
        let tracker = FingerTracker::new();
        assert!(!tracker.is_tracking());
        assert_eq!(tracker.cursor().engaged, false);
    }

    #[test]
    fn test_default_config() {
        let config = FingerTrackingConfig::default();
        assert_eq!(config.primary_finger, Finger::Index);
        assert!(config.prediction_enabled);
    }

    #[test]
    fn test_calibration_session() {
        let mut session = CalibrationSession::new();
        session.start();
        assert_eq!(session.state, CalibrationState::CollectingSamples);
        assert!(session.current_target().is_some());
    }

    #[test]
    fn test_calibration_progress() {
        let session = CalibrationSession::new();
        assert_eq!(session.progress(), 0.0);
    }

    #[test]
    fn test_finger_state_default() {
        let state = FingerState::default();
        assert!(!state.extended);
        assert!((state.curl - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_tracking_calibration_default() {
        let cal = TrackingCalibration::default();
        assert!(!cal.calibrated);
    }

    #[test]
    fn test_pinch_detection() {
        let mut tracker = FingerTracker::new();
        
        // Add thumb and index finger states
        tracker.finger_states.insert(
            (Handedness::Right, Finger::Thumb),
            FingerState {
                tip_position: LocalCoord::new(0.0, 0.0, 0.0),
                extended: true,
                curl: 0.2,
                confidence: 0.9,
                last_seen: Instant::now(),
            },
        );
        tracker.finger_states.insert(
            (Handedness::Right, Finger::Index),
            FingerState {
                tip_position: LocalCoord::new(0.02, 0.0, 0.0), // 2cm apart
                extended: true,
                curl: 0.2,
                confidence: 0.9,
                last_seen: Instant::now(),
            },
        );

        // With 3cm threshold, should be pinching
        assert!(tracker.is_pinching(Handedness::Right, 0.03));
        // With 1cm threshold, should not be pinching
        assert!(!tracker.is_pinching(Handedness::Right, 0.01));
    }

    #[test]
    fn test_get_pinch_distance() {
        let mut tracker = FingerTracker::new();
        
        tracker.finger_states.insert(
            (Handedness::Left, Finger::Thumb),
            FingerState {
                tip_position: LocalCoord::new(0.0, 0.0, 0.0),
                extended: true,
                curl: 0.2,
                confidence: 0.9,
                last_seen: Instant::now(),
            },
        );
        tracker.finger_states.insert(
            (Handedness::Left, Finger::Index),
            FingerState {
                tip_position: LocalCoord::new(0.05, 0.0, 0.0),
                extended: true,
                curl: 0.2,
                confidence: 0.9,
                last_seen: Instant::now(),
            },
        );

        let distance = tracker.get_pinch_distance(Handedness::Left, Finger::Index);
        assert!(distance.is_some());
        assert!((distance.unwrap() - 0.05).abs() < 0.001);
    }
}
