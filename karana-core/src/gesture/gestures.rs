//! Gesture Actions
//!
//! Maps gestures to system actions and provides gesture-based interactions.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::spatial::world_coords::LocalCoord;
use super::{GestureType, DynamicGestureType, TwoHandGestureType, Handedness, SwipeDirection};

/// Action triggered by a gesture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureAction {
    /// No action
    None,
    /// Select/click item under cursor
    Select,
    /// Confirm/enter
    Confirm,
    /// Cancel/back
    Cancel,
    /// Open context menu
    ContextMenu,
    /// Scroll in direction
    Scroll { dx: f32, dy: f32 },
    /// Zoom in/out
    Zoom { factor: f32 },
    /// Rotate object
    Rotate { angle: f32 },
    /// Navigate forward
    Forward,
    /// Navigate back
    Back,
    /// Toggle voice input
    VoiceInput,
    /// Take screenshot
    Screenshot,
    /// Dismiss notification
    Dismiss,
    /// Custom action
    Custom(String),
}

/// Gesture binding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureBinding {
    /// Gesture type
    pub gesture: GestureType,
    /// Required hand
    pub hand: Option<Handedness>,
    /// Minimum hold duration
    pub hold_duration: Option<Duration>,
    /// Action to trigger
    pub action: GestureAction,
    /// Priority (higher = preferred)
    pub priority: u8,
}

impl GestureBinding {
    pub fn new(gesture: GestureType, action: GestureAction) -> Self {
        Self {
            gesture,
            hand: None,
            hold_duration: None,
            action,
            priority: 0,
        }
    }
    
    pub fn with_hand(mut self, hand: Handedness) -> Self {
        self.hand = Some(hand);
        self
    }
    
    pub fn with_hold(mut self, duration: Duration) -> Self {
        self.hold_duration = Some(duration);
        self
    }
    
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// Dynamic gesture binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicGestureBinding {
    pub gesture: DynamicGestureType,
    pub hand: Option<Handedness>,
    pub action: GestureAction,
}

/// Gesture action mapper
pub struct GestureActionMapper {
    /// Static gesture bindings
    bindings: Vec<GestureBinding>,
    /// Dynamic gesture bindings
    dynamic_bindings: Vec<DynamicGestureBinding>,
    /// Two-hand gesture bindings
    two_hand_bindings: HashMap<TwoHandGestureType, GestureAction>,
    /// Custom action handler names
    custom_handlers: HashMap<String, String>,
}

impl GestureActionMapper {
    /// Create with default bindings
    pub fn new() -> Self {
        let mut mapper = Self {
            bindings: vec![],
            dynamic_bindings: vec![],
            two_hand_bindings: HashMap::new(),
            custom_handlers: HashMap::new(),
        };
        mapper.setup_default_bindings();
        mapper
    }
    
    /// Setup default gesture->action mappings
    fn setup_default_bindings(&mut self) {
        // Pinch = select
        self.bindings.push(GestureBinding::new(
            GestureType::Pinch,
            GestureAction::Select,
        ));
        
        // Point = cursor/ray control (no action, just tracking)
        self.bindings.push(GestureBinding::new(
            GestureType::Point,
            GestureAction::None,
        ));
        
        // Open palm = stop/cancel
        self.bindings.push(GestureBinding::new(
            GestureType::OpenPalm,
            GestureAction::Cancel,
        ).with_hold(Duration::from_millis(500)));
        
        // Thumbs up = confirm
        self.bindings.push(GestureBinding::new(
            GestureType::ThumbsUp,
            GestureAction::Confirm,
        ));
        
        // Thumbs down = dismiss/reject
        self.bindings.push(GestureBinding::new(
            GestureType::ThumbsDown,
            GestureAction::Dismiss,
        ));
        
        // Fist + hold = voice input
        self.bindings.push(GestureBinding::new(
            GestureType::Fist,
            GestureAction::VoiceInput,
        ).with_hold(Duration::from_millis(300)));
        
        // Peace = screenshot (fun!)
        self.bindings.push(GestureBinding::new(
            GestureType::Peace,
            GestureAction::Screenshot,
        ).with_hold(Duration::from_millis(500)));
        
        // Two-hand zoom
        self.two_hand_bindings.insert(
            TwoHandGestureType::Zoom,
            GestureAction::Zoom { factor: 1.0 },
        );
        
        // Two-hand rotate
        self.two_hand_bindings.insert(
            TwoHandGestureType::Rotate,
            GestureAction::Rotate { angle: 0.0 },
        );
        
        // Swipe gestures
        self.dynamic_bindings.push(DynamicGestureBinding {
            gesture: DynamicGestureType::Swipe { direction: SwipeDirection::Left },
            hand: None,
            action: GestureAction::Forward,
        });
        
        self.dynamic_bindings.push(DynamicGestureBinding {
            gesture: DynamicGestureType::Swipe { direction: SwipeDirection::Right },
            hand: None,
            action: GestureAction::Back,
        });
    }
    
