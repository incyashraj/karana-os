//! Response Generation System
//!
//! Generates intelligent, context-aware responses for smart glasses.
//! Optimized for voice output and minimal visual display.

use super::*;
use std::collections::HashMap;

/// Response generator
pub struct ResponseGenerator {
    /// Verbosity level
    verbosity: Verbosity,
    /// Response templates
    templates: HashMap<String, Vec<ResponseTemplate>>,
    /// Personalization settings
    personalization: PersonalizationSettings,
    /// Response history for variety
    recent_responses: Vec<String>,
    /// Learning buffer
    learning_buffer: Vec<ResponseFeedback>,
}

impl ResponseGenerator {
    pub fn new(verbosity: Verbosity) -> Self {
        let mut generator = Self {
            verbosity,
            templates: HashMap::new(),
            personalization: PersonalizationSettings::default(),
            recent_responses: Vec::new(),
            learning_buffer: Vec::new(),
        };
        generator.initialize_templates();
        generator
    }
    
    fn initialize_templates(&mut self) {
        // Navigation responses
        self.templates.insert("navigation".to_string(), vec![
            ResponseTemplate {
                pattern: "Starting navigation to {destination}".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec!["destination".to_string()],
            },
            ResponseTemplate {
                pattern: "Navigating to {destination}. The route will take approximately {duration} via {route}".to_string(),
                verbosity: Verbosity::Normal,
                requires_entities: vec!["destination".to_string()],
            },
        ]);
        
        // Call responses
        self.templates.insert("call".to_string(), vec![
            ResponseTemplate {
                pattern: "Calling {contact}".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec!["contact".to_string()],
            },
            ResponseTemplate {
                pattern: "Starting call to {contact}. {call_type} call initiated.".to_string(),
                verbosity: Verbosity::Normal,
                requires_entities: vec!["contact".to_string()],
            },
        ]);
        
        // Message responses
        self.templates.insert("message".to_string(), vec![
            ResponseTemplate {
                pattern: "Message sent to {contact}".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec!["contact".to_string()],
            },
            ResponseTemplate {
                pattern: "I've sent your message to {contact}".to_string(),
                verbosity: Verbosity::Concise,
                requires_entities: vec!["contact".to_string()],
            },
        ]);
        
        // Timer responses
        self.templates.insert("set_timer".to_string(), vec![
            ResponseTemplate {
                pattern: "Timer set for {duration}".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec!["duration".to_string()],
            },
            ResponseTemplate {
                pattern: "I've set a timer for {duration}. I'll notify you when it's done.".to_string(),
                verbosity: Verbosity::Normal,
                requires_entities: vec!["duration".to_string()],
            },
        ]);
        
        // Reminder responses
        self.templates.insert("set_reminder".to_string(), vec![
            ResponseTemplate {
                pattern: "Reminder set".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec![],
            },
            ResponseTemplate {
                pattern: "I'll remind you about {content}".to_string(),
                verbosity: Verbosity::Concise,
                requires_entities: vec!["content".to_string()],
            },
            ResponseTemplate {
                pattern: "I've set a reminder for {content} at {time}".to_string(),
                verbosity: Verbosity::Normal,
                requires_entities: vec!["content".to_string(), "time".to_string()],
            },
        ]);
        
        // Play media responses
        self.templates.insert("play_media".to_string(), vec![
            ResponseTemplate {
                pattern: "Playing".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec![],
            },
            ResponseTemplate {
                pattern: "Now playing {media_name}".to_string(),
                verbosity: Verbosity::Concise,
                requires_entities: vec!["media_name".to_string()],
            },
        ]);
        
        // Volume responses
        self.templates.insert("volume".to_string(), vec![
            ResponseTemplate {
                pattern: "Volume {direction}".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec!["direction".to_string()],
            },
        ]);
        
        // Take photo responses
        self.templates.insert("take_photo".to_string(), vec![
            ResponseTemplate {
                pattern: "Photo captured".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec![],
            },
            ResponseTemplate {
                pattern: "I've taken a photo and saved it to your gallery".to_string(),
                verbosity: Verbosity::Normal,
                requires_entities: vec![],
            },
        ]);
        
        // Balance responses
        self.templates.insert("check_balance".to_string(), vec![
            ResponseTemplate {
                pattern: "Balance: {amount} {currency}".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec!["amount".to_string()],
            },
            ResponseTemplate {
                pattern: "Your current balance is {amount} {currency}".to_string(),
                verbosity: Verbosity::Concise,
                requires_entities: vec!["amount".to_string()],
            },
        ]);
        
        // Transfer responses
        self.templates.insert("transfer".to_string(), vec![
            ResponseTemplate {
                pattern: "Sent {amount} to {recipient}".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec!["amount".to_string(), "recipient".to_string()],
            },
            ResponseTemplate {
                pattern: "I've transferred {amount} {currency} to {recipient}. Transaction confirmed.".to_string(),
                verbosity: Verbosity::Normal,
                requires_entities: vec!["amount".to_string(), "recipient".to_string()],
            },
        ]);
        
        // Search responses
        self.templates.insert("search".to_string(), vec![
            ResponseTemplate {
                pattern: "Here's what I found".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec![],
            },
            ResponseTemplate {
                pattern: "I found some results for '{query}'".to_string(),
                verbosity: Verbosity::Concise,
                requires_entities: vec!["query".to_string()],
            },
        ]);
        
        // Launch app responses
        self.templates.insert("launch_app".to_string(), vec![
            ResponseTemplate {
                pattern: "Opening {app_name}".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec!["app_name".to_string()],
            },
        ]);
        
        // Translate responses
        self.templates.insert("translate".to_string(), vec![
            ResponseTemplate {
                pattern: "Translation: {translation}".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec!["translation".to_string()],
            },
        ]);
        
        // Identify responses
        self.templates.insert("identify_object".to_string(), vec![
            ResponseTemplate {
                pattern: "That's a {object}".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec!["object".to_string()],
            },
            ResponseTemplate {
                pattern: "I see a {object}. {description}".to_string(),
                verbosity: Verbosity::Normal,
                requires_entities: vec!["object".to_string()],
            },
        ]);
        
        // Error templates
        self.templates.insert("error".to_string(), vec![
            ResponseTemplate {
                pattern: "Sorry, I couldn't do that".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec![],
            },
            ResponseTemplate {
                pattern: "I wasn't able to complete that request. {reason}".to_string(),
                verbosity: Verbosity::Normal,
                requires_entities: vec![],
            },
        ]);
        
        // Clarification templates
        self.templates.insert("clarification".to_string(), vec![
            ResponseTemplate {
                pattern: "{question}".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec!["question".to_string()],
            },
        ]);
        
        // Confirmation templates
        self.templates.insert("confirmation".to_string(), vec![
            ResponseTemplate {
                pattern: "Confirm: {action}?".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec!["action".to_string()],
            },
            ResponseTemplate {
                pattern: "Should I {action}? Say 'yes' to confirm.".to_string(),
                verbosity: Verbosity::Normal,
                requires_entities: vec!["action".to_string()],
            },
        ]);
        
        // Greeting templates
        self.templates.insert("greeting".to_string(), vec![
            ResponseTemplate {
                pattern: "Hi!".to_string(),
                verbosity: Verbosity::Minimal,
                requires_entities: vec![],
            },
            ResponseTemplate {
                pattern: "Hello! How can I help?".to_string(),
                verbosity: Verbosity::Concise,
                requires_entities: vec![],
            },
            ResponseTemplate {
                pattern: "Good {time_of_day}! What would you like to do?".to_string(),
                verbosity: Verbosity::Normal,
                requires_entities: vec![],
            },
        ]);
    }
    
