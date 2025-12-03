//! Screen reader functionality for blind and low-vision users

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Verbosity level for screen reader
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verbosity {
    /// Only essential information
    Low,
    /// Standard verbosity
    Medium,
    /// Detailed information
    High,
    /// Maximum detail
    Full,
}

impl Verbosity {
    /// Get announcement delay multiplier
    pub fn delay_multiplier(&self) -> f32 {
        match self {
            Verbosity::Low => 0.5,
            Verbosity::Medium => 1.0,
            Verbosity::High => 1.5,
            Verbosity::Full => 2.0,
        }
    }
}

/// Punctuation reading level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PunctuationLevel {
    /// No punctuation
    None,
    /// Some punctuation (periods, commas)
    Some,
    /// Most punctuation
    Most,
    /// All punctuation
    All,
}

/// Speech configuration
#[derive(Debug, Clone)]
pub struct SpeechConfig {
    /// Speech rate (words per minute)
    pub rate: u32,
    /// Pitch (0.5 - 2.0, 1.0 = normal)
    pub pitch: f32,
    /// Volume (0.0 - 1.0)
    pub volume: f32,
    /// Voice identifier
    pub voice_id: String,
    /// Verbosity level
    pub verbosity: Verbosity,
    /// Punctuation reading level
    pub punctuation: PunctuationLevel,
    /// Read capital letters
    pub announce_capitals: bool,
    /// Announce formatting changes
    pub announce_formatting: bool,
}

impl Default for SpeechConfig {
    fn default() -> Self {
        Self {
            rate: 180,
            pitch: 1.0,
            volume: 1.0,
            voice_id: "default".to_string(),
            verbosity: Verbosity::Medium,
            punctuation: PunctuationLevel::Some,
            announce_capitals: true,
            announce_formatting: false,
        }
    }
}

/// Speech queue entry
#[derive(Debug, Clone)]
pub struct SpeechEntry {
    /// Text to speak
    pub text: String,
    /// Priority (higher = more important)
    pub priority: u8,
    /// Whether this can be interrupted
    pub interruptible: bool,
    /// When queued
    pub queued_at: Instant,
}

/// Focus element for screen reader
#[derive(Debug, Clone)]
pub struct FocusElement {
    /// Element ID
    pub id: String,
    /// Element type (button, text, link, etc.)
    pub element_type: String,
    /// Label/text content
    pub label: String,
    /// Additional description
    pub description: Option<String>,
    /// State (e.g., "checked", "expanded")
    pub state: Option<String>,
    /// Position in list if applicable
    pub position: Option<(usize, usize)>, // (current, total)
}

/// Screen reader system
#[derive(Debug)]
pub struct ScreenReader {
    /// Whether screen reader is enabled
    enabled: bool,
    /// Speech configuration
    speech_config: SpeechConfig,
    /// Speech queue
    speech_queue: VecDeque<SpeechEntry>,
    /// Current focus element
    current_focus: Option<FocusElement>,
    /// Previously focused element
    previous_focus: Option<FocusElement>,
    /// Currently speaking
    is_speaking: bool,
    /// Current speech text
    current_speech: Option<String>,
    /// Speech started at
    speech_start: Option<Instant>,
    /// Navigation mode
    navigation_mode: NavigationMode,
    /// Quick navigation enabled
    quick_nav: bool,
    /// Element types to skip
    skip_elements: Vec<String>,
    /// Last interaction time
    last_interaction: Instant,
}

/// Navigation mode for screen reader
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationMode {
    /// Normal navigation
    Normal,
    /// Forms mode
    Forms,
    /// Browse mode (read-only)
    Browse,
    /// Application mode
    Application,
}

impl ScreenReader {
    /// Create new screen reader
    pub fn new() -> Self {
        Self {
            enabled: false,
            speech_config: SpeechConfig::default(),
            speech_queue: VecDeque::new(),
            current_focus: None,
            previous_focus: None,
            is_speaking: false,
            current_speech: None,
            speech_start: None,
            navigation_mode: NavigationMode::Normal,
            quick_nav: true,
            skip_elements: Vec::new(),
            last_interaction: Instant::now(),
        }
    }
    
    /// Enable screen reader
    pub fn enable(&mut self) {
        self.enabled = true;
        self.speak("Screen reader enabled", 10, false);
    }
    
    /// Disable screen reader
    pub fn disable(&mut self) {
        self.speak("Screen reader disabled", 10, false);
        self.enabled = false;
    }
    
