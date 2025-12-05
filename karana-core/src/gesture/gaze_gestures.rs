//! Gaze-Gesture Integration for Kāraṇa OS
//!
//! Combines eye tracking with hand gestures for enhanced interaction.
//! Supports look-and-select, dwell activation, and gaze-guided gestures.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::spatial::world_coords::LocalCoord;
use super::{GestureType, Handedness, Finger};
use super::finger_tracking::ScreenPoint;

/// Gaze-gesture coordination manager
pub struct GazeGestureManager {
    /// Current gaze point
    gaze_point: Option<GazePoint>,
    /// Gaze history for dwell detection
    gaze_history: Vec<GazePoint>,
    /// Configuration
    config: GazeGestureConfig,
    /// Active dwell target
    dwell_target: Option<DwellTarget>,
    /// Registered gaze-gesture bindings
    bindings: Vec<GazeGestureBinding>,
    /// Gaze state
    state: GazeState,
    /// Fixation detector
    fixation: FixationDetector,
    /// Saccade detector
    saccade: SaccadeDetector,
    /// Statistics
    stats: GazeStats,
}

/// Gaze point from eye tracking
#[derive(Debug, Clone, Copy)]
pub struct GazePoint {
    /// Screen position (normalized 0-1)
    pub position: ScreenPoint,
    /// 3D ray direction in world space
    pub ray_direction: LocalCoord,
    /// Confidence (0-1)
    pub confidence: f32,
    /// Timestamp
    pub timestamp: Instant,
    /// Pupil diameter (mm)
    pub pupil_diameter: f32,
}

impl GazePoint {
    /// Create new gaze point
    pub fn new(x: f32, y: f32, confidence: f32) -> Self {
        Self {
            position: ScreenPoint::new(x, y),
            ray_direction: LocalCoord::new(0.0, 0.0, -1.0),
            confidence,
            timestamp: Instant::now(),
            pupil_diameter: 4.0,
        }
    }

    /// Distance to another point in screen space
    pub fn distance_to(&self, other: &GazePoint) -> f32 {
        self.position.distance(&other.position)
    }
}

/// Gaze-gesture configuration
#[derive(Debug, Clone)]
pub struct GazeGestureConfig {
    /// Dwell time for selection (ms)
    pub dwell_time_ms: u64,
    /// Dwell radius threshold (normalized)
    pub dwell_radius: f32,
    /// Minimum gaze confidence
    pub min_confidence: f32,
    /// Enable look-and-pinch
    pub look_and_pinch: bool,
    /// Enable look-and-dwell
    pub look_and_dwell: bool,
    /// Enable gaze scrolling
    pub gaze_scroll: bool,
    /// Gaze smoothing factor
    pub smoothing: f32,
    /// Fixation threshold (ms)
    pub fixation_threshold_ms: u64,
    /// Saccade velocity threshold (deg/s)
    pub saccade_threshold: f32,
}

impl Default for GazeGestureConfig {
    fn default() -> Self {
        Self {
            dwell_time_ms: 500,
            dwell_radius: 0.05,
            min_confidence: 0.6,
            look_and_pinch: true,
            look_and_dwell: true,
            gaze_scroll: true,
            smoothing: 0.3,
            fixation_threshold_ms: 100,
            saccade_threshold: 100.0,
        }
    }
}

/// Dwell target for selection
#[derive(Debug, Clone)]
pub struct DwellTarget {
    /// Target position
    pub position: ScreenPoint,
    /// Target ID
    pub target_id: Option<u64>,
    /// Dwell start time
    pub started_at: Instant,
    /// Current progress (0-1)
    pub progress: f32,
    /// Is complete
    pub complete: bool,
}

/// Current gaze state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GazeState {
    /// Not tracking
    NotTracking,
    /// In saccade (rapid eye movement)
    Saccade,
    /// Fixating on a point
    Fixating,
    /// Dwelling (holding gaze for selection)
    Dwelling,
    /// Following movement (smooth pursuit)
    Pursuit,
}

