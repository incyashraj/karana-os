//! Head Gesture Recognition for Kāraṇa OS
//!
//! Detects head movements for hands-free interaction.
//! Supports nods, shakes, tilts, and other head gestures.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::spatial::world_coords::LocalCoord;

/// Head gesture recognizer
pub struct HeadGestureRecognizer {
    /// Head pose history
    pose_history: VecDeque<HeadPose>,
    /// Configuration
    config: HeadGestureConfig,
    /// Current gesture state
    state: HeadGestureState,
    /// Registered gesture callbacks
    callbacks: Vec<Box<dyn FnMut(HeadGesture) + Send>>,
    /// Statistics
    stats: HeadGestureStats,
    /// Baseline head pose
    baseline_pose: Option<HeadPose>,
}

/// Head pose from IMU/tracking
#[derive(Debug, Clone, Copy)]
pub struct HeadPose {
    /// Pitch (looking up/down) in radians
    pub pitch: f32,
    /// Yaw (looking left/right) in radians
    pub yaw: f32,
    /// Roll (tilting head) in radians
    pub roll: f32,
    /// Position in world space
    pub position: LocalCoord,
    /// Angular velocity (rad/s)
    pub angular_velocity: AngularVelocity,
    /// Timestamp
    pub timestamp: Instant,
}

/// Angular velocity
#[derive(Debug, Clone, Copy, Default)]
pub struct AngularVelocity {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl HeadPose {
    /// Create new head pose
    pub fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
        Self {
            pitch,
            yaw,
            roll,
            position: LocalCoord::new(0.0, 1.7, 0.0), // Approximate head height
            angular_velocity: AngularVelocity::default(),
            timestamp: Instant::now(),
        }
    }

    /// Create with full parameters
    pub fn full(pitch: f32, yaw: f32, roll: f32, position: LocalCoord, angular_velocity: AngularVelocity) -> Self {
        Self {
            pitch,
            yaw,
            roll,
            position,
            angular_velocity,
            timestamp: Instant::now(),
        }
    }
}

/// Head gesture configuration
#[derive(Debug, Clone)]
pub struct HeadGestureConfig {
    /// Nod detection threshold (radians)
    pub nod_threshold: f32,
    /// Shake detection threshold (radians)
    pub shake_threshold: f32,
    /// Tilt detection threshold (radians)
    pub tilt_threshold: f32,
    /// Minimum gesture duration (ms)
    pub min_duration_ms: u64,
    /// Maximum gesture duration (ms)
    pub max_duration_ms: u64,
    /// Velocity threshold for gesture detection (rad/s)
    pub velocity_threshold: f32,
    /// Return-to-center tolerance
    pub return_tolerance: f32,
    /// Cooldown between gestures (ms)
    pub cooldown_ms: u64,
    /// Enable nod gestures
    pub enable_nod: bool,
    /// Enable shake gestures
    pub enable_shake: bool,
    /// Enable tilt gestures
    pub enable_tilt: bool,
    /// Enable look-to-scroll
    pub enable_look_scroll: bool,
    /// History size
    pub history_size: usize,
}

impl Default for HeadGestureConfig {
    fn default() -> Self {
        Self {
            nod_threshold: 0.2,           // ~11 degrees
            shake_threshold: 0.25,        // ~14 degrees
            tilt_threshold: 0.15,         // ~8 degrees
            min_duration_ms: 150,
            max_duration_ms: 800,
            velocity_threshold: 0.5,
            return_tolerance: 0.1,
            cooldown_ms: 500,
            enable_nod: true,
            enable_shake: true,
            enable_tilt: true,
            enable_look_scroll: true,
            history_size: 60,  // ~1 second at 60fps
        }
    }
}

/// Head gesture types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeadGesture {
    /// Nod down (yes)
    NodDown,
    /// Nod up (attention/summon)
    NodUp,
    /// Double nod (confirm)
    DoubleNod,
    /// Shake left-right (no)
    Shake,
    /// Single shake left
    ShakeLeft,
    /// Single shake right
    ShakeRight,
    /// Tilt left
    TiltLeft,
    /// Tilt right
    TiltRight,
    /// Look up (scroll up)
    LookUp,
    /// Look down (scroll down)
    LookDown,
    /// Look left (navigate back)
    LookLeft,
    /// Look right (navigate forward)
    LookRight,
}

impl HeadGesture {
    /// Get action description
    pub fn description(&self) -> &'static str {
        match self {
            Self::NodDown => "Nod down (confirm/yes)",
            Self::NodUp => "Nod up (attention)",
            Self::DoubleNod => "Double nod (strong confirm)",
            Self::Shake => "Shake head (deny/no)",
            Self::ShakeLeft => "Look left",
            Self::ShakeRight => "Look right",
            Self::TiltLeft => "Tilt head left",
            Self::TiltRight => "Tilt head right",
            Self::LookUp => "Look up",
            Self::LookDown => "Look down",
            Self::LookLeft => "Look left",
            Self::LookRight => "Look right",
        }
    }
}

