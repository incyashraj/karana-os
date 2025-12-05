//! Accessibility Features for Voice
//!
//! Voice-based accessibility support for smart glasses.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Voice accessibility manager
pub struct VoiceAccessibility {
    /// Configuration
    config: AccessibilityConfig,
    /// Screen reader
    screen_reader: ScreenReader,
    /// Navigation assistance
    nav_assist: NavigationAssist,
    /// User adaptations
    adaptations: UserAdaptations,
}

impl VoiceAccessibility {
    /// Create new accessibility manager
    pub fn new(config: AccessibilityConfig) -> Self {
        Self {
            screen_reader: ScreenReader::new(&config),
            nav_assist: NavigationAssist::new(),
            adaptations: UserAdaptations::default(),
            config,
        }
    }

    /// Read current focus
    pub fn read_focus(&self) -> String {
        self.screen_reader.read_current_focus()
    }

    /// Read element
    pub fn read_element(&self, element: &AccessibleElement) -> String {
        self.screen_reader.read_element(element)
    }

    /// Navigate to next element
    pub fn navigate_next(&mut self) -> Option<AccessibleElement> {
        self.nav_assist.next()
    }

    /// Navigate to previous element
    pub fn navigate_previous(&mut self) -> Option<AccessibleElement> {
        self.nav_assist.previous()
    }

    /// Navigate by type
    pub fn navigate_by_type(&mut self, element_type: ElementType) -> Option<AccessibleElement> {
        self.nav_assist.navigate_to_type(element_type)
    }

    /// Set focus
    pub fn set_focus(&mut self, element: AccessibleElement) {
        self.nav_assist.set_focus(element);
    }

    /// Apply voice adaptation
    pub fn apply_adaptation(&mut self, adaptation: VoiceAdaptation) {
        match &adaptation {
            VoiceAdaptation::SlowerSpeech => {
                self.config.speech_rate = 0.7;
            }
            VoiceAdaptation::LouderSpeech => {
                self.config.volume = 1.0;
            }
            VoiceAdaptation::SimplifiedCommands => {
                self.config.simplified_mode = true;
            }
            VoiceAdaptation::ExtendedTimeout => {
                self.config.timeout_multiplier = 2.0;
            }
            VoiceAdaptation::RepeatConfirmation => {
                self.config.repeat_confirmations = true;
            }
            VoiceAdaptation::Custom(params) => {
                for (key, value) in params {
                    self.adaptations.custom.insert(key.clone(), value.clone());
                }
            }
        }
        self.adaptations.active.push(adaptation);
    }

    /// Get configuration
    pub fn config(&self) -> &AccessibilityConfig {
        &self.config
    }

    /// Get adapted timeout
    pub fn adapted_timeout(&self, base_ms: u32) -> u32 {
        (base_ms as f32 * self.config.timeout_multiplier) as u32
    }

    /// Generate help for current context
    pub fn context_help(&self) -> Vec<HelpItem> {
        let mut help = Vec::new();

        help.push(HelpItem {
            command: "read".to_string(),
            description: "Read current focus".to_string(),
            example: "Read this".to_string(),
        });

        help.push(HelpItem {
            command: "next".to_string(),
            description: "Go to next element".to_string(),
            example: "Next item".to_string(),
        });

        help.push(HelpItem {
            command: "previous".to_string(),
            description: "Go to previous element".to_string(),
            example: "Previous item".to_string(),
        });

        help.push(HelpItem {
            command: "select".to_string(),
            description: "Activate current element".to_string(),
            example: "Select this".to_string(),
        });

        help.push(HelpItem {
            command: "help".to_string(),
            description: "Get help".to_string(),
            example: "What can I say".to_string(),
        });

        help
    }
}

impl Default for VoiceAccessibility {
    fn default() -> Self {
        Self::new(AccessibilityConfig::default())
    }
}

/// Accessibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    /// Speech rate (0.5 - 2.0)
    pub speech_rate: f32,
    /// Volume (0.0 - 1.0)
    pub volume: f32,
    /// Simplified command mode
    pub simplified_mode: bool,
    /// Timeout multiplier
    pub timeout_multiplier: f32,
    /// Repeat confirmations
    pub repeat_confirmations: bool,
    /// Verbosity level
    pub verbosity: VerbosityLevel,
    /// Announce navigation
    pub announce_navigation: bool,
    /// Spatial audio enabled
    pub spatial_audio: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            speech_rate: 1.0,
            volume: 0.8,
            simplified_mode: false,
            timeout_multiplier: 1.0,
            repeat_confirmations: false,
            verbosity: VerbosityLevel::Normal,
            announce_navigation: true,
            spatial_audio: true,
        }
    }
}

/// Verbosity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerbosityLevel {
    /// Minimal feedback
    Minimal,
    /// Normal feedback
    Normal,
    /// Detailed feedback
    Verbose,
    /// Full descriptions
    Full,
}

/// Screen reader
pub struct ScreenReader {
    /// Current focus
    current_focus: Option<AccessibleElement>,
    /// Reading queue
    queue: Vec<ReadingItem>,
    /// Configuration
    config: ScreenReaderConfig,
}

impl ScreenReader {
    /// Create new screen reader
    pub fn new(config: &AccessibilityConfig) -> Self {
        Self {
            current_focus: None,
            queue: Vec::new(),
            config: ScreenReaderConfig {
                verbosity: config.verbosity,
                include_hints: true,
                include_shortcuts: true,
            },
        }
    }

    /// Read current focus
    pub fn read_current_focus(&self) -> String {
        if let Some(element) = &self.current_focus {
            self.read_element(element)
        } else {
            "No focus".to_string()
        }
    }

    /// Read element
    pub fn read_element(&self, element: &AccessibleElement) -> String {
        let mut parts = Vec::new();

        // Element type
        parts.push(format!("{:?}", element.element_type));

        // Label/name
        if let Some(label) = &element.label {
            parts.push(label.clone());
        }

        // Value
        if let Some(value) = &element.value {
            parts.push(value.clone());
        }

        // State
        if element.selected {
            parts.push("Selected".to_string());
        }
        if element.disabled {
            parts.push("Disabled".to_string());
        }

        // Role hint
        if self.config.include_hints {
            if let Some(hint) = &element.hint {
                parts.push(hint.clone());
            }
        }

        // Shortcut
        if self.config.include_shortcuts {
            if let Some(shortcut) = &element.shortcut {
                parts.push(format!("Shortcut: {}", shortcut));
            }
        }

        parts.join(". ")
    }

    /// Set focus
    pub fn set_focus(&mut self, element: AccessibleElement) {
        self.current_focus = Some(element);
    }

    /// Queue reading
    pub fn queue_reading(&mut self, text: &str, priority: ReadingPriority) {
        self.queue.push(ReadingItem {
            text: text.to_string(),
            priority,
        });
        self.queue.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Get next reading item
    pub fn next_reading(&mut self) -> Option<String> {
        self.queue.pop().map(|item| item.text)
    }
}

/// Screen reader configuration
struct ScreenReaderConfig {
    verbosity: VerbosityLevel,
    include_hints: bool,
    include_shortcuts: bool,
}

/// Reading item
struct ReadingItem {
    text: String,
    priority: ReadingPriority,
}

/// Reading priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReadingPriority {
    Low,
    Normal,
    High,
    Immediate,
}

/// Accessible element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibleElement {
    /// Element ID
    pub id: String,
    /// Element type
    pub element_type: ElementType,
    /// Label/name
    pub label: Option<String>,
    /// Current value
    pub value: Option<String>,
    /// Hint/description
    pub hint: Option<String>,
    /// Keyboard shortcut
    pub shortcut: Option<String>,
    /// Is selected
    pub selected: bool,
    /// Is disabled
    pub disabled: bool,
    /// Position in space (for spatial audio)
    pub position: Option<(f32, f32, f32)>,
    /// Children
    pub children: Vec<String>,
}

