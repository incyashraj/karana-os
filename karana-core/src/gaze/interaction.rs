//! Gaze-based interaction system
//!
//! Handles dwell selection, gaze gestures, and UI navigation via eye tracking.

use super::{GazePoint, GazeConfig, GazeEvent, BlinkType, BlinkEvent};
use std::collections::HashMap;
use std::time::Instant;

/// Dwell state for a target
#[derive(Debug, Clone)]
struct DwellState {
    /// Target being dwelled on
    target_id: String,
    /// Start time
    start: Instant,
    /// Last update time
    last_update: Instant,
    /// Progress (0-1)
    progress: f32,
    /// Has selection been triggered
    triggered: bool,
}

/// Gaze gesture type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GazeGesture {
    /// Look left then right
    LookLeftRight,
    /// Look right then left
    LookRightLeft,
    /// Look up then down
    LookUpDown,
    /// Look down then up
    LookDownUp,
    /// Circle clockwise
    CircleCW,
    /// Circle counter-clockwise
    CircleCCW,
}

/// Gaze gesture recognizer
pub struct GestureRecognizer {
    /// Recent gaze points for gesture detection
    points: Vec<GazePoint>,
    /// Maximum points to keep
    max_points: usize,
    /// Gesture detection enabled
    enabled: bool,
}

impl GestureRecognizer {
    pub fn new() -> Self {
        Self {
            points: Vec::with_capacity(60),
            max_points: 60,  // 1 second at 60fps
            enabled: true,
        }
    }
    
    /// Add a gaze point
    pub fn add_point(&mut self, point: GazePoint) {
        if self.points.len() >= self.max_points {
            self.points.remove(0);
        }
        self.points.push(point);
    }
    
    /// Check for gesture
    pub fn detect(&self) -> Option<GazeGesture> {
        if self.points.len() < 20 {
            return None;
        }
        
        // Analyze trajectory
        let trajectory = self.analyze_trajectory();
        
        // Match against known gestures
        self.match_gesture(&trajectory)
    }
    
    /// Analyze gaze trajectory
    fn analyze_trajectory(&self) -> Vec<(f32, f32)> {
        // Downsample to key points
        let step = self.points.len() / 10;
        if step == 0 {
            return Vec::new();
        }
        
        self.points
            .iter()
            .step_by(step)
            .map(|p| (p.x, p.y))
            .collect()
    }
    
    /// Match trajectory to known gestures
    fn match_gesture(&self, trajectory: &[(f32, f32)]) -> Option<GazeGesture> {
        if trajectory.len() < 3 {
            return None;
        }
        
        let (start_x, start_y) = trajectory[0];
        let (end_x, end_y) = trajectory[trajectory.len() - 1];
        
        // Find extremes
        let (min_x, max_x, min_y, max_y) = trajectory.iter().fold(
            (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
            |(min_x, max_x, min_y, max_y), &(x, y)| {
                (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
            },
        );
        
        let range_x = max_x - min_x;
        let range_y = max_y - min_y;
        
        // Need significant movement
        if range_x < 0.2 && range_y < 0.2 {
            return None;
        }
        
        // Horizontal gesture
        if range_x > range_y * 1.5 && range_x > 0.3 {
            // Find if it went left-right or right-left
            let mid_idx = trajectory.len() / 2;
            let mid_x = trajectory[mid_idx].0;
            
            if mid_x < start_x.min(end_x) {
                // Went left first
                return Some(GazeGesture::LookLeftRight);
            } else if mid_x > start_x.max(end_x) {
                // Went right first
                return Some(GazeGesture::LookRightLeft);
            }
        }
        
        // Vertical gesture
        if range_y > range_x * 1.5 && range_y > 0.3 {
            let mid_idx = trajectory.len() / 2;
            let mid_y = trajectory[mid_idx].1;
            
            if mid_y < start_y.min(end_y) {
                return Some(GazeGesture::LookUpDown);
            } else if mid_y > start_y.max(end_y) {
                return Some(GazeGesture::LookDownUp);
            }
        }
        
        // Check for circular motion
        if let Some(direction) = self.detect_circle(trajectory) {
            return Some(if direction > 0.0 {
                GazeGesture::CircleCW
            } else {
                GazeGesture::CircleCCW
            });
        }
        
        None
    }
    
    /// Detect circular motion, returns signed area (positive = CW)
    fn detect_circle(&self, trajectory: &[(f32, f32)]) -> Option<f32> {
        if trajectory.len() < 8 {
            return None;
        }
        
        // Calculate signed area (shoelace formula)
        let mut area = 0.0;
        let n = trajectory.len();
        
        for i in 0..n {
            let (x1, y1) = trajectory[i];
            let (x2, y2) = trajectory[(i + 1) % n];
            area += (x2 - x1) * (y2 + y1);
        }
        
        area /= 2.0;
        
        // Need significant enclosed area for a circle
        if area.abs() > 0.02 {
            Some(area)
        } else {
            None
        }
    }
    
    /// Clear gesture history
    pub fn clear(&mut self) {
        self.points.clear();
    }
}

impl Default for GestureRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Gaze interaction manager
pub struct GazeInteraction {
    config: GazeConfig,
    
    /// Current dwell state
    dwell: Option<DwellState>,
    
    /// Registered interactive targets
    targets: HashMap<String, InteractiveTarget>,
    
    /// Gaze gesture recognizer
    gesture: GestureRecognizer,
    
    /// Last gaze point
    last_point: Option<GazePoint>,
    
    /// Blink action bindings
    blink_actions: HashMap<BlinkType, String>,
}

/// Interactive target that can be selected via gaze
#[derive(Debug, Clone)]
pub struct InteractiveTarget {
    /// Target ID
    pub id: String,
    /// Bounding box (x, y, width, height) - normalized
    pub bounds: (f32, f32, f32, f32),
    /// Custom dwell time (ms), None = use default
    pub dwell_time_ms: Option<u32>,
    /// Action to perform on selection
    pub action: String,
    /// Is selectable
    pub selectable: bool,
}

impl InteractiveTarget {
    pub fn new(id: impl Into<String>, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            id: id.into(),
            bounds: (x, y, w, h),
            dwell_time_ms: None,
            action: String::new(),
            selectable: true,
        }
    }
    
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = action.into();
        self
    }
    
