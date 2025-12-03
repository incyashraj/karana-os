//! Accessibility System for Kāraṇa OS AR Glasses
//! 
//! Comprehensive accessibility features for users with visual, hearing,
//! motor, or cognitive disabilities. Essential for inclusive AR experiences.

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub mod vision;
pub mod hearing;
pub mod motor;
pub mod cognitive;
pub mod screen_reader;
pub mod magnifier;

pub use vision::{VisionAssist, ColorMode, ContrastMode};
pub use hearing::{HearingAssist, CaptionStyle, SoundType};
pub use motor::{MotorAssist, DwellMode, SwitchConfig, InputMethod};
pub use cognitive::{CognitiveAssist, ReadingMode, FocusLevel};
pub use screen_reader::{ScreenReader, SpeechConfig, Verbosity};
pub use magnifier::{Magnifier, MagnifierMode, LensShape};

/// Accessibility engine
#[derive(Debug)]
pub struct AccessibilityEngine {
    /// Vision assistance settings
    vision: VisionAssist,
    /// Hearing accessibility settings
    hearing: HearingAssist,
    /// Motor/mobility assistance
    motor: MotorAssist,
    /// Cognitive assistance
    cognitive: CognitiveAssist,
    /// Screen reader
    screen_reader: ScreenReader,
    /// Magnifier
    magnifier: Magnifier,
    /// Global enabled state
    enabled: bool,
    /// Active accessibility profiles
    profiles: HashMap<String, AccessibilityProfile>,
    /// Current active profile
    active_profile: Option<String>,
    /// Quick access shortcuts
    shortcuts: HashMap<String, AccessibilityAction>,
    /// Statistics
    stats: AccessibilityStats,
}

/// Accessibility profile
#[derive(Debug, Clone)]
pub struct AccessibilityProfile {
    /// Profile name
    pub name: String,
    /// Vision settings enabled
    pub vision_enabled: bool,
    /// Color mode
    pub color_mode: ColorMode,
    /// Contrast level
    pub contrast_mode: ContrastMode,
    /// Text size multiplier
    pub text_scale: f32,
    /// Screen reader enabled
    pub screen_reader_enabled: bool,
    /// Speech rate (words per minute)
    pub speech_rate: u32,
    /// Captions enabled
    pub captions_enabled: bool,
    /// Magnifier enabled
    pub magnifier_enabled: bool,
    /// Magnification level
    pub magnification: f32,
    /// Reduced motion
    pub reduced_motion: bool,
    /// Dwell selection enabled
    pub dwell_enabled: bool,
    /// Dwell time (ms)
    pub dwell_time_ms: u32,
}

impl Default for AccessibilityProfile {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            vision_enabled: false,
            color_mode: ColorMode::Normal,
            contrast_mode: ContrastMode::Normal,
            text_scale: 1.0,
            screen_reader_enabled: false,
            speech_rate: 150,
            captions_enabled: false,
            magnifier_enabled: false,
            magnification: 2.0,
            reduced_motion: false,
            dwell_enabled: false,
            dwell_time_ms: 1000,
        }
    }
}

/// Accessibility action types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessibilityAction {
    /// Toggle screen reader
    ToggleScreenReader,
    /// Toggle magnifier
    ToggleMagnifier,
    /// Increase text size
    IncreaseTextSize,
    /// Decrease text size
    DecreaseTextSize,
    /// Toggle high contrast
    ToggleHighContrast,
    /// Toggle color inversion
    ToggleColorInversion,
    /// Toggle captions
    ToggleCaptions,
    /// Toggle reduced motion
    ToggleReducedMotion,
    /// Speak current focus
    SpeakCurrentFocus,
    /// Emergency SOS
    EmergencySOS,
    /// Read scene description
    DescribeScene,
}

/// Accessibility statistics
#[derive(Debug, Default)]
pub struct AccessibilityStats {
    /// Screen reader usage time
    pub screen_reader_time: Duration,
    /// Magnifier usage time
    pub magnifier_time: Duration,
    /// Caption views
    pub captions_displayed: u64,
    /// Scene descriptions requested
    pub scene_descriptions: u64,
    /// Feature toggles count
    pub feature_toggles: u64,
}

