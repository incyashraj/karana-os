//! Posture Monitoring for Kāraṇa OS AR Glasses
//!
//! Tracks head/neck position and detects poor posture patterns.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use nalgebra::Vector3;
use super::{HealthAlert, AlertType};

/// Posture state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostureState {
    /// Good posture
    Good,
    /// Slightly forward
    SlightlyForward,
    /// Significantly forward (neck strain risk)
    Forward,
    /// Tilted to side
    Tilted,
    /// Looking down too much
    LookingDown,
    /// Looking up too much
    LookingUp,
    /// Unknown (insufficient data)
    Unknown,
}

impl PostureState {
    /// Is this considered good posture?
    pub fn is_good(&self) -> bool {
        matches!(self, PostureState::Good | PostureState::Unknown)
    }
    
    /// Get description
    pub fn description(&self) -> &str {
        match self {
            PostureState::Good => "Good posture maintained",
            PostureState::SlightlyForward => "Head slightly forward",
            PostureState::Forward => "Head too far forward - neck strain risk",
            PostureState::Tilted => "Head tilted to one side",
            PostureState::LookingDown => "Looking down too much",
            PostureState::LookingUp => "Looking up for extended period",
            PostureState::Unknown => "Posture unknown",
        }
    }
}

/// Neck position data
#[derive(Debug, Clone)]
pub struct NeckPosition {
    /// Forward angle (0 = neutral, positive = forward)
    pub forward_angle: f32,
    /// Side tilt angle (0 = neutral, positive = right tilt)
    pub tilt_angle: f32,
    /// Rotation angle (0 = forward, positive = right)
    pub rotation_angle: f32,
    /// Timestamp
    pub timestamp: Instant,
}

impl NeckPosition {
    /// Threshold for forward lean (degrees)
    const FORWARD_THRESHOLD: f32 = 15.0;
    /// Threshold for side tilt (degrees)
    const TILT_THRESHOLD: f32 = 10.0;
    /// Threshold for extended looking down (degrees)
    const DOWN_THRESHOLD: f32 = 30.0;
    /// Threshold for extended looking up (degrees)
    const UP_THRESHOLD: f32 = 20.0;
    
    /// Evaluate posture state
    pub fn evaluate(&self) -> PostureState {
        if self.forward_angle.abs() < Self::FORWARD_THRESHOLD * 0.5 &&
           self.tilt_angle.abs() < Self::TILT_THRESHOLD {
            return PostureState::Good;
        }
        
        if self.forward_angle > Self::FORWARD_THRESHOLD {
            return PostureState::Forward;
        }
        
        if self.forward_angle > Self::FORWARD_THRESHOLD * 0.5 {
            return PostureState::SlightlyForward;
        }
        
        if self.forward_angle < -Self::UP_THRESHOLD {
            return PostureState::LookingUp;
        }
        
        if self.forward_angle > Self::DOWN_THRESHOLD {
            return PostureState::LookingDown;
        }
        
        if self.tilt_angle.abs() > Self::TILT_THRESHOLD {
            return PostureState::Tilted;
        }
        
        PostureState::Good
    }
}

/// Posture monitor
#[derive(Debug)]
pub struct PostureMonitor {
    /// Recent position history
    history: VecDeque<NeckPosition>,
    /// History window duration
    window: Duration,
    /// Current posture state
    current_state: PostureState,
    /// Duration of current state
    state_duration: Duration,
    /// Bad posture threshold (continuous bad posture time before alert)
    bad_posture_threshold: Duration,
    /// Calibration offset
    calibration_offset: Vector3<f32>,
    /// Is calibrated
    is_calibrated: bool,
    /// Good posture time
    good_posture_time: Duration,
    /// Bad posture time
    bad_posture_time: Duration,
    /// Session start
    session_start: Instant,
}

