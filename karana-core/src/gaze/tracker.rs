//! Eye tracking hardware interface
//!
//! Abstracts different eye tracking hardware and provides a unified interface.

use super::{EyeFrame, GazePoint, GazeRay, PupilData, EyeOpenness, Eye};
use std::time::Instant;

/// Eye tracker status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackerStatus {
    Disconnected,
    Connecting,
    Connected,
    Tracking,
    Error,
}

/// Eye tracker capabilities
#[derive(Debug, Clone, Default)]
pub struct TrackerCapabilities {
    /// Supports 3D gaze tracking
    pub gaze_3d: bool,
    /// Supports pupil diameter
    pub pupil_size: bool,
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// Supports individual eye tracking
    pub binocular: bool,
    /// Supports blink detection
    pub blink_detection: bool,
}

/// Eye tracking hardware interface
pub struct EyeTracker {
    status: TrackerStatus,
    capabilities: TrackerCapabilities,
    last_frame: Option<EyeFrame>,
    frame_count: u64,
    start_time: Instant,
}

impl EyeTracker {
    pub fn new() -> Self {
        Self {
            status: TrackerStatus::Disconnected,
            capabilities: TrackerCapabilities {
                gaze_3d: true,
                pupil_size: true,
                sample_rate: 60,
                binocular: true,
                blink_detection: true,
            },
            last_frame: None,
            frame_count: 0,
            start_time: Instant::now(),
        }
    }
    
    /// Connect to eye tracker
    pub fn connect(&mut self) -> Result<(), TrackerError> {
        self.status = TrackerStatus::Connecting;
        
        // Simulate connection
        self.status = TrackerStatus::Connected;
        Ok(())
    }
    
    /// Disconnect from eye tracker
    pub fn disconnect(&mut self) {
        self.status = TrackerStatus::Disconnected;
    }
    
    /// Start tracking
    pub fn start_tracking(&mut self) -> Result<(), TrackerError> {
        if self.status != TrackerStatus::Connected {
            return Err(TrackerError::NotConnected);
        }
        
        self.status = TrackerStatus::Tracking;
        Ok(())
    }
    
    /// Stop tracking
    pub fn stop_tracking(&mut self) {
        if self.status == TrackerStatus::Tracking {
            self.status = TrackerStatus::Connected;
        }
    }
    
    /// Get current status
    pub fn status(&self) -> TrackerStatus {
        self.status
    }
    
    /// Get capabilities
    pub fn capabilities(&self) -> &TrackerCapabilities {
        &self.capabilities
    }
    
    /// Get latest frame (non-blocking)
    pub fn get_frame(&mut self) -> Option<EyeFrame> {
        if self.status != TrackerStatus::Tracking {
            return None;
        }
        
        // Generate simulated frame
        let frame = self.generate_simulated_frame();
        self.last_frame = Some(frame.clone());
        self.frame_count += 1;
        
        Some(frame)
    }
    
    /// Wait for next frame (blocking with timeout)
    pub fn wait_frame(&mut self, timeout_ms: u32) -> Result<EyeFrame, TrackerError> {
        if self.status != TrackerStatus::Tracking {
            return Err(TrackerError::NotTracking);
        }
        
        // Simulate waiting
        std::thread::sleep(std::time::Duration::from_millis(
            (1000 / self.capabilities.sample_rate) as u64
        ));
        
        self.get_frame().ok_or(TrackerError::Timeout)
    }
    
    /// Generate simulated eye frame for testing
    fn generate_simulated_frame(&self) -> EyeFrame {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        
        // Simulate eye movement with slow drift + micro-saccades
        let base_x = 0.5 + 0.1 * (elapsed * 0.5).sin();
        let base_y = 0.5 + 0.08 * (elapsed * 0.3).cos();
        
        // Add micro-movements
        let noise_x = 0.005 * ((elapsed * 20.0).sin() + (elapsed * 31.0).cos());
        let noise_y = 0.005 * ((elapsed * 23.0).cos() + (elapsed * 29.0).sin());
        
        let gaze_x = (base_x + noise_x).clamp(0.0, 1.0);
        let gaze_y = (base_y + noise_y).clamp(0.0, 1.0);
        
        // Simulate occasional blinks
        let blink_cycle = (elapsed * 0.2) % 1.0;  // Blink every ~5 seconds
        let openness = if blink_cycle > 0.98 || blink_cycle < 0.02 {
            0.1  // Closing/opening
        } else {
            0.95
        };
        
        // Pupil dilation varies slightly
        let pupil_diameter = 4.0 + 0.5 * (elapsed * 0.1).sin();
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        EyeFrame {
            left_pupil: Some(PupilData {
                x: gaze_x - 0.03,
                y: gaze_y,
                diameter: pupil_diameter,
                confidence: 0.95,
            }),
            right_pupil: Some(PupilData {
                x: gaze_x + 0.03,
                y: gaze_y,
                diameter: pupil_diameter,
                confidence: 0.95,
            }),
            openness: EyeOpenness {
                left: openness,
                right: openness,
            },
            gaze_point: GazePoint::new(gaze_x, gaze_y, 0.95),
            gaze_ray: GazeRay::new(
                [0.0, 0.0, 0.0],
                [gaze_x - 0.5, -(gaze_y - 0.5), -1.0],
            ),
            timestamp,
        }
    }
    
    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
    
    /// Set custom sample rate
    pub fn set_sample_rate(&mut self, rate: u32) {
        self.capabilities.sample_rate = rate.clamp(30, 240);
    }
}

impl Default for EyeTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Eye tracker errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackerError {
    NotConnected,
    NotTracking,
    Timeout,
    DeviceError,
    CalibrationFailed,
}

impl std::fmt::Display for TrackerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackerError::NotConnected => write!(f, "Eye tracker not connected"),
            TrackerError::NotTracking => write!(f, "Tracking not started"),
            TrackerError::Timeout => write!(f, "Frame timeout"),
            TrackerError::DeviceError => write!(f, "Device error"),
            TrackerError::CalibrationFailed => write!(f, "Calibration failed"),
        }
    }
}

impl std::error::Error for TrackerError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tracker_creation() {
        let tracker = EyeTracker::new();
        assert_eq!(tracker.status(), TrackerStatus::Disconnected);
    }
    
    #[test]
    fn test_tracker_connection() {
        let mut tracker = EyeTracker::new();
        
        assert!(tracker.connect().is_ok());
        assert_eq!(tracker.status(), TrackerStatus::Connected);
    }
    
    #[test]
    fn test_tracker_start_tracking() {
        let mut tracker = EyeTracker::new();
        tracker.connect().unwrap();
        
        assert!(tracker.start_tracking().is_ok());
        assert_eq!(tracker.status(), TrackerStatus::Tracking);
    }
    
    #[test]
    fn test_get_frame() {
        let mut tracker = EyeTracker::new();
        tracker.connect().unwrap();
        tracker.start_tracking().unwrap();
        
        let frame = tracker.get_frame();
        assert!(frame.is_some());
        
        let frame = frame.unwrap();
        assert!(frame.gaze_point.confidence > 0.0);
    }
    
    #[test]
    fn test_capabilities() {
        let tracker = EyeTracker::new();
        let caps = tracker.capabilities();
        
        assert!(caps.binocular);
        assert!(caps.blink_detection);
        assert_eq!(caps.sample_rate, 60);
    }
}
