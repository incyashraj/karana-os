// Kāraṇa OS - AR Window Gesture Interactions
// Gesture-based control for AR windows and UI elements

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::spatial::world_coords::LocalCoord;
use super::{Handedness, Finger, GestureType, DynamicGestureType, SwipeDirection, GestureAction};
use super::finger_tracking::{ScreenPoint, FingerCursor, FingerTracker};

/// AR window identifier
pub type WindowId = u64;

/// AR window interaction controller
pub struct ARWindowController {
    /// Registered windows
    windows: HashMap<WindowId, ARWindow>,
    /// Active/focused window
    focused_window: Option<WindowId>,
    /// Interaction state
    interaction_state: InteractionState,
    /// Gesture mappings
    gesture_mappings: WindowGestureMappings,
    /// Configuration
    config: ARWindowConfig,
    /// Interaction history
    history: Vec<WindowInteraction>,
    /// Max history
    max_history: usize,
}

/// AR window representation
#[derive(Debug, Clone)]
pub struct ARWindow {
    /// Unique ID
    pub id: WindowId,
    /// Window name/title
    pub title: String,
    /// Position in screen space (normalized)
    pub position: ScreenPoint,
    /// Size in screen space (normalized)
    pub size: ScreenPoint,
    /// 3D world position (for depth)
    pub world_position: LocalCoord,
    /// Z-order (higher = on top)
    pub z_order: i32,
    /// Window state
    pub state: WindowState,
    /// Is interactive
    pub interactive: bool,
    /// Opacity (0-1)
    pub opacity: f32,
    /// Scroll position
    pub scroll_offset: ScreenPoint,
    /// Max scroll bounds
    pub scroll_max: ScreenPoint,
}

/// Window state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowState {
    /// Normal display
    Normal,
    /// Minimized
    Minimized,
    /// Maximized
    Maximized,
    /// Being dragged
    Dragging,
    /// Being resized
    Resizing,
    /// Being scrolled
    Scrolling,
    /// Closing animation
    Closing,
    /// Hidden
    Hidden,
}

/// Current interaction state
#[derive(Debug, Clone)]
pub struct InteractionState {
    /// Current mode
    pub mode: InteractionMode,
    /// Target window (if any)
    pub target_window: Option<WindowId>,
    /// Interaction start position
    pub start_position: Option<ScreenPoint>,
    /// Current position
    pub current_position: Option<ScreenPoint>,
    /// Pinch state (if pinching)
    pub pinch_state: Option<PinchState>,
    /// Swipe state (if swiping)
    pub swipe_state: Option<SwipeState>,
    /// Time started
    pub started_at: Option<Instant>,
}

/// Interaction modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionMode {
    /// No active interaction
    Idle,
    /// Hovering over element
    Hovering,
    /// Selecting (pinch held)
    Selecting,
    /// Dragging window
    Dragging,
    /// Resizing window
    Resizing,
    /// Scrolling content
    Scrolling,
    /// Pinch-to-close gesture
    PinchClosing,
    /// Two-hand interaction
    TwoHand,
}

/// Pinch gesture state
#[derive(Debug, Clone)]
pub struct PinchState {
    /// Initial pinch distance
    pub initial_distance: f32,
    /// Current pinch distance
    pub current_distance: f32,
    /// Pinch center point
    pub center: ScreenPoint,
    /// Is closing gesture (fingers coming together)
    pub is_closing: bool,
}

/// Swipe gesture state
#[derive(Debug, Clone)]
pub struct SwipeState {
    /// Swipe direction
    pub direction: SwipeDirection,
    /// Swipe velocity
    pub velocity: f32,
    /// Distance traveled
    pub distance: f32,
    /// Start position
    pub start: ScreenPoint,
}

