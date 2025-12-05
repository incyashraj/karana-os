// Kāraṇa OS - AI Slot Filler
// Manages slot filling for multi-turn conversations

use std::collections::HashMap;
use crate::ai_layer::entities::{EntityExtractor, ExtractedEntity, EntityType};
use crate::ai_layer::intent::ResolvedIntent;
use crate::ai_layer::AiContext;
use crate::ai_layer::context_manager::ContextManager;

/// Slot filler that manages required and optional slots for intents
pub struct SlotFiller {
    /// Slot definitions per intent
    slot_definitions: HashMap<String, Vec<SlotDefinition>>,
    /// Current slot values being filled
    current_slots: HashMap<String, SlotValue>,
    /// Entity extractor for extraction
    entity_extractor: EntityExtractor,
    /// Validation rules
    validators: HashMap<String, Box<dyn Fn(&str) -> bool + Send + Sync>>,
}

/// Definition of a slot
#[derive(Debug, Clone)]
pub struct SlotDefinition {
    /// Slot name
    pub name: String,
    /// Whether slot is required
    pub required: bool,
    /// Expected entity type
    pub entity_type: EntityType,
    /// Prompt to ask for this slot
    pub prompt: String,
    /// Alternative prompts
    pub alt_prompts: Vec<String>,
    /// Default value if not provided
    pub default: Option<String>,
    /// Validation regex pattern
    pub validation_pattern: Option<String>,
    /// Examples for the user
    pub examples: Vec<String>,
}

/// Value of a filled slot
#[derive(Debug, Clone)]
pub struct SlotValue {
    /// The value
    pub value: String,
    /// How confident we are in this value
    pub confidence: f32,
    /// How the slot was filled
    pub source: SlotSource,
    /// Whether confirmed by user
    pub confirmed: bool,
}

/// How a slot was filled
#[derive(Debug, Clone, PartialEq)]
pub enum SlotSource {
    /// Extracted from user input
    Extracted,
    /// From conversation context
    Context,
    /// From user preference
    Preference,
    /// User explicitly provided
    UserProvided,
    /// Default value
    Default,
}

/// Result of slot filling attempt
#[derive(Debug, Clone)]
pub struct SlotFillingResult {
    /// Whether all required slots are filled
    pub complete: bool,
    /// Current slot values
    pub slots: HashMap<String, SlotValue>,
    /// Missing required slots
    pub missing_required: Vec<String>,
    /// Missing optional slots
    pub missing_optional: Vec<String>,
    /// Next slot to ask for
    pub next_prompt: Option<SlotPrompt>,
}

/// Prompt for a slot
#[derive(Debug, Clone)]
pub struct SlotPrompt {
    /// Slot name
    pub slot: String,
    /// Question to ask
    pub question: String,
    /// Expected type description
    pub expected_type: String,
    /// Examples
    pub examples: Vec<String>,
}

impl SlotFiller {
    /// Create a new slot filler
    pub fn new() -> Self {
        let mut filler = Self {
            slot_definitions: HashMap::new(),
            current_slots: HashMap::new(),
            entity_extractor: EntityExtractor::new(),
            validators: HashMap::new(),
        };
        filler.initialize_definitions();
        filler
    }
    
