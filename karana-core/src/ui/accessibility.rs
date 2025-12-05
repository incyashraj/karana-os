//! UI Accessibility System
//! 
//! Comprehensive accessibility support for AR interfaces:
//! - Screen reader support
//! - High contrast modes
//! - Motion reduction
//! - Voice control integration
//! - Haptic feedback

use std::collections::HashMap;

/// Accessibility state
#[derive(Debug, Clone, Default)]
pub struct AccessibilityState {
    /// Screen reader enabled
    pub screen_reader_enabled: bool,
    /// Reduce motion preference
    pub reduce_motion: bool,
    /// High contrast mode
    pub high_contrast: bool,
    /// Larger text scale
    pub text_scale: f32,
    /// Voice control enabled
    pub voice_control: bool,
    /// Haptic feedback enabled
    pub haptic_feedback: bool,
    /// Focus indicators visible
    pub show_focus: bool,
    /// Color blindness mode
    pub color_blindness_mode: ColorBlindnessMode,
    /// Current announcement queue
    announcements: Vec<Announcement>,
    /// Focus path for navigation
    focus_path: Vec<u64>,
}

impl AccessibilityState {
    pub fn new() -> Self {
        Self {
            text_scale: 1.0,
            show_focus: true,
            ..Default::default()
        }
    }

    /// Queue an announcement for screen readers
    pub fn announce(&mut self, message: impl Into<String>, priority: AnnouncementPriority) {
        self.announcements.push(Announcement {
            message: message.into(),
            priority,
        });
    }

    /// Get and clear pending announcements
    pub fn drain_announcements(&mut self) -> Vec<Announcement> {
        std::mem::take(&mut self.announcements)
    }

    /// Set focus path for spatial navigation
    pub fn set_focus_path(&mut self, path: Vec<u64>) {
        self.focus_path = path;
    }

    /// Get scaled text size
    pub fn scaled_text(&self, base_size: f32) -> f32 {
        base_size * self.text_scale
    }
}

/// Accessibility announcement
#[derive(Debug, Clone)]
pub struct Announcement {
    /// Message to announce
    pub message: String,
    /// Priority level
    pub priority: AnnouncementPriority,
}

/// Announcement priority
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnouncementPriority {
    /// Normal priority (queued)
    Normal,
    /// High priority (interrupts current)
    High,
    /// Assertive (immediate, interrupts everything)
    Assertive,
}

/// Color blindness simulation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorBlindnessMode {
    #[default]
    None,
    /// Red-green (most common)
    Protanopia,
    /// Red-green
    Deuteranopia,
    /// Blue-yellow
    Tritanopia,
    /// Complete color blindness
    Achromatopsia,
}

/// Accessibility info for a widget
#[derive(Debug, Clone)]
pub struct AccessibilityInfo {
    /// Role of the element
    pub role: AccessibilityRole,
    /// Accessible label
    pub label: String,
    /// Current value (for sliders, etc.)
    pub value: Option<String>,
    /// Usage hint
    pub hint: Option<String>,
    /// Can receive focus
    pub focusable: bool,
    /// Available actions
    pub actions: Vec<AccessibilityAction>,
}

impl Default for AccessibilityInfo {
    fn default() -> Self {
        Self {
            role: AccessibilityRole::None,
            label: String::new(),
            value: None,
            hint: None,
            focusable: false,
            actions: Vec::new(),
        }
    }
}

impl AccessibilityInfo {
    pub fn new(role: AccessibilityRole, label: impl Into<String>) -> Self {
        Self {
            role,
            label: label.into(),
            ..Default::default()
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn focusable(mut self) -> Self {
        self.focusable = true;
        self
    }

    pub fn with_actions(mut self, actions: Vec<AccessibilityAction>) -> Self {
        self.actions = actions;
        self
    }
}

/// Accessibility role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccessibilityRole {
    #[default]
    None,
    Button,
    Link,
    Checkbox,
    Radio,
    Switch,
    Slider,
    TextField,
    SearchField,
    Text,
    StaticText,
    Heading,
    Image,
    List,
    ListItem,
    Table,
    TableRow,
    TableCell,
    Tab,
    TabList,
    TabPanel,
    Dialog,
    Alert,
    AlertDialog,
    Menu,
    MenuItem,
    MenuBar,
    ProgressBar,
    ScrollView,
    Toolbar,
    NavigationBar,
    Group,
    Region,
    Application,
    Window,
    Popup,
    Tooltip,
}

/// Accessibility action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessibilityAction {
    Press,
    LongPress,
    Focus,
    SetValue,
    Increment,
    Decrement,
    Toggle,
    Expand,
    Collapse,
    Select,
    Dismiss,
    Scroll,
    ScrollToTop,
    ScrollToBottom,
    Copy,
    Paste,
    Delete,
    Custom(u32),
}

