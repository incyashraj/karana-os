//! # Kāraṇa Multimodal Command System
//!
//! Integrates voice, gesture, and gaze inputs for unified AR command processing.
//!
//! ## Command Sources
//! - Voice: Natural language commands via Whisper
//! - Gesture: Hand tracking actions (pinch, point, etc.)
//! - Gaze: Eye tracking with dwell selection
//! - Hybrid: Combined inputs (e.g., look + speak, point + pinch)
//!
//! ## Architecture
//! ```text
//! Voice ──┐
//! Gesture ├──→ Command Fusion → Intent Resolver → AR Tab Actions
//! Gaze ───┘
//! ```

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::gesture::GestureType;
use crate::gaze::{GazeEvent, GazePoint, BlinkType};

/// Command input source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputSource {
    Voice,
    Gesture,
    Gaze,
    Touch,  // Future: touchpad on glasses
    Hybrid,
}

/// Voice command category
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceCategory {
    /// Tab management ("open tab", "close tab", etc.)
    TabControl,
    /// Navigation ("go back", "scroll down", etc.)
    Navigation,
    /// Content interaction ("click", "select", "read this")
    Interaction,
    /// System commands ("show battery", "take screenshot")
    System,
    /// AR spatial commands ("move here", "resize", "anchor")
    Spatial,
    /// Information queries ("what is this", "translate")
    Query,
    /// Confirmation ("yes", "no", "cancel")
    Confirmation,
    /// Dictation (free-form text input)
    Dictation,
    /// Unknown/Other
    Unknown,
}

/// Parsed voice command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommand {
    /// Raw transcription
    pub transcription: String,
    /// Detected command category
    pub category: VoiceCategory,
    /// Extracted entities
    pub entities: HashMap<String, String>,
    /// Confidence (0-1)
    pub confidence: f32,
    /// Target tab (if any)
    pub target_tab: Option<String>,
    /// Timestamp
    pub timestamp: u64,
}

/// Gesture command with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureCommand {
    /// Gesture type
    pub gesture: GestureType,
    /// Position in space
    pub position: [f32; 3],
    /// Duration held
    pub duration_ms: u32,
    /// Confidence
    pub confidence: f32,
    /// Target tab at gesture location (u64 id)
    pub target_tab: Option<u64>,
    /// Timestamp
    pub timestamp: u64,
}

/// Gaze command with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazeCommand {
    /// Gaze event type
    pub event_type: GazeCommandType,
    /// Screen position
    pub point: GazePoint,
    /// Dwell duration (if applicable)
    pub dwell_ms: Option<u32>,
    /// Target element ID
    pub target_element: Option<String>,
    /// Target tab (u64 id)
    pub target_tab: Option<u64>,
    /// Timestamp
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GazeCommandType {
    Dwell,
    SingleBlink,
    DoubleBlink,
    LongBlink,
    FixationStart,
    FixationEnd,
}

/// Unified multimodal command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultimodalCommand {
    Voice(VoiceCommand),
    Gesture(GestureCommand),
    Gaze(GazeCommand),
    /// Combined inputs that happened together
    Hybrid {
        voice: Option<VoiceCommand>,
        gesture: Option<GestureCommand>,
        gaze: Option<GazeCommand>,
    },
}

impl MultimodalCommand {
    pub fn source(&self) -> InputSource {
        match self {
            MultimodalCommand::Voice(_) => InputSource::Voice,
            MultimodalCommand::Gesture(_) => InputSource::Gesture,
            MultimodalCommand::Gaze(_) => InputSource::Gaze,
            MultimodalCommand::Hybrid { .. } => InputSource::Hybrid,
        }
    }
    
    pub fn timestamp(&self) -> u64 {
        match self {
            MultimodalCommand::Voice(v) => v.timestamp,
            MultimodalCommand::Gesture(g) => g.timestamp,
            MultimodalCommand::Gaze(g) => g.timestamp,
            MultimodalCommand::Hybrid { voice, gesture, gaze } => {
                voice.as_ref().map(|v| v.timestamp)
                    .or_else(|| gesture.as_ref().map(|g| g.timestamp))
                    .or_else(|| gaze.as_ref().map(|g| g.timestamp))
                    .unwrap_or(0)
            }
        }
    }
}