    /// Initialize slot definitions for all intents
    fn initialize_definitions(&mut self) {
        // Call intent
        self.slot_definitions.insert("call".to_string(), vec![
            SlotDefinition {
                name: "contact".to_string(),
                required: true,
                entity_type: EntityType::Contact,
                prompt: "Who would you like to call?".to_string(),
                alt_prompts: vec![
                    "Who should I call?".to_string(),
                    "What's the name or number?".to_string(),
                ],
                default: None,
                validation_pattern: None,
                examples: vec!["Mom".to_string(), "John".to_string(), "555-1234".to_string()],
            },
        ]);
        
        // Message intent
        self.slot_definitions.insert("message".to_string(), vec![
            SlotDefinition {
                name: "contact".to_string(),
                required: true,
                entity_type: EntityType::Contact,
                prompt: "Who would you like to message?".to_string(),
                alt_prompts: vec!["Who's the message for?".to_string()],
                default: None,
                validation_pattern: None,
                examples: vec!["Mom".to_string(), "John".to_string()],
            },
            SlotDefinition {
                name: "content".to_string(),
                required: true,
                entity_type: EntityType::Other,
                prompt: "What would you like the message to say?".to_string(),
                alt_prompts: vec!["What's the message?".to_string()],
                default: None,
                validation_pattern: None,
                examples: vec!["I'm on my way".to_string(), "Be there in 10".to_string()],
            },
        ]);
        
        // Navigate intent
        self.slot_definitions.insert("navigate".to_string(), vec![
            SlotDefinition {
                name: "destination".to_string(),
                required: true,
                entity_type: EntityType::Location,
                prompt: "Where would you like to go?".to_string(),
                alt_prompts: vec![
                    "What's the destination?".to_string(),
                    "Where to?".to_string(),
                ],
                default: None,
                validation_pattern: None,
                examples: vec!["Home".to_string(), "Work".to_string(), "123 Main St".to_string()],
            },
            SlotDefinition {
                name: "mode".to_string(),
                required: false,
                entity_type: EntityType::TravelMode,
                prompt: "How would you like to get there?".to_string(),
                alt_prompts: vec!["Walking, driving, or transit?".to_string()],
                default: Some("walking".to_string()),
                validation_pattern: None,
                examples: vec!["walking".to_string(), "driving".to_string()],
            },
        ]);
        
        // Reminder intent
        self.slot_definitions.insert("reminder".to_string(), vec![
            SlotDefinition {
                name: "content".to_string(),
                required: true,
                entity_type: EntityType::Other,
                prompt: "What should I remind you about?".to_string(),
                alt_prompts: vec!["What's the reminder?".to_string()],
                default: None,
                validation_pattern: None,
                examples: vec!["Meeting".to_string(), "Take medicine".to_string()],
            },
            SlotDefinition {
                name: "time".to_string(),
                required: true,
                entity_type: EntityType::Time,
                prompt: "When should I remind you?".to_string(),
                alt_prompts: vec!["What time?".to_string()],
                default: None,
                validation_pattern: None,
                examples: vec!["3pm".to_string(), "in 10 minutes".to_string()],
            },
        ]);
        
        // Timer intent
        self.slot_definitions.insert("timer".to_string(), vec![
            SlotDefinition {
                name: "duration".to_string(),
                required: true,
                entity_type: EntityType::Duration,
                prompt: "How long should I set the timer for?".to_string(),
                alt_prompts: vec!["For how long?".to_string()],
                default: None,
                validation_pattern: None,
                examples: vec!["5 minutes".to_string(), "1 hour".to_string()],
            },
            SlotDefinition {
                name: "label".to_string(),
                required: false,
                entity_type: EntityType::Other,
                prompt: "What's this timer for?".to_string(),
                alt_prompts: vec!["Any label for the timer?".to_string()],
                default: None,
                validation_pattern: None,
                examples: vec!["Cooking".to_string(), "Break".to_string()],
            },
        ]);
        
        // Play music intent
        self.slot_definitions.insert("play_music".to_string(), vec![
            SlotDefinition {
                name: "query".to_string(),
                required: false,
                entity_type: EntityType::Other,
                prompt: "What would you like to listen to?".to_string(),
                alt_prompts: vec!["Any specific song or artist?".to_string()],
                default: None,
                validation_pattern: None,
                examples: vec!["Jazz".to_string(), "Beatles".to_string(), "My playlist".to_string()],
            },
        ]);
        
        // Search intent
        self.slot_definitions.insert("search".to_string(), vec![
            SlotDefinition {
                name: "query".to_string(),
                required: true,
                entity_type: EntityType::Other,
                prompt: "What would you like to search for?".to_string(),
                alt_prompts: vec!["What are you looking for?".to_string()],
                default: None,
                validation_pattern: None,
                examples: vec!["Weather".to_string(), "News".to_string(), "Restaurants nearby".to_string()],
            },
        ]);
        
        // Email intent
        self.slot_definitions.insert("email".to_string(), vec![
            SlotDefinition {
                name: "to".to_string(),
                required: true,
                entity_type: EntityType::Email,
                prompt: "Who should I send the email to?".to_string(),
                alt_prompts: vec!["What's the recipient's email?".to_string()],
                default: None,
                validation_pattern: Some(r"^[^\s@]+@[^\s@]+\.[^\s@]+$".to_string()),
                examples: vec!["john@example.com".to_string()],
            },
            SlotDefinition {
                name: "subject".to_string(),
                required: false,
                entity_type: EntityType::Other,
                prompt: "What's the subject?".to_string(),
                alt_prompts: vec!["Email subject?".to_string()],
                default: Some("No subject".to_string()),
                validation_pattern: None,
                examples: vec!["Meeting tomorrow".to_string()],
            },
            SlotDefinition {
                name: "body".to_string(),
                required: true,
                entity_type: EntityType::Other,
                prompt: "What should the email say?".to_string(),
                alt_prompts: vec!["What's the message?".to_string()],
                default: None,
                validation_pattern: None,
                examples: vec!["Looking forward to our meeting".to_string()],
            },
        ]);
        
        // Photo intent
        self.slot_definitions.insert("photo".to_string(), vec![
            SlotDefinition {
                name: "mode".to_string(),
                required: false,
                entity_type: EntityType::Other,
                prompt: "Photo, video, or something else?".to_string(),
                alt_prompts: vec!["What type of capture?".to_string()],
                default: Some("photo".to_string()),
                validation_pattern: None,
                examples: vec!["photo".to_string(), "video".to_string(), "panorama".to_string()],
            },
        ]);
        
        // Translate intent
        self.slot_definitions.insert("translate".to_string(), vec![
            SlotDefinition {
                name: "text".to_string(),
                required: true,
                entity_type: EntityType::Other,
                prompt: "What would you like to translate?".to_string(),
                alt_prompts: vec!["What should I translate?".to_string()],
                default: None,
                validation_pattern: None,
                examples: vec!["Hello".to_string(), "Thank you".to_string()],
            },
            SlotDefinition {
                name: "target_language".to_string(),
                required: true,
                entity_type: EntityType::Language,
                prompt: "What language should I translate to?".to_string(),
                alt_prompts: vec!["To which language?".to_string()],
                default: None,
                validation_pattern: None,
                examples: vec!["Spanish".to_string(), "French".to_string(), "Japanese".to_string()],
            },
        ]);
    }
    