/// Gesture mappings for window interactions
#[derive(Debug, Clone)]
pub struct WindowGestureMappings {
    /// Pinch to select/click
    pub pinch_select: bool,
    /// Pinch closing gesture to close window
    pub pinch_close_enabled: bool,
    /// Minimum pinch-close distance
    pub pinch_close_threshold: f32,
    /// Swipe to scroll
    pub swipe_scroll: bool,
    /// Swipe scroll sensitivity
    pub scroll_sensitivity: f32,
    /// Grab to drag
    pub grab_drag: bool,
    /// Two-finger pinch to resize
    pub pinch_resize: bool,
    /// Fist to minimize
    pub fist_minimize: bool,
    /// Open palm to show all windows
    pub palm_show_all: bool,
}

impl Default for WindowGestureMappings {
    fn default() -> Self {
        Self {
            pinch_select: true,
            pinch_close_enabled: true,
            pinch_close_threshold: 0.15, // 15% of screen distance
            swipe_scroll: true,
            scroll_sensitivity: 2.0,
            grab_drag: true,
            pinch_resize: true,
            fist_minimize: true,
            palm_show_all: true,
        }
    }
}

/// AR window configuration
#[derive(Debug, Clone)]
pub struct ARWindowConfig {
    /// Minimum window size
    pub min_window_size: ScreenPoint,
    /// Maximum window size
    pub max_window_size: ScreenPoint,
    /// Window snap distance
    pub snap_distance: f32,
    /// Enable window snapping
    pub snap_enabled: bool,
    /// Animation duration (ms)
    pub animation_ms: u32,
    /// Inertia for scrolling
    pub scroll_inertia: f32,
    /// Max scroll velocity
    pub max_scroll_velocity: f32,
}

impl Default for ARWindowConfig {
    fn default() -> Self {
        Self {
            min_window_size: ScreenPoint::new(0.1, 0.1),
            max_window_size: ScreenPoint::new(1.0, 1.0),
            snap_distance: 0.02,
            snap_enabled: true,
            animation_ms: 200,
            scroll_inertia: 0.95,
            max_scroll_velocity: 0.5,
        }
    }
}

/// Window interaction event
#[derive(Debug, Clone)]
pub struct WindowInteraction {
    /// Interaction type
    pub interaction_type: WindowInteractionType,
    /// Target window
    pub window_id: WindowId,
    /// Timestamp
    pub timestamp: Instant,
    /// Position
    pub position: ScreenPoint,
}

/// Types of window interactions
#[derive(Debug, Clone, PartialEq)]
pub enum WindowInteractionType {
    /// Window selected/clicked
    Select,
    /// Window focused
    Focus,
    /// Window unfocused
    Unfocus,
    /// Window dragged
    Drag { delta: ScreenPoint },
    /// Window resized
    Resize { new_size: ScreenPoint },
    /// Window scrolled
    Scroll { delta: ScreenPoint },
    /// Window closed
    Close,
    /// Window minimized
    Minimize,
    /// Window maximized
    Maximize,
    /// Window restored
    Restore,
}

/// Result of processing gestures
#[derive(Debug, Clone)]
pub struct GestureProcessResult {
    /// Actions to execute
    pub actions: Vec<WindowAction>,
    /// Updated interaction state
    pub state: InteractionState,
    /// Haptic feedback to trigger
    pub haptic: Option<HapticFeedback>,
}

/// Window action to execute
#[derive(Debug, Clone)]
pub enum WindowAction {
    /// Focus window
    Focus(WindowId),
    /// Close window
    Close(WindowId),
    /// Minimize window
    Minimize(WindowId),
    /// Maximize window
    Maximize(WindowId),
    /// Move window
    Move { window: WindowId, position: ScreenPoint },
    /// Resize window
    Resize { window: WindowId, size: ScreenPoint },
    /// Scroll window content
    Scroll { window: WindowId, delta: ScreenPoint },
    /// Select element at position
    Select { window: WindowId, position: ScreenPoint },
    /// Show all windows (expose view)
    ShowAll,
    /// Hide window
    Hide(WindowId),
}