    /// Generate response for resolved intent
    pub fn generate(
        &mut self,
        intent: &ResolvedIntent,
        entities: &[ExtractedEntity],
        reasoning: Option<&ReasoningResult>,
        context: &AiContext,
    ) -> AiResponse {
        // Build entity map for template substitution
        let entity_map = self.build_entity_map(entities, context);
        
        // Handle special cases first
        if !intent.feasible {
            return self.generate_infeasible_response(intent, &entity_map);
        }
        
        if !intent.missing_slots.is_empty() {
            return self.generate_clarification_response(intent, &entity_map);
        }
        
        if intent.requires_confirmation {
            return self.generate_confirmation_response(intent, &entity_map);
        }
        
        // Generate main response
        let text = self.select_and_fill_template(&intent.name, &entity_map);
        
        // Add reasoning results if available
        let text = if let Some(reason) = reasoning {
            if reason.has_answer() {
                format!("{} {}", text, reason.get_summary())
            } else {
                text
            }
        } else {
            text
        };
        
        // Generate actions
        let actions = self.generate_actions(intent, entities);
        
        // Generate suggestions
        let suggestions = self.generate_suggestions(intent, context);
        
        // Track response
        self.track_response(&text);
        
        AiResponse {
            text,
            confidence: intent.confidence,
            intent: Some(intent.clone()),
            entities: entities.to_vec(),
            actions,
            suggestions,
            response_type: ResponseType::ActionResult,
            needs_clarification: false,
            clarification_question: None,
        }
    }
    