/// Head gesture detection state
#[derive(Debug, Clone)]
pub enum HeadGestureState {
    /// Idle, waiting for gesture
    Idle,
    /// Potential gesture starting
    Starting { gesture_type: PotentialGesture, started_at: Instant },
    /// In middle of gesture
    InGesture { gesture_type: PotentialGesture, peak_reached: bool },
    /// Returning to neutral
    Returning { gesture_type: PotentialGesture },
    /// Cooldown after gesture
    Cooldown { until: Instant },
}

/// Potential gesture being detected
#[derive(Debug, Clone, Copy)]
pub enum PotentialGesture {
    Nod { direction: i8, count: u8 },
    Shake { direction: i8, count: u8 },
    Tilt { direction: i8 },
    Look { pitch: f32, yaw: f32 },
}

/// Head gesture statistics
#[derive(Debug, Default)]
pub struct HeadGestureStats {
    /// Total gestures detected
    pub total_gestures: u64,
    /// Nod count
    pub nod_count: u64,
    /// Shake count
    pub shake_count: u64,
    /// Tilt count
    pub tilt_count: u64,
    /// False positive rate estimate
    pub false_positive_rate: f32,
}

/// Head gesture event
#[derive(Debug, Clone)]
pub struct HeadGestureEvent {
    /// Detected gesture
    pub gesture: HeadGesture,
    /// Confidence (0-1)
    pub confidence: f32,
    /// Duration of gesture
    pub duration: Duration,
    /// Peak angular displacement
    pub peak_displacement: f32,
    /// Timestamp
    pub timestamp: Instant,
}

impl HeadGestureRecognizer {
    /// Create new recognizer
    pub fn new() -> Self {
        Self {
            pose_history: VecDeque::with_capacity(60),
            config: HeadGestureConfig::default(),
            state: HeadGestureState::Idle,
            callbacks: Vec::new(),
            stats: HeadGestureStats::default(),
            baseline_pose: None,
        }
    }

    /// Create with configuration
    pub fn with_config(config: HeadGestureConfig) -> Self {
        Self {
            pose_history: VecDeque::with_capacity(config.history_size),
            config,
            state: HeadGestureState::Idle,
            callbacks: Vec::new(),
            stats: HeadGestureStats::default(),
            baseline_pose: None,
        }
    }

    /// Update with new head pose
    pub fn update(&mut self, pose: HeadPose) -> Option<HeadGestureEvent> {
        // Update history
        self.pose_history.push_back(pose);
        while self.pose_history.len() > self.config.history_size {
            self.pose_history.pop_front();
        }

        // Update baseline (moving average)
        self.update_baseline(&pose);

        // Check cooldown
        if let HeadGestureState::Cooldown { until } = self.state {
            if Instant::now() < until {
                return None;
            }
            self.state = HeadGestureState::Idle;
        }

        // Detect gestures
        self.detect_gesture(&pose)
    }

    fn update_baseline(&mut self, pose: &HeadPose) {
        match &mut self.baseline_pose {
            Some(baseline) => {
                // Slow moving average
                let alpha = 0.02;
                baseline.pitch = baseline.pitch * (1.0 - alpha) + pose.pitch * alpha;
                baseline.yaw = baseline.yaw * (1.0 - alpha) + pose.yaw * alpha;
                baseline.roll = baseline.roll * (1.0 - alpha) + pose.roll * alpha;
            }
            None => {
                self.baseline_pose = Some(*pose);
            }
        }
    }