impl AccessibilityEngine {
    /// Create new accessibility engine
    pub fn new() -> Self {
        Self {
            vision: VisionAssist::new(),
            hearing: HearingAssist::new(),
            motor: MotorAssist::new(),
            cognitive: CognitiveAssist::new(),
            screen_reader: ScreenReader::new(),
            magnifier: Magnifier::new(),
            enabled: true,
            profiles: Self::create_default_profiles(),
            active_profile: None,
            shortcuts: Self::create_default_shortcuts(),
            stats: AccessibilityStats::default(),
        }
    }
    
    /// Create default accessibility profiles
    fn create_default_profiles() -> HashMap<String, AccessibilityProfile> {
        let mut profiles = HashMap::new();
        
        // Default profile
        profiles.insert("default".to_string(), AccessibilityProfile::default());
        
        // Low vision profile
        profiles.insert("low_vision".to_string(), AccessibilityProfile {
            name: "Low Vision".to_string(),
            vision_enabled: true,
            color_mode: ColorMode::Normal,
            contrast_mode: ContrastMode::High,
            text_scale: 1.5,
            screen_reader_enabled: true,
            speech_rate: 150,
            captions_enabled: true,
            magnifier_enabled: true,
            magnification: 2.5,
            reduced_motion: false,
            dwell_enabled: false,
            dwell_time_ms: 1000,
        });
        
        // Blind user profile
        profiles.insert("blind".to_string(), AccessibilityProfile {
            name: "Blind/Screen Reader".to_string(),
            vision_enabled: true,
            color_mode: ColorMode::Normal,
            contrast_mode: ContrastMode::Normal,
            text_scale: 1.0,
            screen_reader_enabled: true,
            speech_rate: 200,
            captions_enabled: false,
            magnifier_enabled: false,
            magnification: 1.0,
            reduced_motion: true,
            dwell_enabled: false,
            dwell_time_ms: 1000,
        });
        
        // Motor impairment profile
        profiles.insert("motor".to_string(), AccessibilityProfile {
            name: "Motor Assistance".to_string(),
            vision_enabled: false,
            color_mode: ColorMode::Normal,
            contrast_mode: ContrastMode::Normal,
            text_scale: 1.2,
            screen_reader_enabled: false,
            speech_rate: 150,
            captions_enabled: false,
            magnifier_enabled: false,
            magnification: 1.0,
            reduced_motion: true,
            dwell_enabled: true,
            dwell_time_ms: 800,
        });
        
        // Hearing impairment profile  
        profiles.insert("hearing".to_string(), AccessibilityProfile {
            name: "Hearing Assistance".to_string(),
            vision_enabled: false,
            color_mode: ColorMode::Normal,
            contrast_mode: ContrastMode::Normal,
            text_scale: 1.0,
            screen_reader_enabled: false,
            speech_rate: 150,
            captions_enabled: true,
            magnifier_enabled: false,
            magnification: 1.0,
            reduced_motion: false,
            dwell_enabled: false,
            dwell_time_ms: 1000,
        });
        
        // Cognitive assistance profile
        profiles.insert("cognitive".to_string(), AccessibilityProfile {
            name: "Cognitive Assistance".to_string(),
            vision_enabled: false,
            color_mode: ColorMode::Normal,
            contrast_mode: ContrastMode::Normal,
            text_scale: 1.3,
            screen_reader_enabled: false,
            speech_rate: 120,
            captions_enabled: true,
            magnifier_enabled: false,
            magnification: 1.0,
            reduced_motion: true,
            dwell_enabled: true,
            dwell_time_ms: 1500,
        });
        
        profiles
    }
    