    /// Build entity map for template substitution
    fn build_entity_map(&self, entities: &[ExtractedEntity], context: &AiContext) -> HashMap<String, String> {
        let mut map = HashMap::new();
        
        // Add entities
        for entity in entities {
            let key = entity.slot_name.clone()
                .unwrap_or_else(|| entity.entity_type.to_string());
            map.insert(key, entity.normalized_value.clone());
        }
        
        // Add context values
        if let Some(location) = &context.location {
            map.insert("current_location".to_string(), location.clone());
        }
        
        map.insert("time_of_day".to_string(), format!("{:?}", context.time_of_day).to_lowercase());
        
        // Add defaults for optional template values
        map.entry("currency".to_string()).or_insert_with(|| "SOL".to_string());
        map.entry("route".to_string()).or_insert_with(|| "the fastest route".to_string());
        map.entry("call_type".to_string()).or_insert_with(|| "Voice".to_string());
        map.entry("duration".to_string()).or_insert_with(|| "unknown duration".to_string());
        
        map
    }
    
    /// Select appropriate template and fill placeholders
    fn select_and_fill_template(&self, intent: &str, entities: &HashMap<String, String>) -> String {
        if let Some(templates) = self.templates.get(intent) {
            // Find best matching template for verbosity
            let template = templates.iter()
                .filter(|t| t.verbosity == self.verbosity || t.verbosity == Verbosity::Minimal)
                .filter(|t| t.has_required_entities(entities))
                .last()
                .or_else(|| templates.first());
            
            if let Some(t) = template {
                return self.fill_template(&t.pattern, entities);
            }
        }
        
        // Fallback response
        format!("Done")
    }
    
