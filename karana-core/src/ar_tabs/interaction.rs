//! # Tab Interaction
//!
//! Handles gaze tracking, voice commands, and gesture controls
//! for interacting with AR tabs.

use std::collections::VecDeque;
use super::tab::{TabId, ARTab};
use crate::spatial::WorldPosition;

/// Tab interaction manager
#[derive(Debug)]
pub struct TabInteraction {
    /// Gaze state
    gaze: GazeState,
    /// Dwell configuration
    dwell_config: DwellConfig,
    /// Current target tab
    target_tab: Option<TabId>,
    /// Gesture history
    gesture_history: VecDeque<GestureEvent>,
    /// Voice command history
    voice_history: VecDeque<VoiceTabCommand>,
}

impl TabInteraction {
    /// Create a new interaction manager
    pub fn new() -> Self {
        Self {
            gaze: GazeState::default(),
            dwell_config: DwellConfig::default(),
            target_tab: None,
            gesture_history: VecDeque::with_capacity(100),
            voice_history: VecDeque::with_capacity(100),
        }
    }
    
    /// Create with custom dwell configuration
    pub fn with_dwell_config(dwell_config: DwellConfig) -> Self {
        Self {
            dwell_config,
            ..Self::new()
        }
    }
    
    /// Process gaze update
    pub fn on_gaze(
        &mut self,
        gaze_point: &WorldPosition,
        tabs: &[&ARTab],
        timestamp_ms: u64,
    ) -> Option<InteractionEvent> {
        // Find which tab is gazed at
        let target = tabs.iter()
            .find(|tab| tab.interaction_zone.contains(gaze_point))
            .map(|tab| tab.id);
        
        // Update gaze state
        self.gaze.position = gaze_point.clone();
        self.gaze.timestamp = timestamp_ms;
        
        if target != self.target_tab {
            // Gaze moved to different tab or off tabs
            if let Some(old_target) = self.target_tab {
                // Fire gaze exit event for old target
                self.gaze.dwell_start = None;
            }
            
            self.target_tab = target;
            
            if let Some(new_target) = target {
                // Start dwell timer for new target
                self.gaze.dwell_start = Some(timestamp_ms);
                return Some(InteractionEvent::GazeEnter(new_target));
            }
        } else if let Some(tab_id) = target {
            // Still looking at same tab
            if let Some(dwell_start) = self.gaze.dwell_start {
                let dwell_duration = timestamp_ms.saturating_sub(dwell_start);
                
                // Check if dwell completed
                if dwell_duration >= self.dwell_config.select_time_ms as u64 
                    && !self.gaze.dwell_completed {
                    self.gaze.dwell_completed = true;
                    return Some(InteractionEvent::DwellSelect(tab_id));
                }
                
                // Fire progress event
                if dwell_duration < self.dwell_config.select_time_ms as u64 {
                    let progress = dwell_duration as f32 / self.dwell_config.select_time_ms as f32;
                    return Some(InteractionEvent::DwellProgress { tab_id, progress });
                }
            }
            
            // Update cursor position within tab
            if let Some(tab) = tabs.iter().find(|t| t.id == tab_id) {
                let local = tab.interaction_zone.to_local(gaze_point);
                return Some(InteractionEvent::CursorMove { tab_id, position: local });
            }
        }
        
        None
    }
    
    /// Process voice command for tabs
    pub fn on_voice(&mut self, command: &str) -> Option<InteractionEvent> {
        let cmd = VoiceTabCommand::parse(command)?;
        
        // Add to history
        self.voice_history.push_front(cmd.clone());
        if self.voice_history.len() > 100 {
            self.voice_history.pop_back();
        }
        
        Some(cmd.to_interaction_event(self.target_tab))
    }
    