/// Resolved action from multimodal command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolvedAction {
    /// Open a new AR tab
    OpenTab { url: Option<String>, title: Option<String> },
    /// Close a tab
    CloseTab { tab_id: u64 },
    /// Focus a tab
    FocusTab { tab_id: u64 },
    /// Move tab to new position
    MoveTab { tab_id: u64, position: [f32; 3] },
    /// Resize tab
    ResizeTab { tab_id: u64, size: (f32, f32) },
    /// Scroll content
    Scroll { tab_id: u64, direction: ScrollDirection, amount: f32 },
    /// Click/select element
    Select { tab_id: Option<u64>, element_id: Option<String> },
    /// Navigate back/forward
    Navigate { direction: NavDirection },
    /// Text input
    TextInput { text: String },
    /// Take screenshot
    Screenshot { target: Option<u64> },
    /// Show/hide HUD element
    ToggleHud { element: String, visible: bool },
    /// Query for information
    Query { query: String, target: Option<u64> },
    /// Confirm pending action
    Confirm { action_id: String },
    /// Cancel pending action
    Cancel { action_id: Option<String> },
    /// Pin/anchor tab in space
    AnchorTab { tab_id: u64 },
    /// Minimize tab
    MinimizeTab { tab_id: u64 },
    /// Maximize/restore tab
    MaximizeTab { tab_id: u64 },
    /// Custom action
    Custom { action: String, params: HashMap<String, String> },
    /// No action (command not recognized)
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavDirection {
    Back,
    Forward,
    Home,
    Refresh,
}

/// Configuration for multimodal command processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalConfig {
    /// Voice command enabled
    pub voice_enabled: bool,
    /// Gesture command enabled
    pub gesture_enabled: bool,
    /// Gaze command enabled
    pub gaze_enabled: bool,
    /// Time window for combining inputs (ms)
    pub fusion_window_ms: u32,
    /// Minimum confidence for voice commands
    pub voice_confidence_threshold: f32,
    /// Minimum confidence for gesture commands
    pub gesture_confidence_threshold: f32,
    /// Enable hybrid commands
    pub hybrid_enabled: bool,
    /// Voice command timeout (ms)
    pub voice_timeout_ms: u32,
}

impl Default for MultimodalConfig {
    fn default() -> Self {
        Self {
            voice_enabled: true,
            gesture_enabled: true,
            gaze_enabled: true,
            fusion_window_ms: 500,
            voice_confidence_threshold: 0.5,
            gesture_confidence_threshold: 0.7,
            hybrid_enabled: true,
            voice_timeout_ms: 5000,
        }
    }
}

/// Voice command parser
pub struct VoiceParser {
    /// Command patterns
    patterns: Vec<CommandPattern>,
}

/// Pattern for matching voice commands
#[derive(Debug, Clone)]
struct CommandPattern {
    keywords: Vec<String>,
    category: VoiceCategory,
    action_template: String,
    entities: Vec<String>,
}

impl VoiceParser {
    pub fn new() -> Self {
        let patterns = Self::default_patterns();
        Self { patterns }
    }
    