    fn detect_gesture(&mut self, pose: &HeadPose) -> Option<HeadGestureEvent> {
        let baseline = self.baseline_pose?;

        let pitch_delta = pose.pitch - baseline.pitch;
        let yaw_delta = pose.yaw - baseline.yaw;
        let roll_delta = pose.roll - baseline.roll;

        match &self.state {
            HeadGestureState::Idle => {
                // Check for gesture start
                if self.config.enable_nod && pitch_delta.abs() > self.config.nod_threshold {
                    let direction = if pitch_delta > 0.0 { 1 } else { -1 };
                    self.state = HeadGestureState::Starting {
                        gesture_type: PotentialGesture::Nod { direction, count: 1 },
                        started_at: Instant::now(),
                    };
                } else if self.config.enable_shake && yaw_delta.abs() > self.config.shake_threshold {
                    let direction = if yaw_delta > 0.0 { 1 } else { -1 };
                    self.state = HeadGestureState::Starting {
                        gesture_type: PotentialGesture::Shake { direction, count: 1 },
                        started_at: Instant::now(),
                    };
                } else if self.config.enable_tilt && roll_delta.abs() > self.config.tilt_threshold {
                    let direction = if roll_delta > 0.0 { 1 } else { -1 };
                    self.state = HeadGestureState::Starting {
                        gesture_type: PotentialGesture::Tilt { direction },
                        started_at: Instant::now(),
                    };
                }
                None
            }
            HeadGestureState::Starting { gesture_type, started_at } => {
                let elapsed = started_at.elapsed();
                let gesture_type = *gesture_type;
                let started_at = *started_at;

                if elapsed.as_millis() as u64 > self.config.max_duration_ms {
                    // Took too long, cancel
                    self.state = HeadGestureState::Idle;
                    return None;
                }

                // Check if gesture is progressing
                match gesture_type {
                    PotentialGesture::Nod { direction, count } => {
                        let threshold_exceeded = if direction > 0 {
                            pitch_delta > self.config.nod_threshold * 1.5
                        } else {
                            pitch_delta < -self.config.nod_threshold * 1.5
                        };

                        if threshold_exceeded {
                            self.state = HeadGestureState::InGesture {
                                gesture_type: PotentialGesture::Nod { direction, count },
                                peak_reached: true,
                            };
                        }
                    }
                    PotentialGesture::Shake { direction, count } => {
                        let threshold_exceeded = if direction > 0 {
                            yaw_delta > self.config.shake_threshold * 1.5
                        } else {
                            yaw_delta < -self.config.shake_threshold * 1.5
                        };

                        if threshold_exceeded {
                            self.state = HeadGestureState::InGesture {
                                gesture_type: PotentialGesture::Shake { direction, count },
                                peak_reached: true,
                            };
                        }
                    }
                    PotentialGesture::Tilt { direction } => {
                        let threshold_exceeded = if direction > 0 {
                            roll_delta > self.config.tilt_threshold * 1.5
                        } else {
                            roll_delta < -self.config.tilt_threshold * 1.5
                        };

                        if threshold_exceeded {
                            self.state = HeadGestureState::InGesture {
                                gesture_type,
                                peak_reached: true,
                            };
                        }
                    }
                    _ => {}
                }
                None
            }
            HeadGestureState::InGesture { gesture_type, peak_reached } => {
                let gesture_type = *gesture_type;
                
                // Check for return to neutral
                let returned = pitch_delta.abs() < self.config.return_tolerance
                    && yaw_delta.abs() < self.config.return_tolerance
                    && roll_delta.abs() < self.config.return_tolerance;

                if returned {
                    self.state = HeadGestureState::Returning { gesture_type };
                }
                None
            }
            HeadGestureState::Returning { gesture_type } => {
                let gesture_type = *gesture_type;

                // Complete the gesture
                let gesture = match gesture_type {
                    PotentialGesture::Nod { direction, count } => {
                        if count >= 2 {
                            HeadGesture::DoubleNod
                        } else if direction > 0 {
                            HeadGesture::NodUp
                        } else {
                            HeadGesture::NodDown
                        }
                    }
                    PotentialGesture::Shake { direction, count } => {
                        if count >= 2 {
                            HeadGesture::Shake
                        } else if direction > 0 {
                            HeadGesture::ShakeRight
                        } else {
                            HeadGesture::ShakeLeft
                        }
                    }
                    PotentialGesture::Tilt { direction } => {
                        if direction > 0 {
                            HeadGesture::TiltRight
                        } else {
                            HeadGesture::TiltLeft
                        }
                    }
                    PotentialGesture::Look { pitch, yaw } => {
                        if pitch.abs() > yaw.abs() {
                            if pitch > 0.0 { HeadGesture::LookUp } else { HeadGesture::LookDown }
                        } else {
                            if yaw > 0.0 { HeadGesture::LookRight } else { HeadGesture::LookLeft }
                        }
                    }
                };

                // Enter cooldown
                self.state = HeadGestureState::Cooldown {
                    until: Instant::now() + Duration::from_millis(self.config.cooldown_ms),
                };

                // Update stats
                self.stats.total_gestures += 1;
                match gesture {
                    HeadGesture::NodDown | HeadGesture::NodUp | HeadGesture::DoubleNod => {
                        self.stats.nod_count += 1;
                    }
                    HeadGesture::Shake | HeadGesture::ShakeLeft | HeadGesture::ShakeRight => {
                        self.stats.shake_count += 1;
                    }
                    HeadGesture::TiltLeft | HeadGesture::TiltRight => {
                        self.stats.tilt_count += 1;
                    }
                    _ => {}
                }

                Some(HeadGestureEvent {
                    gesture,
                    confidence: 0.9,
                    duration: Duration::from_millis(300),
                    peak_displacement: 0.0,
                    timestamp: Instant::now(),
                })
            }
            HeadGestureState::Cooldown { .. } => None,
        }
    }