    /// Create default shortcuts
    fn create_default_shortcuts() -> HashMap<String, AccessibilityAction> {
        let mut shortcuts = HashMap::new();
        
        shortcuts.insert("triple_tap".to_string(), AccessibilityAction::ToggleScreenReader);
        shortcuts.insert("pinch_zoom".to_string(), AccessibilityAction::ToggleMagnifier);
        shortcuts.insert("voice_contrast".to_string(), AccessibilityAction::ToggleHighContrast);
        shortcuts.insert("voice_describe".to_string(), AccessibilityAction::DescribeScene);
        shortcuts.insert("long_press_sos".to_string(), AccessibilityAction::EmergencySOS);
        
        shortcuts
    }
    
    /// Apply accessibility profile
    pub fn apply_profile(&mut self, profile_name: &str) -> bool {
        if let Some(profile) = self.profiles.get(profile_name).cloned() {
            // Apply vision settings
            self.vision.set_color_mode(profile.color_mode);
            self.vision.set_contrast_mode(profile.contrast_mode);
            self.vision.set_text_scale(profile.text_scale);
            
            // Apply screen reader
            if profile.screen_reader_enabled {
                self.screen_reader.enable();
                self.screen_reader.set_speech_rate(profile.speech_rate);
            } else {
                self.screen_reader.disable();
            }
            
            // Apply magnifier
            if profile.magnifier_enabled {
                self.magnifier.enable();
                self.magnifier.set_zoom(profile.magnification);
            } else {
                self.magnifier.disable();
            }
            
            // Apply captions
            self.hearing.set_captions_enabled(profile.captions_enabled);
            
            // Apply motor settings
            if profile.dwell_enabled {
                self.motor.set_dwell_mode(DwellMode::Simple);
                self.motor.set_dwell_time(std::time::Duration::from_millis(profile.dwell_time_ms as u64));
            } else {
                self.motor.set_dwell_mode(DwellMode::Disabled);
            }
            
            // Apply cognitive settings
            if profile.reduced_motion {
                self.cognitive.set_reduce_motion(true);
            } else {
                self.cognitive.set_reduce_motion(false);
            }
            
            self.active_profile = Some(profile_name.to_string());
            self.stats.feature_toggles += 1;
            
            true
        } else {
            false
        }
    }
    
    /// Get current profile name
    pub fn current_profile(&self) -> Option<&str> {
        self.active_profile.as_deref()
    }
    
    /// Execute accessibility action
    pub fn execute_action(&mut self, action: AccessibilityAction) {
        self.stats.feature_toggles += 1;
        
        match action {
            AccessibilityAction::ToggleScreenReader => {
                self.screen_reader.toggle();
            }
            AccessibilityAction::ToggleMagnifier => {
                self.magnifier.toggle();
            }
            AccessibilityAction::IncreaseTextSize => {
                let current = self.vision.text_scale();
                self.vision.set_text_scale((current + 0.1).min(3.0));
            }
            AccessibilityAction::DecreaseTextSize => {
                let current = self.vision.text_scale();
                self.vision.set_text_scale((current - 0.1).max(0.5));
            }
            AccessibilityAction::ToggleHighContrast => {
                self.vision.toggle_high_contrast();
            }
            AccessibilityAction::ToggleColorInversion => {
                self.vision.toggle_inversion();
            }
            AccessibilityAction::ToggleCaptions => {
                let current = self.hearing.captions_enabled();
                self.hearing.set_captions_enabled(!current);
            }
            AccessibilityAction::ToggleReducedMotion => {
                let current = self.cognitive.reduce_motion();
                self.cognitive.set_reduce_motion(!current);
            }
            AccessibilityAction::SpeakCurrentFocus => {
                // Trigger screen reader to speak current focus
                self.screen_reader.speak_focus();
            }
            AccessibilityAction::EmergencySOS => {
                self.trigger_emergency_sos();
            }
            AccessibilityAction::DescribeScene => {
                self.stats.scene_descriptions += 1;
                self.screen_reader.describe_scene();
            }
        }
    }
    
    /// Trigger emergency SOS
    fn trigger_emergency_sos(&mut self) {
        // In real implementation, this would:
        // 1. Send location to emergency contacts
        // 2. Play loud alert
        // 3. Display emergency info on screen
        // 4. Connect to emergency services if configured
    }
    