    fn default_patterns() -> Vec<CommandPattern> {
        vec![
            // Tab control
            CommandPattern {
                keywords: vec!["open".into(), "new".into(), "tab".into()],
                category: VoiceCategory::TabControl,
                action_template: "open_tab".into(),
                entities: vec!["url".into()],
            },
            CommandPattern {
                keywords: vec!["close".into(), "tab".into()],
                category: VoiceCategory::TabControl,
                action_template: "close_tab".into(),
                entities: vec!["target".into()],
            },
            CommandPattern {
                keywords: vec!["switch".into(), "tab".into()],
                category: VoiceCategory::TabControl,
                action_template: "focus_tab".into(),
                entities: vec!["target".into()],
            },
            CommandPattern {
                keywords: vec!["minimize".into()],
                category: VoiceCategory::TabControl,
                action_template: "minimize_tab".into(),
                entities: vec!["target".into()],
            },
            CommandPattern {
                keywords: vec!["maximize".into()],
                category: VoiceCategory::TabControl,
                action_template: "maximize_tab".into(),
                entities: vec!["target".into()],
            },
            // Navigation
            CommandPattern {
                keywords: vec!["go".into(), "back".into()],
                category: VoiceCategory::Navigation,
                action_template: "navigate_back".into(),
                entities: vec![],
            },
            CommandPattern {
                keywords: vec!["go".into(), "forward".into()],
                category: VoiceCategory::Navigation,
                action_template: "navigate_forward".into(),
                entities: vec![],
            },
            CommandPattern {
                keywords: vec!["scroll".into(), "down".into()],
                category: VoiceCategory::Navigation,
                action_template: "scroll_down".into(),
                entities: vec!["amount".into()],
            },
            CommandPattern {
                keywords: vec!["scroll".into(), "up".into()],
                category: VoiceCategory::Navigation,
                action_template: "scroll_up".into(),
                entities: vec!["amount".into()],
            },
            // Interaction
            CommandPattern {
                keywords: vec!["click".into(), "select".into()],
                category: VoiceCategory::Interaction,
                action_template: "select".into(),
                entities: vec!["target".into()],
            },
            CommandPattern {
                keywords: vec!["read".into(), "this".into()],
                category: VoiceCategory::Interaction,
                action_template: "read_aloud".into(),
                entities: vec!["target".into()],
            },
            // Spatial
            CommandPattern {
                keywords: vec!["move".into(), "here".into()],
                category: VoiceCategory::Spatial,
                action_template: "move_tab".into(),
                entities: vec!["target".into()],
            },
            CommandPattern {
                keywords: vec!["anchor".into(), "pin".into()],
                category: VoiceCategory::Spatial,
                action_template: "anchor_tab".into(),
                entities: vec!["target".into()],
            },
            CommandPattern {
                keywords: vec!["resize".into()],
                category: VoiceCategory::Spatial,
                action_template: "resize_tab".into(),
                entities: vec!["size".into()],
            },
            // Query
            CommandPattern {
                keywords: vec!["what".into(), "is".into()],
                category: VoiceCategory::Query,
                action_template: "query_identify".into(),
                entities: vec!["target".into()],
            },
            CommandPattern {
                keywords: vec!["translate".into()],
                category: VoiceCategory::Query,
                action_template: "query_translate".into(),
                entities: vec!["text".into(), "language".into()],
            },
            // System
            CommandPattern {
                keywords: vec!["screenshot".into()],
                category: VoiceCategory::System,
                action_template: "screenshot".into(),
                entities: vec![],
            },
            CommandPattern {
                keywords: vec!["show".into(), "battery".into()],
                category: VoiceCategory::System,
                action_template: "show_battery".into(),
                entities: vec![],
            },
            // Confirmation
            CommandPattern {
                keywords: vec!["yes".into(), "confirm".into(), "okay".into()],
                category: VoiceCategory::Confirmation,
                action_template: "confirm".into(),
                entities: vec![],
            },
            CommandPattern {
                keywords: vec!["no".into(), "cancel".into()],
                category: VoiceCategory::Confirmation,
                action_template: "cancel".into(),
                entities: vec![],
            },
        ]
    }
    
    /// Parse transcribed text into a voice command
    pub fn parse(&self, transcription: &str) -> VoiceCommand {
        let lower = transcription.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        
        let mut best_match: Option<(usize, &CommandPattern)> = None;
        
        for pattern in &self.patterns {
            let mut match_count = 0;
            for keyword in &pattern.keywords {
                if words.iter().any(|w| w.contains(keyword.as_str())) {
                    match_count += 1;
                }
            }
            
            if match_count > 0 {
                if best_match.is_none() || match_count > best_match.unwrap().0 {
                    best_match = Some((match_count, pattern));
                }
            }
        }
        
        let (category, confidence) = if let Some((count, pattern)) = best_match {
            let conf = count as f32 / pattern.keywords.len() as f32;
            (pattern.category.clone(), conf)
        } else {
            (VoiceCategory::Unknown, 0.0)
        };
        
        // Extract entities (simple heuristic)
        let entities = self.extract_entities(transcription, &category);
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        VoiceCommand {
            transcription: transcription.to_string(),
            category,
            entities,
            confidence,
            target_tab: None,
            timestamp,
        }
    }
    
