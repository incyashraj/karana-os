// Phase 49: User Personas System
// Adapt UX complexity based on user expertise level

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::ux::simple_intents::SimpleIntent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserPersona {
    Casual,
    Everyday,
    Power,
    Developer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Advanced,
    Technical,
}

pub struct PersonaManager {
    current_persona: UserPersona,
    feature_visibility: HashMap<String, ComplexityLevel>,
}

impl PersonaManager {
    pub fn new(persona: UserPersona) -> Self {
        let mut feature_visibility = HashMap::new();
        
        feature_visibility.insert("basic_navigation".to_string(), ComplexityLevel::Simple);
        feature_visibility.insert("voice_commands".to_string(), ComplexityLevel::Simple);
        feature_visibility.insert("payment".to_string(), ComplexityLevel::Moderate);
        feature_visibility.insert("privacy_settings".to_string(), ComplexityLevel::Moderate);
        feature_visibility.insert("custom_shortcuts".to_string(), ComplexityLevel::Advanced);
        feature_visibility.insert("automation_rules".to_string(), ComplexityLevel::Advanced);
        feature_visibility.insert("system_logs".to_string(), ComplexityLevel::Technical);
        feature_visibility.insert("blockchain_explorer".to_string(), ComplexityLevel::Technical);
        feature_visibility.insert("api_access".to_string(), ComplexityLevel::Technical);
        feature_visibility.insert("developer_console".to_string(), ComplexityLevel::Technical);
        feature_visibility.insert("performance_profiler".to_string(), ComplexityLevel::Technical);
        feature_visibility.insert("network_inspector".to_string(), ComplexityLevel::Technical);
        
        Self {
            current_persona: persona,
            feature_visibility,
        }
    }

    pub fn is_feature_visible(&self, feature: &str) -> bool {
        let complexity = self.feature_visibility.get(feature)
            .copied()
            .unwrap_or(ComplexityLevel::Simple);
        
        match self.current_persona {
            UserPersona::Casual => matches!(complexity, ComplexityLevel::Simple),
            UserPersona::Everyday => matches!(complexity, ComplexityLevel::Simple | ComplexityLevel::Moderate),
            UserPersona::Power => !matches!(complexity, ComplexityLevel::Technical),
            UserPersona::Developer => true,
        }
    }

    pub fn current_persona(&self) -> UserPersona {
        self.current_persona
    }

    pub fn set_persona(&mut self, persona: UserPersona) {
        self.current_persona = persona;
    }

    pub fn get_visible_features(&self) -> Vec<String> {
        self.feature_visibility.keys()
            .filter(|f| self.is_feature_visible(f))
            .cloned()
            .collect()
    }

    pub fn needs_confirmation(&self, intent: &SimpleIntent) -> bool {
        match intent {
            SimpleIntent::Pay { .. } => true,
            SimpleIntent::Share { .. } => {
                matches!(self.current_persona, UserPersona::Casual | UserPersona::Everyday)
            },
            SimpleIntent::Settings { .. } => {
                matches!(self.current_persona, UserPersona::Casual)
            },
            _ => false,
        }
    }

    pub fn simplify_term(&self, term: &str) -> String {
        if matches!(self.current_persona, UserPersona::Developer | UserPersona::Power) {
            return term.to_string();
        }
        
        match term {
            s if s.contains("blockchain") => "secure ledger".to_string(),
            s if s.contains("transaction") => "payment".to_string(),
            s if s.contains("cryptocurrency") => "digital money".to_string(),
            s if s.contains("decentralized") => "peer-to-peer".to_string(),
            s if s.contains("consensus") => "agreement".to_string(),
            s if s.contains("hash") => "fingerprint".to_string(),
            _ => term.to_string(),
        }
    }

    pub fn onboarding_duration(&self) -> u32 {
        match self.current_persona {
            UserPersona::Casual => 3,
            UserPersona::Everyday => 5,
            UserPersona::Power => 10,
            UserPersona::Developer => 15,
        }
    }