/// Haptic feedback types
#[derive(Debug, Clone)]
pub enum HapticFeedback {
    /// Light tap
    Tap,
    /// Selection feedback
    Selection,
    /// Action completed
    Success,
    /// Action failed
    Error,
    /// Scroll boundary reached
    Boundary,
}

impl ARWindowController {
    /// Create new controller
    pub fn new() -> Self {
        Self::with_config(ARWindowConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: ARWindowConfig) -> Self {
        Self {
            windows: HashMap::new(),
            focused_window: None,
            interaction_state: InteractionState::default(),
            gesture_mappings: WindowGestureMappings::default(),
            config,
            history: Vec::new(),
            max_history: 500,
        }
    }

    /// Register a window
    pub fn register_window(&mut self, window: ARWindow) {
        self.windows.insert(window.id, window);
    }

    /// Remove a window
    pub fn remove_window(&mut self, id: WindowId) -> Option<ARWindow> {
        if self.focused_window == Some(id) {
            self.focused_window = None;
        }
        self.windows.remove(&id)
    }

    /// Get window by ID
    pub fn get_window(&self, id: WindowId) -> Option<&ARWindow> {
        self.windows.get(&id)
    }

    /// Get mutable window by ID
    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut ARWindow> {
        self.windows.get_mut(&id)
    }

    /// Process finger cursor for window interactions
    pub fn process_cursor(&mut self, cursor: &FingerCursor, pinch_distance: Option<f32>) -> GestureProcessResult {
        let mut result = GestureProcessResult {
            actions: Vec::new(),
            state: self.interaction_state.clone(),
            haptic: None,
        };

        if !cursor.engaged {
            // Cursor not active - reset interaction if needed
            if self.interaction_state.mode != InteractionMode::Idle {
                result.state.mode = InteractionMode::Idle;
                result.state.target_window = None;
            }
            self.interaction_state = result.state.clone();
            return result;
        }

        // Find window under cursor
        let window_under_cursor = self.find_window_at(cursor.screen_position);

        // Update hover state
        if let Some(window_id) = window_under_cursor {
            if self.interaction_state.mode == InteractionMode::Idle {
                result.state.mode = InteractionMode::Hovering;
                result.state.target_window = Some(window_id);
            }
        }

        // Process pinch gesture
        if let Some(distance) = pinch_distance {
            result = self.process_pinch(cursor, distance, window_under_cursor, result);
        }

        // Update current position
        result.state.current_position = Some(cursor.screen_position);
        self.interaction_state = result.state.clone();

        result
    }

    fn process_pinch(
        &mut self,
        cursor: &FingerCursor,
        distance: f32,
        window_under_cursor: Option<WindowId>,
        mut result: GestureProcessResult,
    ) -> GestureProcessResult {
        let pinch_threshold = 0.03; // 3cm threshold for pinch detection

        // Check for pinch-to-close gesture
        if distance < pinch_threshold && self.gesture_mappings.pinch_close_enabled {
            if let Some(pinch_state) = &self.interaction_state.pinch_state {
                // Check if pinch is closing (fingers coming together)
                let distance_delta = pinch_state.initial_distance - distance;

                if distance_delta > self.gesture_mappings.pinch_close_threshold {
                    // Trigger close action
                    if let Some(window_id) = self.interaction_state.target_window.or(window_under_cursor) {
                        result.actions.push(WindowAction::Close(window_id));
                        result.haptic = Some(HapticFeedback::Success);
                        result.state.mode = InteractionMode::Idle;
                        result.state.pinch_state = None;

                        self.record_interaction(WindowInteraction {
                            interaction_type: WindowInteractionType::Close,
                            window_id,
                            timestamp: Instant::now(),
                            position: cursor.screen_position,
                        });

                        return result;
                    }
                }
            } else {
                // Start tracking pinch
                result.state.pinch_state = Some(PinchState {
                    initial_distance: distance,
                    current_distance: distance,
                    center: cursor.screen_position,
                    is_closing: true,
                });
                result.state.mode = InteractionMode::PinchClosing;
            }
        }

        // Handle pinch-to-select
        if distance < pinch_threshold && self.gesture_mappings.pinch_select {
            if self.interaction_state.mode == InteractionMode::Hovering {
                result.state.mode = InteractionMode::Selecting;
                result.haptic = Some(HapticFeedback::Selection);

                if let Some(window_id) = window_under_cursor {
                    result.actions.push(WindowAction::Select {
                        window: window_id,
                        position: cursor.screen_position,
                    });

                    // Focus window if not focused
                    if self.focused_window != Some(window_id) {
                        result.actions.push(WindowAction::Focus(window_id));
                        self.focused_window = Some(window_id);
                    }

                    self.record_interaction(WindowInteraction {
                        interaction_type: WindowInteractionType::Select,
                        window_id,
                        timestamp: Instant::now(),
                        position: cursor.screen_position,
                    });
                }
            }
        }

        // Update pinch state
        if let Some(ref mut pinch_state) = result.state.pinch_state {
            pinch_state.current_distance = distance;
        }

        result
    }

    /// Process swipe gesture for scrolling
    pub fn process_swipe(&mut self, direction: SwipeDirection, velocity: f32) -> GestureProcessResult {
        let mut result = GestureProcessResult {
            actions: Vec::new(),
            state: self.interaction_state.clone(),
            haptic: None,
        };

        if !self.gesture_mappings.swipe_scroll {
            return result;
        }

        let window_id = self.focused_window.or(self.interaction_state.target_window);

        if let Some(id) = window_id {
            let scroll_delta = match direction {
                SwipeDirection::Up => ScreenPoint::new(0.0, -velocity * self.gesture_mappings.scroll_sensitivity),
                SwipeDirection::Down => ScreenPoint::new(0.0, velocity * self.gesture_mappings.scroll_sensitivity),
                SwipeDirection::Left => ScreenPoint::new(-velocity * self.gesture_mappings.scroll_sensitivity, 0.0),
                SwipeDirection::Right => ScreenPoint::new(velocity * self.gesture_mappings.scroll_sensitivity, 0.0),
                _ => ScreenPoint::new(0.0, 0.0),
            };

            result.actions.push(WindowAction::Scroll {
                window: id,
                delta: scroll_delta,
            });

            // Update window scroll offset
            if let Some(window) = self.windows.get_mut(&id) {
                window.scroll_offset.x = (window.scroll_offset.x + scroll_delta.x)
                    .clamp(0.0, window.scroll_max.x);
                window.scroll_offset.y = (window.scroll_offset.y + scroll_delta.y)
                    .clamp(0.0, window.scroll_max.y);

                // Check for boundary haptic
                if window.scroll_offset.y <= 0.0 || window.scroll_offset.y >= window.scroll_max.y {
                    result.haptic = Some(HapticFeedback::Boundary);
                }
            }

            self.record_interaction(WindowInteraction {
                interaction_type: WindowInteractionType::Scroll { delta: scroll_delta },
                window_id: id,
                timestamp: Instant::now(),
                position: ScreenPoint::new(0.5, 0.5),
            });
        }

        result
    }

    /// Process static gesture
    pub fn process_gesture(&mut self, gesture: GestureType, hand: Handedness) -> GestureProcessResult {
        let mut result = GestureProcessResult {
            actions: Vec::new(),
            state: self.interaction_state.clone(),
            haptic: None,
        };

        match gesture {
            GestureType::Fist if self.gesture_mappings.fist_minimize => {
                if let Some(window_id) = self.focused_window {
                    result.actions.push(WindowAction::Minimize(window_id));
                    result.haptic = Some(HapticFeedback::Success);
                    self.focused_window = None;
                }
            }
            GestureType::OpenPalm if self.gesture_mappings.palm_show_all => {
                result.actions.push(WindowAction::ShowAll);
                result.haptic = Some(HapticFeedback::Tap);
            }
            GestureType::Grab if self.gesture_mappings.grab_drag => {
                if let Some(window_id) = self.interaction_state.target_window {
                    result.state.mode = InteractionMode::Dragging;
                    result.state.start_position = self.interaction_state.current_position;
                }
            }
            _ => {}
        }

        result
    }

    /// Find window at screen position (accounting for z-order)
    fn find_window_at(&self, position: ScreenPoint) -> Option<WindowId> {
        let mut candidates: Vec<_> = self.windows.values()
            .filter(|w| w.state != WindowState::Hidden && w.state != WindowState::Minimized)
            .filter(|w| {
                position.x >= w.position.x &&
                position.x <= w.position.x + w.size.x &&
                position.y >= w.position.y &&
                position.y <= w.position.y + w.size.y
            })
            .collect();

        // Sort by z-order (highest first)
        candidates.sort_by(|a, b| b.z_order.cmp(&a.z_order));

        candidates.first().map(|w| w.id)
    }

    /// Record interaction in history
    fn record_interaction(&mut self, interaction: WindowInteraction) {
        self.history.push(interaction);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Get focused window
    pub fn focused(&self) -> Option<WindowId> {
        self.focused_window
    }

    /// Set focused window
    pub fn set_focus(&mut self, id: Option<WindowId>) {
        self.focused_window = id;
    }

    /// Get all visible windows
    pub fn visible_windows(&self) -> Vec<&ARWindow> {
        self.windows.values()
            .filter(|w| w.state != WindowState::Hidden)
            .collect()
    }

    /// Get interaction history
    pub fn history(&self) -> &[WindowInteraction] {
        &self.history
    }

    /// Apply window action
    pub fn apply_action(&mut self, action: &WindowAction) {
        match action {
            WindowAction::Focus(id) => {
                self.focused_window = Some(*id);
            }
            WindowAction::Close(id) => {
                if let Some(window) = self.windows.get_mut(id) {
                    window.state = WindowState::Closing;
                }
            }
            WindowAction::Minimize(id) => {
                if let Some(window) = self.windows.get_mut(id) {
                    window.state = WindowState::Minimized;
                }
            }
            WindowAction::Maximize(id) => {
                if let Some(window) = self.windows.get_mut(id) {
                    window.state = WindowState::Maximized;
                    window.position = ScreenPoint::new(0.0, 0.0);
                    window.size = ScreenPoint::new(1.0, 1.0);
                }
            }
            WindowAction::Move { window, position } => {
                if let Some(w) = self.windows.get_mut(window) {
                    w.position = *position;
                }
            }
            WindowAction::Resize { window, size } => {
                if let Some(w) = self.windows.get_mut(window) {
                    w.size = ScreenPoint::new(
                        size.x.clamp(self.config.min_window_size.x, self.config.max_window_size.x),
                        size.y.clamp(self.config.min_window_size.y, self.config.max_window_size.y),
                    );
                }
            }
            WindowAction::Scroll { window, delta } => {
                if let Some(w) = self.windows.get_mut(window) {
                    w.scroll_offset.x = (w.scroll_offset.x + delta.x).clamp(0.0, w.scroll_max.x);
                    w.scroll_offset.y = (w.scroll_offset.y + delta.y).clamp(0.0, w.scroll_max.y);
                }
            }
            WindowAction::Hide(id) => {
                if let Some(window) = self.windows.get_mut(id) {
                    window.state = WindowState::Hidden;
                }
            }
            _ => {}
        }
    }
}

impl Default for ARWindowController {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            mode: InteractionMode::Idle,
            target_window: None,
            start_position: None,
            current_position: None,
            pinch_state: None,
            swipe_state: None,
            started_at: None,
        }
    }
}