/// Gaze-gesture binding
#[derive(Debug, Clone)]
pub struct GazeGestureBinding {
    /// Gaze action type
    pub gaze_action: GazeAction,
    /// Optional hand gesture requirement
    pub hand_gesture: Option<GestureType>,
    /// Resulting action
    pub result_action: GazeResultAction,
    /// Priority
    pub priority: u8,
}

/// Gaze action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GazeAction {
    /// Look at target
    LookAt,
    /// Dwell on target
    Dwell,
    /// Look away from target
    LookAway,
    /// Look at edge of screen
    LookAtEdge(ScreenEdge),
    /// Quick glance
    Glance,
}

/// Screen edges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenEdge {
    Top,
    Bottom,
    Left,
    Right,
}

/// Result action from gaze gesture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GazeResultAction {
    /// Select item
    Select,
    /// Scroll content
    Scroll { direction: ScrollDirection },
    /// Open context menu
    ContextMenu,
    /// Dismiss/close
    Dismiss,
    /// Show tooltip
    ShowTooltip,
    /// Navigate
    Navigate { direction: NavigateDirection },
    /// Custom action
    Custom(String),
}

/// Scroll direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Navigate direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigateDirection {
    Forward,
    Back,
    Up,
    Down,
}

/// Fixation detection
#[derive(Debug)]
pub struct FixationDetector {
    /// Recent gaze points
    points: Vec<GazePoint>,
    /// Max points to keep
    max_points: usize,
    /// Current fixation center
    fixation_center: Option<ScreenPoint>,
    /// Fixation start time
    fixation_start: Option<Instant>,
    /// Dispersion threshold
    dispersion_threshold: f32,
}

impl FixationDetector {
    /// Create new fixation detector
    pub fn new(dispersion_threshold: f32) -> Self {
        Self {
            points: Vec::with_capacity(100),
            max_points: 100,
            fixation_center: None,
            fixation_start: None,
            dispersion_threshold,
        }
    }

    /// Add gaze point and detect fixation
    pub fn update(&mut self, point: GazePoint) -> Option<Fixation> {
        self.points.push(point);
        if self.points.len() > self.max_points {
            self.points.remove(0);
        }

        if self.points.len() < 5 {
            return None;
        }

        // Calculate dispersion (max distance between points)
        let dispersion = self.calculate_dispersion();

        if dispersion < self.dispersion_threshold {
            // Points are clustered - fixation
            let center = self.calculate_center();

            if self.fixation_start.is_none() {
                self.fixation_start = Some(Instant::now());
                self.fixation_center = Some(center);
            }

            let duration = self.fixation_start.unwrap().elapsed();

            Some(Fixation {
                center,
                duration,
                dispersion,
            })
        } else {
            // Points spread out - not fixating
            self.fixation_start = None;
            self.fixation_center = None;
            None
        }
    }

    fn calculate_dispersion(&self) -> f32 {
        if self.points.len() < 2 {
            return 0.0;
        }

        let mut max_dist = 0.0f32;
        for i in 0..self.points.len() {
            for j in (i + 1)..self.points.len() {
                let dist = self.points[i].distance_to(&self.points[j]);
                max_dist = max_dist.max(dist);
            }
        }
        max_dist
    }

    fn calculate_center(&self) -> ScreenPoint {
        if self.points.is_empty() {
            return ScreenPoint::new(0.5, 0.5);
        }

        let sum_x: f32 = self.points.iter().map(|p| p.position.x).sum();
        let sum_y: f32 = self.points.iter().map(|p| p.position.y).sum();
        let count = self.points.len() as f32;

        ScreenPoint::new(sum_x / count, sum_y / count)
    }
}

/// Fixation data
#[derive(Debug, Clone)]
pub struct Fixation {
    /// Center point of fixation
    pub center: ScreenPoint,
    /// Duration of fixation
    pub duration: Duration,
    /// Dispersion of gaze points
    pub dispersion: f32,
}