    /// Start filling slots for an intent
    pub fn start(&mut self, intent_name: &str) {
        self.current_slots.clear();
        
        // Initialize with defaults
        if let Some(definitions) = self.slot_definitions.get(intent_name) {
            for def in definitions {
                if let Some(default) = &def.default {
                    self.current_slots.insert(def.name.clone(), SlotValue {
                        value: default.clone(),
                        confidence: 0.5,
                        source: SlotSource::Default,
                        confirmed: false,
                    });
                }
            }
        }
    }
    
    /// Fill slots from user input and context
    pub fn fill(
        &mut self,
        intent: &ResolvedIntent,
        input: &str,
        context: &AiContext,
        context_manager: Option<&ContextManager>,
    ) -> SlotFillingResult {
        let definitions = self.slot_definitions.get(&intent.name)
            .cloned()
            .unwrap_or_default();
        
        // Extract entities from input
        let entities = self.entity_extractor.extract(input, context);
        
        // Match entities to slots
        for def in &definitions {
            if !self.current_slots.contains_key(&def.name) || 
               self.current_slots.get(&def.name).map(|s| s.source == SlotSource::Default).unwrap_or(false) {
                // Try to fill from entities
                if let Some(entity) = self.find_entity_for_slot(&def, &entities) {
                    self.current_slots.insert(def.name.clone(), SlotValue {
                        value: entity.normalized_value.clone(),
                        confidence: entity.confidence,
                        source: SlotSource::Extracted,
                        confirmed: false,
                    });
                }
                // Try to fill from context
                else if let Some(cm) = context_manager {
                    if let Some(value) = cm.get_recent_entity(&self.entity_type_to_string(&def.entity_type)) {
                        self.current_slots.insert(def.name.clone(), SlotValue {
                            value,
                            confidence: 0.6,
                            source: SlotSource::Context,
                            confirmed: false,
                        });
                    }
                }
            }
        }
        
        // Also try to fill content/text slots from remaining input
        self.fill_text_slots(&definitions, input, &entities);
        
        // Build result
        self.build_result(&intent.name)
    }
    