    /// Process gesture
    pub fn on_gesture(&mut self, gesture: TabGesture, timestamp_ms: u64) -> Option<InteractionEvent> {
        let event = GestureEvent {
            gesture: gesture.clone(),
            timestamp: timestamp_ms,
            target_tab: self.target_tab,
        };
        
        // Add to history
        self.gesture_history.push_front(event);
        if self.gesture_history.len() > 100 {
            self.gesture_history.pop_back();
        }
        
        match gesture {
            TabGesture::Pinch { scale } => {
                self.target_tab.map(|id| InteractionEvent::Resize { tab_id: id, scale })
            }
            TabGesture::Swipe { direction, velocity } => {
                self.target_tab.map(|id| InteractionEvent::SwipeNav { 
                    tab_id: id, 
                    direction, 
                    velocity 
                })
            }
            TabGesture::Tap => {
                self.target_tab.map(|id| InteractionEvent::Click(id))
            }
            TabGesture::DoubleTap => {
                self.target_tab.map(|id| InteractionEvent::DoubleClick(id))
            }
            TabGesture::LongPress { duration_ms } => {
                self.target_tab.map(|id| InteractionEvent::LongPress { 
                    tab_id: id, 
                    duration_ms 
                })
            }
            TabGesture::Drag { delta } => {
                self.target_tab.map(|id| InteractionEvent::Drag { 
                    tab_id: id, 
                    delta 
                })
            }
            TabGesture::TwoFingerScroll { delta } => {
                self.target_tab.map(|id| InteractionEvent::Scroll { 
                    tab_id: id, 
                    direction: if delta.1 > 0.0 { 
                        super::browser::ScrollDirection::Down 
                    } else { 
                        super::browser::ScrollDirection::Up 
                    },
                    amount: delta.1.abs(),
                })
            }
            TabGesture::Rotate { angle } => {
                self.target_tab.map(|id| InteractionEvent::Rotate { 
                    tab_id: id, 
                    angle 
                })
            }
            TabGesture::Grab => {
                self.target_tab.map(|id| InteractionEvent::GrabStart(id))
            }
            TabGesture::Release => {
                self.target_tab.map(|id| InteractionEvent::GrabEnd(id))
            }
            TabGesture::Dismiss => {
                self.target_tab.map(InteractionEvent::Minimize)
            }
        }
    }
    
    /// Reset dwell timer (e.g., after click)
    pub fn reset_dwell(&mut self) {
        self.gaze.dwell_start = None;
        self.gaze.dwell_completed = false;
    }
    
    /// Get current target tab
    pub fn target(&self) -> Option<TabId> {
        self.target_tab
    }
    
    /// Get gaze state
    pub fn gaze_state(&self) -> &GazeState {
        &self.gaze
    }
    
    /// Check if currently dwelling
    pub fn is_dwelling(&self) -> bool {
        self.gaze.dwell_start.is_some() && !self.gaze.dwell_completed
    }
    
    /// Get dwell progress (0.0 - 1.0)
    pub fn dwell_progress(&self, current_time_ms: u64) -> Option<f32> {
        self.gaze.dwell_start.map(|start| {
            let elapsed = current_time_ms.saturating_sub(start);
            (elapsed as f32 / self.dwell_config.select_time_ms as f32).min(1.0)
        })
    }
}

impl Default for TabInteraction {
    fn default() -> Self {
        Self::new()
    }
}

/// Gaze state
#[derive(Debug, Clone, Default)]
pub struct GazeState {
    /// Current gaze position
    pub position: WorldPosition,
    /// Timestamp
    pub timestamp: u64,
    /// When dwell started
    pub dwell_start: Option<u64>,
    /// Whether dwell selection completed
    pub dwell_completed: bool,
    /// Gaze velocity for smoothing
    pub velocity: (f32, f32, f32),
    /// Confidence (0.0 - 1.0)
    pub confidence: f32,
}

/// Dwell configuration
#[derive(Debug, Clone)]
pub struct DwellConfig {
    /// Time to dwell for selection (ms)
    pub select_time_ms: u32,
    /// Radius within which dwell is maintained (meters)
    pub dwell_radius_m: f32,
    /// Whether to show progress indicator
    pub show_progress: bool,
    /// Haptic feedback on selection
    pub haptic_on_select: bool,
}

