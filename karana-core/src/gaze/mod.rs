//! # Kāraṇa Gaze Tracking System
//!
//! Advanced eye-tracking and gaze-based interaction for AR interfaces.
//!
//! ## Features
//! - Eye position tracking (pupil center detection)
//! - Gaze ray casting in 3D space
//! - Dwell selection (look-to-select)
//! - Blink detection (single, double, long)
//! - Saccade and fixation analysis
//! - Attention heatmaps
//! - Calibration system

pub mod tracker;
pub mod calibration;
pub mod analysis;
pub mod interaction;

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Maximum number of gaze samples to keep in history
pub const GAZE_HISTORY_SIZE: usize = 120;  // 2 seconds at 60Hz

/// Default dwell time for selection (ms)
pub const DEFAULT_DWELL_MS: u32 = 500;

/// Minimum blink duration (ms)
pub const MIN_BLINK_MS: u32 = 50;

/// Maximum blink duration (ms) - longer is a prolonged close
pub const MAX_BLINK_MS: u32 = 400;

/// Double blink max interval (ms)
pub const DOUBLE_BLINK_INTERVAL_MS: u32 = 500;

/// Eye identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Eye {
    Left,
    Right,
}

/// 2D gaze point on the display (normalized 0-1)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct GazePoint {
    /// X coordinate (0 = left, 1 = right)
    pub x: f32,
    /// Y coordinate (0 = top, 1 = bottom)
    pub y: f32,
    /// Confidence (0-1)
    pub confidence: f32,
    /// Timestamp
    pub timestamp: u64,
}

impl GazePoint {
    pub fn new(x: f32, y: f32, confidence: f32) -> Self {
        Self {
            x,
            y,
            confidence,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
    
    /// Distance to another gaze point
    pub fn distance_to(&self, other: &GazePoint) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// 3D gaze ray in world space
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct GazeRay {
    /// Ray origin (eye position)
    pub origin: [f32; 3],
    /// Ray direction (normalized)
    pub direction: [f32; 3],
    /// Confidence
    pub confidence: f32,
}

impl GazeRay {
    pub fn new(origin: [f32; 3], direction: [f32; 3]) -> Self {
        // Normalize direction
        let len = (direction[0].powi(2) + direction[1].powi(2) + direction[2].powi(2)).sqrt();
        let dir = if len > 0.0 {
            [direction[0] / len, direction[1] / len, direction[2] / len]
        } else {
            [0.0, 0.0, -1.0]  // Default forward
        };
        
        Self {
            origin,
            direction: dir,
            confidence: 1.0,
        }
    }
    
    /// Get point along ray at distance t
    pub fn point_at(&self, t: f32) -> [f32; 3] {
        [
            self.origin[0] + self.direction[0] * t,
            self.origin[1] + self.direction[1] * t,
            self.origin[2] + self.direction[2] * t,
        ]
    }
    
    /// Intersect with a plane (returns t parameter, None if parallel)
    pub fn intersect_plane(&self, plane_normal: [f32; 3], plane_d: f32) -> Option<f32> {
        let denom = plane_normal[0] * self.direction[0]
            + plane_normal[1] * self.direction[1]
            + plane_normal[2] * self.direction[2];
            
        if denom.abs() < 1e-6 {
            return None;  // Parallel
        }
        
        let num = -(plane_normal[0] * self.origin[0]
            + plane_normal[1] * self.origin[1]
            + plane_normal[2] * self.origin[2]
            + plane_d);
            
        Some(num / denom)
    }
}

/// Eye state (open/closed/blinking)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EyeState {
    Open,
    Closing,
    Closed,
    Opening,
}

impl Default for EyeState {
    fn default() -> Self {
        Self::Open
    }
}

/// Blink type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlinkType {
    Single,
    Double,
    Long,  // Prolonged close (> 400ms)
}

/// Blink event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlinkEvent {
    pub blink_type: BlinkType,
    pub eye: Option<Eye>,  // None = both eyes
    pub duration_ms: u32,
    pub timestamp: u64,
}