    /// Fill template placeholders with entity values
    fn fill_template(&self, template: &str, entities: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        
        for (key, value) in entities {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        
        // Remove unfilled placeholders using simple loop
        while let Some(start) = result.find('{') {
            if let Some(end) = result[start..].find('}') {
                result = format!("{}{}", &result[..start], &result[start + end + 1..]);
            } else {
                break;
            }
        }
        
        // Clean up extra whitespace
        result.split_whitespace().collect::<Vec<_>>().join(" ")
    }
    
    /// Generate response for infeasible intent
    fn generate_infeasible_response(&self, intent: &ResolvedIntent, _entities: &HashMap<String, String>) -> AiResponse {
        let text = if let Some(alt) = &intent.alternative {
            format!("That's not available on glasses. {}", alt)
        } else {
            "That action isn't available on smart glasses".to_string()
        };
        
        AiResponse {
            text,
            confidence: 1.0,
            intent: Some(intent.clone()),
            entities: vec![],
            actions: vec![],
            suggestions: vec![],
            response_type: ResponseType::Error,
            needs_clarification: false,
            clarification_question: None,
        }
    }
    
    /// Generate clarification response for missing slots
    fn generate_clarification_response(&self, intent: &ResolvedIntent, _entities: &HashMap<String, String>) -> AiResponse {
        let question = if let Some(slot) = intent.missing_slots.first() {
            self.get_clarification_question(slot)
        } else {
            "Could you provide more details?".to_string()
        };
        
        AiResponse {
            text: question.clone(),
            confidence: intent.confidence,
            intent: Some(intent.clone()),
            entities: vec![],
            actions: vec![],
            suggestions: vec![],
            response_type: ResponseType::Clarification,
            needs_clarification: true,
            clarification_question: Some(question),
        }
    }
    
    /// Generate confirmation response
    fn generate_confirmation_response(&self, intent: &ResolvedIntent, entities: &HashMap<String, String>) -> AiResponse {
        let action_description = self.describe_action(intent, entities);
        let text = format!("Should I {}? Say 'yes' to confirm.", action_description);
        
        AiResponse {
            text: text.clone(),
            confidence: intent.confidence,
            intent: Some(intent.clone()),
            entities: vec![],
            actions: vec![],
            suggestions: vec![],
            response_type: ResponseType::Confirmation,
            needs_clarification: true,
            clarification_question: Some(text),
        }
    }
    
    /// Get clarification question for slot
    fn get_clarification_question(&self, slot: &str) -> String {
        match slot {
            "destination" => "Where would you like to go?".to_string(),
            "contact" => "Who should I contact?".to_string(),
            "duration" => "How long?".to_string(),
            "content" | "message_content" => "What would you like to say?".to_string(),
            "amount" => "How much?".to_string(),
            "recipient" => "Who should I send to?".to_string(),
            "time" => "When?".to_string(),
            "query" => "What would you like to search for?".to_string(),
            "app_name" => "Which app?".to_string(),
            _ => format!("What {}?", slot),
        }
    }
    
    /// Describe action for confirmation
    fn describe_action(&self, intent: &ResolvedIntent, entities: &HashMap<String, String>) -> String {
        let default_amount = "the amount".to_string();
        let default_them = "them".to_string();
        
        match intent.name.as_str() {
            "transfer" => {
                let amount = entities.get("amount").unwrap_or(&default_amount);
                let recipient = entities.get("recipient").unwrap_or(&default_them);
                format!("send {} to {}", amount, recipient)
            }
            "message" => {
                let contact = entities.get("contact").unwrap_or(&default_them);
                format!("send a message to {}", contact)
            }
            "call" => {
                let contact = entities.get("contact").unwrap_or(&default_them);
                format!("call {}", contact)
            }
            _ => format!("proceed with {}", intent.name),
        }
    }
    
    /// Generate actions to execute
    fn generate_actions(&self, intent: &ResolvedIntent, entities: &[ExtractedEntity]) -> Vec<AiAction> {
        let mut actions = Vec::new();
        
        let entity_map: HashMap<String, String> = entities.iter()
            .map(|e| (e.slot_name.clone().unwrap_or(e.entity_type.to_string()), e.normalized_value.clone()))
            .collect();
        
        let action = AiAction {
            action_type: intent.name.clone(),
            parameters: entity_map,
            priority: if intent.requires_confirmation {
                ActionPriority::Normal
            } else {
                ActionPriority::High
            },
            requires_confirmation: intent.requires_confirmation,
        };
        
        actions.push(action);
        actions
    }
    
    /// Generate follow-up suggestions
    fn generate_suggestions(&self, intent: &ResolvedIntent, context: &AiContext) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        match intent.name.as_str() {
            "navigation" => {
                suggestions.push(Suggestion {
                    text: "Start with traffic view".to_string(),
                    action: "show_traffic".to_string(),
                    confidence: 0.7,
                    reason: "Traffic info often helpful".to_string(),
                });
            }
            "play_media" => {
                suggestions.push(Suggestion {
                    text: "Play similar".to_string(),
                    action: "play_similar".to_string(),
                    confidence: 0.6,
                    reason: "Continue listening".to_string(),
                });
            }
            "take_photo" => {
                suggestions.push(Suggestion {
                    text: "Take another".to_string(),
                    action: "take_photo".to_string(),
                    confidence: 0.5,
                    reason: "Often want multiple shots".to_string(),
                });
            }
            "check_balance" => {
                suggestions.push(Suggestion {
                    text: "View transactions".to_string(),
                    action: "view_transactions".to_string(),
                    confidence: 0.6,
                    reason: "Related action".to_string(),
                });
            }
            _ => {}
        }
        
        suggestions
    }
    
    /// Track response for variety
    fn track_response(&mut self, response: &str) {
        self.recent_responses.push(response.to_string());
        if self.recent_responses.len() > 10 {
            self.recent_responses.remove(0);
        }
    }
    
    /// Learn from user feedback
    pub fn learn_from_feedback(&mut self, feedback: &AiFeedback) {
        self.learning_buffer.push(ResponseFeedback {
            interaction_id: feedback.interaction_id,
            helpful: feedback.accepted,
            rating: feedback.rating,
        });
        
        // Adjust verbosity based on feedback patterns
        if self.learning_buffer.len() >= 20 {
            self.apply_learning();
        }
    }
    
    fn apply_learning(&mut self) {
        // Analyze feedback to adjust verbosity
        let ratings: Vec<u8> = self.learning_buffer.iter()
            .filter_map(|f| f.rating)
            .collect();
        
        if !ratings.is_empty() {
            let avg_rating = ratings.iter().map(|&r| r as f32).sum::<f32>() / ratings.len() as f32;
            
            // If ratings are low, try different verbosity
            if avg_rating < 5.0 {
                self.verbosity = match self.verbosity {
                    Verbosity::Minimal => Verbosity::Concise,
                    Verbosity::Concise => Verbosity::Normal,
                    Verbosity::Normal => Verbosity::Detailed,
                    Verbosity::Detailed => Verbosity::Concise,
                };
            }
        }
        
        self.learning_buffer.clear();
    }
    