    /// Add custom binding
    pub fn add_binding(&mut self, binding: GestureBinding) {
        self.bindings.push(binding);
        // Sort by priority (descending)
        self.bindings.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    /// Get action for static gesture
    pub fn get_action(&self, gesture: GestureType, hand: Handedness, held_ms: u64) -> Option<GestureAction> {
        for binding in &self.bindings {
            if binding.gesture != gesture {
                continue;
            }
            
            if let Some(required_hand) = binding.hand {
                if required_hand != hand {
                    continue;
                }
            }
            
            if let Some(hold_duration) = binding.hold_duration {
                if held_ms < hold_duration.as_millis() as u64 {
                    continue;
                }
            }
            
            return Some(binding.action.clone());
        }
        None
    }
    
    /// Get action for dynamic gesture
    pub fn get_dynamic_action(&self, gesture: DynamicGestureType, hand: Handedness) -> Option<GestureAction> {
        for binding in &self.dynamic_bindings {
            if binding.gesture != gesture {
                continue;
            }
            
            if let Some(required_hand) = binding.hand {
                if required_hand != hand {
                    continue;
                }
            }
            
            return Some(binding.action.clone());
        }
        None
    }
    
    /// Get action for two-hand gesture
    pub fn get_two_hand_action(&self, gesture: TwoHandGestureType) -> Option<GestureAction> {
        self.two_hand_bindings.get(&gesture).cloned()
    }
    
    /// Clear all bindings
    pub fn clear(&mut self) {
        self.bindings.clear();
        self.dynamic_bindings.clear();
        self.two_hand_bindings.clear();
    }
}

impl Default for GestureActionMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Spatial gesture zone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureZone {
    /// Zone identifier
    pub id: String,
    /// Center position
    pub center: LocalCoord,
    /// Radius (meters)
    pub radius: f32,
    /// Special handling for this zone
    pub zone_type: GestureZoneType,
    /// Custom action overrides
    pub action_overrides: HashMap<GestureType, GestureAction>,
}

/// Types of gesture zones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureZoneType {
    /// Normal zone
    Normal,
    /// Keyboard area (2D input)
    Keyboard,
    /// Slider/dial (rotational input)
    Control,
    /// No gestures (pass through)
    NoGesture,
    /// Custom behavior
    Custom(String),
}

impl GestureZone {
    /// Create a new zone
    pub fn new(id: String, center: LocalCoord, radius: f32) -> Self {
        Self {
            id,
            center,
            radius,
            zone_type: GestureZoneType::Normal,
            action_overrides: HashMap::new(),
        }
    }
    
    /// Check if point is in zone
    pub fn contains(&self, point: &LocalCoord) -> bool {
        self.center.distance_to(point) <= self.radius
    }
    
    /// Add action override for this zone
    pub fn override_action(&mut self, gesture: GestureType, action: GestureAction) {
        self.action_overrides.insert(gesture, action);
    }
    
    /// Get overridden action if any
    pub fn get_override(&self, gesture: GestureType) -> Option<&GestureAction> {
        self.action_overrides.get(&gesture)
    }
}