/// Accessibility node tree for screen readers
#[derive(Debug, Clone)]
pub struct AccessibilityTree {
    /// Root nodes
    pub roots: Vec<AccessibilityNode>,
    /// Node map by ID
    pub nodes: HashMap<u64, AccessibilityNode>,
    /// Current focused node
    pub focused: Option<u64>,
}

impl Default for AccessibilityTree {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessibilityTree {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            nodes: HashMap::new(),
            focused: None,
        }
    }

    /// Add a node to the tree
    pub fn add_node(&mut self, node: AccessibilityNode) {
        let id = node.id;
        if let Some(parent_id) = node.parent {
            if let Some(parent) = self.nodes.get_mut(&parent_id) {
                parent.children.push(id);
            }
        } else {
            self.roots.push(node.clone());
        }
        self.nodes.insert(id, node);
    }

    /// Get node by ID
    pub fn get_node(&self, id: u64) -> Option<&AccessibilityNode> {
        self.nodes.get(&id)
    }

    /// Find next focusable node
    pub fn next_focusable(&self, current: Option<u64>) -> Option<u64> {
        let mut focusable: Vec<u64> = self.nodes.values()
            .filter(|n| n.info.focusable)
            .map(|n| n.id)
            .collect();
        focusable.sort(); // Ensure deterministic order
        
        if focusable.is_empty() {
            return None;
        }

        match current {
            Some(curr) => {
                let pos = focusable.iter().position(|&id| id == curr);
                match pos {
                    Some(i) => Some(focusable[(i + 1) % focusable.len()]),
                    None => Some(focusable[0]),
                }
            }
            None => Some(focusable[0]),
        }
    }

    /// Find previous focusable node
    pub fn previous_focusable(&self, current: Option<u64>) -> Option<u64> {
        let mut focusable: Vec<u64> = self.nodes.values()
            .filter(|n| n.info.focusable)
            .map(|n| n.id)
            .collect();
        focusable.sort(); // Ensure deterministic order
        
        if focusable.is_empty() {
            return None;
        }

        match current {
            Some(curr) => {
                let pos = focusable.iter().position(|&id| id == curr);
                match pos {
                    Some(0) => Some(focusable[focusable.len() - 1]),
                    Some(i) => Some(focusable[i - 1]),
                    None => Some(focusable[focusable.len() - 1]),
                }
            }
            None => Some(focusable[focusable.len() - 1]),
        }
    }

    /// Generate description for current focus
    pub fn describe_focus(&self) -> Option<String> {
        self.focused
            .and_then(|id| self.nodes.get(&id))
            .map(|node| {
                let mut desc = node.info.label.clone();
                if let Some(value) = &node.info.value {
                    desc.push_str(&format!(", value: {}", value));
                }
                if let Some(hint) = &node.info.hint {
                    desc.push_str(&format!(". {}", hint));
                }
                desc
            })
    }
}

/// Accessibility node
#[derive(Debug, Clone)]
pub struct AccessibilityNode {
    /// Widget ID
    pub id: u64,
    /// Parent ID
    pub parent: Option<u64>,
    /// Children IDs
    pub children: Vec<u64>,
    /// Accessibility info
    pub info: AccessibilityInfo,
    /// Screen bounds
    pub bounds: crate::ui::Rect,
}

/// Haptic feedback patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HapticPattern {
    /// Light tap
    Light,
    /// Medium tap
    Medium,
    /// Heavy tap
    Heavy,
    /// Selection changed
    Selection,
    /// Error occurred
    Error,
    /// Success
    Success,
    /// Warning
    Warning,
    /// Custom pattern (intensity 0-100, duration ms)
    Custom { intensity: u8, duration_ms: u16 },
}