    /// Set verbosity level
    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
    }
}

impl Default for ResponseGenerator {
    fn default() -> Self {
        Self::new(Verbosity::Concise)
    }
}

/// Response template
#[derive(Debug, Clone)]
struct ResponseTemplate {
    pattern: String,
    verbosity: Verbosity,
    requires_entities: Vec<String>,
}

impl ResponseTemplate {
    fn has_required_entities(&self, entities: &HashMap<String, String>) -> bool {
        self.requires_entities.iter().all(|e| entities.contains_key(e))
    }
}

/// Personalization settings
#[derive(Debug, Clone, Default)]
struct PersonalizationSettings {
    preferred_verbosity: Option<Verbosity>,
    use_formal_language: bool,
    include_emojis: bool,
}

/// Response feedback record
#[derive(Debug, Clone)]
struct ResponseFeedback {
    interaction_id: u64,
    helpful: bool,
    rating: Option<u8>,
}

/// Reasoning result from reasoning engine
#[derive(Debug, Clone, Default)]
pub struct ReasoningResult {
    pub answer: Option<String>,
    pub confidence: f32,
    pub sources: Vec<String>,
    pub reasoning_steps: Vec<String>,
}

impl ReasoningResult {
    pub fn has_answer(&self) -> bool {
        self.answer.is_some() && !self.answer.as_ref().unwrap().is_empty()
    }
    
    pub fn get_summary(&self) -> String {
        self.answer.clone().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_response_generator_creation() {
        let generator = ResponseGenerator::new(Verbosity::Concise);
        assert!(!generator.templates.is_empty());
    }
    
    #[test]
    fn test_template_filling() {
        let generator = ResponseGenerator::new(Verbosity::Normal);
        let mut entities = HashMap::new();
        entities.insert("destination".to_string(), "coffee shop".to_string());
        
        let result = generator.fill_template("Going to {destination}", &entities);
        assert_eq!(result, "Going to coffee shop");
    }
    
    #[test]
    fn test_generate_response() {
        let mut generator = ResponseGenerator::new(Verbosity::Minimal);
        let context = AiContext::default();
        
        let intent = ResolvedIntent {
            name: "take_photo".to_string(),
            category: IntentCategory::Camera,
            confidence: 0.9,
            requires_confirmation: false,
            requires_reasoning: false,
            missing_slots: vec![],
            feasible: true,
            alternative: None,
            continues_previous: false,
        };
        
        let response = generator.generate(&intent, &[], None, &context);
        
        assert!(!response.text.is_empty());
        assert_eq!(response.response_type, ResponseType::ActionResult);
    }
    
    #[test]
    fn test_clarification_response() {
        let mut generator = ResponseGenerator::new(Verbosity::Concise);
        let context = AiContext::default();
        
        let intent = ResolvedIntent {
            name: "navigation".to_string(),
            category: IntentCategory::Navigation,
            confidence: 0.8,
            requires_confirmation: false,
            requires_reasoning: false,
            missing_slots: vec!["destination".to_string()],
            feasible: true,
            alternative: None,
            continues_previous: false,
        };
        
        let response = generator.generate(&intent, &[], None, &context);
        
        assert!(response.needs_clarification);
        assert_eq!(response.response_type, ResponseType::Clarification);
    }
    
    #[test]
    fn test_verbosity_affects_response() {
        let mut generator_minimal = ResponseGenerator::new(Verbosity::Minimal);
        let mut generator_normal = ResponseGenerator::new(Verbosity::Normal);
        let context = AiContext::default();
        
        let intent = ResolvedIntent {
            name: "take_photo".to_string(),
            category: IntentCategory::Camera,
            confidence: 0.9,
            requires_confirmation: false,
            requires_reasoning: false,
            missing_slots: vec![],
            feasible: true,
            alternative: None,
            continues_previous: false,
        };
        
        let minimal_response = generator_minimal.generate(&intent, &[], None, &context);
        let normal_response = generator_normal.generate(&intent, &[], None, &context);
        
        // Normal response should be longer or equal
        assert!(normal_response.text.len() >= minimal_response.text.len());
    }
}
