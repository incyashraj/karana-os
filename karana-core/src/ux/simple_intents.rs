// Phase 49: Simplified Intent System
// Hide technical complexity behind voice-like natural intents

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple, user-friendly intents that hide technical complexity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SimpleIntent {
    Show { target: String },
    Find { query: String },
    Send { to: String, content: String },
    Pay { recipient: String, amount: f64 },
    Remember { note: String },
    Remind { task: String, when: String },
    Open { app: String },
    Close { app: String },
    Navigate { destination: String },
    Call { contact: String },
    Record { media_type: String },
    Share { content: String, target: String },
    Settings { category: String },
    Help { topic: String },
}

/// Privacy level for operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrivacyLevel {
    Public,
    Private,
    Sensitive,
    Critical,
}

/// Expanded technical representation of an intent
#[derive(Debug, Clone)]
pub struct ExpandedIntent {
    pub commands: Vec<String>,
    pub parameters: HashMap<String, String>,
    pub privacy_level: PrivacyLevel,
    pub description: String,
}

/// Converts simple intents to technical layer operations
pub struct IntentExpander {
    command_map: HashMap<String, Vec<String>>,
}

impl IntentExpander {
    pub fn new() -> Self {
        let mut command_map = HashMap::new();
        
        command_map.insert("show_messages".to_string(), vec![
            "ai_layer::activate_app('messages')".to_string(),
            "ui::focus_window('com.karana.messages')".to_string(),
        ]);
        
        command_map.insert("pay".to_string(), vec![
            "wallet::prepare_transaction".to_string(),
            "economy::execute_payment".to_string(),
            "notifications::send_receipt".to_string(),
        ]);
        
        command_map.insert("record_video".to_string(), vec![
            "camera::start_capture('video')".to_string(),
            "storage::allocate_space".to_string(),
            "privacy::check_permissions".to_string(),
        ]);
        
        Self { command_map }
    }

    /// Expand a simple intent into technical operations
    pub fn expand_intent(&self, intent: &SimpleIntent) -> ExpandedIntent {
        match intent {
            SimpleIntent::Show { target } => {
                let key = format!("show_{}", target);
                let commands = self.command_map.get(&key)
                    .cloned()
                    .unwrap_or_else(|| vec![format!("ui::show('{}')", target)]);
                
                ExpandedIntent {
                    commands,
                    parameters: HashMap::from([("target".to_string(), target.clone())]),
                    privacy_level: PrivacyLevel::Public,
                    description: format!("Display {} on screen", target),
                }
            },
            SimpleIntent::Pay { recipient, amount } => {
                let commands = self.command_map.get("pay")
                    .cloned()
                    .unwrap_or_default();
                
                ExpandedIntent {
                    commands,
                    parameters: HashMap::from([
                        ("recipient".to_string(), recipient.clone()),
                        ("amount".to_string(), amount.to_string()),
                    ]),
                    privacy_level: PrivacyLevel::Critical,
                    description: format!("Send {} tokens to {}", amount, recipient),
                }
            },
            SimpleIntent::Record { media_type } => {
                let key = format!("record_{}", media_type);
                let commands = self.command_map.get(&key)
                    .cloned()
                    .unwrap_or_else(|| vec![format!("camera::record('{}')", media_type)]);
                
                ExpandedIntent {
                    commands,
                    parameters: HashMap::from([("media_type".to_string(), media_type.clone())]),
                    privacy_level: PrivacyLevel::Sensitive,
                    description: format!("Start {} recording", media_type),
                }
            },
            SimpleIntent::Navigate { destination } => {
                ExpandedIntent {
                    commands: vec![format!("navigation::route_to('{}')", destination)],
                    parameters: HashMap::from([("destination".to_string(), destination.clone())]),
                    privacy_level: PrivacyLevel::Private,
                    description: format!("Navigate to {}", destination),
                }
            },
            _ => {
                ExpandedIntent {
                    commands: vec!["intent::execute".to_string()],
                    parameters: HashMap::new(),
                    privacy_level: PrivacyLevel::Public,
                    description: "Execute intent".to_string(),
                }
            }
        }
    }
}