impl HapticPattern {
    /// Get intensity (0-100)
    pub fn intensity(&self) -> u8 {
        match self {
            HapticPattern::Light => 30,
            HapticPattern::Medium => 50,
            HapticPattern::Heavy => 80,
            HapticPattern::Selection => 40,
            HapticPattern::Error => 70,
            HapticPattern::Success => 60,
            HapticPattern::Warning => 65,
            HapticPattern::Custom { intensity, .. } => *intensity,
        }
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> u16 {
        match self {
            HapticPattern::Light => 10,
            HapticPattern::Medium => 20,
            HapticPattern::Heavy => 30,
            HapticPattern::Selection => 15,
            HapticPattern::Error => 100,
            HapticPattern::Success => 50,
            HapticPattern::Warning => 75,
            HapticPattern::Custom { duration_ms, .. } => *duration_ms,
        }
    }
}

/// Accessibility manager
#[derive(Debug)]
pub struct AccessibilityManager {
    /// Current state
    pub state: AccessibilityState,
    /// Accessibility tree
    pub tree: AccessibilityTree,
    /// Haptic feedback queue
    haptic_queue: Vec<HapticPattern>,
    /// Focus trap stack (for dialogs/modals)
    focus_traps: Vec<Vec<u64>>,
}

impl Default for AccessibilityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessibilityManager {
    pub fn new() -> Self {
        Self {
            state: AccessibilityState::new(),
            tree: AccessibilityTree::new(),
            haptic_queue: Vec::new(),
            focus_traps: Vec::new(),
        }
    }

    /// Enable screen reader support
    pub fn enable_screen_reader(&mut self) {
        self.state.screen_reader_enabled = true;
        self.state.show_focus = true;
    }

    /// Enable reduced motion
    pub fn enable_reduced_motion(&mut self) {
        self.state.reduce_motion = true;
    }

    /// Enable high contrast
    pub fn enable_high_contrast(&mut self) {
        self.state.high_contrast = true;
    }

    /// Set text scale
    pub fn set_text_scale(&mut self, scale: f32) {
        self.state.text_scale = scale.clamp(0.5, 3.0);
    }

    /// Queue announcement
    pub fn announce(&mut self, message: impl Into<String>) {
        self.state.announce(message, AnnouncementPriority::Normal);
    }

    /// Queue assertive announcement
    pub fn announce_assertive(&mut self, message: impl Into<String>) {
        self.state.announce(message, AnnouncementPriority::Assertive);
    }

    /// Request haptic feedback
    pub fn haptic(&mut self, pattern: HapticPattern) {
        if self.state.haptic_feedback {
            self.haptic_queue.push(pattern);
        }
    }

    /// Drain haptic queue
    pub fn drain_haptics(&mut self) -> Vec<HapticPattern> {
        std::mem::take(&mut self.haptic_queue)
    }

    /// Push focus trap (for modal dialogs)
    pub fn push_focus_trap(&mut self, focusable_ids: Vec<u64>) {
        self.focus_traps.push(focusable_ids);
    }

    /// Pop focus trap
    pub fn pop_focus_trap(&mut self) {
        self.focus_traps.pop();
    }

    /// Focus next element
    pub fn focus_next(&mut self) {
        if let Some(trap) = self.focus_traps.last() {
            // Within focus trap
            if let Some(current) = self.tree.focused {
                let pos = trap.iter().position(|&id| id == current);
                self.tree.focused = match pos {
                    Some(i) => Some(trap[(i + 1) % trap.len()]),
                    None => trap.first().copied(),
                };
            } else {
                self.tree.focused = trap.first().copied();
            }
        } else {
            // Normal navigation
            self.tree.focused = self.tree.next_focusable(self.tree.focused);
        }

        // Announce focus change
        if let Some(desc) = self.tree.describe_focus() {
            self.announce(desc);
        }
    }