    /// Toggle screen reader
    pub fn toggle(&mut self) {
        if self.enabled {
            self.disable();
        } else {
            self.enable();
        }
    }
    
    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Set speech rate
    pub fn set_speech_rate(&mut self, rate: u32) {
        self.speech_config.rate = rate.clamp(50, 500);
    }
    
    /// Get speech rate
    pub fn speech_rate(&self) -> u32 {
        self.speech_config.rate
    }
    
    /// Set pitch
    pub fn set_pitch(&mut self, pitch: f32) {
        self.speech_config.pitch = pitch.clamp(0.5, 2.0);
    }
    
    /// Set volume
    pub fn set_volume(&mut self, volume: f32) {
        self.speech_config.volume = volume.clamp(0.0, 1.0);
    }
    
    /// Set verbosity
    pub fn set_verbosity(&mut self, level: Verbosity) {
        self.speech_config.verbosity = level;
    }
    
    /// Get verbosity
    pub fn verbosity(&self) -> Verbosity {
        self.speech_config.verbosity
    }
    
    /// Set speech configuration
    pub fn set_speech_config(&mut self, config: SpeechConfig) {
        self.speech_config = config;
    }
    
    /// Get speech configuration
    pub fn speech_config(&self) -> &SpeechConfig {
        &self.speech_config
    }
    
    /// Add speech to queue
    pub fn speak(&mut self, text: &str, priority: u8, interruptible: bool) {
        let entry = SpeechEntry {
            text: text.to_string(),
            priority,
            interruptible,
            queued_at: Instant::now(),
        };
        
        // Insert based on priority
        let pos = self.speech_queue
            .iter()
            .position(|e| e.priority < priority)
            .unwrap_or(self.speech_queue.len());
        
        self.speech_queue.insert(pos, entry);
    }
    
    /// Speak immediately (interrupts current speech)
    pub fn speak_now(&mut self, text: &str) {
        // Clear interruptible entries
        self.speech_queue.retain(|e| !e.interruptible);
        
        // Stop current speech if interruptible
        if self.is_speaking {
            self.stop_speech();
        }
        
        // Add new speech at front
        self.speak(text, 255, false);
    }
    
    /// Stop current speech
    pub fn stop_speech(&mut self) {
        self.is_speaking = false;
        self.current_speech = None;
        self.speech_start = None;
    }
    
    /// Clear speech queue
    pub fn clear_queue(&mut self) {
        self.speech_queue.clear();
        self.stop_speech();
    }
    
    /// Get next speech to process
    pub fn next_speech(&mut self) -> Option<String> {
        if self.is_speaking {
            return None;
        }
        
        if let Some(entry) = self.speech_queue.pop_front() {
            self.is_speaking = true;
            self.current_speech = Some(entry.text.clone());
            self.speech_start = Some(Instant::now());
            Some(entry.text)
        } else {
            None
        }
    }
    
    /// Mark current speech as finished
    pub fn speech_finished(&mut self) {
        self.is_speaking = false;
        self.current_speech = None;
        self.speech_start = None;
    }
    
    /// Check if currently speaking
    pub fn is_speaking(&self) -> bool {
        self.is_speaking
    }
    
    /// Set focus to element
    pub fn set_focus(&mut self, element: FocusElement) {
        self.previous_focus = self.current_focus.take();
        self.current_focus = Some(element);
        self.last_interaction = Instant::now();
        
        // Announce new focus
        if self.enabled {
            self.announce_focus();
        }
    }
    
    /// Get current focus
    pub fn current_focus(&self) -> Option<&FocusElement> {
        self.current_focus.as_ref()
    }
    
    /// Announce current focus
    fn announce_focus(&mut self) {
        if let Some(focus) = &self.current_focus {
            let mut announcement = String::new();
            
            // Element type and label
            announcement.push_str(&focus.label);
            
            // Add type based on verbosity
            if self.speech_config.verbosity >= Verbosity::Medium {
                announcement.push_str(", ");
                announcement.push_str(&focus.element_type);
            }
            
            // Add state if present
            if let Some(state) = &focus.state {
                announcement.push_str(", ");
                announcement.push_str(state);
            }
            
            // Add position if in list
            if self.speech_config.verbosity >= Verbosity::High {
                if let Some((current, total)) = focus.position {
                    announcement.push_str(&format!(", {} of {}", current, total));
                }
            }
            
            // Add description
            if self.speech_config.verbosity >= Verbosity::High {
                if let Some(desc) = &focus.description {
                    announcement.push_str(", ");
                    announcement.push_str(desc);
                }
            }
            
            self.speak(&announcement, 5, true);
        }
    }
    