impl Default for ARWindow {
    fn default() -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self {
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            title: String::new(),
            position: ScreenPoint::new(0.1, 0.1),
            size: ScreenPoint::new(0.3, 0.3),
            world_position: LocalCoord::default(),
            z_order: 0,
            state: WindowState::Normal,
            interactive: true,
            opacity: 1.0,
            scroll_offset: ScreenPoint::new(0.0, 0.0),
            scroll_max: ScreenPoint::new(0.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_creation() {
        let controller = ARWindowController::new();
        assert!(controller.focused().is_none());
        assert!(controller.visible_windows().is_empty());
    }

    #[test]
    fn test_register_window() {
        let mut controller = ARWindowController::new();
        let window = ARWindow {
            id: 1,
            title: "Test Window".to_string(),
            ..Default::default()
        };

        controller.register_window(window);
        assert!(controller.get_window(1).is_some());
        assert_eq!(controller.get_window(1).unwrap().title, "Test Window");
    }

    #[test]
    fn test_remove_window() {
        let mut controller = ARWindowController::new();
        let window = ARWindow {
            id: 42,
            ..Default::default()
        };

        controller.register_window(window);
        assert!(controller.get_window(42).is_some());

        let removed = controller.remove_window(42);
        assert!(removed.is_some());
        assert!(controller.get_window(42).is_none());
    }

    #[test]
    fn test_find_window_at_position() {
        let mut controller = ARWindowController::new();
        let window = ARWindow {
            id: 1,
            position: ScreenPoint::new(0.2, 0.2),
            size: ScreenPoint::new(0.4, 0.4),
            ..Default::default()
        };

        controller.register_window(window);

        // Point inside window
        let found = controller.find_window_at(ScreenPoint::new(0.4, 0.4));
        assert_eq!(found, Some(1));

        // Point outside window
        let found = controller.find_window_at(ScreenPoint::new(0.1, 0.1));
        assert!(found.is_none());
    }

    #[test]
    fn test_window_z_order() {
        let mut controller = ARWindowController::new();
        
        let window1 = ARWindow {
            id: 1,
            position: ScreenPoint::new(0.2, 0.2),
            size: ScreenPoint::new(0.4, 0.4),
            z_order: 0,
            ..Default::default()
        };
        let window2 = ARWindow {
            id: 2,
            position: ScreenPoint::new(0.3, 0.3),
            size: ScreenPoint::new(0.4, 0.4),
            z_order: 1, // Higher z-order
            ..Default::default()
        };

        controller.register_window(window1);
        controller.register_window(window2);

        // Overlapping point should return higher z-order window
        let found = controller.find_window_at(ScreenPoint::new(0.4, 0.4));
        assert_eq!(found, Some(2));
    }

    #[test]
    fn test_apply_move_action() {
        let mut controller = ARWindowController::new();
        controller.register_window(ARWindow {
            id: 1,
            position: ScreenPoint::new(0.0, 0.0),
            ..Default::default()
        });

        controller.apply_action(&WindowAction::Move {
            window: 1,
            position: ScreenPoint::new(0.5, 0.5),
        });

        let window = controller.get_window(1).unwrap();
        assert!((window.position.x - 0.5).abs() < 0.001);
        assert!((window.position.y - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_apply_resize_action() {
        let mut controller = ARWindowController::new();
        controller.register_window(ARWindow {
            id: 1,
            size: ScreenPoint::new(0.3, 0.3),
            ..Default::default()
        });

        controller.apply_action(&WindowAction::Resize {
            window: 1,
            size: ScreenPoint::new(0.5, 0.5),
        });

        let window = controller.get_window(1).unwrap();
        assert!((window.size.x - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_apply_scroll_action() {
        let mut controller = ARWindowController::new();
        controller.register_window(ARWindow {
            id: 1,
            scroll_offset: ScreenPoint::new(0.0, 0.0),
            scroll_max: ScreenPoint::new(0.0, 2.0),
            ..Default::default()
        });

        controller.apply_action(&WindowAction::Scroll {
            window: 1,
            delta: ScreenPoint::new(0.0, 0.5),
        });

        let window = controller.get_window(1).unwrap();
        assert!((window.scroll_offset.y - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_scroll_clamping() {
        let mut controller = ARWindowController::new();
        controller.register_window(ARWindow {
            id: 1,
            scroll_offset: ScreenPoint::new(0.0, 0.0),
            scroll_max: ScreenPoint::new(0.0, 1.0),
            ..Default::default()
        });

        // Try to scroll beyond max
        controller.apply_action(&WindowAction::Scroll {
            window: 1,
            delta: ScreenPoint::new(0.0, 5.0),
        });

        let window = controller.get_window(1).unwrap();
        assert!((window.scroll_offset.y - 1.0).abs() < 0.001); // Clamped to max
    }

    #[test]
    fn test_focus_management() {
        let mut controller = ARWindowController::new();
        controller.register_window(ARWindow { id: 1, ..Default::default() });
        controller.register_window(ARWindow { id: 2, ..Default::default() });

        assert!(controller.focused().is_none());

        controller.apply_action(&WindowAction::Focus(1));
        assert_eq!(controller.focused(), Some(1));

        controller.apply_action(&WindowAction::Focus(2));
        assert_eq!(controller.focused(), Some(2));
    }

    #[test]
    fn test_minimize_action() {
        let mut controller = ARWindowController::new();
        controller.register_window(ARWindow { id: 1, ..Default::default() });
        controller.set_focus(Some(1));

        controller.apply_action(&WindowAction::Minimize(1));

        let window = controller.get_window(1).unwrap();
        assert_eq!(window.state, WindowState::Minimized);
    }

    #[test]
    fn test_maximize_action() {
        let mut controller = ARWindowController::new();
        controller.register_window(ARWindow {
            id: 1,
            position: ScreenPoint::new(0.2, 0.2),
            size: ScreenPoint::new(0.3, 0.3),
            ..Default::default()
        });

        controller.apply_action(&WindowAction::Maximize(1));

        let window = controller.get_window(1).unwrap();
        assert_eq!(window.state, WindowState::Maximized);
        assert!((window.size.x - 1.0).abs() < 0.001);
        assert!((window.size.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_swipe_scroll() {
        let mut controller = ARWindowController::new();
        controller.register_window(ARWindow {
            id: 1,
            scroll_max: ScreenPoint::new(0.0, 10.0),
            ..Default::default()
        });
        controller.set_focus(Some(1));

        let result = controller.process_swipe(SwipeDirection::Down, 0.1);
        assert!(!result.actions.is_empty());

        // Check scroll action was created
        let has_scroll = result.actions.iter().any(|a| matches!(a, WindowAction::Scroll { .. }));
        assert!(has_scroll);
    }

    #[test]
    fn test_gesture_mappings_default() {
        let mappings = WindowGestureMappings::default();
        assert!(mappings.pinch_select);
        assert!(mappings.pinch_close_enabled);
        assert!(mappings.swipe_scroll);
        assert!(mappings.grab_drag);
    }

    #[test]
    fn test_window_default() {
        let w1 = ARWindow::default();
        let w2 = ARWindow::default();
        assert_ne!(w1.id, w2.id); // Each default should have unique ID
    }

    #[test]
    fn test_interaction_state_default() {
        let state = InteractionState::default();
        assert_eq!(state.mode, InteractionMode::Idle);
        assert!(state.target_window.is_none());
    }

    #[test]
    fn test_visible_windows_filter() {
        let mut controller = ARWindowController::new();
        controller.register_window(ARWindow {
            id: 1,
            state: WindowState::Normal,
            ..Default::default()
        });
        controller.register_window(ARWindow {
            id: 2,
            state: WindowState::Hidden,
            ..Default::default()
        });

        let visible = controller.visible_windows();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, 1);
    }
}