    pub fn with_dwell_time(mut self, ms: u32) -> Self {
        self.dwell_time_ms = Some(ms);
        self
    }
    
    pub fn contains(&self, point: &GazePoint) -> bool {
        let (x, y, w, h) = self.bounds;
        point.x >= x && point.x <= x + w && point.y >= y && point.y <= y + h
    }
}

impl GazeInteraction {
    pub fn new(config: GazeConfig) -> Self {
        Self {
            config,
            dwell: None,
            targets: HashMap::new(),
            gesture: GestureRecognizer::new(),
            last_point: None,
            blink_actions: HashMap::new(),
        }
    }
    
    /// Update with new gaze point, returns event if selection triggered
    pub fn update(&mut self, point: &GazePoint, timestamp: u64) -> Option<GazeEvent> {
        self.last_point = Some(*point);
        self.gesture.add_point(*point);
        
        // Find target under gaze
        let target_id = self.find_target(point);
        
        // Update dwell state
        let event = self.update_dwell(target_id.as_deref(), point);
        
        event
    }
    
    /// Find target at gaze point
    fn find_target(&self, point: &GazePoint) -> Option<String> {
        for (id, target) in &self.targets {
            if target.selectable && target.contains(point) {
                return Some(id.clone());
            }
        }
        None
    }
    
    /// Update dwell state
    fn update_dwell(&mut self, target_id: Option<&str>, point: &GazePoint) -> Option<GazeEvent> {
        match (&mut self.dwell, target_id) {
            (Some(dwell), Some(id)) if dwell.target_id == id => {
                // Continue dwelling on same target
                dwell.last_update = Instant::now();
                
                let dwell_time = self.targets
                    .get(id)
                    .and_then(|t| t.dwell_time_ms)
                    .unwrap_or(self.config.dwell_time_ms);
                
                let elapsed_ms = dwell.start.elapsed().as_millis() as u32;
                dwell.progress = (elapsed_ms as f32 / dwell_time as f32).min(1.0);
                
                if elapsed_ms >= dwell_time && !dwell.triggered {
                    dwell.triggered = true;
                    return Some(GazeEvent::Dwell {
                        point: *point,
                        duration_ms: elapsed_ms,
                    });
                }
            }
            (_, Some(id)) => {
                // Start dwelling on new target
                self.dwell = Some(DwellState {
                    target_id: id.to_string(),
                    start: Instant::now(),
                    last_update: Instant::now(),
                    progress: 0.0,
                    triggered: false,
                });
            }
            (Some(_), None) => {
                // Gaze left target
                self.dwell = None;
            }
            (None, None) => {
                // No target, nothing to do
            }
        }
        
        None
    }
    
    /// Register an interactive target
    pub fn register_target(&mut self, target: InteractiveTarget) {
        self.targets.insert(target.id.clone(), target);
    }
    
