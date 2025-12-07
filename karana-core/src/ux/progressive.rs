// Kāraṇa OS - Phase 58: Progressive Disclosure UX
// Simplified UX layer hiding complexity, smart defaults, onboarding

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User experience level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UXLevel {
    /// Beginner - hide all complexity
    Beginner,
    
    /// Intermediate - show some advanced options
    Intermediate,
    
    /// Advanced - show most features
    Advanced,
    
    /// Expert - full access to all features
    Expert,
}

impl UXLevel {
    /// Get features visible at this level
    pub fn visible_features(&self) -> Vec<&'static str> {
        match self {
            Self::Beginner => vec![
                "voice_commands",
                "camera_capture",
                "basic_navigation",
            ],
            Self::Intermediate => vec![
                "voice_commands",
                "camera_capture",
                "basic_navigation",
                "ar_overlays",
                "notifications",
                "settings",
            ],
            Self::Advanced => vec![
                "voice_commands",
                "camera_capture",
                "basic_navigation",
                "ar_overlays",
                "notifications",
                "settings",
                "blockchain_wallet",
                "governance",
                "privacy_controls",
            ],
            Self::Expert => vec![
                "all_features",
            ],
        }
    }
}

/// Smart default configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartDefaults {
    pub auto_brightness: bool,
    pub adaptive_ai: bool,
    pub power_saving: bool,
    pub privacy_mode: PrivacyPreset,
    pub notification_level: NotificationLevel,
}

/// Privacy presets
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PrivacyPreset {
    /// Maximum privacy, minimal data collection
    Maximum,
    
    /// Balanced privacy and functionality
    Balanced,
    
    /// Minimal privacy, maximum functionality
    Minimal,
}

/// Notification level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NotificationLevel {
    Critical,
    Important,
    All,
}

impl Default for SmartDefaults {
    fn default() -> Self {
        Self {
            auto_brightness: true,
            adaptive_ai: true,
            power_saving: true,
            privacy_mode: PrivacyPreset::Balanced,
            notification_level: NotificationLevel::Important,
        }
    }
}

/// Onboarding step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub tutorial_text: String,
    pub action: Option<String>,
    pub completed: bool,
}

/// Progressive disclosure manager
pub struct ProgressiveUX {
    ux_level: UXLevel,
    defaults: SmartDefaults,
    onboarding_steps: Vec<OnboardingStep>,
    user_progress: HashMap<String, bool>,
}

impl ProgressiveUX {
    /// Create new progressive UX manager
    pub fn new() -> Self {
        Self {
            ux_level: UXLevel::Beginner,
            defaults: SmartDefaults::default(),
            onboarding_steps: Self::create_onboarding_steps(),
            user_progress: HashMap::new(),
        }
    }
    
    /// Create onboarding steps
    fn create_onboarding_steps() -> Vec<OnboardingStep> {
        vec![
            OnboardingStep {
                id: "welcome".to_string(),
                title: "Welcome to Kāraṇa OS".to_string(),
                description: "Your sovereign AR operating system".to_string(),
                tutorial_text: "Kāraṇa OS puts you in control of your data and digital life.".to_string(),
                action: None,
                completed: false,
            },
            OnboardingStep {
                id: "voice_setup".to_string(),
                title: "Voice Commands".to_string(),
                description: "Learn basic voice controls".to_string(),
                tutorial_text: "Try saying 'Hey Kāraṇa' to activate voice commands.".to_string(),
                action: Some("test_voice".to_string()),
                completed: false,
            },
            OnboardingStep {
                id: "ar_basics".to_string(),
                title: "AR Basics".to_string(),
                description: "Understanding your AR display".to_string(),
                tutorial_text: "Look around - content stays anchored in your space.".to_string(),
                action: Some("show_ar_demo".to_string()),
                completed: false,
            },
            OnboardingStep {
                id: "privacy_intro".to_string(),
                title: "Privacy First".to_string(),
                description: "Your data stays with you".to_string(),
                tutorial_text: "All processing happens on-device by default.".to_string(),
                action: None,
                completed: false,
            },
        ]
    }
    
    /// Set UX level
    pub fn set_level(&mut self, level: UXLevel) {
        self.ux_level = level;
    }
    
    /// Get current UX level
    pub fn level(&self) -> UXLevel {
        self.ux_level
    }
    
    /// Check if feature should be visible
    pub fn is_feature_visible(&self, feature: &str) -> bool {
        let visible = self.ux_level.visible_features();
        visible.contains(&feature) || visible.contains(&"all_features")
    }
    
    /// Get next onboarding step
    pub fn next_onboarding_step(&self) -> Option<&OnboardingStep> {
        self.onboarding_steps.iter().find(|s| !s.completed)
    }
    
    /// Complete onboarding step
    pub fn complete_step(&mut self, step_id: &str) -> Result<()> {
        if let Some(step) = self.onboarding_steps.iter_mut().find(|s| s.id == step_id) {
            step.completed = true;
            self.user_progress.insert(step_id.to_string(), true);
        }
        Ok(())
    }
    
    /// Get smart defaults
    pub fn defaults(&self) -> &SmartDefaults {
        &self.defaults
    }
    
    /// Update defaults
    pub fn set_defaults(&mut self, defaults: SmartDefaults) {
        self.defaults = defaults;
    }
    
    /// Get onboarding progress percentage
    pub fn onboarding_progress(&self) -> f32 {
        let total = self.onboarding_steps.len() as f32;
        let completed = self.onboarding_steps.iter().filter(|s| s.completed).count() as f32;
        (completed / total) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ux_levels() {
        assert_eq!(UXLevel::Beginner.visible_features().len(), 3);
        assert!(UXLevel::Expert.visible_features().contains(&"all_features"));
    }
    
    #[test]
    fn test_progressive_ux() {
        let mut ux = ProgressiveUX::new();
        assert_eq!(ux.level(), UXLevel::Beginner);
        
        ux.set_level(UXLevel::Advanced);
        assert!(ux.is_feature_visible("blockchain_wallet"));
    }
    
    #[test]
    fn test_onboarding() {
        let mut ux = ProgressiveUX::new();
        assert_eq!(ux.onboarding_progress(), 0.0);
        
        ux.complete_step("welcome").unwrap();
        assert!(ux.onboarding_progress() > 0.0);
    }
}