/// Eye openness measurement
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct EyeOpenness {
    /// Left eye openness (0 = closed, 1 = fully open)
    pub left: f32,
    /// Right eye openness
    pub right: f32,
}

impl EyeOpenness {
    pub fn average(&self) -> f32 {
        (self.left + self.right) / 2.0
    }
    
    pub fn is_closed(&self) -> bool {
        self.average() < 0.2
    }
    
    pub fn is_open(&self) -> bool {
        self.average() > 0.6
    }
}

/// Pupil data from eye tracker
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PupilData {
    /// Pupil center X (image coordinates)
    pub x: f32,
    /// Pupil center Y
    pub y: f32,
    /// Pupil diameter (mm)
    pub diameter: f32,
    /// Confidence
    pub confidence: f32,
}

/// Single eye tracking frame
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EyeFrame {
    /// Left eye pupil
    pub left_pupil: Option<PupilData>,
    /// Right eye pupil
    pub right_pupil: Option<PupilData>,
    /// Eye openness
    pub openness: EyeOpenness,
    /// Gaze point (combined)
    pub gaze_point: GazePoint,
    /// 3D gaze ray
    pub gaze_ray: GazeRay,
    /// Frame timestamp
    pub timestamp: u64,
}

/// Fixation (sustained gaze at a point)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fixation {
    /// Center point
    pub center: GazePoint,
    /// Start time
    pub start_ms: u64,
    /// Duration
    pub duration_ms: u32,
    /// Number of samples
    pub sample_count: u32,
    /// Dispersion (spread of points)
    pub dispersion: f32,
}

impl Fixation {
    pub fn is_dwell(&self, threshold_ms: u32) -> bool {
        self.duration_ms >= threshold_ms
    }
}

/// Saccade (rapid eye movement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Saccade {
    /// Start point
    pub start: GazePoint,
    /// End point
    pub end: GazePoint,
    /// Duration (ms)
    pub duration_ms: u32,
    /// Peak velocity (degrees/sec)
    pub peak_velocity: f32,
    /// Amplitude (degrees)
    pub amplitude: f32,
}

/// Gaze event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GazeEvent {
    /// Fixation started
    FixationStart(Fixation),
    /// Fixation updated (ongoing)
    FixationUpdate(Fixation),
    /// Fixation ended
    FixationEnd(Fixation),
    /// Dwell selection triggered
    Dwell { point: GazePoint, duration_ms: u32 },
    /// Blink detected
    Blink(BlinkEvent),
    /// Saccade detected
    Saccade(Saccade),
    /// Gaze entered a region
    EnterRegion { region_id: String, point: GazePoint },
    /// Gaze exited a region
    ExitRegion { region_id: String },
}

/// Configuration for gaze tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazeConfig {
    /// Enable gaze tracking
    pub enabled: bool,
    /// Dwell time for selection (ms)
    pub dwell_time_ms: u32,
    /// Fixation dispersion threshold (normalized)
    pub fixation_dispersion: f32,
    /// Minimum fixation duration (ms)
    pub min_fixation_ms: u32,
    /// Saccade velocity threshold (deg/s)
    pub saccade_velocity_threshold: f32,
    /// Smoothing factor (0 = none, 1 = max)
    pub smoothing: f32,
    /// Blink detection enabled
    pub detect_blinks: bool,
    /// Double blink detection
    pub detect_double_blinks: bool,
}

impl Default for GazeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dwell_time_ms: DEFAULT_DWELL_MS,
            fixation_dispersion: 0.03,  // 3% of screen
            min_fixation_ms: 100,
            saccade_velocity_threshold: 30.0,  // degrees per second
            smoothing: 0.3,
            detect_blinks: true,
            detect_double_blinks: true,
        }
    }
}