    fn extract_entities(&self, text: &str, category: &VoiceCategory) -> HashMap<String, String> {
        let mut entities = HashMap::new();
        let lower = text.to_lowercase();
        
        // Extract URLs
        if lower.contains("http") || lower.contains("www") {
            for word in text.split_whitespace() {
                if word.starts_with("http") || word.starts_with("www") {
                    entities.insert("url".to_string(), word.to_string());
                    break;
                }
            }
        }
        
        // Extract numbers (for scroll amount, tab numbers, etc.)
        for word in text.split_whitespace() {
            if let Ok(n) = word.parse::<i32>() {
                entities.insert("number".to_string(), n.to_string());
                break;
            }
        }
        
        // Extract ordinals (first, second, etc.)
        let ordinals = [
            ("first", "1"), ("second", "2"), ("third", "3"),
            ("fourth", "4"), ("fifth", "5"), ("last", "-1"),
        ];
        for (word, num) in ordinals {
            if lower.contains(word) {
                entities.insert("ordinal".to_string(), num.to_string());
                break;
            }
        }
        
        entities
    }
}

impl Default for VoiceParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Multimodal command fusion engine
pub struct CommandFusion {
    config: MultimodalConfig,
    
    /// Recent voice commands
    voice_buffer: VecDeque<VoiceCommand>,
    /// Recent gesture commands
    gesture_buffer: VecDeque<GestureCommand>,
    /// Recent gaze commands
    gaze_buffer: VecDeque<GazeCommand>,
    
    /// Voice parser
    voice_parser: VoiceParser,
    
    /// Pending hybrid command
    pending_hybrid: Option<HybridBuilder>,
    
    /// Action history
    action_history: VecDeque<ResolvedAction>,
}

/// Builder for hybrid commands
struct HybridBuilder {
    start_time: Instant,
    voice: Option<VoiceCommand>,
    gesture: Option<GestureCommand>,
    gaze: Option<GazeCommand>,
}

impl HybridBuilder {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            voice: None,
            gesture: None,
            gaze: None,
        }
    }
    
    fn is_expired(&self, window_ms: u32) -> bool {
        self.start_time.elapsed() > Duration::from_millis(window_ms as u64)
    }
    
    fn has_input(&self) -> bool {
        self.voice.is_some() || self.gesture.is_some() || self.gaze.is_some()
    }
    
    fn build(self) -> MultimodalCommand {
        MultimodalCommand::Hybrid {
            voice: self.voice,
            gesture: self.gesture,
            gaze: self.gaze,
        }
    }
}

impl CommandFusion {
    pub fn new(config: MultimodalConfig) -> Self {
        Self {
            config,
            voice_buffer: VecDeque::with_capacity(10),
            gesture_buffer: VecDeque::with_capacity(20),
            gaze_buffer: VecDeque::with_capacity(20),
            voice_parser: VoiceParser::new(),
            pending_hybrid: None,
            action_history: VecDeque::with_capacity(50),
        }
    }
    
    /// Process voice transcription
    pub fn process_voice(&mut self, transcription: &str) -> Option<MultimodalCommand> {
        if !self.config.voice_enabled {
            return None;
        }
        
        let command = self.voice_parser.parse(transcription);
        
        if command.confidence < self.config.voice_confidence_threshold {
            return None;
        }
        
        self.voice_buffer.push_back(command.clone());
        if self.voice_buffer.len() > 10 {
            self.voice_buffer.pop_front();
        }
        
        // Check for hybrid command
        if self.config.hybrid_enabled {
            if let Some(ref mut builder) = self.pending_hybrid {
                builder.voice = Some(command.clone());
                
                if builder.is_expired(self.config.fusion_window_ms) || builder.has_input() {
                    let hybrid = self.pending_hybrid.take().unwrap().build();
                    return Some(hybrid);
                }
                return None;
            }
        }
        
        Some(MultimodalCommand::Voice(command))
    }
    