impl PostureMonitor {
    /// Create new posture monitor
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            window: Duration::from_secs(30),
            current_state: PostureState::Unknown,
            state_duration: Duration::ZERO,
            bad_posture_threshold: Duration::from_secs(60),
            calibration_offset: Vector3::zeros(),
            is_calibrated: false,
            good_posture_time: Duration::ZERO,
            bad_posture_time: Duration::ZERO,
            session_start: Instant::now(),
        }
    }
    
    /// Calibrate neutral position
    pub fn calibrate(&mut self, current_rotation: Vector3<f32>) {
        self.calibration_offset = current_rotation;
        self.is_calibrated = true;
    }
    
    /// Record head position
    pub fn record_position(&mut self, _position: Vector3<f32>, rotation: Vector3<f32>) {
        // Apply calibration offset
        let adjusted = rotation - self.calibration_offset;
        
        let neck_position = NeckPosition {
            forward_angle: adjusted.x.to_degrees(), // Pitch
            tilt_angle: adjusted.z.to_degrees(),    // Roll
            rotation_angle: adjusted.y.to_degrees(), // Yaw
            timestamp: Instant::now(),
        };
        
        self.history.push_back(neck_position);
        
        // Clean old data
        let cutoff = Instant::now() - self.window;
        while self.history.front().map(|p| p.timestamp < cutoff).unwrap_or(false) {
            self.history.pop_front();
        }
    }
    
    /// Get current posture state
    pub fn current_state(&self) -> PostureState {
        self.current_state
    }
    
    /// Is current posture good?
    pub fn is_good_posture(&self) -> bool {
        self.current_state.is_good()
    }
    
    /// Calculate average posture over window
    fn calculate_average_posture(&self) -> PostureState {
        if self.history.is_empty() {
            return PostureState::Unknown;
        }
        
        // Average the angles
        let count = self.history.len() as f32;
        let avg_forward: f32 = self.history.iter()
            .map(|p| p.forward_angle)
            .sum::<f32>() / count;
        let avg_tilt: f32 = self.history.iter()
            .map(|p| p.tilt_angle)
            .sum::<f32>() / count;
        
        let avg_position = NeckPosition {
            forward_angle: avg_forward,
            tilt_angle: avg_tilt,
            rotation_angle: 0.0,
            timestamp: Instant::now(),
        };
        
        avg_position.evaluate()
    }
    
    /// Get good posture percentage
    pub fn good_posture_percentage(&self) -> f32 {
        let total = self.good_posture_time + self.bad_posture_time;
        if total.as_secs() == 0 {
            return 100.0;
        }
        
        (self.good_posture_time.as_secs_f32() / total.as_secs_f32()) * 100.0
    }
    
    /// Update monitor
    pub fn update(&mut self, delta: Duration) -> Option<HealthAlert> {
        let new_state = self.calculate_average_posture();
        
        // Track good vs bad posture time
        if new_state.is_good() {
            self.good_posture_time += delta;
        } else {
            self.bad_posture_time += delta;
        }
        
        // Track state duration
        if new_state != self.current_state {
            self.current_state = new_state;
            self.state_duration = Duration::ZERO;
        } else {
            self.state_duration += delta;
        }
        
        // Generate alert for sustained bad posture
        if !new_state.is_good() && self.state_duration >= self.bad_posture_threshold {
            let alert_type = match new_state {
                PostureState::Forward | PostureState::SlightlyForward => AlertType::NeckStrain,
                _ => AlertType::PoorPosture,
            };
            
            return Some(HealthAlert::new(
                alert_type,
                format!("Posture alert: {}", new_state.description()),
            ));
        }
        
        None
    }
    
    /// Get latest neck position
    pub fn latest_position(&self) -> Option<&NeckPosition> {
        self.history.back()
    }
    
    /// Reset session
    pub fn reset_session(&mut self) {
        self.history.clear();
        self.good_posture_time = Duration::ZERO;
        self.bad_posture_time = Duration::ZERO;
        self.current_state = PostureState::Unknown;
        self.session_start = Instant::now();
    }
}

impl Default for PostureMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_posture_monitor_creation() {
        let monitor = PostureMonitor::new();
        assert_eq!(monitor.current_state(), PostureState::Unknown);
    }
    
    #[test]
    fn test_posture_state_good() {
        let position = NeckPosition {
            forward_angle: 5.0,
            tilt_angle: 3.0,
            rotation_angle: 0.0,
            timestamp: Instant::now(),
        };
        
        assert_eq!(position.evaluate(), PostureState::Good);
    }
    
    #[test]
    fn test_posture_state_forward() {
        let position = NeckPosition {
            forward_angle: 25.0,
            tilt_angle: 0.0,
            rotation_angle: 0.0,
            timestamp: Instant::now(),
        };
        
        assert_eq!(position.evaluate(), PostureState::Forward);
    }
    
    #[test]
    fn test_posture_state_tilted() {
        let position = NeckPosition {
            forward_angle: 0.0,
            tilt_angle: 15.0,
            rotation_angle: 0.0,
            timestamp: Instant::now(),
        };
        
        assert_eq!(position.evaluate(), PostureState::Tilted);
    }
    
    #[test]
    fn test_record_position() {
        let mut monitor = PostureMonitor::new();
        
        monitor.record_position(
            Vector3::new(0.0, 1.7, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
        );
        
        assert!(monitor.latest_position().is_some());
    }
    
    #[test]
    fn test_good_posture_percentage() {
        let monitor = PostureMonitor::new();
        // With no data, should be 100%
        assert_eq!(monitor.good_posture_percentage(), 100.0);
    }
    
    #[test]
    fn test_calibration() {
        let mut monitor = PostureMonitor::new();
        monitor.calibrate(Vector3::new(0.1, 0.0, 0.05));
        
        assert!(monitor.is_calibrated);
    }
}