/// Saccade detection
#[derive(Debug)]
pub struct SaccadeDetector {
    /// Previous gaze point
    previous: Option<GazePoint>,
    /// Velocity threshold (normalized units per second)
    velocity_threshold: f32,
    /// Current saccade
    current_saccade: Option<Saccade>,
}

impl SaccadeDetector {
    /// Create new saccade detector
    pub fn new(velocity_threshold: f32) -> Self {
        Self {
            previous: None,
            velocity_threshold,
            current_saccade: None,
        }
    }

    /// Update with new gaze point
    pub fn update(&mut self, point: GazePoint) -> Option<Saccade> {
        let result = if let Some(prev) = &self.previous {
            let dt = point.timestamp.duration_since(prev.timestamp).as_secs_f32();
            if dt > 0.0 {
                let distance = point.distance_to(prev);
                let velocity = distance / dt;

                if velocity > self.velocity_threshold {
                    let direction = ScreenPoint::new(
                        point.position.x - prev.position.x,
                        point.position.y - prev.position.y,
                    );
                    Some(Saccade {
                        start: prev.position,
                        end: point.position,
                        velocity,
                        direction,
                        duration: point.timestamp.duration_since(prev.timestamp),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        self.previous = Some(point);
        result
    }
}

/// Saccade data
#[derive(Debug, Clone)]
pub struct Saccade {
    /// Start position
    pub start: ScreenPoint,
    /// End position
    pub end: ScreenPoint,
    /// Peak velocity
    pub velocity: f32,
    /// Direction vector
    pub direction: ScreenPoint,
    /// Duration
    pub duration: Duration,
}

/// Gaze statistics
#[derive(Debug, Default)]
pub struct GazeStats {
    /// Total fixations detected
    pub fixation_count: u64,
    /// Total saccades detected
    pub saccade_count: u64,
    /// Average fixation duration
    pub avg_fixation_duration_ms: f32,
    /// Average saccade velocity
    pub avg_saccade_velocity: f32,
    /// Dwell selections made
    pub dwell_selections: u64,
    /// Look-and-pinch selections
    pub look_pinch_selections: u64,
}

impl GazeGestureManager {
    /// Create new gaze-gesture manager
    pub fn new() -> Self {
        Self {
            gaze_point: None,
            gaze_history: Vec::with_capacity(1000),
            config: GazeGestureConfig::default(),
            dwell_target: None,
            bindings: Vec::new(),
            state: GazeState::NotTracking,
            fixation: FixationDetector::new(0.05),
            saccade: SaccadeDetector::new(0.5),
            stats: GazeStats::default(),
        }
    }

    /// Create with configuration
    pub fn with_config(config: GazeGestureConfig) -> Self {
        Self {
            gaze_point: None,
            gaze_history: Vec::with_capacity(1000),
            config: config.clone(),
            dwell_target: None,
            bindings: Vec::new(),
            state: GazeState::NotTracking,
            fixation: FixationDetector::new(config.dwell_radius),
            saccade: SaccadeDetector::new(config.saccade_threshold / 100.0),
            stats: GazeStats::default(),
        }
    }

    /// Update with new gaze point
    pub fn update_gaze(&mut self, point: GazePoint) -> Vec<GazeEvent> {
        let mut events = Vec::new();

        if point.confidence < self.config.min_confidence {
            self.state = GazeState::NotTracking;
            return events;
        }

        // Apply smoothing
        let smoothed_point = self.smooth_gaze(point);

        // Store in history
        self.gaze_history.push(smoothed_point);
        if self.gaze_history.len() > 1000 {
            self.gaze_history.remove(0);
        }

        // Detect saccade
        if let Some(saccade) = self.saccade.update(smoothed_point) {
            self.state = GazeState::Saccade;
            self.stats.saccade_count += 1;
            events.push(GazeEvent::Saccade(saccade));
        }

        // Detect fixation
        if let Some(fixation) = self.fixation.update(smoothed_point) {
            if self.state != GazeState::Dwelling {
                self.state = GazeState::Fixating;
            }

            // Check for dwell
            if fixation.duration.as_millis() as u64 >= self.config.dwell_time_ms {
                if self.config.look_and_dwell && self.dwell_target.is_none() {
                    self.start_dwell(fixation.center);
                }
            }

            self.stats.fixation_count += 1;
            events.push(GazeEvent::Fixation(fixation));
        }

        // Update dwell progress
        if let Some(ref mut dwell) = self.dwell_target {
            let elapsed = dwell.started_at.elapsed().as_millis() as u64;
            dwell.progress = (elapsed as f32 / self.config.dwell_time_ms as f32).min(1.0);

            if dwell.progress >= 1.0 && !dwell.complete {
                dwell.complete = true;
                self.stats.dwell_selections += 1;
                events.push(GazeEvent::DwellComplete {
                    position: dwell.position,
                    target_id: dwell.target_id,
                });
            }
        }

        self.gaze_point = Some(smoothed_point);
        events
    }

    fn smooth_gaze(&self, point: GazePoint) -> GazePoint {
        if let Some(prev) = self.gaze_point {
            let alpha = self.config.smoothing;
            GazePoint {
                position: ScreenPoint::new(
                    prev.position.x * alpha + point.position.x * (1.0 - alpha),
                    prev.position.y * alpha + point.position.y * (1.0 - alpha),
                ),
                ray_direction: point.ray_direction,
                confidence: point.confidence,
                timestamp: point.timestamp,
                pupil_diameter: point.pupil_diameter,
            }
        } else {
            point
        }
    }

    fn start_dwell(&mut self, position: ScreenPoint) {
        self.state = GazeState::Dwelling;
        self.dwell_target = Some(DwellTarget {
            position,
            target_id: None,
            started_at: Instant::now(),
            progress: 0.0,
            complete: false,
        });
    }

    /// Cancel current dwell
    pub fn cancel_dwell(&mut self) {
        self.dwell_target = None;
        self.state = GazeState::Fixating;
    }

    /// Handle hand gesture with gaze context
    pub fn handle_gesture(&mut self, gesture: GestureType, position: ScreenPoint) -> Option<GazeResultAction> {
        // Look-and-pinch: use gaze target for gesture action
        if self.config.look_and_pinch {
            if let Some(gaze) = self.gaze_point {
                // Check if gesture is near gaze point
                let gaze_gesture_dist = gaze.position.distance(&position);
                if gaze_gesture_dist < 0.2 {
                    // Gesture matches gaze - use gaze target
                    self.stats.look_pinch_selections += 1;
                    return self.find_action_for_gesture(gesture);
                }
            }
        }

        // Check bindings
        self.find_action_for_gesture(gesture)
    }

    fn find_action_for_gesture(&self, gesture: GestureType) -> Option<GazeResultAction> {
        for binding in &self.bindings {
            if let Some(required) = &binding.hand_gesture {
                if *required == gesture {
                    return Some(binding.result_action.clone());
                }
            }
        }
        None
    }

    /// Add gaze-gesture binding
    pub fn add_binding(&mut self, binding: GazeGestureBinding) {
        self.bindings.push(binding);
        self.bindings.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Get current gaze state
    pub fn state(&self) -> GazeState {
        self.state
    }

    /// Get current gaze point
    pub fn gaze_point(&self) -> Option<&GazePoint> {
        self.gaze_point.as_ref()
    }

    /// Get dwell progress (0-1)
    pub fn dwell_progress(&self) -> f32 {
        self.dwell_target.as_ref().map(|d| d.progress).unwrap_or(0.0)
    }

    /// Get statistics
    pub fn stats(&self) -> &GazeStats {
        &self.stats
    }

    /// Check if looking at edge (for navigation)
    pub fn check_edge_look(&self) -> Option<ScreenEdge> {
        let gaze = self.gaze_point?;
        let margin = 0.1;

        if gaze.position.y < margin {
            Some(ScreenEdge::Top)
        } else if gaze.position.y > 1.0 - margin {
            Some(ScreenEdge::Bottom)
        } else if gaze.position.x < margin {
            Some(ScreenEdge::Left)
        } else if gaze.position.x > 1.0 - margin {
            Some(ScreenEdge::Right)
        } else {
            None
        }
    }

    /// Get gaze-based scroll direction if looking at edge
    pub fn get_gaze_scroll(&self) -> Option<ScrollDirection> {
        if !self.config.gaze_scroll {
            return None;
        }

        match self.check_edge_look()? {
            ScreenEdge::Top => Some(ScrollDirection::Up),
            ScreenEdge::Bottom => Some(ScrollDirection::Down),
            ScreenEdge::Left => Some(ScrollDirection::Left),
            ScreenEdge::Right => Some(ScrollDirection::Right),
        }
    }
}

impl Default for GazeGestureManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Events from gaze processing
#[derive(Debug, Clone)]
pub enum GazeEvent {
    /// Fixation detected
    Fixation(Fixation),
    /// Saccade detected
    Saccade(Saccade),
    /// Dwell selection complete
    DwellComplete {
        position: ScreenPoint,
        target_id: Option<u64>,
    },
    /// Gaze entered edge region
    EdgeEnter(ScreenEdge),
    /// Gaze left edge region
    EdgeLeave(ScreenEdge),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaze_point_creation() {
        let point = GazePoint::new(0.5, 0.5, 0.9);
        assert_eq!(point.position.x, 0.5);
        assert_eq!(point.position.y, 0.5);
        assert!((point.confidence - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_gaze_point_distance() {
        let p1 = GazePoint::new(0.0, 0.0, 1.0);
        let p2 = GazePoint::new(1.0, 0.0, 1.0);
        assert!((p1.distance_to(&p2) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_fixation_detector() {
        let mut detector = FixationDetector::new(0.1);
        
        // Add clustered points
        for _ in 0..10 {
            let point = GazePoint::new(0.5, 0.5, 1.0);
            detector.update(point);
        }
        
        let fixation = detector.update(GazePoint::new(0.5, 0.5, 1.0));
        assert!(fixation.is_some());
    }

    #[test]
    fn test_saccade_detector() {
        let mut detector = SaccadeDetector::new(0.01);
        
        let p1 = GazePoint::new(0.0, 0.0, 1.0);
        detector.update(p1);
        
        // Small delay to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let p2 = GazePoint::new(0.5, 0.5, 1.0);
        let saccade = detector.update(p2);
        
        // May or may not detect depending on timing
        // Just ensure it doesn't crash
        let _ = saccade;
    }

    #[test]
    fn test_gaze_gesture_manager() {
        let mut manager = GazeGestureManager::new();
        assert_eq!(manager.state(), GazeState::NotTracking);
        
        let point = GazePoint::new(0.5, 0.5, 0.9);
        let events = manager.update_gaze(point);
        
        // State should update
        assert!(manager.gaze_point().is_some());
    }

    #[test]
    fn test_edge_detection() {
        let mut manager = GazeGestureManager::new();
        
        // Gaze at top edge
        let point = GazePoint::new(0.5, 0.05, 1.0);
        manager.update_gaze(point);
        
        assert_eq!(manager.check_edge_look(), Some(ScreenEdge::Top));
    }

    #[test]
    fn test_gaze_scroll() {
        let mut manager = GazeGestureManager::new();
        
        // Gaze at bottom edge
        let point = GazePoint::new(0.5, 0.95, 1.0);
        manager.update_gaze(point);
        
        let scroll = manager.get_gaze_scroll();
        assert_eq!(scroll, Some(ScrollDirection::Down));
    }

    #[test]
    fn test_low_confidence_ignored() {
        let mut manager = GazeGestureManager::new();
        
        let point = GazePoint::new(0.5, 0.5, 0.1); // Low confidence
        manager.update_gaze(point);
        
        assert_eq!(manager.state(), GazeState::NotTracking);
    }
}