    /// Process gesture command
    pub fn process_gesture(&mut self, gesture_type: GestureType, position: [f32; 3], confidence: f32) -> Option<MultimodalCommand> {
        if !self.config.gesture_enabled {
            return None;
        }
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let command = GestureCommand {
            gesture: gesture_type,
            position,
            duration_ms: 0,
            confidence,
            target_tab: None,  // Resolved later
            timestamp,
        };
        
        if command.confidence < self.config.gesture_confidence_threshold {
            return None;
        }
        
        self.gesture_buffer.push_back(command.clone());
        if self.gesture_buffer.len() > 20 {
            self.gesture_buffer.pop_front();
        }
        
        // Check for hybrid command
        if self.config.hybrid_enabled {
            if let Some(ref mut builder) = self.pending_hybrid {
                builder.gesture = Some(command.clone());
                
                if builder.is_expired(self.config.fusion_window_ms) {
                    let hybrid = self.pending_hybrid.take().unwrap().build();
                    return Some(hybrid);
                }
                return None;
            } else {
                // Start new hybrid builder
                let mut builder = HybridBuilder::new();
                builder.gesture = Some(command);
                self.pending_hybrid = Some(builder);
                return None;
            }
        }
        
        Some(MultimodalCommand::Gesture(command))
    }
    
    /// Process gaze event
    pub fn process_gaze(&mut self, event: &GazeEvent) -> Option<MultimodalCommand> {
        if !self.config.gaze_enabled {
            return None;
        }
        
        let command = match event {
            GazeEvent::Dwell { point, duration_ms } => {
                Some(GazeCommand {
                    event_type: GazeCommandType::Dwell,
                    point: *point,
                    dwell_ms: Some(*duration_ms),
                    target_element: None,
                    target_tab: None,
                    timestamp: point.timestamp,
                })
            }
            GazeEvent::Blink(blink) => {
                let event_type = match blink.blink_type {
                    BlinkType::Single => GazeCommandType::SingleBlink,
                    BlinkType::Double => GazeCommandType::DoubleBlink,
                    BlinkType::Long => GazeCommandType::LongBlink,
                };
                Some(GazeCommand {
                    event_type,
                    point: GazePoint::default(),
                    dwell_ms: None,
                    target_element: None,
                    target_tab: None,
                    timestamp: blink.timestamp,
                })
            }
            GazeEvent::FixationStart(fixation) => {
                Some(GazeCommand {
                    event_type: GazeCommandType::FixationStart,
                    point: fixation.center,
                    dwell_ms: Some(fixation.duration_ms),
                    target_element: None,
                    target_tab: None,
                    timestamp: fixation.start_ms,
                })
            }
            GazeEvent::FixationEnd(fixation) => {
                Some(GazeCommand {
                    event_type: GazeCommandType::FixationEnd,
                    point: fixation.center,
                    dwell_ms: Some(fixation.duration_ms),
                    target_element: None,
                    target_tab: None,
                    timestamp: fixation.start_ms,
                })
            }
            _ => None,
        };
        
        let command = command?;
        
        self.gaze_buffer.push_back(command.clone());
        if self.gaze_buffer.len() > 20 {
            self.gaze_buffer.pop_front();
        }
        
        // Check for hybrid command
        if self.config.hybrid_enabled {
            if let Some(ref mut builder) = self.pending_hybrid {
                builder.gaze = Some(command.clone());
                // Don't complete on gaze alone - wait for voice or gesture
                return None;
            }
        }
        
        Some(MultimodalCommand::Gaze(command))
    }
    