/// Region of interest for gaze detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazeRegion {
    /// Unique identifier
    pub id: String,
    /// Bounding box (x, y, width, height) - normalized
    pub bounds: (f32, f32, f32, f32),
    /// Is gaze currently in this region
    pub is_focused: bool,
    /// Total focus time (ms)
    pub focus_time_ms: u32,
    /// Number of visits
    pub visit_count: u32,
}

impl GazeRegion {
    pub fn new(id: impl Into<String>, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id: id.into(),
            bounds: (x, y, width, height),
            is_focused: false,
            focus_time_ms: 0,
            visit_count: 0,
        }
    }
    
    pub fn contains(&self, point: &GazePoint) -> bool {
        let (x, y, w, h) = self.bounds;
        point.x >= x && point.x <= x + w && point.y >= y && point.y <= y + h
    }
}

/// Gaze statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GazeStats {
    /// Total tracking time (ms)
    pub total_time_ms: u64,
    /// Number of fixations
    pub fixation_count: u32,
    /// Average fixation duration (ms)
    pub avg_fixation_duration_ms: f32,
    /// Number of saccades
    pub saccade_count: u32,
    /// Average saccade amplitude (degrees)
    pub avg_saccade_amplitude: f32,
    /// Number of blinks
    pub blink_count: u32,
    /// Blink rate (blinks per minute)
    pub blink_rate: f32,
}

/// Main gaze tracking engine
pub struct GazeEngine {
    config: GazeConfig,
    
    /// Eye tracking hardware interface
    tracker: tracker::EyeTracker,
    
    /// Calibration data
    calibration: calibration::GazeCalibration,
    
    /// Gaze analyzer
    analyzer: analysis::GazeAnalyzer,
    
    /// Interaction manager
    interaction: interaction::GazeInteraction,
    
    /// Recent gaze history
    history: VecDeque<EyeFrame>,
    
    /// Registered regions
    regions: HashMap<String, GazeRegion>,
    
    /// Current fixation (if any)
    current_fixation: Option<Fixation>,
    
    /// Last blink time (for double-blink detection)
    last_blink_time: Option<Instant>,
    
    /// Statistics
    stats: GazeStats,
    
    /// Event queue
    event_queue: VecDeque<GazeEvent>,
    
    /// Start time
    start_time: Instant,
}

impl GazeEngine {
    pub fn new(config: GazeConfig) -> Self {
        Self {
            tracker: tracker::EyeTracker::new(),
            calibration: calibration::GazeCalibration::default(),
            analyzer: analysis::GazeAnalyzer::new(config.clone()),
            interaction: interaction::GazeInteraction::new(config.clone()),
            history: VecDeque::with_capacity(GAZE_HISTORY_SIZE),
            regions: HashMap::new(),
            current_fixation: None,
            last_blink_time: None,
            stats: GazeStats::default(),
            event_queue: VecDeque::new(),
            start_time: Instant::now(),
            config,
        }
    }
    
    /// Process a new eye frame
    pub fn process_frame(&mut self, frame: EyeFrame) {
        // Add to history
        if self.history.len() >= GAZE_HISTORY_SIZE {
            self.history.pop_front();
        }
        self.history.push_back(frame.clone());
        
        // Apply calibration
        let calibrated_point = self.calibration.apply(&frame.gaze_point);
        
        // Smooth gaze point
        let smoothed = self.smooth_gaze(&calibrated_point);
        
        // Check blinks
        if self.config.detect_blinks {
            self.detect_blinks(&frame);
        }
        
        // Analyze for fixations/saccades
        if let Some(event) = self.analyzer.analyze(&smoothed, frame.timestamp) {
            self.event_queue.push_back(event);
        }
        
        // Check regions
        self.check_regions(&smoothed);
        
        // Update interaction (dwell, etc.)
        if let Some(event) = self.interaction.update(&smoothed, frame.timestamp) {
            self.event_queue.push_back(event);
        }
        
        // Update stats
        self.stats.total_time_ms = self.start_time.elapsed().as_millis() as u64;
    }
    