impl Default for DwellConfig {
    fn default() -> Self {
        Self {
            select_time_ms: 500,
            dwell_radius_m: 0.05,
            show_progress: true,
            haptic_on_select: true,
        }
    }
}

impl DwellConfig {
    /// Fast dwell (300ms)
    pub fn fast() -> Self {
        Self {
            select_time_ms: 300,
            ..Self::default()
        }
    }
    
    /// Slow dwell (800ms, less accidental triggers)
    pub fn slow() -> Self {
        Self {
            select_time_ms: 800,
            ..Self::default()
        }
    }
    
    /// Disable dwell selection
    pub fn disabled() -> Self {
        Self {
            select_time_ms: u32::MAX,
            show_progress: false,
            haptic_on_select: false,
            ..Self::default()
        }
    }
}

/// Interaction events
#[derive(Debug, Clone)]
pub enum InteractionEvent {
    /// Gaze entered a tab
    GazeEnter(TabId),
    /// Gaze exited a tab
    GazeExit(TabId),
    /// Cursor moved within tab
    CursorMove { tab_id: TabId, position: (f32, f32) },
    /// Dwell selection in progress
    DwellProgress { tab_id: TabId, progress: f32 },
    /// Dwell selection completed
    DwellSelect(TabId),
    /// Click on tab
    Click(TabId),
    /// Double click on tab
    DoubleClick(TabId),
    /// Long press on tab
    LongPress { tab_id: TabId, duration_ms: u64 },
    /// Scroll within tab
    Scroll { tab_id: TabId, direction: super::browser::ScrollDirection, amount: f32 },
    /// Resize tab
    Resize { tab_id: TabId, scale: f32 },
    /// Swipe navigation
    SwipeNav { tab_id: TabId, direction: SwipeDirection, velocity: f32 },
    /// Drag tab
    Drag { tab_id: TabId, delta: (f32, f32, f32) },
    /// Rotate tab
    Rotate { tab_id: TabId, angle: f32 },
    /// Started grabbing tab
    GrabStart(TabId),
    /// Released grabbed tab
    GrabEnd(TabId),
    /// Close tab
    Close(TabId),
    /// Minimize tab
    Minimize(TabId),
    /// Maximize/restore tab
    Maximize(TabId),
    /// Focus tab
    Focus(TabId),
    /// Switch to next tab
    NextTab,
    /// Switch to previous tab
    PrevTab,
    /// Close all tabs
    CloseAll,
    /// Voice input started
    VoiceInputStart,
    /// Voice input ended
    VoiceInputEnd(String),
    /// Navigate to URL
    Navigate(String),
    /// Go back
    GoBack,
    /// Go forward
    GoForward,
    /// Reload
    Reload,
}

/// Swipe direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Voice commands for tabs
#[derive(Debug, Clone)]
pub enum VoiceTabCommand {
    /// Scroll the page
    Scroll { direction: super::browser::ScrollDirection, amount: super::browser::ScrollAmount },
    /// Click at current cursor
    Click,
    /// Close current tab
    Close,
    /// Minimize current tab
    Minimize,
    /// Maximize current tab
    Maximize,
    /// Navigate to URL
    Navigate(String),
    /// Go back
    GoBack,
    /// Go forward
    GoForward,
    /// Reload page
    Reload,
    /// Zoom in
    ZoomIn,
    /// Zoom out
    ZoomOut,
    /// Find text
    Find(String),
    /// Read page content
    ReadPage,
    /// Switch tab
    SwitchTab(String),
    /// New tab
    NewTab(Option<String>),
    /// Next tab
    NextTab,
    /// Previous tab
    PrevTab,
    /// Close all tabs
    CloseAll,
    /// Show tabs
    ShowTabs,
}

impl VoiceTabCommand {
    /// Parse a voice command string
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.to_lowercase();
        let words: Vec<&str> = input.split_whitespace().collect();
        
        if words.is_empty() {
            return None;
        }
        