    /// Resolve command to action
    pub fn resolve(&mut self, command: &MultimodalCommand) -> ResolvedAction {
        let action = match command {
            MultimodalCommand::Voice(voice) => self.resolve_voice(voice),
            MultimodalCommand::Gesture(gesture) => self.resolve_gesture(gesture),
            MultimodalCommand::Gaze(gaze) => self.resolve_gaze(gaze),
            MultimodalCommand::Hybrid { voice, gesture, gaze } => {
                self.resolve_hybrid(voice.as_ref(), gesture.as_ref(), gaze.as_ref())
            }
        };
        
        // Record in history
        self.action_history.push_back(action.clone());
        if self.action_history.len() > 50 {
            self.action_history.pop_front();
        }
        
        action
    }
    
    fn resolve_voice(&self, voice: &VoiceCommand) -> ResolvedAction {
        match voice.category {
            VoiceCategory::TabControl => {
                let lower = voice.transcription.to_lowercase();
                if lower.contains("open") || lower.contains("new") {
                    let url = voice.entities.get("url").cloned();
                    ResolvedAction::OpenTab { url, title: None }
                } else if lower.contains("close") {
                    ResolvedAction::CloseTab { tab_id: 0 }  // TODO: resolve target
                } else if lower.contains("minimize") {
                    ResolvedAction::MinimizeTab { tab_id: 0 }
                } else if lower.contains("maximize") {
                    ResolvedAction::MaximizeTab { tab_id: 0 }
                } else {
                    ResolvedAction::None
                }
            }
            VoiceCategory::Navigation => {
                let lower = voice.transcription.to_lowercase();
                if lower.contains("back") {
                    ResolvedAction::Navigate { direction: NavDirection::Back }
                } else if lower.contains("forward") {
                    ResolvedAction::Navigate { direction: NavDirection::Forward }
                } else if lower.contains("scroll") {
                    let direction = if lower.contains("down") {
                        ScrollDirection::Down
                    } else if lower.contains("up") {
                        ScrollDirection::Up
                    } else if lower.contains("left") {
                        ScrollDirection::Left
                    } else {
                        ScrollDirection::Right
                    };
                    ResolvedAction::Scroll {
                        tab_id: 0,
                        direction,
                        amount: 100.0,
                    }
                } else {
                    ResolvedAction::None
                }
            }
            VoiceCategory::Interaction => {
                let lower = voice.transcription.to_lowercase();
                if lower.contains("click") || lower.contains("select") {
                    ResolvedAction::Select {
                        tab_id: None,
                        element_id: None,
                    }
                } else {
                    ResolvedAction::None
                }
            }
            VoiceCategory::System => {
                let lower = voice.transcription.to_lowercase();
                if lower.contains("screenshot") {
                    ResolvedAction::Screenshot { target: None }
                } else {
                    ResolvedAction::None
                }
            }
            VoiceCategory::Spatial => {
                let lower = voice.transcription.to_lowercase();
                if lower.contains("anchor") || lower.contains("pin") {
                    ResolvedAction::AnchorTab { tab_id: 0 }
                } else if lower.contains("move") {
                    ResolvedAction::MoveTab {
                        tab_id: 0,
                        position: [0.0, 0.0, 0.0],
                    }
                } else {
                    ResolvedAction::None
                }
            }
            VoiceCategory::Query => {
                ResolvedAction::Query {
                    query: voice.transcription.clone(),
                    target: None,
                }
            }
            VoiceCategory::Confirmation => {
                let lower = voice.transcription.to_lowercase();
                if lower.contains("yes") || lower.contains("confirm") || lower.contains("okay") {
                    ResolvedAction::Confirm { action_id: String::new() }
                } else {
                    ResolvedAction::Cancel { action_id: None }
                }
            }
            VoiceCategory::Dictation => {
                ResolvedAction::TextInput { text: voice.transcription.clone() }
            }
            VoiceCategory::Unknown => ResolvedAction::None,
        }
    }
    