    /// Speak current focus (explicitly triggered)
    pub fn speak_focus(&mut self) {
        if let Some(focus) = &self.current_focus {
            let mut announcement = format!("{}, {}", focus.label, focus.element_type);
            
            if let Some(state) = &focus.state {
                announcement.push_str(", ");
                announcement.push_str(state);
            }
            
            if let Some(desc) = &focus.description {
                announcement.push_str(". ");
                announcement.push_str(desc);
            }
            
            self.speak_now(&announcement);
        } else {
            self.speak_now("No element focused");
        }
    }
    
    /// Describe current scene
    pub fn describe_scene(&mut self) {
        // In real implementation, this would:
        // 1. Get scene analysis from AI/vision system
        // 2. Generate natural language description
        // 3. Speak the description
        self.speak("Scene description not available", 3, true);
    }
    
    /// Set navigation mode
    pub fn set_navigation_mode(&mut self, mode: NavigationMode) {
        let old_mode = self.navigation_mode;
        self.navigation_mode = mode;
        
        if self.enabled && old_mode != mode {
            let mode_name = match mode {
                NavigationMode::Normal => "Normal mode",
                NavigationMode::Forms => "Forms mode",
                NavigationMode::Browse => "Browse mode",
                NavigationMode::Application => "Application mode",
            };
            self.speak(mode_name, 8, true);
        }
    }
    
    /// Get navigation mode
    pub fn navigation_mode(&self) -> NavigationMode {
        self.navigation_mode
    }
    
    /// Toggle quick navigation
    pub fn toggle_quick_nav(&mut self) {
        self.quick_nav = !self.quick_nav;
        if self.enabled {
            let status = if self.quick_nav { "on" } else { "off" };
            self.speak(&format!("Quick navigation {}", status), 8, true);
        }
    }
    
    /// Check if quick nav is enabled
    pub fn quick_nav_enabled(&self) -> bool {
        self.quick_nav
    }
    
    /// Get speech queue length
    pub fn queue_length(&self) -> usize {
        self.speech_queue.len()
    }
}

impl Default for ScreenReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_screen_reader_creation() {
        let sr = ScreenReader::new();
        assert!(!sr.is_enabled());
        assert_eq!(sr.navigation_mode(), NavigationMode::Normal);
    }
    
    #[test]
    fn test_enable_disable() {
        let mut sr = ScreenReader::new();
        
        sr.enable();
        assert!(sr.is_enabled());
        
        sr.disable();
        assert!(!sr.is_enabled());
    }
    
    #[test]
    fn test_toggle() {
        let mut sr = ScreenReader::new();
        
        sr.toggle();
        assert!(sr.is_enabled());
        
        sr.toggle();
        assert!(!sr.is_enabled());
    }
    
    #[test]
    fn test_speech_queue() {
        let mut sr = ScreenReader::new();
        
        sr.speak("First", 1, true);
        sr.speak("Second", 5, true);
        sr.speak("Third", 3, true);
        
        // Higher priority should come first
        assert_eq!(sr.next_speech(), Some("Second".to_string()));
        sr.speech_finished();
        
        assert_eq!(sr.next_speech(), Some("Third".to_string()));
        sr.speech_finished();
        
        assert_eq!(sr.next_speech(), Some("First".to_string()));
    }
    
    #[test]
    fn test_speech_rate() {
        let mut sr = ScreenReader::new();
        
        sr.set_speech_rate(200);
        assert_eq!(sr.speech_rate(), 200);
        
        // Test clamping
        sr.set_speech_rate(1000);
        assert_eq!(sr.speech_rate(), 500);
    }
    
    #[test]
    fn test_focus_announcement() {
        let mut sr = ScreenReader::new();
        sr.enable();
        
        let element = FocusElement {
            id: "btn1".to_string(),
            element_type: "button".to_string(),
            label: "Submit".to_string(),
            description: None,
            state: None,
            position: None,
        };
        
        sr.set_focus(element);
        
        assert!(sr.current_focus().is_some());
        assert_eq!(sr.current_focus().unwrap().label, "Submit");
        
        // Should have queued announcement
        assert!(sr.queue_length() > 0);
    }
    
    #[test]
    fn test_navigation_mode() {
        let mut sr = ScreenReader::new();
        sr.enable();
        
        sr.set_navigation_mode(NavigationMode::Forms);
        assert_eq!(sr.navigation_mode(), NavigationMode::Forms);
    }
    
    #[test]
    fn test_verbosity() {
        let mut sr = ScreenReader::new();
        
        sr.set_verbosity(Verbosity::High);
        assert_eq!(sr.verbosity(), Verbosity::High);
    }
}