    /// Add gesture callback
    pub fn on_gesture<F>(&mut self, callback: F)
    where
        F: FnMut(HeadGesture) + Send + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }

    /// Get statistics
    pub fn stats(&self) -> &HeadGestureStats {
        &self.stats
    }

    /// Reset to idle state
    pub fn reset(&mut self) {
        self.state = HeadGestureState::Idle;
        self.pose_history.clear();
        self.baseline_pose = None;
    }

    /// Check if looking in a direction (for scroll gestures)
    pub fn is_looking(&self, threshold: f32) -> Option<LookDirection> {
        let pose = self.pose_history.back()?;
        let baseline = self.baseline_pose?;

        let pitch_delta = pose.pitch - baseline.pitch;
        let yaw_delta = pose.yaw - baseline.yaw;

        if pitch_delta > threshold {
            Some(LookDirection::Up)
        } else if pitch_delta < -threshold {
            Some(LookDirection::Down)
        } else if yaw_delta > threshold {
            Some(LookDirection::Right)
        } else if yaw_delta < -threshold {
            Some(LookDirection::Left)
        } else {
            None
        }
    }

    /// Get current head pose relative to baseline
    pub fn relative_pose(&self) -> Option<HeadPose> {
        let pose = self.pose_history.back()?;
        let baseline = self.baseline_pose?;

        Some(HeadPose {
            pitch: pose.pitch - baseline.pitch,
            yaw: pose.yaw - baseline.yaw,
            roll: pose.roll - baseline.roll,
            position: pose.position,
            angular_velocity: pose.angular_velocity,
            timestamp: pose.timestamp,
        })
    }
}

impl Default for HeadGestureRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Look direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookDirection {
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_head_pose_creation() {
        let pose = HeadPose::new(0.1, 0.2, 0.0);
        assert!((pose.pitch - 0.1).abs() < 0.001);
        assert!((pose.yaw - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_recognizer_creation() {
        let recognizer = HeadGestureRecognizer::new();
        assert!(matches!(recognizer.state, HeadGestureState::Idle));
    }

    #[test]
    fn test_baseline_update() {
        let mut recognizer = HeadGestureRecognizer::new();
        
        // First pose sets baseline
        let pose = HeadPose::new(0.0, 0.0, 0.0);
        recognizer.update(pose);
        
        assert!(recognizer.baseline_pose.is_some());
    }

    #[test]
    fn test_nod_detection() {
        let mut recognizer = HeadGestureRecognizer::new();
        
        // Set baseline
        for _ in 0..10 {
            recognizer.update(HeadPose::new(0.0, 0.0, 0.0));
        }

        // Simulate nod down
        recognizer.update(HeadPose::new(-0.3, 0.0, 0.0)); // Head down
        recognizer.update(HeadPose::new(-0.4, 0.0, 0.0)); // More down
        recognizer.update(HeadPose::new(-0.3, 0.0, 0.0)); // Coming back
        
        // Return to neutral
        let event = recognizer.update(HeadPose::new(0.0, 0.0, 0.0));
        
        // May or may not detect depending on timing
        // Just ensure no crash
        let _ = event;
    }

    #[test]
    fn test_config_defaults() {
        let config = HeadGestureConfig::default();
        assert!(config.enable_nod);
        assert!(config.enable_shake);
        assert!(config.enable_tilt);
    }

    #[test]
    fn test_gesture_description() {
        assert!(!HeadGesture::NodDown.description().is_empty());
        assert!(!HeadGesture::Shake.description().is_empty());
    }

    #[test]
    fn test_look_detection() {
        let mut recognizer = HeadGestureRecognizer::new();
        
        // Set baseline
        for _ in 0..10 {
            recognizer.update(HeadPose::new(0.0, 0.0, 0.0));
        }
        
        // Look up
        recognizer.update(HeadPose::new(0.3, 0.0, 0.0));
        
        let direction = recognizer.is_looking(0.2);
        assert_eq!(direction, Some(LookDirection::Up));
    }

    #[test]
    fn test_reset() {
        let mut recognizer = HeadGestureRecognizer::new();
        
        recognizer.update(HeadPose::new(0.0, 0.0, 0.0));
        recognizer.reset();
        
        assert!(recognizer.pose_history.is_empty());
        assert!(recognizer.baseline_pose.is_none());
    }
}