    fn resolve_gesture(&self, gesture: &GestureCommand) -> ResolvedAction {
        match gesture.gesture {
            GestureType::Pinch => {
                // Pinch = select/click
                ResolvedAction::Select {
                    tab_id: gesture.target_tab,
                    element_id: None,
                }
            }
            GestureType::Point => {
                // Point = focus
                if let Some(tab_id) = gesture.target_tab {
                    ResolvedAction::FocusTab { tab_id }
                } else {
                    ResolvedAction::None
                }
            }
            GestureType::Fist => {
                // Fist = grab/move
                ResolvedAction::MoveTab {
                    tab_id: gesture.target_tab.unwrap_or(0),
                    position: gesture.position,
                }
            }
            GestureType::OpenPalm => {
                // Open palm = stop/cancel
                ResolvedAction::Cancel { action_id: None }
            }
            GestureType::ThumbsUp => {
                // Thumbs up = confirm
                ResolvedAction::Confirm { action_id: String::new() }
            }
            GestureType::ThumbsDown => {
                // Thumbs down = reject/close
                if let Some(tab_id) = gesture.target_tab {
                    ResolvedAction::CloseTab { tab_id }
                } else {
                    ResolvedAction::Cancel { action_id: None }
                }
            }
            _ => ResolvedAction::None,
        }
    }
    
    fn resolve_gaze(&self, gaze: &GazeCommand) -> ResolvedAction {
        match gaze.event_type {
            GazeCommandType::Dwell => {
                // Dwell = select
                ResolvedAction::Select {
                    tab_id: gaze.target_tab,
                    element_id: gaze.target_element.clone(),
                }
            }
            GazeCommandType::DoubleBlink => {
                // Double blink = confirm
                ResolvedAction::Confirm { action_id: String::new() }
            }
            GazeCommandType::LongBlink => {
                // Long blink = cancel
                ResolvedAction::Cancel { action_id: None }
            }
            _ => ResolvedAction::None,
        }
    }
    
    fn resolve_hybrid(
        &self,
        voice: Option<&VoiceCommand>,
        gesture: Option<&GestureCommand>,
        gaze: Option<&GazeCommand>,
    ) -> ResolvedAction {
        // Hybrid resolution: combine context from all inputs
        
        // Voice + Point gesture = voice command targeted at pointed location
        if let (Some(voice), Some(gesture)) = (voice, gesture) {
            if gesture.gesture == GestureType::Point {
                // Use voice command but target the pointed tab
                let mut base_action = self.resolve_voice(voice);
                
                // Update target if action has one
                match &mut base_action {
                    ResolvedAction::CloseTab { tab_id } |
                    ResolvedAction::FocusTab { tab_id } |
                    ResolvedAction::MinimizeTab { tab_id } |
                    ResolvedAction::MaximizeTab { tab_id } |
                    ResolvedAction::AnchorTab { tab_id } => {
                        if let Some(target) = gesture.target_tab {
                            *tab_id = target;
                        }
                    }
                    ResolvedAction::MoveTab { tab_id, position } => {
                        if let Some(target) = gesture.target_tab {
                            *tab_id = target;
                        }
                        *position = gesture.position;
                    }
                    _ => {}
                }
                
                return base_action;
            }
        }
        
        // Gaze + Voice = voice command targeted at gazed element
        if let (Some(voice), Some(gaze)) = (voice, gaze) {
            let mut base_action = self.resolve_voice(voice);
            
            // Update target from gaze
            if let ResolvedAction::Select { tab_id, element_id } = &mut base_action {
                *tab_id = gaze.target_tab;
                *element_id = gaze.target_element.clone();
            }
            
            return base_action;
        }
        
        // Fallback: resolve individual inputs
        if let Some(voice) = voice {
            return self.resolve_voice(voice);
        }
        if let Some(gesture) = gesture {
            return self.resolve_gesture(gesture);
        }
        if let Some(gaze) = gaze {
            return self.resolve_gaze(gaze);
        }
        
        ResolvedAction::None
    }
    
    /// Get last resolved action
    pub fn last_action(&self) -> Option<&ResolvedAction> {
        self.action_history.back()
    }
    
    /// Flush pending hybrid command
    pub fn flush(&mut self) -> Option<MultimodalCommand> {
        self.pending_hybrid.take().map(|b| b.build())
    }
    
