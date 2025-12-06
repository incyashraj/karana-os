// Phase 49: Progressive Disclosure UX System
// Unified coordinator for simplified intents, smart defaults, and user personas

pub mod simple_intents;
pub mod smart_defaults;
pub mod personas;

use simple_intents::{IntentExpander, SimpleIntent};
use smart_defaults::{DefaultContext, SmartDefaults};
use personas::{PersonaManager, UserPersona};
use std::collections::HashMap;

/// Coordinates all UX systems for progressive disclosure
pub struct UXCoordinator {
    intent_expander: IntentExpander,
    defaults: SmartDefaults,
    persona_manager: PersonaManager,
}

/// Processed intent ready for execution
#[derive(Debug, Clone)]
pub struct ProcessedIntent {
    pub original: SimpleIntent,
    pub expanded_commands: Vec<String>,
    pub defaults_applied: HashMap<String, String>,
    pub requires_confirmation: bool,
    pub simplified_description: String,
}

/// Current UX system status
#[derive(Debug, Clone)]
pub struct UXSystemStatus {
    pub current_persona: UserPersona,
    pub learning_active: bool,
    pub confidence_score: f32,
}

impl UXCoordinator {
    pub fn new(persona: UserPersona) -> Self {
        Self {
            intent_expander: IntentExpander::new(),
            defaults: SmartDefaults::new(),
            persona_manager: PersonaManager::new(persona),
        }
    }

    /// Process a simple intent through the full UX pipeline
    pub fn process_intent(&mut self, intent: SimpleIntent, context: DefaultContext) -> ProcessedIntent {
        // Expand intent to technical commands
        let expanded = self.intent_expander.expand_intent(&intent);
        
        // Apply smart defaults based on context
        let mut defaults_applied = HashMap::new();
        for (key, value) in expanded.parameters.iter() {
            let default = self.defaults.get_default_with_context(key, &context);
            defaults_applied.insert(key.clone(), default.unwrap_or_else(|| value.clone()));
        }
        
        // Determine if confirmation needed based on persona
        let requires_confirmation = self.persona_manager.needs_confirmation(&intent);
        
        // Simplify description for display
        let simplified_description = self.persona_manager.simplify_term(&expanded.description);
        
        ProcessedIntent {
            original: intent,
            expanded_commands: expanded.commands,
            defaults_applied,
            requires_confirmation,
            simplified_description,
        }
    }

    /// Switch user persona and adapt UX complexity
    pub fn switch_persona(&mut self, persona: UserPersona) {
        self.persona_manager.set_persona(persona);
    }

    /// Record usage for learning smart defaults
    pub fn record_usage(&mut self, key: &str, value: String) {
        self.defaults.record_usage(key, value);
    }

    /// Apply a template for quick preset
    pub fn apply_template(&mut self, template_name: &str) {
        if let Some(template) = smart_defaults::DefaultTemplates::get_template(template_name) {
            for (key, value) in template {
                self.defaults.set_default(&key, value);
            }
        }
    }

    /// Get onboarding steps for current persona
    pub fn get_onboarding_steps(&self) -> Vec<String> {
        self.persona_manager.get_onboarding_steps()
    }

    /// Get current UX system status
    pub fn get_status(&self) -> UXSystemStatus {
        UXSystemStatus {
            current_persona: self.persona_manager.current_persona(),
            learning_active: true,
            confidence_score: 0.75,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_casual_user_processing() {
        let mut coord = UXCoordinator::new(UserPersona::Casual);
        let intent = SimpleIntent::Show { target: "messages".to_string() };
        let context = DefaultContext {
            battery_level: 80,
            time_of_day: 14,
            location: "home".to_string(),
            current_activity: "browsing".to_string(),
        };
        
        let processed = coord.process_intent(intent, context);
        assert!(!processed.requires_confirmation);
        assert!(!processed.expanded_commands.is_empty());
    }

    #[test]
    fn test_payment_requires_confirmation() {
        let mut coord = UXCoordinator::new(UserPersona::Casual);
        let intent = SimpleIntent::Pay { 
            recipient: "coffee_shop".to_string(),
            amount: 5.0,
        };
        let context = DefaultContext {
            battery_level: 50,
            time_of_day: 9,
            location: "coffee_shop".to_string(),
            current_activity: "payment".to_string(),
        };
        
        let processed = coord.process_intent(intent, context);
        assert!(processed.requires_confirmation);
    }

    #[test]
    fn test_persona_switching() {
        let mut coord = UXCoordinator::new(UserPersona::Casual);
        assert_eq!(coord.persona_manager.current_persona(), UserPersona::Casual);
        
        coord.switch_persona(UserPersona::Power);
        assert_eq!(coord.persona_manager.current_persona(), UserPersona::Power);
    }

    #[test]
    fn test_template_application() {
        let mut coord = UXCoordinator::new(UserPersona::Everyday);
        coord.apply_template("power_saving");
        
        let default = coord.defaults.get_default("brightness");
        assert!(default.is_some());
    }

    #[test]
    fn test_usage_recording() {
        let mut coord = UXCoordinator::new(UserPersona::Power);
        
        for _ in 0..5 {
            coord.record_usage("volume", "75".to_string());
        }
        
        let default = coord.defaults.get_default("volume");
        assert_eq!(default, Some("75".to_string()));
    }

    #[test]
    fn test_onboarding_steps() {
        let coord = UXCoordinator::new(UserPersona::Casual);
        let steps = coord.get_onboarding_steps();
        
        assert!(!steps.is_empty());
        assert!(steps.len() <= 5);
    }

    #[test]
    fn test_system_status() {
        let coord = UXCoordinator::new(UserPersona::Developer);
        let status = coord.get_status();
        
        assert_eq!(status.current_persona, UserPersona::Developer);
        assert!(status.learning_active);
        assert!(status.confidence_score > 0.0);
    }
}