    pub fn get_onboarding_steps(&self) -> Vec<String> {
        match self.current_persona {
            UserPersona::Casual => vec![
                "Welcome! Let's get started".to_string(),
                "Try saying 'Show messages'".to_string(),
                "Swipe left to see apps".to_string(),
            ],
            UserPersona::Everyday => vec![
                "Welcome to K??ra???a OS".to_string(),
                "Voice commands and gestures".to_string(),
                "Your digital wallet".to_string(),
                "Privacy settings".to_string(),
            ],
            UserPersona::Power => vec![
                "Advanced features overview".to_string(),
                "Custom shortcuts".to_string(),
                "Automation rules".to_string(),
                "Multi-app workflows".to_string(),
                "Performance monitoring".to_string(),
            ],
            UserPersona::Developer => vec![
                "Developer mode activated".to_string(),
                "API access and keys".to_string(),
                "System logs and debugging".to_string(),
                "Blockchain integration".to_string(),
                "Custom app development".to_string(),
                "Performance profiling tools".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_visibility() {
        let manager = PersonaManager::new(UserPersona::Casual);
        assert!(manager.is_feature_visible("basic_navigation"));
        assert!(manager.is_feature_visible("voice_commands"));
        assert!(!manager.is_feature_visible("automation_rules"));
        assert!(!manager.is_feature_visible("developer_console"));
    }

    #[test]
    fn test_persona_manager() {
        let mut manager = PersonaManager::new(UserPersona::Everyday);
        assert_eq!(manager.current_persona(), UserPersona::Everyday);
        assert!(manager.is_feature_visible("payment"));
        assert!(!manager.is_feature_visible("developer_console"));
        
        manager.set_persona(UserPersona::Developer);
        assert!(manager.is_feature_visible("developer_console"));
    }

    #[test]
    fn test_persona_switching() {
        let mut manager = PersonaManager::new(UserPersona::Casual);
        let casual_features = manager.get_visible_features();
        
        manager.set_persona(UserPersona::Developer);
        let dev_features = manager.get_visible_features();
        
        assert!(dev_features.len() > casual_features.len());
    }

    #[test]
    fn test_visible_feature_categories() {
        let casual = PersonaManager::new(UserPersona::Casual);
        let power = PersonaManager::new(UserPersona::Power);
        let dev = PersonaManager::new(UserPersona::Developer);
        
        let casual_count = casual.get_visible_features().len();
        let power_count = power.get_visible_features().len();
        let dev_count = dev.get_visible_features().len();
        
        assert!(casual_count < power_count);
        assert!(power_count < dev_count);
    }

    #[test]
    fn test_user_preferences() {
        let manager = PersonaManager::new(UserPersona::Power);
        assert!(manager.is_feature_visible("custom_shortcuts"));
        assert!(manager.is_feature_visible("automation_rules"));
        assert!(!manager.is_feature_visible("system_logs"));
    }

    #[test]
    fn test_confirmation_logic() {
        let casual = PersonaManager::new(UserPersona::Casual);
        let dev = PersonaManager::new(UserPersona::Developer);
        
        let payment = SimpleIntent::Pay { recipient: "store".to_string(), amount: 10.0 };
        let share = SimpleIntent::Share { content: "photo".to_string(), target: "friend".to_string() };
        let settings = SimpleIntent::Settings { category: "display".to_string() };
        
        assert!(casual.needs_confirmation(&payment));
        assert!(dev.needs_confirmation(&payment));
        assert!(casual.needs_confirmation(&share));
        assert!(!dev.needs_confirmation(&share));
        assert!(casual.needs_confirmation(&settings));
        assert!(!dev.needs_confirmation(&settings));
    }

    #[test]
    fn test_term_simplification() {
        let casual = PersonaManager::new(UserPersona::Casual);
        let dev = PersonaManager::new(UserPersona::Developer);
        
        assert_eq!(casual.simplify_term("blockchain transaction"), "secure ledger");
        assert_eq!(dev.simplify_term("blockchain transaction"), "blockchain transaction");
        assert_eq!(casual.simplify_term("cryptocurrency payment"), "digital money");
        assert_eq!(casual.simplify_term("decentralized network"), "peer-to-peer");
    }

    #[test]
    fn test_onboarding_duration() {
        assert_eq!(PersonaManager::new(UserPersona::Casual).onboarding_duration(), 3);
        assert_eq!(PersonaManager::new(UserPersona::Everyday).onboarding_duration(), 5);
        assert_eq!(PersonaManager::new(UserPersona::Power).onboarding_duration(), 10);
        assert_eq!(PersonaManager::new(UserPersona::Developer).onboarding_duration(), 15);
    }

    #[test]
    fn test_onboarding_steps() {
        let casual = PersonaManager::new(UserPersona::Casual);
        let dev = PersonaManager::new(UserPersona::Developer);
        
        let casual_steps = casual.get_onboarding_steps();
        let dev_steps = dev.get_onboarding_steps();
        
        assert!(casual_steps.len() < 5);
        assert!(dev_steps.len() > casual_steps.len());
        assert!(dev_steps.iter().any(|s| s.contains("API") || s.contains("Developer")));
    }
}