    /// Fill text/content slots from input
    fn fill_text_slots(&mut self, definitions: &[SlotDefinition], input: &str, entities: &[ExtractedEntity]) {
        for def in definitions {
            if matches!(def.entity_type, EntityType::Other) && 
               !self.current_slots.contains_key(&def.name) {
                // Get text excluding recognized entities
                let remaining = self.get_remaining_text(input, entities);
                if !remaining.trim().is_empty() {
                    self.current_slots.insert(def.name.clone(), SlotValue {
                        value: remaining,
                        confidence: 0.7,
                        source: SlotSource::Extracted,
                        confirmed: false,
                    });
                }
            }
        }
    }
    
    /// Get text that wasn't recognized as entities
    fn get_remaining_text(&self, input: &str, entities: &[ExtractedEntity]) -> String {
        // Remove common trigger words
        let trigger_words = ["call", "text", "message", "remind", "set", "navigate", "go", "play", "search", "find", "send", "email", "take", "translate"];
        
        let mut result = input.to_lowercase();
        
        for word in &trigger_words {
            result = result.replace(word, "");
        }
        
        // Remove entity spans
        let mut chars: Vec<char> = result.chars().collect();
        for entity in entities {
            if entity.start_pos < chars.len() && entity.end_pos <= chars.len() {
                for i in entity.start_pos..entity.end_pos {
                    chars[i] = ' ';
                }
            }
        }
        
        chars.into_iter().collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Find entity that matches slot type
    fn find_entity_for_slot<'a>(&self, def: &SlotDefinition, entities: &'a [ExtractedEntity]) -> Option<&'a ExtractedEntity> {
        entities.iter().find(|e| e.entity_type == def.entity_type)
    }
    
    /// Convert entity type to string for context lookup
    fn entity_type_to_string(&self, entity_type: &EntityType) -> String {
        match entity_type {
            EntityType::Time => "time".to_string(),
            EntityType::Duration => "duration".to_string(),
            EntityType::Number => "number".to_string(),
            EntityType::Amount => "amount".to_string(),
            EntityType::Email => "email".to_string(),
            EntityType::Phone => "phone".to_string(),
            EntityType::Url => "url".to_string(),
            EntityType::Location => "location".to_string(),
            EntityType::Contact => "contact".to_string(),
            EntityType::App => "app".to_string(),
            EntityType::Direction => "direction".to_string(),
            EntityType::TravelMode => "travel_mode".to_string(),
            EntityType::MediaType => "media_type".to_string(),
            EntityType::Language => "language".to_string(),
            EntityType::Percentage => "percentage".to_string(),
            EntityType::Other => "other".to_string(),
        }
    }
    
    /// Build slot filling result
    fn build_result(&self, intent_name: &str) -> SlotFillingResult {
        let definitions = self.slot_definitions.get(intent_name)
            .cloned()
            .unwrap_or_default();
        
        let mut missing_required = Vec::new();
        let mut missing_optional = Vec::new();
        
        for def in &definitions {
            if !self.current_slots.contains_key(&def.name) {
                if def.required {
                    missing_required.push(def.name.clone());
                } else {
                    missing_optional.push(def.name.clone());
                }
            }
        }
        
        let next_prompt = if let Some(missing) = missing_required.first() {
            definitions.iter()
                .find(|d| d.name == *missing)
                .map(|d| SlotPrompt {
                    slot: d.name.clone(),
                    question: d.prompt.clone(),
                    expected_type: self.entity_type_to_string(&d.entity_type),
                    examples: d.examples.clone(),
                })
        } else {
            None
        };
        
        SlotFillingResult {
            complete: missing_required.is_empty(),
            slots: self.current_slots.clone(),
            missing_required,
            missing_optional,
            next_prompt,
        }
    }
    
    /// Fill a specific slot with a value
    pub fn fill_slot(&mut self, slot_name: &str, value: &str, confirmed: bool) {
        self.current_slots.insert(slot_name.to_string(), SlotValue {
            value: value.to_string(),
            confidence: 1.0,
            source: SlotSource::UserProvided,
            confirmed,
        });
    }
    
    /// Get current value for a slot
    pub fn get_slot(&self, slot_name: &str) -> Option<&SlotValue> {
        self.current_slots.get(slot_name)
    }
    
    /// Get all current slot values
    pub fn get_all_slots(&self) -> &HashMap<String, SlotValue> {
        &self.current_slots
    }
    
    /// Clear all slots
    pub fn clear(&mut self) {
        self.current_slots.clear();
    }
    
    /// Validate a slot value
    pub fn validate(&self, intent_name: &str, slot_name: &str, value: &str) -> bool {
        if let Some(definitions) = self.slot_definitions.get(intent_name) {
            if let Some(def) = definitions.iter().find(|d| d.name == slot_name) {
                // Check custom validator
                if let Some(validator) = self.validators.get(slot_name) {
                    if !validator(value) {
                        return false;
                    }
                }
                
                // Check pattern (simple validation without regex)
                if let Some(_pattern) = &def.validation_pattern {
                    // Basic email validation
                    if slot_name == "email" || slot_name == "to" {
                        if !value.contains('@') || !value.contains('.') {
                            return false;
                        }
                    }
                }
                
                return true;
            }
        }
        true
    }
    
    /// Get prompt for missing slot
    pub fn get_prompt(&self, intent_name: &str, slot_name: &str) -> Option<String> {
        if let Some(definitions) = self.slot_definitions.get(intent_name) {
            if let Some(def) = definitions.iter().find(|d| d.name == slot_name) {
                return Some(def.prompt.clone());
            }
        }
        None
    }
    
    /// Get examples for a slot
    pub fn get_examples(&self, intent_name: &str, slot_name: &str) -> Vec<String> {
        if let Some(definitions) = self.slot_definitions.get(intent_name) {
            if let Some(def) = definitions.iter().find(|d| d.name == slot_name) {
                return def.examples.clone();
            }
        }
        vec![]
    }
    
    /// Suggest values for a slot based on context and history
    pub fn suggest_values(
        &self,
        slot_name: &str,
        context_manager: Option<&ContextManager>,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if let Some(cm) = context_manager {
            // Get from recent context
            if let Some(recent) = cm.get_recent_entity(slot_name) {
                suggestions.push(recent);
            }
            
            // Get frequent contacts/apps if relevant
            match slot_name {
                "contact" => {
                    suggestions.extend(cm.get_suggested_contacts(3));
                }
                "app" => {
                    suggestions.extend(cm.get_suggested_apps(3));
                }
                _ => {}
            }
        }
        
        suggestions.truncate(5);
        suggestions
    }
    
    /// Check if all required slots are filled
    pub fn is_complete(&self, intent_name: &str) -> bool {
        if let Some(definitions) = self.slot_definitions.get(intent_name) {
            for def in definitions {
                if def.required && !self.current_slots.contains_key(&def.name) {
                    return false;
                }
            }
            return true;
        }
        true // Unknown intent, assume complete
    }
    
    /// Get missing required slots
    pub fn get_missing_required(&self, intent_name: &str) -> Vec<String> {
        let mut missing = Vec::new();
        
        if let Some(definitions) = self.slot_definitions.get(intent_name) {
            for def in definitions {
                if def.required && !self.current_slots.contains_key(&def.name) {
                    missing.push(def.name.clone());
                }
            }
        }
        
        missing
    }
    
    /// Confirm a slot value
    pub fn confirm_slot(&mut self, slot_name: &str) {
        if let Some(slot) = self.current_slots.get_mut(slot_name) {
            slot.confirmed = true;
            slot.confidence = 1.0;
        }
    }
    
    /// Reject a slot value
    pub fn reject_slot(&mut self, slot_name: &str) {
        self.current_slots.remove(slot_name);
    }
}