    /// Focus previous element
    pub fn focus_previous(&mut self) {
        if let Some(trap) = self.focus_traps.last() {
            // Within focus trap
            if let Some(current) = self.tree.focused {
                let pos = trap.iter().position(|&id| id == current);
                self.tree.focused = match pos {
                    Some(0) => Some(trap[trap.len() - 1]),
                    Some(i) => Some(trap[i - 1]),
                    None => trap.last().copied(),
                };
            } else {
                self.tree.focused = trap.last().copied();
            }
        } else {
            // Normal navigation
            self.tree.focused = self.tree.previous_focusable(self.tree.focused);
        }

        // Announce focus change
        if let Some(desc) = self.tree.describe_focus() {
            self.announce(desc);
        }
    }

    /// Get animation duration scale (for reduced motion)
    pub fn animation_scale(&self) -> f32 {
        if self.state.reduce_motion {
            0.0 // Instant animations
        } else {
            1.0
        }
    }
}

/// High contrast color adjustments
pub fn high_contrast_color(color: crate::ui::Color, is_background: bool) -> crate::ui::Color {
    use crate::ui::Color;
    
    // Calculate luminance
    let luminance = 0.299 * color.r as f32 / 255.0 
        + 0.587 * color.g as f32 / 255.0 
        + 0.114 * color.b as f32 / 255.0;
    
    if is_background {
        // Make backgrounds either pure black or white
        if luminance > 0.5 {
            Color::WHITE
        } else {
            Color::BLACK
        }
    } else {
        // Make foregrounds opposite of background
        if luminance > 0.5 {
            Color::BLACK
        } else {
            Color::WHITE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessibility_state() {
        let mut state = AccessibilityState::new();
        assert!(!state.screen_reader_enabled);
        assert!((state.text_scale - 1.0).abs() < 0.001);

        state.announce("Test", AnnouncementPriority::Normal);
        let announcements = state.drain_announcements();
        assert_eq!(announcements.len(), 1);
        assert_eq!(announcements[0].message, "Test");
    }

    #[test]
    fn test_text_scale() {
        let state = AccessibilityState { text_scale: 1.5, ..Default::default() };
        assert!((state.scaled_text(16.0) - 24.0).abs() < 0.001);
    }

    #[test]
    fn test_accessibility_info() {
        let info = AccessibilityInfo::new(AccessibilityRole::Button, "Submit")
            .with_hint("Double tap to submit form")
            .focusable();

        assert_eq!(info.role, AccessibilityRole::Button);
        assert_eq!(info.label, "Submit");
        assert!(info.focusable);
    }

    #[test]
    fn test_accessibility_tree_navigation() {
        let mut tree = AccessibilityTree::new();
        
        tree.add_node(AccessibilityNode {
            id: 1,
            parent: None,
            children: vec![],
            info: AccessibilityInfo::new(AccessibilityRole::Button, "First").focusable(),
            bounds: crate::ui::Rect::zero(),
        });
        
        tree.add_node(AccessibilityNode {
            id: 2,
            parent: None,
            children: vec![],
            info: AccessibilityInfo::new(AccessibilityRole::Button, "Second").focusable(),
            bounds: crate::ui::Rect::zero(),
        });

        assert_eq!(tree.next_focusable(None), Some(1));
        assert_eq!(tree.next_focusable(Some(1)), Some(2));
        assert_eq!(tree.next_focusable(Some(2)), Some(1));
    }

    #[test]
    fn test_haptic_pattern() {
        assert!(HapticPattern::Light.intensity() < HapticPattern::Heavy.intensity());
        assert!(HapticPattern::Light.duration_ms() < HapticPattern::Heavy.duration_ms());
    }

    #[test]
    fn test_accessibility_manager() {
        let mut manager = AccessibilityManager::new();
        manager.enable_screen_reader();
        assert!(manager.state.screen_reader_enabled);

        manager.set_text_scale(2.0);
        assert!((manager.state.text_scale - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_animation_scale() {
        let mut manager = AccessibilityManager::new();
        assert!((manager.animation_scale() - 1.0).abs() < 0.001);

        manager.enable_reduced_motion();
        assert!((manager.animation_scale() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_focus_trap() {
        let mut manager = AccessibilityManager::new();
        manager.push_focus_trap(vec![10, 20, 30]);
        
        manager.tree.focused = Some(10);
        manager.focus_next();
        assert_eq!(manager.tree.focused, Some(20));

        manager.pop_focus_trap();
    }
}