    /// Smooth gaze point using history
    fn smooth_gaze(&self, point: &GazePoint) -> GazePoint {
        if self.config.smoothing <= 0.0 || self.history.is_empty() {
            return *point;
        }
        
        let alpha = self.config.smoothing.min(1.0);
        let window = (alpha * 10.0) as usize + 1;
        
        let recent: Vec<_> = self.history.iter().rev().take(window).collect();
        if recent.is_empty() {
            return *point;
        }
        
        let mut x_sum = point.x;
        let mut y_sum = point.y;
        let mut weight_sum = 1.0;
        
        for (i, frame) in recent.iter().enumerate() {
            let weight = 1.0 / (i + 2) as f32;
            x_sum += frame.gaze_point.x * weight;
            y_sum += frame.gaze_point.y * weight;
            weight_sum += weight;
        }
        
        GazePoint::new(
            x_sum / weight_sum,
            y_sum / weight_sum,
            point.confidence,
        )
    }
    
    /// Detect blinks from eye openness
    fn detect_blinks(&mut self, frame: &EyeFrame) {
        let openness = frame.openness.average();
        
        // Check for closed eyes
        if openness < 0.2 {
            // Eyes closed - check if this is start of blink
            if self.analyzer.eye_state == EyeState::Open {
                self.analyzer.eye_state = EyeState::Closing;
                self.analyzer.blink_start = Some(Instant::now());
            }
        } else if openness > 0.6 {
            // Eyes open - check if this ends a blink
            if self.analyzer.eye_state == EyeState::Closing || self.analyzer.eye_state == EyeState::Closed {
                if let Some(start) = self.analyzer.blink_start.take() {
                    let duration_ms = start.elapsed().as_millis() as u32;
                    
                    if duration_ms >= MIN_BLINK_MS {
                        let blink_type = if duration_ms > MAX_BLINK_MS {
                            BlinkType::Long
                        } else if self.config.detect_double_blinks {
                            // Check for double blink
                            if let Some(last) = self.last_blink_time {
                                if last.elapsed().as_millis() as u32 <= DOUBLE_BLINK_INTERVAL_MS {
                                    self.last_blink_time = None;
                                    BlinkType::Double
                                } else {
                                    self.last_blink_time = Some(Instant::now());
                                    BlinkType::Single
                                }
                            } else {
                                self.last_blink_time = Some(Instant::now());
                                BlinkType::Single
                            }
                        } else {
                            BlinkType::Single
                        };
                        
                        let event = BlinkEvent {
                            blink_type,
                            eye: None,
                            duration_ms,
                            timestamp: frame.timestamp,
                        };
                        
                        self.stats.blink_count += 1;
                        self.event_queue.push_back(GazeEvent::Blink(event));
                    }
                }
                self.analyzer.eye_state = EyeState::Open;
            }
        }
    }
    
    /// Check if gaze is in any registered region
    fn check_regions(&mut self, point: &GazePoint) {
        for (id, region) in self.regions.iter_mut() {
            let was_focused = region.is_focused;
            let is_focused = region.contains(point);
            
            if is_focused && !was_focused {
                region.is_focused = true;
                region.visit_count += 1;
                self.event_queue.push_back(GazeEvent::EnterRegion {
                    region_id: id.clone(),
                    point: *point,
                });
            } else if !is_focused && was_focused {
                region.is_focused = false;
                self.event_queue.push_back(GazeEvent::ExitRegion {
                    region_id: id.clone(),
                });
            }
        }
    }
    
    /// Register a region for gaze detection
    pub fn register_region(&mut self, region: GazeRegion) {
        self.regions.insert(region.id.clone(), region);
    }
    
    /// Unregister a region
    pub fn unregister_region(&mut self, id: &str) {
        self.regions.remove(id);
    }
    
    /// Get current gaze point
    pub fn current_gaze(&self) -> Option<&GazePoint> {
        self.history.back().map(|f| &f.gaze_point)
    }
    