impl Default for AccessibleElement {
    fn default() -> Self {
        Self {
            id: String::new(),
            element_type: ElementType::Unknown,
            label: None,
            value: None,
            hint: None,
            shortcut: None,
            selected: false,
            disabled: false,
            position: None,
            children: Vec::new(),
        }
    }
}

/// Element type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementType {
    Button,
    Link,
    TextField,
    Checkbox,
    RadioButton,
    Slider,
    List,
    ListItem,
    Menu,
    MenuItem,
    Tab,
    Window,
    Dialog,
    Alert,
    Image,
    Text,
    Heading,
    Landmark,
    Unknown,
}

/// Navigation assistance
pub struct NavigationAssist {
    /// Element tree
    elements: Vec<AccessibleElement>,
    /// Current index
    current_index: usize,
    /// Navigation history
    history: Vec<usize>,
}

impl NavigationAssist {
    /// Create new navigation assist
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            current_index: 0,
            history: Vec::new(),
        }
    }

    /// Set elements
    pub fn set_elements(&mut self, elements: Vec<AccessibleElement>) {
        self.elements = elements;
        self.current_index = 0;
        self.history.clear();
    }

    /// Get current element
    pub fn current(&self) -> Option<&AccessibleElement> {
        self.elements.get(self.current_index)
    }

    /// Navigate to next
    pub fn next(&mut self) -> Option<AccessibleElement> {
        if self.elements.is_empty() {
            return None;
        }

        self.history.push(self.current_index);
        self.current_index = (self.current_index + 1) % self.elements.len();
        self.elements.get(self.current_index).cloned()
    }

    /// Navigate to previous
    pub fn previous(&mut self) -> Option<AccessibleElement> {
        if self.elements.is_empty() {
            return None;
        }

        self.history.push(self.current_index);
        self.current_index = if self.current_index == 0 {
            self.elements.len() - 1
        } else {
            self.current_index - 1
        };
        self.elements.get(self.current_index).cloned()
    }

    /// Navigate to element type
    pub fn navigate_to_type(&mut self, element_type: ElementType) -> Option<AccessibleElement> {
        let start = self.current_index;
        let mut idx = (start + 1) % self.elements.len();

        while idx != start {
            if self.elements[idx].element_type == element_type {
                self.history.push(self.current_index);
                self.current_index = idx;
                return self.elements.get(idx).cloned();
            }
            idx = (idx + 1) % self.elements.len();
        }

        None
    }

    /// Set focus to element
    pub fn set_focus(&mut self, element: AccessibleElement) {
        if let Some(idx) = self.elements.iter().position(|e| e.id == element.id) {
            self.history.push(self.current_index);
            self.current_index = idx;
        } else {
            self.elements.push(element);
            self.history.push(self.current_index);
            self.current_index = self.elements.len() - 1;
        }
    }

    /// Go back in history
    pub fn back(&mut self) -> Option<AccessibleElement> {
        if let Some(idx) = self.history.pop() {
            self.current_index = idx;
            self.elements.get(idx).cloned()
        } else {
            None
        }
    }
}

impl Default for NavigationAssist {
    fn default() -> Self {
        Self::new()
    }
}

/// User adaptations
#[derive(Debug, Clone, Default)]
pub struct UserAdaptations {
    /// Active adaptations
    pub active: Vec<VoiceAdaptation>,
    /// Custom parameters
    pub custom: HashMap<String, String>,
}

/// Voice adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceAdaptation {
    /// Slower speech rate
    SlowerSpeech,
    /// Louder output
    LouderSpeech,
    /// Simplified commands
    SimplifiedCommands,
    /// Extended timeouts
    ExtendedTimeout,
    /// Repeat confirmations
    RepeatConfirmation,
    /// Custom parameters
    Custom(HashMap<String, String>),
}