    /// Process shortcut trigger
    pub fn process_shortcut(&mut self, shortcut: &str) -> bool {
        if let Some(action) = self.shortcuts.get(shortcut).cloned() {
            self.execute_action(action);
            true
        } else {
            false
        }
    }
    
    /// Get vision assist
    pub fn vision(&self) -> &VisionAssist {
        &self.vision
    }
    
    /// Get mutable vision assist
    pub fn vision_mut(&mut self) -> &mut VisionAssist {
        &mut self.vision
    }
    
    /// Get screen reader
    pub fn screen_reader(&self) -> &ScreenReader {
        &self.screen_reader
    }
    
    /// Get mutable screen reader
    pub fn screen_reader_mut(&mut self) -> &mut ScreenReader {
        &mut self.screen_reader
    }
    
    /// Get magnifier
    pub fn magnifier(&self) -> &Magnifier {
        &self.magnifier
    }
    
    /// Get mutable magnifier
    pub fn magnifier_mut(&mut self) -> &mut Magnifier {
        &mut self.magnifier
    }
    
    /// Get hearing accessibility
    pub fn hearing(&self) -> &HearingAssist {
        &self.hearing
    }
    
    /// Get mutable hearing accessibility
    pub fn hearing_mut(&mut self) -> &mut HearingAssist {
        &mut self.hearing
    }
    
    /// Get motor assist
    pub fn motor(&self) -> &MotorAssist {
        &self.motor
    }
    
    /// Get cognitive assist
    pub fn cognitive(&self) -> &CognitiveAssist {
        &self.cognitive
    }
    
    /// Check if screen reader is active
    pub fn is_screen_reader_active(&self) -> bool {
        self.screen_reader.is_enabled()
    }
    
    /// Check if reduced motion is enabled
    pub fn is_reduced_motion(&self) -> bool {
        self.cognitive.reduce_motion()
    }
    
    /// Get text scale
    pub fn text_scale(&self) -> f32 {
        self.vision.text_scale()
    }
    
    /// Get statistics
    pub fn stats(&self) -> &AccessibilityStats {
        &self.stats
    }
    
    /// List available profiles
    pub fn list_profiles(&self) -> Vec<&str> {
        self.profiles.keys().map(|s| s.as_str()).collect()
    }
    
    /// Add custom profile
    pub fn add_profile(&mut self, name: &str, profile: AccessibilityProfile) {
        self.profiles.insert(name.to_string(), profile);
    }
}

impl Default for AccessibilityEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_engine_creation() {
        let engine = AccessibilityEngine::new();
        assert!(engine.list_profiles().len() > 0);
    }
    
    #[test]
    fn test_apply_profile() {
        let mut engine = AccessibilityEngine::new();
        
        assert!(engine.apply_profile("low_vision"));
        assert_eq!(engine.current_profile(), Some("low_vision"));
        assert!(engine.is_screen_reader_active());
    }
    
    #[test]
    fn test_toggle_actions() {
        let mut engine = AccessibilityEngine::new();
        
        let initial = engine.is_screen_reader_active();
        engine.execute_action(AccessibilityAction::ToggleScreenReader);
        assert_ne!(engine.is_screen_reader_active(), initial);
    }
    
    #[test]
    fn test_text_scale() {
        let mut engine = AccessibilityEngine::new();
        
        let initial = engine.text_scale();
        engine.execute_action(AccessibilityAction::IncreaseTextSize);
        assert!(engine.text_scale() > initial);
    }
    
    #[test]
    fn test_shortcut_processing() {
        let mut engine = AccessibilityEngine::new();
        
        assert!(engine.process_shortcut("triple_tap"));
        assert!(engine.is_screen_reader_active());
    }
    
    #[test]
    fn test_default_profiles() {
        let engine = AccessibilityEngine::new();
        
        let profiles = engine.list_profiles();
        assert!(profiles.contains(&"default"));
        assert!(profiles.contains(&"low_vision"));
        assert!(profiles.contains(&"blind"));
        assert!(profiles.contains(&"motor"));
        assert!(profiles.contains(&"hearing"));
    }
}