    /// Get current 3D gaze ray
    pub fn current_ray(&self) -> Option<&GazeRay> {
        self.history.back().map(|f| &f.gaze_ray)
    }
    
    /// Poll for events
    pub fn poll_event(&mut self) -> Option<GazeEvent> {
        self.event_queue.pop_front()
    }
    
    /// Get all pending events
    pub fn drain_events(&mut self) -> Vec<GazeEvent> {
        self.event_queue.drain(..).collect()
    }
    
    /// Get statistics
    pub fn stats(&self) -> &GazeStats {
        &self.stats
    }
    
    /// Check if calibrated
    pub fn is_calibrated(&self) -> bool {
        self.calibration.is_complete()
    }
    
    /// Start calibration process
    pub fn start_calibration(&mut self) {
        self.calibration.start();
    }
    
    /// Record calibration point
    pub fn record_calibration_point(&mut self, target: GazePoint, measured: GazePoint) {
        self.calibration.add_point(target, measured);
    }
    
    /// Finish calibration
    pub fn finish_calibration(&mut self) -> bool {
        self.calibration.compute()
    }
    
    /// Get gaze history
    pub fn history(&self) -> &VecDeque<EyeFrame> {
        &self.history
    }
    
    /// Update configuration
    pub fn set_config(&mut self, config: GazeConfig) {
        self.config = config.clone();
        self.analyzer = analysis::GazeAnalyzer::new(config.clone());
        self.interaction = interaction::GazeInteraction::new(config);
    }
    
    /// Get current configuration
    pub fn config(&self) -> &GazeConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gaze_point() {
        let p1 = GazePoint::new(0.0, 0.0, 1.0);
        let p2 = GazePoint::new(1.0, 0.0, 1.0);
        
        assert!((p1.distance_to(&p2) - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_gaze_ray() {
        let ray = GazeRay::new([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]);
        
        let point = ray.point_at(5.0);
        assert!((point[2] - (-5.0)).abs() < 0.001);
    }
    
    #[test]
    fn test_gaze_ray_plane_intersection() {
        let ray = GazeRay::new([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]);
        
        // Plane at z = -2 (normal pointing +z)
        let t = ray.intersect_plane([0.0, 0.0, 1.0], 2.0);
        assert!(t.is_some());
        assert!((t.unwrap() - 2.0).abs() < 0.001);
    }
    
    #[test]
    fn test_gaze_region() {
        let region = GazeRegion::new("test", 0.0, 0.0, 0.5, 0.5);
        
        let inside = GazePoint::new(0.25, 0.25, 1.0);
        let outside = GazePoint::new(0.75, 0.75, 1.0);
        
        assert!(region.contains(&inside));
        assert!(!region.contains(&outside));
    }
    
    #[test]
    fn test_eye_openness() {
        let open = EyeOpenness { left: 0.9, right: 0.9 };
        let closed = EyeOpenness { left: 0.1, right: 0.1 };
        
        assert!(open.is_open());
        assert!(!open.is_closed());
        assert!(closed.is_closed());
        assert!(!closed.is_open());
    }
    
    #[test]
    fn test_gaze_config_default() {
        let config = GazeConfig::default();
        
        assert!(config.enabled);
        assert_eq!(config.dwell_time_ms, 500);
        assert!(config.detect_blinks);
    }
    
    #[test]
    fn test_gaze_engine_creation() {
        let engine = GazeEngine::new(GazeConfig::default());
        
        assert!(engine.current_gaze().is_none());
        assert!(!engine.is_calibrated());
    }
    
    #[test]
    fn test_fixation_dwell() {
        let fixation = Fixation {
            center: GazePoint::new(0.5, 0.5, 1.0),
            start_ms: 0,
            duration_ms: 600,
            sample_count: 36,
            dispersion: 0.02,
        };
        
        assert!(fixation.is_dwell(500));
        assert!(!fixation.is_dwell(700));
    }
}
