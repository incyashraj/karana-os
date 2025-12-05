//! Intent Resolution System
//!
//! Resolves and refines intents based on context, dialogue state, and user history.

use super::*;
use super::nlu::NluResult;
use super::dialogue::DialogueState;
use std::collections::HashMap;

/// Intent resolver
pub struct IntentResolver {
    /// Intent definitions
    intent_defs: HashMap<String, IntentDefinition>,
    /// Intent hierarchy (parent -> children)
    hierarchy: HashMap<String, Vec<String>>,
    /// Disambiguation rules
    disambiguation_rules: Vec<DisambiguationRule>,
    /// User-specific intent preferences
    user_preferences: HashMap<String, f32>,
    /// Learning buffer for reinforcement
    learning_buffer: Vec<IntentFeedbackRecord>,
}

impl IntentResolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            intent_defs: HashMap::new(),
            hierarchy: HashMap::new(),
            disambiguation_rules: Vec::new(),
            user_preferences: HashMap::new(),
            learning_buffer: Vec::new(),
        };
        resolver.initialize_intents();
        resolver.initialize_disambiguation();
        resolver
    }
    
    fn initialize_intents(&mut self) {
        // Navigation intents
        self.add_intent(IntentDefinition {
            name: "navigation".to_string(),
            category: IntentCategory::Navigation,
            required_slots: vec!["destination".to_string()],
            optional_slots: vec!["travel_mode".to_string(), "departure_time".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "navigate to".to_string(),
                "directions to".to_string(),
                "take me to".to_string(),
            ],
        });
        
        // Communication intents
        self.add_intent(IntentDefinition {
            name: "call".to_string(),
            category: IntentCategory::Communication,
            required_slots: vec!["contact".to_string()],
            optional_slots: vec!["call_type".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "call".to_string(),
                "phone".to_string(),
                "dial".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "message".to_string(),
            category: IntentCategory::Communication,
            required_slots: vec!["contact".to_string()],
            optional_slots: vec!["message_content".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "message".to_string(),
                "text".to_string(),
                "send message to".to_string(),
            ],
        });
        
        // Media intents
        self.add_intent(IntentDefinition {
            name: "play_media".to_string(),
            category: IntentCategory::Media,
            required_slots: vec![],
            optional_slots: vec!["media_name".to_string(), "artist".to_string(), "playlist".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "play".to_string(),
                "play music".to_string(),
                "play song".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "volume".to_string(),
            category: IntentCategory::Media,
            required_slots: vec!["direction".to_string()],
            optional_slots: vec!["level".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "volume up".to_string(),
                "louder".to_string(),
                "softer".to_string(),
            ],
        });
        
        // Camera intents
        self.add_intent(IntentDefinition {
            name: "take_photo".to_string(),
            category: IntentCategory::Camera,
            required_slots: vec![],
            optional_slots: vec!["mode".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "take a photo".to_string(),
                "capture".to_string(),
                "take picture".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "record_video".to_string(),
            category: IntentCategory::Camera,
            required_slots: vec![],
            optional_slots: vec!["duration".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "record video".to_string(),
                "start recording".to_string(),
            ],
        });
        
        // Timer/Reminder intents
        self.add_intent(IntentDefinition {
            name: "set_timer".to_string(),
            category: IntentCategory::Productivity,
            required_slots: vec!["duration".to_string()],
            optional_slots: vec!["label".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "set timer".to_string(),
                "timer for".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "set_reminder".to_string(),
            category: IntentCategory::Productivity,
            required_slots: vec!["content".to_string()],
            optional_slots: vec!["time".to_string(), "location".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "remind me".to_string(),
                "set reminder".to_string(),
            ],
        });
        
        // Wallet/Blockchain intents
        self.add_intent(IntentDefinition {
            name: "check_balance".to_string(),
            category: IntentCategory::Finance,
            required_slots: vec![],
            optional_slots: vec!["currency".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "check balance".to_string(),
                "how much".to_string(),
                "show wallet".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "transfer".to_string(),
            category: IntentCategory::Finance,
            required_slots: vec!["amount".to_string(), "recipient".to_string()],
            optional_slots: vec!["currency".to_string()],
            confirmation_required: true, // Always confirm financial actions
            feasible_on_glasses: true,
            example_phrases: vec![
                "send".to_string(),
                "transfer".to_string(),
                "pay".to_string(),
            ],
        });
        
        // Question/Search intents
        self.add_intent(IntentDefinition {
            name: "question".to_string(),
            category: IntentCategory::Information,
            required_slots: vec!["query".to_string()],
            optional_slots: vec![],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "what is".to_string(),
                "who is".to_string(),
                "how do".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "search".to_string(),
            category: IntentCategory::Information,
            required_slots: vec!["query".to_string()],
            optional_slots: vec!["source".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "search".to_string(),
                "find".to_string(),
                "look up".to_string(),
            ],
        });
        
        // Vision/AR intents
        self.add_intent(IntentDefinition {
            name: "identify_object".to_string(),
            category: IntentCategory::Vision,
            required_slots: vec![],
            optional_slots: vec![],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "what is this".to_string(),
                "identify".to_string(),
                "what am i looking at".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "translate".to_string(),
            category: IntentCategory::Vision,
            required_slots: vec![],
            optional_slots: vec!["target_language".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "translate".to_string(),
                "what does this say".to_string(),
            ],
        });
        
        // App control intents
        self.add_intent(IntentDefinition {
            name: "launch_app".to_string(),
            category: IntentCategory::System,
            required_slots: vec!["app_name".to_string()],
            optional_slots: vec![],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "open".to_string(),
                "launch".to_string(),
                "start".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "close_app".to_string(),
            category: IntentCategory::System,
            required_slots: vec![],
            optional_slots: vec!["app_name".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "close".to_string(),
                "quit".to_string(),
                "exit".to_string(),
            ],
        });
        
        // Settings intents
        self.add_intent(IntentDefinition {
            name: "adjust_brightness".to_string(),
            category: IntentCategory::Settings,
            required_slots: vec!["direction".to_string()],
            optional_slots: vec!["level".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "brightness".to_string(),
                "brighter".to_string(),
                "dimmer".to_string(),
            ],
        });
        
        // Gallery/Photos intents
        self.add_intent(IntentDefinition {
            name: "show_gallery".to_string(),
            category: IntentCategory::Media,
            required_slots: vec![],
            optional_slots: vec!["filter".to_string(), "date".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "show photos".to_string(),
                "open gallery".to_string(),
                "show pictures".to_string(),
                "my photos".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "show_photo".to_string(),
            category: IntentCategory::Media,
            required_slots: vec![],
            optional_slots: vec!["query".to_string(), "date".to_string(), "location".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "show me".to_string(),
                "show that photo".to_string(),
                "show picture from".to_string(),
                "last photo".to_string(),
                "recent photos".to_string(),
            ],
        });
        
        // Download intents
        self.add_intent(IntentDefinition {
            name: "download".to_string(),
            category: IntentCategory::System,
            required_slots: vec!["item".to_string()],
            optional_slots: vec!["source".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "download".to_string(),
                "get".to_string(),
                "save".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "show_downloads".to_string(),
            category: IntentCategory::System,
            required_slots: vec![],
            optional_slots: vec![],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "show downloads".to_string(),
                "my downloads".to_string(),
                "downloaded files".to_string(),
            ],
        });
        
        // Notifications intents
        self.add_intent(IntentDefinition {
            name: "show_notifications".to_string(),
            category: IntentCategory::System,
            required_slots: vec![],
            optional_slots: vec!["app".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "show notifications".to_string(),
                "any notifications".to_string(),
                "what's new".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "clear_notifications".to_string(),
            category: IntentCategory::System,
            required_slots: vec![],
            optional_slots: vec!["app".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "clear notifications".to_string(),
                "dismiss all".to_string(),
            ],
        });
        
        // Media control intents
        self.add_intent(IntentDefinition {
            name: "pause_media".to_string(),
            category: IntentCategory::Media,
            required_slots: vec![],
            optional_slots: vec![],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "pause".to_string(),
                "stop music".to_string(),
                "pause playback".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "skip_track".to_string(),
            category: IntentCategory::Media,
            required_slots: vec![],
            optional_slots: vec!["direction".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "next song".to_string(),
                "skip".to_string(),
                "previous".to_string(),
                "go back".to_string(),
            ],
        });
        
        // Email intents
        self.add_intent(IntentDefinition {
            name: "send_email".to_string(),
            category: IntentCategory::Communication,
            required_slots: vec!["recipient".to_string()],
            optional_slots: vec!["subject".to_string(), "body".to_string()],
            confirmation_required: true,
            feasible_on_glasses: true,
            example_phrases: vec![
                "send email".to_string(),
                "email to".to_string(),
                "compose email".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "check_email".to_string(),
            category: IntentCategory::Communication,
            required_slots: vec![],
            optional_slots: vec!["filter".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "check email".to_string(),
                "any new emails".to_string(),
                "show inbox".to_string(),
            ],
        });
        
        // Weather/Info intents
        self.add_intent(IntentDefinition {
            name: "weather".to_string(),
            category: IntentCategory::Information,
            required_slots: vec![],
            optional_slots: vec!["location".to_string(), "time".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "weather".to_string(),
                "is it going to rain".to_string(),
                "temperature".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "time".to_string(),
            category: IntentCategory::Information,
            required_slots: vec![],
            optional_slots: vec!["timezone".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "what time".to_string(),
                "time".to_string(),
                "what's the time".to_string(),
            ],
        });
        
        // Calendar intents
        self.add_intent(IntentDefinition {
            name: "show_calendar".to_string(),
            category: IntentCategory::Productivity,
            required_slots: vec![],
            optional_slots: vec!["date".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "show calendar".to_string(),
                "my schedule".to_string(),
                "what's on my calendar".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "create_event".to_string(),
            category: IntentCategory::Productivity,
            required_slots: vec!["title".to_string()],
            optional_slots: vec!["time".to_string(), "duration".to_string(), "location".to_string()],
            confirmation_required: true,
            feasible_on_glasses: true,
            example_phrases: vec![
                "create event".to_string(),
                "schedule meeting".to_string(),
                "add to calendar".to_string(),
            ],
        });
        
        // Contact intents
        self.add_intent(IntentDefinition {
            name: "show_contact".to_string(),
            category: IntentCategory::Communication,
            required_slots: vec!["contact".to_string()],
            optional_slots: vec![],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "show contact".to_string(),
                "contact info for".to_string(),
                "details for".to_string(),
            ],
        });
        
        // Map/Location intents
        self.add_intent(IntentDefinition {
            name: "show_map".to_string(),
            category: IntentCategory::Navigation,
            required_slots: vec![],
            optional_slots: vec!["location".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "show map".to_string(),
                "where am i".to_string(),
                "current location".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "find_nearby".to_string(),
            category: IntentCategory::Navigation,
            required_slots: vec!["place_type".to_string()],
            optional_slots: vec![],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "find nearby".to_string(),
                "where is the nearest".to_string(),
                "restaurants near me".to_string(),
                "coffee shops".to_string(),
            ],
        });
        
        // System control intents
        self.add_intent(IntentDefinition {
            name: "toggle_dnd".to_string(),
            category: IntentCategory::Settings,
            required_slots: vec![],
            optional_slots: vec!["state".to_string(), "duration".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "do not disturb".to_string(),
                "silence notifications".to_string(),
                "quiet mode".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "battery_status".to_string(),
            category: IntentCategory::System,
            required_slots: vec![],
            optional_slots: vec![],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "battery".to_string(),
                "how much battery".to_string(),
                "power level".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "toggle_wifi".to_string(),
            category: IntentCategory::Settings,
            required_slots: vec![],
            optional_slots: vec!["state".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "wifi".to_string(),
                "turn on wifi".to_string(),
                "connect to wifi".to_string(),
            ],
        });
        
        self.add_intent(IntentDefinition {
            name: "toggle_bluetooth".to_string(),
            category: IntentCategory::Settings,
            required_slots: vec![],
            optional_slots: vec!["state".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "bluetooth".to_string(),
                "turn on bluetooth".to_string(),
                "pair device".to_string(),
            ],
        });
        
        // Share intent
        self.add_intent(IntentDefinition {
            name: "share".to_string(),
            category: IntentCategory::Communication,
            required_slots: vec![],
            optional_slots: vec!["item".to_string(), "contact".to_string(), "app".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "share".to_string(),
                "share this".to_string(),
                "send to".to_string(),
            ],
        });
        
        // Read aloud intent
        self.add_intent(IntentDefinition {
            name: "read_aloud".to_string(),
            category: IntentCategory::Information,
            required_slots: vec![],
            optional_slots: vec!["content".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "read this".to_string(),
                "read aloud".to_string(),
                "what does this say".to_string(),
            ],
        });
        
        // Help intent
        self.add_intent(IntentDefinition {
            name: "help".to_string(),
            category: IntentCategory::System,
            required_slots: vec![],
            optional_slots: vec!["topic".to_string()],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "help".to_string(),
                "what can you do".to_string(),
                "how do i".to_string(),
            ],
        });
        
        // Stop/Cancel intent
        self.add_intent(IntentDefinition {
            name: "cancel".to_string(),
            category: IntentCategory::System,
            required_slots: vec![],
            optional_slots: vec![],
            confirmation_required: false,
            feasible_on_glasses: true,
            example_phrases: vec![
                "cancel".to_string(),
                "stop".to_string(),
                "nevermind".to_string(),
                "forget it".to_string(),
            ],
        });
        
        // Infeasible intents (for explanation)
        self.add_intent(IntentDefinition {
            name: "desktop_app".to_string(),
            category: IntentCategory::Infeasible,
            required_slots: vec![],
            optional_slots: vec![],
            confirmation_required: false,
            feasible_on_glasses: false,
            example_phrases: vec![
                "open vs code".to_string(),
                "open photoshop".to_string(),
                "open excel".to_string(),
            ],
        });
    }
    
    fn add_intent(&mut self, def: IntentDefinition) {
        self.intent_defs.insert(def.name.clone(), def);
    }
    
    fn initialize_disambiguation(&mut self) {
        // Disambiguate "play" based on context
        self.disambiguation_rules.push(DisambiguationRule {
            ambiguous_intent: "play".to_string(),
            context_key: "current_app".to_string(),
            context_values: vec![("music_app".to_string(), "play_media".to_string())],
            default_resolution: "play_media".to_string(),
        });
        
        // Disambiguate "send" based on entities
        self.disambiguation_rules.push(DisambiguationRule {
            ambiguous_intent: "send".to_string(),
            context_key: "entity_type".to_string(),
            context_values: vec![
                ("contact".to_string(), "message".to_string()),
                ("amount".to_string(), "transfer".to_string()),
            ],
            default_resolution: "message".to_string(),
        });
        
        // Disambiguate "open" based on what follows
        self.disambiguation_rules.push(DisambiguationRule {
            ambiguous_intent: "open".to_string(),
            context_key: "object_type".to_string(),
            context_values: vec![
                ("app".to_string(), "launch_app".to_string()),
                ("file".to_string(), "open_file".to_string()),
                ("settings".to_string(), "open_settings".to_string()),
            ],
            default_resolution: "launch_app".to_string(),
        });
    }
    
    /// Resolve intent from NLU result
    pub fn resolve(
        &self,
        nlu_result: &NluResult,
        dialogue_state: &DialogueState,
        context: &AiContext,
    ) -> ResolvedIntent {
        let intent_name = nlu_result.primary_intent.clone().unwrap_or_else(|| "unknown".to_string());
        
        // Check if we have a definition for this intent
        let def = self.intent_defs.get(&intent_name);
        
        // Check feasibility
        let (feasible, alternative) = if let Some(d) = def {
            if d.feasible_on_glasses {
                (true, None)
            } else {
                (false, Some(self.get_alternative(&intent_name)))
            }
        } else {
            (true, None) // Unknown intents are assumed feasible
        };
        
        // Check for disambiguation
        let resolved_intent = self.apply_disambiguation(&intent_name, context);
        
        // Get required slots
        let missing_slots = if let Some(d) = self.intent_defs.get(&resolved_intent) {
            self.get_missing_slots(&d.required_slots, dialogue_state)
        } else {
            vec![]
        };
        
        // Calculate confidence with context boost
        let mut confidence = nlu_result.confidence;
        if let Some(pref) = self.user_preferences.get(&resolved_intent) {
            confidence = (confidence + pref * 0.2).min(1.0);
        }
        
        // Check if this continues a previous intent
        let continues_previous = dialogue_state.current_intent.as_ref() == Some(&resolved_intent);
        
        ResolvedIntent {
            name: resolved_intent.clone(),
            category: def.map(|d| d.category).unwrap_or(IntentCategory::Unknown),
            confidence,
            requires_confirmation: def.map(|d| d.confirmation_required).unwrap_or(false),
            requires_reasoning: self.requires_reasoning(&resolved_intent),
            missing_slots,
            feasible,
            alternative,
            continues_previous,
        }
    }
    
    /// Apply disambiguation rules
    fn apply_disambiguation(&self, intent: &str, context: &AiContext) -> String {
        for rule in &self.disambiguation_rules {
            if rule.ambiguous_intent == intent {
                // Check context for resolution
                for (value, resolution) in &rule.context_values {
                    if context.current_app.as_ref() == Some(value) {
                        return resolution.clone();
                    }
                }
                return rule.default_resolution.clone();
            }
        }
        intent.to_string()
    }
    
    /// Get missing required slots
    fn get_missing_slots(&self, required: &[String], state: &DialogueState) -> Vec<String> {
        required.iter()
            .filter(|slot| !state.filled_slots.contains_key(*slot))
            .cloned()
            .collect()
    }
    
    /// Get alternative for infeasible action
    fn get_alternative(&self, intent: &str) -> String {
        match intent {
            "desktop_app" => "I can show code snippets or sync notes to your desktop instead".to_string(),
            _ => "This action isn't available on glasses, but I can help with something similar".to_string(),
        }
    }
    
    /// Check if intent requires complex reasoning
    fn requires_reasoning(&self, intent: &str) -> bool {
        matches!(intent, "question" | "search" | "identify_object" | "translate")
    }
    
    /// Learn from user feedback
    pub fn learn_from_feedback(&mut self, feedback: &AiFeedback) {
        self.learning_buffer.push(IntentFeedbackRecord {
            interaction_id: feedback.interaction_id,
            accepted: feedback.accepted,
            correction: feedback.correction.clone(),
        });
        
        // Apply learning periodically
        if self.learning_buffer.len() >= 10 {
            self.apply_learning();
        }
    }
    
    /// Apply accumulated learning
    fn apply_learning(&mut self) {
        // Adjust preferences based on feedback
        for record in &self.learning_buffer {
            if let Some(correction) = &record.correction {
                // Boost preference for corrected intent
                let pref = self.user_preferences.entry(correction.clone()).or_insert(0.0);
                *pref = (*pref + 0.1).min(0.5);
            }
        }
        self.learning_buffer.clear();
    }
    
    /// Get intent definition
    pub fn get_intent_def(&self, name: &str) -> Option<&IntentDefinition> {
        self.intent_defs.get(name)
    }
}

impl Default for IntentResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Intent definition
#[derive(Debug, Clone)]
pub struct IntentDefinition {
    pub name: String,
    pub category: IntentCategory,
    pub required_slots: Vec<String>,
    pub optional_slots: Vec<String>,
    pub confirmation_required: bool,
    pub feasible_on_glasses: bool,
    pub example_phrases: Vec<String>,
}

/// Intent categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntentCategory {
    Navigation,
    Communication,
    Media,
    Camera,
    Productivity,
    Finance,
    Information,
    Vision,
    System,
    Settings,
    Infeasible,
    Unknown,
}

/// Resolved intent
#[derive(Debug, Clone)]
pub struct ResolvedIntent {
    pub name: String,
    pub category: IntentCategory,
    pub confidence: f32,
    pub requires_confirmation: bool,
    pub requires_reasoning: bool,
    pub missing_slots: Vec<String>,
    pub feasible: bool,
    pub alternative: Option<String>,
    pub continues_previous: bool,
}

/// Disambiguation rule
#[derive(Debug, Clone)]
struct DisambiguationRule {
    ambiguous_intent: String,
    context_key: String,
    context_values: Vec<(String, String)>,
    default_resolution: String,
}

/// Intent feedback record for learning
#[derive(Debug, Clone)]
struct IntentFeedbackRecord {
    interaction_id: u64,
    accepted: bool,
    correction: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_intent_resolver_creation() {
        let resolver = IntentResolver::new();
        assert!(resolver.intent_defs.contains_key("navigation"));
        assert!(resolver.intent_defs.contains_key("call"));
    }
    
    #[test]
    fn test_intent_definition() {
        let resolver = IntentResolver::new();
        let nav = resolver.get_intent_def("navigation").unwrap();
        
        assert_eq!(nav.category, IntentCategory::Navigation);
        assert!(nav.required_slots.contains(&"destination".to_string()));
    }
    
    #[test]
    fn test_transfer_requires_confirmation() {
        let resolver = IntentResolver::new();
        let transfer = resolver.get_intent_def("transfer").unwrap();
        
        assert!(transfer.confirmation_required);
    }
    
    #[test]
    fn test_requires_reasoning() {
        let resolver = IntentResolver::new();
        
        assert!(resolver.requires_reasoning("question"));
        assert!(resolver.requires_reasoning("search"));
        assert!(!resolver.requires_reasoning("call"));
    }
}