/// Help item
#[derive(Debug, Clone)]
pub struct HelpItem {
    /// Command phrase
    pub command: String,
    /// Description
    pub description: String,
    /// Example usage
    pub example: String,
}

/// Spatial audio positioning
pub struct SpatialAudio {
    /// Listener position
    listener_pos: (f32, f32, f32),
    /// Listener orientation
    listener_dir: (f32, f32, f32),
}

impl SpatialAudio {
    /// Create new spatial audio
    pub fn new() -> Self {
        Self {
            listener_pos: (0.0, 0.0, 0.0),
            listener_dir: (0.0, 0.0, 1.0),
        }
    }

    /// Set listener position
    pub fn set_listener(&mut self, pos: (f32, f32, f32), dir: (f32, f32, f32)) {
        self.listener_pos = pos;
        self.listener_dir = dir;
    }

    /// Calculate spatial parameters for element
    pub fn calculate_spatial(&self, element_pos: (f32, f32, f32)) -> SpatialParams {
        let dx = element_pos.0 - self.listener_pos.0;
        let dy = element_pos.1 - self.listener_pos.1;
        let dz = element_pos.2 - self.listener_pos.2;

        let distance = (dx * dx + dy * dy + dz * dz).sqrt();

        // Calculate pan (-1 to 1)
        let pan = if distance > 0.01 {
            dx / distance
        } else {
            0.0
        };

        // Calculate volume attenuation
        let volume = 1.0 / (1.0 + distance * 0.1);

        SpatialParams {
            pan,
            volume,
            distance,
        }
    }
}

impl Default for SpatialAudio {
    fn default() -> Self {
        Self::new()
    }
}

/// Spatial audio parameters
#[derive(Debug, Clone)]
pub struct SpatialParams {
    /// Pan (-1 left to 1 right)
    pub pan: f32,
    /// Volume (0 to 1)
    pub volume: f32,
    /// Distance
    pub distance: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_accessibility() {
        let accessibility = VoiceAccessibility::default();
        assert_eq!(accessibility.config().speech_rate, 1.0);
    }

    #[test]
    fn test_read_element() {
        let accessibility = VoiceAccessibility::default();

        let element = AccessibleElement {
            id: "btn1".to_string(),
            element_type: ElementType::Button,
            label: Some("Submit".to_string()),
            ..Default::default()
        };

        let reading = accessibility.read_element(&element);
        assert!(reading.contains("Button"));
        assert!(reading.contains("Submit"));
    }

    #[test]
    fn test_navigation() {
        let mut nav = NavigationAssist::new();

        nav.set_elements(vec![
            AccessibleElement {
                id: "1".to_string(),
                element_type: ElementType::Button,
                label: Some("One".to_string()),
                ..Default::default()
            },
            AccessibleElement {
                id: "2".to_string(),
                element_type: ElementType::Button,
                label: Some("Two".to_string()),
                ..Default::default()
            },
        ]);

        let next = nav.next();
        assert!(next.is_some());
        assert_eq!(next.unwrap().label, Some("Two".to_string()));
    }

    #[test]
    fn test_adaptation() {
        let mut accessibility = VoiceAccessibility::default();

        accessibility.apply_adaptation(VoiceAdaptation::SlowerSpeech);
        assert_eq!(accessibility.config().speech_rate, 0.7);

        accessibility.apply_adaptation(VoiceAdaptation::ExtendedTimeout);
        assert_eq!(accessibility.adapted_timeout(1000), 2000);
    }

    #[test]
    fn test_spatial_audio() {
        let mut spatial = SpatialAudio::new();
        spatial.set_listener((0.0, 0.0, 0.0), (0.0, 0.0, 1.0));

        let params = spatial.calculate_spatial((1.0, 0.0, 0.0));
        assert!(params.pan > 0.0); // Right side
    }

    #[test]
    fn test_context_help() {
        let accessibility = VoiceAccessibility::default();
        let help = accessibility.context_help();
        assert!(!help.is_empty());
    }
}