        match words.as_slice() {
            // Scrolling
            ["scroll", "down"] | ["page", "down"] => {
                Some(Self::Scroll { 
                    direction: super::browser::ScrollDirection::Down, 
                    amount: super::browser::ScrollAmount::Page 
                })
            }
            ["scroll", "up"] | ["page", "up"] => {
                Some(Self::Scroll { 
                    direction: super::browser::ScrollDirection::Up, 
                    amount: super::browser::ScrollAmount::Page 
                })
            }
            ["scroll", "down", "a", "bit"] | ["down", "a", "bit"] => {
                Some(Self::Scroll { 
                    direction: super::browser::ScrollDirection::Down, 
                    amount: super::browser::ScrollAmount::HalfPage 
                })
            }
            ["scroll", "up", "a", "bit"] | ["up", "a", "bit"] => {
                Some(Self::Scroll { 
                    direction: super::browser::ScrollDirection::Up, 
                    amount: super::browser::ScrollAmount::HalfPage 
                })
            }
            ["scroll", "to", "top"] | ["go", "to", "top"] | ["top"] => {
                Some(Self::Scroll { 
                    direction: super::browser::ScrollDirection::Up, 
                    amount: super::browser::ScrollAmount::ToEnd 
                })
            }
            ["scroll", "to", "bottom"] | ["go", "to", "bottom"] | ["bottom"] => {
                Some(Self::Scroll { 
                    direction: super::browser::ScrollDirection::Down, 
                    amount: super::browser::ScrollAmount::ToEnd 
                })
            }
            
            // Click
            ["click"] | ["select"] | ["tap"] => Some(Self::Click),
            
            // Tab management
            ["close", "tab"] | ["close", "this"] | ["close"] => Some(Self::Close),
            ["minimize"] | ["minimize", "tab"] => Some(Self::Minimize),
            ["maximize"] | ["maximize", "tab"] | ["fullscreen"] => Some(Self::Maximize),
            ["new", "tab"] => Some(Self::NewTab(None)),
            ["next", "tab"] | ["switch", "tab"] => Some(Self::NextTab),
            ["previous", "tab"] | ["prev", "tab"] => Some(Self::PrevTab),
            ["close", "all", "tabs"] | ["close", "all"] => Some(Self::CloseAll),
            ["show", "tabs"] | ["list", "tabs"] | ["my", "tabs"] => Some(Self::ShowTabs),
            
            // Navigation
            ["go", "back"] | ["back"] => Some(Self::GoBack),
            ["go", "forward"] | ["forward"] => Some(Self::GoForward),
            ["reload"] | ["refresh"] => Some(Self::Reload),
            
            // Zoom
            ["zoom", "in"] | ["bigger"] => Some(Self::ZoomIn),
            ["zoom", "out"] | ["smaller"] => Some(Self::ZoomOut),
            
            // Read
            ["read", "page"] | ["read", "this"] | ["read", "aloud"] => Some(Self::ReadPage),
            
            // Navigate to URL or search
            _ => {
                // Check for "go to <url>" or "open <url>"
                if words.len() >= 2 {
                    match words[0] {
                        "go" | "goto" | "open" | "navigate" => {
                            let rest = words[1..].join(" ");
                            if rest.contains('.') || rest.starts_with("http") {
                                Some(Self::Navigate(rest))
                            } else {
                                // Treat as search
                                Some(Self::Navigate(format!("https://google.com/search?q={}", rest.replace(' ', "+"))))
                            }
                        }
                        "find" | "search" => {
                            let query = words[1..].join(" ");
                            Some(Self::Find(query))
                        }
                        "switch" => {
                            let name = words[1..].join(" ");
                            Some(Self::SwitchTab(name))
                        }
                        "new" if words.len() >= 2 && words[1] == "tab" => {
                            if words.len() > 2 {
                                let url = words[2..].join(" ");
                                Some(Self::NewTab(Some(url)))
                            } else {
                                Some(Self::NewTab(None))
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
        }
    }
    
    /// Convert to interaction event
    pub fn to_interaction_event(&self, target_tab: Option<TabId>) -> InteractionEvent {
        match self {
            Self::Scroll { direction, .. } => {
                if let Some(id) = target_tab {
                    InteractionEvent::Scroll { 
                        tab_id: id, 
                        direction: *direction,
                        amount: 0.5,
                    }
                } else {
                    InteractionEvent::VoiceInputEnd("No tab focused".to_string())
                }
            }
            Self::Click => {
                target_tab.map(InteractionEvent::Click)
                    .unwrap_or(InteractionEvent::VoiceInputEnd("No tab focused".to_string()))
            }
            Self::Close => {
                target_tab.map(InteractionEvent::Close)
                    .unwrap_or(InteractionEvent::VoiceInputEnd("No tab to close".to_string()))
            }
            Self::Minimize => {
                target_tab.map(InteractionEvent::Minimize)
                    .unwrap_or(InteractionEvent::VoiceInputEnd("No tab to minimize".to_string()))
            }
            Self::Maximize => {
                target_tab.map(InteractionEvent::Maximize)
                    .unwrap_or(InteractionEvent::VoiceInputEnd("No tab to maximize".to_string()))
            }
            Self::Navigate(url) => InteractionEvent::Navigate(url.clone()),
            Self::GoBack => InteractionEvent::GoBack,
            Self::GoForward => InteractionEvent::GoForward,
            Self::Reload => InteractionEvent::Reload,
            Self::ZoomIn | Self::ZoomOut => {
                let scale = if matches!(self, Self::ZoomIn) { 1.25 } else { 0.8 };
                target_tab.map(|id| InteractionEvent::Resize { tab_id: id, scale })
                    .unwrap_or(InteractionEvent::VoiceInputEnd("No tab focused".to_string()))
            }
            Self::NextTab => InteractionEvent::NextTab,
            Self::PrevTab => InteractionEvent::PrevTab,
            Self::CloseAll => InteractionEvent::CloseAll,
            Self::NewTab(url) => {
                InteractionEvent::Navigate(url.clone().unwrap_or_else(|| "about:blank".to_string()))
            }
            Self::Find(query) => {
                InteractionEvent::VoiceInputEnd(format!("Find: {}", query))
            }
            Self::ReadPage => {
                InteractionEvent::VoiceInputEnd("Reading page...".to_string())
            }
            Self::SwitchTab(name) => {
                InteractionEvent::VoiceInputEnd(format!("Switch to: {}", name))
            }
            Self::ShowTabs => {
                InteractionEvent::VoiceInputEnd("Showing tabs...".to_string())
            }
        }
    }
}

/// Gesture types
#[derive(Debug, Clone)]
pub enum GestureType {
    /// Pinch to resize
    Pinch { scale: f32 },
    /// Swipe gesture
    Swipe { direction: SwipeDirection },
    /// Tap gesture
    Tap,
    /// Long press
    LongPress,
}

/// Tab gestures (more detailed than generic GestureType)
#[derive(Debug, Clone)]
pub enum TabGesture {
    /// Pinch to resize
    Pinch { scale: f32 },
    /// Swipe for navigation
    Swipe { direction: SwipeDirection, velocity: f32 },
    /// Single tap
    Tap,
    /// Double tap
    DoubleTap,
    /// Long press
    LongPress { duration_ms: u64 },
    /// Drag/move
    Drag { delta: (f32, f32, f32) },
    /// Two-finger scroll
    TwoFingerScroll { delta: (f32, f32) },
    /// Rotate gesture
    Rotate { angle: f32 },
    /// Grab gesture (hand close)
    Grab,
    /// Release gesture (hand open)
    Release,
    /// Dismiss gesture (wave away)
    Dismiss,
}

/// Gesture event with metadata
#[derive(Debug, Clone)]
pub struct GestureEvent {
    pub gesture: TabGesture,
    pub timestamp: u64,
    pub target_tab: Option<TabId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::{SpatialAnchor, WorldPosition, AnchorContent, AnchorState, Quaternion};

    fn create_test_tab(x: f32, y: f32, z: f32) -> ARTab {
        use super::super::tab::{ARTab, TabContent};
        
        let anchor = SpatialAnchor {
            id: 1,
            position: WorldPosition::from_local(x, y, z),
            orientation: Quaternion::identity(),
            visual_signature: [0u8; 32],
            content_hash: [0u8; 32],
            content: AnchorContent::Text { text: "test".to_string() },
            state: AnchorState::Active,
            confidence: 1.0,
            created_at: 0,
            updated_at: 0,
            owner_did: None,
            label: None,
        };
        ARTab::new(TabContent::browser("https://test.com"), anchor)
    }

    #[test]
    fn test_interaction_creation() {
        let interaction = TabInteraction::new();
        assert!(interaction.target().is_none());
        assert!(!interaction.is_dwelling());
    }

    #[test]
    fn test_gaze_enter() {
        let mut interaction = TabInteraction::new();
        let tab = create_test_tab(0.0, 1.5, 2.0);
        let tabs = vec![&tab];
        
        // Gaze at tab
        let gaze_pos = WorldPosition::from_local(0.0, 1.5, 2.0);
        let event = interaction.on_gaze(&gaze_pos, &tabs, 1000);
        
        assert!(matches!(event, Some(InteractionEvent::GazeEnter(_))));
        assert_eq!(interaction.target(), Some(tab.id));
    }

    #[test]
    fn test_dwell_selection() {
        let mut interaction = TabInteraction::with_dwell_config(DwellConfig {
            select_time_ms: 100,
            ..DwellConfig::default()
        });
        
        let tab = create_test_tab(0.0, 1.5, 2.0);
        let tabs = vec![&tab];
        let gaze_pos = WorldPosition::from_local(0.0, 1.5, 2.0);
        
        // First gaze - enters
        interaction.on_gaze(&gaze_pos, &tabs, 0);
        assert!(interaction.is_dwelling());
        
        // Progress at 50%
        let event = interaction.on_gaze(&gaze_pos, &tabs, 50);
        assert!(matches!(event, Some(InteractionEvent::DwellProgress { progress, .. }) if progress < 1.0));
        
        // Dwell complete at 100ms
        let event = interaction.on_gaze(&gaze_pos, &tabs, 100);
        assert!(matches!(event, Some(InteractionEvent::DwellSelect(_))));
    }

    #[test]
    fn test_cursor_move() {
        let mut interaction = TabInteraction::new();
        let tab = create_test_tab(0.0, 1.5, 2.0);
        let tabs = vec![&tab];
        
        // Enter tab
        let gaze_pos = WorldPosition::from_local(0.0, 1.5, 2.0);
        interaction.on_gaze(&gaze_pos, &tabs, 0);
        
        // Mark dwell complete to get cursor events
        interaction.gaze.dwell_completed = true;
        
        // Move within tab - use timestamp > dwell select time (500ms default)
        let gaze_pos2 = WorldPosition::from_local(0.1, 1.6, 2.0);
        let event = interaction.on_gaze(&gaze_pos2, &tabs, 600);
        
        assert!(matches!(event, Some(InteractionEvent::CursorMove { .. })));
    }

    #[test]
    fn test_voice_command_parsing() {
        // Scroll commands
        assert!(matches!(
            VoiceTabCommand::parse("scroll down"),
            Some(VoiceTabCommand::Scroll { direction: super::super::browser::ScrollDirection::Down, .. })
        ));
        
        assert!(matches!(
            VoiceTabCommand::parse("page up"),
            Some(VoiceTabCommand::Scroll { direction: super::super::browser::ScrollDirection::Up, .. })
        ));
        
        // Tab commands
        assert!(matches!(VoiceTabCommand::parse("close tab"), Some(VoiceTabCommand::Close)));
        assert!(matches!(VoiceTabCommand::parse("minimize"), Some(VoiceTabCommand::Minimize)));
        assert!(matches!(VoiceTabCommand::parse("next tab"), Some(VoiceTabCommand::NextTab)));
        
        // Navigation
        assert!(matches!(VoiceTabCommand::parse("go back"), Some(VoiceTabCommand::GoBack)));
        assert!(matches!(VoiceTabCommand::parse("reload"), Some(VoiceTabCommand::Reload)));
        
        // URL navigation
        if let Some(VoiceTabCommand::Navigate(url)) = VoiceTabCommand::parse("go to google.com") {
            assert!(url.contains("google.com"));
        } else {
            panic!("Expected Navigate");
        }
        
        // Search
        if let Some(VoiceTabCommand::Navigate(url)) = VoiceTabCommand::parse("open weather forecast") {
            assert!(url.contains("google.com/search"));
        } else {
            panic!("Expected search Navigate");
        }
    }

    #[test]
    fn test_voice_command_to_event() {
        let tab_id = uuid::Uuid::new_v4();
        
        let cmd = VoiceTabCommand::Close;
        let event = cmd.to_interaction_event(Some(tab_id));
        assert!(matches!(event, InteractionEvent::Close(_)));
        
        let cmd = VoiceTabCommand::Close;
        let event = cmd.to_interaction_event(None);
        assert!(matches!(event, InteractionEvent::VoiceInputEnd(_)));
    }

    #[test]
    fn test_gesture_handling() {
        let mut interaction = TabInteraction::new();
        let tab = create_test_tab(0.0, 1.5, 2.0);
        let tabs = vec![&tab];
        
        // Set target
        let gaze_pos = WorldPosition::from_local(0.0, 1.5, 2.0);
        interaction.on_gaze(&gaze_pos, &tabs, 0);
        
        // Pinch gesture
        let event = interaction.on_gesture(TabGesture::Pinch { scale: 1.5 }, 100);
        assert!(matches!(event, Some(InteractionEvent::Resize { scale, .. }) if (scale - 1.5).abs() < 0.001));
        
        // Swipe gesture
        let event = interaction.on_gesture(
            TabGesture::Swipe { direction: SwipeDirection::Left, velocity: 1.0 }, 
            200
        );
        assert!(matches!(event, Some(InteractionEvent::SwipeNav { direction: SwipeDirection::Left, .. })));
        
        // Tap gesture
        let event = interaction.on_gesture(TabGesture::Tap, 300);
        assert!(matches!(event, Some(InteractionEvent::Click(_))));
    }

    #[test]
    fn test_dwell_config() {
        let default = DwellConfig::default();
        assert_eq!(default.select_time_ms, 500);
        
        let fast = DwellConfig::fast();
        assert_eq!(fast.select_time_ms, 300);
        
        let slow = DwellConfig::slow();
        assert_eq!(slow.select_time_ms, 800);
        
        let disabled = DwellConfig::disabled();
        assert_eq!(disabled.select_time_ms, u32::MAX);
    }

    #[test]
    fn test_dwell_progress() {
        let mut interaction = TabInteraction::with_dwell_config(DwellConfig {
            select_time_ms: 1000,
            ..DwellConfig::default()
        });
        
        let tab = create_test_tab(0.0, 1.5, 2.0);
        let tabs = vec![&tab];
        let gaze_pos = WorldPosition::from_local(0.0, 1.5, 2.0);
        
        // Start dwelling
        interaction.on_gaze(&gaze_pos, &tabs, 0);
        
        // Check progress at 50%
        let progress = interaction.dwell_progress(500).unwrap();
        assert!((progress - 0.5).abs() < 0.001);
        
        // Check progress at 100%
        let progress = interaction.dwell_progress(1000).unwrap();
        assert!((progress - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_reset_dwell() {
        let mut interaction = TabInteraction::new();
        let tab = create_test_tab(0.0, 1.5, 2.0);
        let tabs = vec![&tab];
        let gaze_pos = WorldPosition::from_local(0.0, 1.5, 2.0);
        
        // Start dwelling
        interaction.on_gaze(&gaze_pos, &tabs, 0);
        assert!(interaction.is_dwelling());
        
        // Reset
        interaction.reset_dwell();
        assert!(!interaction.is_dwelling());
        assert!(interaction.gaze.dwell_start.is_none());
    }
}