    /// Clear all buffers
    pub fn clear(&mut self) {
        self.voice_buffer.clear();
        self.gesture_buffer.clear();
        self.gaze_buffer.clear();
        self.pending_hybrid = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_voice_parser() {
        let parser = VoiceParser::new();
        
        let cmd = parser.parse("open a new tab");
        assert_eq!(cmd.category, VoiceCategory::TabControl);
        assert!(cmd.confidence > 0.0);
        
        let cmd = parser.parse("go back");
        assert_eq!(cmd.category, VoiceCategory::Navigation);
        
        let cmd = parser.parse("some random words");
        assert_eq!(cmd.category, VoiceCategory::Unknown);
    }
    
    #[test]
    fn test_voice_entities() {
        let parser = VoiceParser::new();
        
        let cmd = parser.parse("open tab https://example.com");
        assert!(cmd.entities.contains_key("url"));
        
        let cmd = parser.parse("switch to the third tab");
        assert!(cmd.entities.contains_key("ordinal"));
        assert_eq!(cmd.entities.get("ordinal"), Some(&"3".to_string()));
    }
    
    #[test]
    fn test_command_fusion_creation() {
        let config = MultimodalConfig::default();
        let fusion = CommandFusion::new(config);
        
        assert!(fusion.last_action().is_none());
    }
    
    #[test]
    fn test_process_voice() {
        let config = MultimodalConfig::default();
        let mut fusion = CommandFusion::new(config);
        
        let cmd = fusion.process_voice("close this tab");
        assert!(cmd.is_some());
        
        if let Some(MultimodalCommand::Voice(voice)) = cmd {
            assert_eq!(voice.category, VoiceCategory::TabControl);
        }
    }
    
    #[test]
    fn test_resolve_voice_commands() {
        let config = MultimodalConfig::default();
        let mut fusion = CommandFusion::new(config);
        
        let voice = VoiceCommand {
            transcription: "open new tab".to_string(),
            category: VoiceCategory::TabControl,
            entities: HashMap::new(),
            confidence: 0.9,
            target_tab: None,
            timestamp: 0,
        };
        
        let action = fusion.resolve(&MultimodalCommand::Voice(voice));
        assert!(matches!(action, ResolvedAction::OpenTab { .. }));
    }
    
    #[test]
    fn test_resolve_gesture() {
        let config = MultimodalConfig::default();
        let mut fusion = CommandFusion::new(config);
        
        let gesture = GestureCommand {
            gesture: GestureType::Pinch,
            position: [0.0, 0.0, 0.0],
            duration_ms: 100,
            confidence: 0.9,
            target_tab: Some(1),
            timestamp: 0,
        };
        
        let action = fusion.resolve(&MultimodalCommand::Gesture(gesture));
        assert!(matches!(action, ResolvedAction::Select { .. }));
    }
    
    #[test]
    fn test_navigation_actions() {
        let config = MultimodalConfig::default();
        let mut fusion = CommandFusion::new(config);
        
        let voice = VoiceCommand {
            transcription: "go back".to_string(),
            category: VoiceCategory::Navigation,
            entities: HashMap::new(),
            confidence: 0.9,
            target_tab: None,
            timestamp: 0,
        };
        
        let action = fusion.resolve(&MultimodalCommand::Voice(voice));
        assert!(matches!(action, ResolvedAction::Navigate { direction: NavDirection::Back }));
    }
    
    #[test]
    fn test_scroll_direction() {
        assert_ne!(ScrollDirection::Up, ScrollDirection::Down);
        assert_ne!(ScrollDirection::Left, ScrollDirection::Right);
    }
    
    #[test]
    fn test_input_source() {
        let voice = MultimodalCommand::Voice(VoiceCommand {
            transcription: "test".to_string(),
            category: VoiceCategory::Unknown,
            entities: HashMap::new(),
            confidence: 0.5,
            target_tab: None,
            timestamp: 0,
        });
        
        assert_eq!(voice.source(), InputSource::Voice);
    }
}