/// Gesture tutorial/training system
#[derive(Debug, Clone)]
pub struct GestureTutorial {
    /// Tutorial gestures to learn
    gestures: Vec<TutorialGesture>,
    /// Current lesson index
    current: usize,
    /// Completed gestures
    completed: Vec<GestureType>,
    /// Start time
    started_at: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct TutorialGesture {
    pub gesture: GestureType,
    pub name: String,
    pub description: String,
    pub difficulty: u8,
}

impl GestureTutorial {
    /// Create new tutorial
    pub fn new() -> Self {
        Self {
            gestures: vec![
                TutorialGesture {
                    gesture: GestureType::OpenPalm,
                    name: "Open Palm".to_string(),
                    description: "Hold your hand open with all fingers spread".to_string(),
                    difficulty: 1,
                },
                TutorialGesture {
                    gesture: GestureType::Pinch,
                    name: "Pinch".to_string(),
                    description: "Touch your thumb and index finger together".to_string(),
                    difficulty: 1,
                },
                TutorialGesture {
                    gesture: GestureType::Point,
                    name: "Point".to_string(),
                    description: "Extend your index finger, curl others".to_string(),
                    difficulty: 2,
                },
                TutorialGesture {
                    gesture: GestureType::ThumbsUp,
                    name: "Thumbs Up".to_string(),
                    description: "Make a fist with thumb extended upward".to_string(),
                    difficulty: 2,
                },
                TutorialGesture {
                    gesture: GestureType::Peace,
                    name: "Peace Sign".to_string(),
                    description: "Extend index and middle fingers in V shape".to_string(),
                    difficulty: 2,
                },
            ],
            current: 0,
            completed: vec![],
            started_at: None,
        }
    }
    
    /// Start tutorial
    pub fn start(&mut self) {
        self.current = 0;
        self.completed.clear();
        self.started_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        );
    }
    
    /// Get current gesture to practice
    pub fn current_gesture(&self) -> Option<&TutorialGesture> {
        self.gestures.get(self.current)
    }
    
    /// Mark current gesture as completed
    pub fn complete_current(&mut self) -> bool {
        if let Some(gesture) = self.gestures.get(self.current) {
            self.completed.push(gesture.gesture);
            self.current += 1;
            true
        } else {
            false
        }
    }
    
    /// Check if tutorial is complete
    pub fn is_complete(&self) -> bool {
        self.current >= self.gestures.len()
    }
    
    /// Get progress (0-1)
    pub fn progress(&self) -> f32 {
        if self.gestures.is_empty() {
            1.0
        } else {
            self.completed.len() as f32 / self.gestures.len() as f32
        }
    }
}

impl Default for GestureTutorial {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gesture_binding() {
        let binding = GestureBinding::new(GestureType::Pinch, GestureAction::Select)
            .with_hand(Handedness::Right)
            .with_priority(5);
        
        assert_eq!(binding.gesture, GestureType::Pinch);
        assert_eq!(binding.hand, Some(Handedness::Right));
        assert_eq!(binding.priority, 5);
    }
    
    #[test]
    fn test_action_mapper() {
        let mapper = GestureActionMapper::new();
        
        // Pinch should map to select
        let action = mapper.get_action(GestureType::Pinch, Handedness::Right, 0);
        assert!(matches!(action, Some(GestureAction::Select)));
    }
    
    #[test]
    fn test_hold_duration() {
        let mapper = GestureActionMapper::new();
        
        // Fist with short hold should not trigger
        let action = mapper.get_action(GestureType::Fist, Handedness::Right, 100);
        assert!(action.is_none());
        
        // Fist with long hold should trigger voice input
        let action = mapper.get_action(GestureType::Fist, Handedness::Right, 500);
        assert!(matches!(action, Some(GestureAction::VoiceInput)));
    }
    
    #[test]
    fn test_gesture_zone() {
        let zone = GestureZone::new(
            "test".to_string(),
            LocalCoord::new(0.0, 0.0, 0.0),
            0.5,
        );
        
        assert!(zone.contains(&LocalCoord::new(0.0, 0.0, 0.0)));
        assert!(zone.contains(&LocalCoord::new(0.3, 0.0, 0.0)));
        assert!(!zone.contains(&LocalCoord::new(1.0, 0.0, 0.0)));
    }
    
    #[test]
    fn test_tutorial() {
        let mut tutorial = GestureTutorial::new();
        tutorial.start();
        
        assert!(!tutorial.is_complete());
        assert!(tutorial.current_gesture().is_some());
        
        // Complete all gestures
        while !tutorial.is_complete() {
            tutorial.complete_current();
        }
        
        assert!(tutorial.is_complete());
        assert_eq!(tutorial.progress(), 1.0);
    }
}