    /// Unregister a target
    pub fn unregister_target(&mut self, id: &str) {
        self.targets.remove(id);
        if let Some(dwell) = &self.dwell {
            if dwell.target_id == id {
                self.dwell = None;
            }
        }
    }
    
    /// Get dwell progress on current target (0-1)
    pub fn dwell_progress(&self) -> f32 {
        self.dwell.as_ref().map(|d| d.progress).unwrap_or(0.0)
    }
    
    /// Get current dwell target
    pub fn dwell_target(&self) -> Option<&str> {
        self.dwell.as_ref().map(|d| d.target_id.as_str())
    }
    
    /// Bind an action to a blink type
    pub fn bind_blink(&mut self, blink_type: BlinkType, action: impl Into<String>) {
        self.blink_actions.insert(blink_type, action.into());
    }
    
    /// Handle blink event, returns action if bound
    pub fn handle_blink(&self, blink: &BlinkEvent) -> Option<&str> {
        self.blink_actions.get(&blink.blink_type).map(|s| s.as_str())
    }
    
    /// Check for gaze gesture
    pub fn check_gesture(&self) -> Option<GazeGesture> {
        self.gesture.detect()
    }
    
    /// Clear gesture history
    pub fn clear_gesture(&mut self) {
        self.gesture.clear();
    }
    
    /// Get all registered targets
    pub fn targets(&self) -> &HashMap<String, InteractiveTarget> {
        &self.targets
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_interactive_target() {
        let target = InteractiveTarget::new("btn", 0.4, 0.4, 0.2, 0.2);
        
        let inside = GazePoint::new(0.5, 0.5, 1.0);
        let outside = GazePoint::new(0.1, 0.1, 1.0);
        
        assert!(target.contains(&inside));
        assert!(!target.contains(&outside));
    }
    
    #[test]
    fn test_target_with_action() {
        let target = InteractiveTarget::new("btn", 0.0, 0.0, 0.5, 0.5)
            .with_action("click")
            .with_dwell_time(300);
        
        assert_eq!(target.action, "click");
        assert_eq!(target.dwell_time_ms, Some(300));
    }
    
    #[test]
    fn test_interaction_manager() {
        let config = GazeConfig::default();
        let mut interaction = GazeInteraction::new(config);
        
        // Register a target
        let target = InteractiveTarget::new("btn1", 0.4, 0.4, 0.2, 0.2);
        interaction.register_target(target);
        
        assert_eq!(interaction.targets().len(), 1);
    }
    
    #[test]
    fn test_dwell_detection() {
        let mut config = GazeConfig::default();
        config.dwell_time_ms = 100;  // Short for testing
        let mut interaction = GazeInteraction::new(config);
        
        let target = InteractiveTarget::new("btn1", 0.4, 0.4, 0.2, 0.2);
        interaction.register_target(target);
        
        // Look at target
        let point = GazePoint::new(0.5, 0.5, 1.0);
        interaction.update(&point, 0);
        
        assert!(interaction.dwell_target().is_some());
        assert_eq!(interaction.dwell_target().unwrap(), "btn1");
    }
    
    #[test]
    fn test_dwell_reset_on_leave() {
        let config = GazeConfig::default();
        let mut interaction = GazeInteraction::new(config);
        
        let target = InteractiveTarget::new("btn1", 0.4, 0.4, 0.2, 0.2);
        interaction.register_target(target);
        
        // Look at target
        interaction.update(&GazePoint::new(0.5, 0.5, 1.0), 0);
        assert!(interaction.dwell_target().is_some());
        
        // Look away
        interaction.update(&GazePoint::new(0.1, 0.1, 1.0), 100);
        assert!(interaction.dwell_target().is_none());
    }
    
    #[test]
    fn test_blink_binding() {
        let config = GazeConfig::default();
        let mut interaction = GazeInteraction::new(config);
        
        interaction.bind_blink(BlinkType::Double, "confirm");
        
        let blink = BlinkEvent {
            blink_type: BlinkType::Double,
            eye: None,
            duration_ms: 100,
            timestamp: 0,
        };
        
        assert_eq!(interaction.handle_blink(&blink), Some("confirm"));
    }
    
    #[test]
    fn test_gesture_recognizer() {
        let mut recognizer = GestureRecognizer::new();
        
        // Not enough points
        assert!(recognizer.detect().is_none());
        
        // Add some points but not a gesture
        for i in 0..20 {
            recognizer.add_point(GazePoint::new(0.5, 0.5, 1.0));
        }
        
        // Still no gesture (staying still)
        assert!(recognizer.detect().is_none());
    }
}