/// Converts technical operations to simple descriptions
pub struct IntentSimplifier;

impl IntentSimplifier {
    pub fn simplify(command: &str) -> String {
        if command.contains("wallet::prepare_transaction") {
            "Preparing payment...".to_string()
        } else if command.contains("camera::start_capture") {
            "Starting camera...".to_string()
        } else if command.contains("navigation::route_to") {
            "Finding best route...".to_string()
        } else if command.contains("ui::show") {
            "Opening...".to_string()
        } else {
            "Working...".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_intent_show() {
        let expander = IntentExpander::new();
        let intent = SimpleIntent::Show { target: "messages".to_string() };
        let expanded = expander.expand_intent(&intent);
        
        assert!(!expanded.commands.is_empty());
        assert_eq!(expanded.privacy_level, PrivacyLevel::Public);
        assert!(expanded.description.contains("messages"));
    }

    #[test]
    fn test_payment_intent() {
        let expander = IntentExpander::new();
        let intent = SimpleIntent::Pay { 
            recipient: "alice".to_string(),
            amount: 10.0,
        };
        let expanded = expander.expand_intent(&intent);
        
        assert_eq!(expanded.privacy_level, PrivacyLevel::Critical);
        assert!(expanded.commands.len() >= 1);
        assert_eq!(expanded.parameters.get("amount").unwrap(), "10");
    }

    #[test]
    fn test_intent_simplification() {
        let simplified = IntentSimplifier::simplify("wallet::prepare_transaction");
        assert_eq!(simplified, "Preparing payment...");
        
        let simplified = IntentSimplifier::simplify("camera::start_capture");
        assert_eq!(simplified, "Starting camera...");
    }

    #[test]
    fn test_recording_intent() {
        let expander = IntentExpander::new();
        let intent = SimpleIntent::Record { media_type: "video".to_string() };
        let expanded = expander.expand_intent(&intent);
        
        assert_eq!(expanded.privacy_level, PrivacyLevel::Sensitive);
        assert!(expanded.description.contains("video"));
    }

    #[test]
    fn test_navigation_intent() {
        let expander = IntentExpander::new();
        let intent = SimpleIntent::Navigate { destination: "home".to_string() };
        let expanded = expander.expand_intent(&intent);
        
        assert_eq!(expanded.privacy_level, PrivacyLevel::Private);
        assert!(expanded.commands[0].contains("navigation"));
    }

    #[test]
    fn test_privacy_levels() {
        let expander = IntentExpander::new();
        
        let show = expander.expand_intent(&SimpleIntent::Show { target: "menu".to_string() });
        assert_eq!(show.privacy_level, PrivacyLevel::Public);
        
        let pay = expander.expand_intent(&SimpleIntent::Pay { recipient: "bob".to_string(), amount: 5.0 });
        assert_eq!(pay.privacy_level, PrivacyLevel::Critical);
        
        let nav = expander.expand_intent(&SimpleIntent::Navigate { destination: "work".to_string() });
        assert_eq!(nav.privacy_level, PrivacyLevel::Private);
    }

    #[test]
    fn test_intent_confirmation_logic() {
        let expander = IntentExpander::new();
        
        let payment = expander.expand_intent(&SimpleIntent::Pay { 
            recipient: "store".to_string(),
            amount: 100.0,
        });
        assert_eq!(payment.privacy_level, PrivacyLevel::Critical);
        
        let show = expander.expand_intent(&SimpleIntent::Show { target: "calendar".to_string() });
        assert_eq!(show.privacy_level, PrivacyLevel::Public);
    }

    #[test]
    fn test_default_expansion() {
        let expander = IntentExpander::new();
        let intent = SimpleIntent::Find { query: "coffee shops".to_string() };
        let expanded = expander.expand_intent(&intent);
        
        assert!(!expanded.commands.is_empty());
        assert_eq!(expanded.privacy_level, PrivacyLevel::Public);
    }
}