impl Default for SlotFiller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_layer::intent::IntentCategory;
    
    #[test]
    fn test_slot_filler_creation() {
        let filler = SlotFiller::new();
        assert!(!filler.slot_definitions.is_empty());
    }
    
    #[test]
    fn test_start_intent() {
        let mut filler = SlotFiller::new();
        filler.start("navigate");
        
        // Navigate has default mode = walking
        assert!(filler.current_slots.get("mode").is_some());
    }
    
    #[test]
    fn test_fill_slots() {
        let mut filler = SlotFiller::new();
        filler.start("timer");
        
        let intent = ResolvedIntent {
            name: "timer".to_string(),
            category: IntentCategory::Productivity,
            confidence: 0.9,
            requires_confirmation: false,
            requires_reasoning: false,
            missing_slots: vec![],
            feasible: true,
            alternative: None,
            continues_previous: false,
        };
        
        let context = AiContext::default();
        let result = filler.fill(&intent, "set timer for 5 minutes", &context, None);
        
        // Duration should be filled
        assert!(filler.current_slots.contains_key("duration"));
    }
    
    #[test]
    fn test_fill_slot_manually() {
        let mut filler = SlotFiller::new();
        filler.start("call");
        
        filler.fill_slot("contact", "Mom", true);
        
        let slot = filler.get_slot("contact");
        assert!(slot.is_some());
        assert_eq!(slot.unwrap().value, "Mom");
        assert!(slot.unwrap().confirmed);
    }
    
    #[test]
    fn test_is_complete() {
        let mut filler = SlotFiller::new();
        filler.start("call");
        
        assert!(!filler.is_complete("call"));
        
        filler.fill_slot("contact", "Mom", false);
        
        assert!(filler.is_complete("call"));
    }
    
    #[test]
    fn test_missing_required() {
        let mut filler = SlotFiller::new();
        filler.start("message");
        
        let missing = filler.get_missing_required("message");
        
        assert!(missing.contains(&"contact".to_string()));
        assert!(missing.contains(&"content".to_string()));
    }
    
    #[test]
    fn test_validate() {
        let filler = SlotFiller::new();
        
        // Valid email
        assert!(filler.validate("email", "to", "test@example.com"));
        
        // Invalid email
        assert!(!filler.validate("email", "to", "notanemail"));
    }
    
    #[test]
    fn test_get_prompt() {
        let filler = SlotFiller::new();
        
        let prompt = filler.get_prompt("call", "contact");
        assert!(prompt.is_some());
        assert!(prompt.unwrap().contains("call"));
    }
    
    #[test]
    fn test_confirm_reject() {
        let mut filler = SlotFiller::new();
        filler.start("call");
        filler.fill_slot("contact", "John", false);
        
        assert!(!filler.get_slot("contact").unwrap().confirmed);
        
        filler.confirm_slot("contact");
        assert!(filler.get_slot("contact").unwrap().confirmed);
        
        filler.reject_slot("contact");
        assert!(filler.get_slot("contact").is_none());
    }
}
