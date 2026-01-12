// Kāraṇa OS - Enhanced Dialogue Manager
// Phase 4: Conversational Intelligence with Context and Memory

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::SystemTime;
use chrono::Timelike;

use super::KaranaAI;

/// Single conversation turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub user: String,
    pub assistant: String,
    pub timestamp: SystemTime,
    pub intent: Option<String>,
    pub confidence: f32,
}

/// User profile for personalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub name: Option<String>,
    pub preferences: Vec<Preference>,
    pub common_queries: Vec<String>,
    pub interaction_style: InteractionStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    pub category: String,
    pub value: String,
    pub learned_from: String, // "explicit" or "implicit"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InteractionStyle {
    Concise,    // Prefers short answers
    Detailed,   // Wants explanations
    Technical,  // Appreciates technical details
    Casual,     // Conversational tone
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            name: None,
            preferences: Vec::new(),
            common_queries: Vec::new(),
            interaction_style: InteractionStyle::Concise,
        }
    }
}

/// Enhanced Dialogue Manager with memory and personality
pub struct DialogueManager {
    ai: Arc<StdMutex<KaranaAI>>,
    conversation_history: VecDeque<ConversationTurn>,
    max_history: usize,
    user_profile: UserProfile,
    system_personality: String,
}

impl DialogueManager {
    pub fn new(ai: Arc<StdMutex<KaranaAI>>) -> Self {
        Self {
            ai,
            conversation_history: VecDeque::new(),
            max_history: 10,
            user_profile: UserProfile::default(),
            system_personality: "You are Kāraṇa, an intelligent AI assistant for smart glasses. \
                You are helpful, concise, and proactive. You remember past conversations and \
                provide contextual responses. You can see through the glasses camera, access \
                blockchain data, and control system functions.".to_string(),
        }
    }

    /// Generate contextual response with conversation memory
    pub fn generate_response(&mut self, user_input: &str) -> Result<String> {
        log::info!("[Dialogue] Processing: {}", user_input);
        
        // Build conversation context
        let context = self.build_context();
        
        // Adapt response based on user style
        let style_instruction = match self.user_profile.interaction_style {
            InteractionStyle::Concise => "Keep your response brief and to the point.",
            InteractionStyle::Detailed => "Provide a detailed explanation.",
            InteractionStyle::Technical => "Include technical details and specifications.",
            InteractionStyle::Casual => "Use a friendly, conversational tone.",
        };
        
        // Construct prompt with personality and context
        let prompt = format!(
            "{}\n\n{}\n\n{}\n\nUser: {}\n\nKāraṇa:",
            self.system_personality,
            context,
            style_instruction,
            user_input
        );
        
        // Generate response (scope the lock)
        let response = {
            let mut ai = self.ai.lock().unwrap();
            ai.predict(&prompt, 200)?
        };
        
        // Store in history (after lock is released)
        self.add_to_history(user_input, &response, None, 0.9);
        
        // Learn from interaction
        self.learn_from_interaction(user_input, &response);
        
        Ok(response)
    }

    /// Generate response with explicit intent
    pub fn generate_response_with_intent(&mut self, user_input: &str, intent: &str, confidence: f32) -> Result<String> {
        let context = self.build_context();
        
        let prompt = format!(
            "{}\n\n{}\n\nDetected Intent: {}\n\nUser: {}\n\nKāraṇa:",
            self.system_personality,
            context,
            intent,
            user_input
        );
        
        let response = {
            let mut ai = self.ai.lock().unwrap();
            ai.predict(&prompt, 200)?
        };
        
        self.add_to_history(user_input, &response, Some(intent.to_string()), confidence);
        
        Ok(response)
    }

    /// Add turn to conversation history
    fn add_to_history(&mut self, user: &str, assistant: &str, intent: Option<String>, confidence: f32) {
        self.conversation_history.push_back(ConversationTurn {
            user: user.to_string(),
            assistant: assistant.to_string(),
            timestamp: SystemTime::now(),
            intent,
            confidence,
        });
        
        // Prune old history
        while self.conversation_history.len() > self.max_history {
            self.conversation_history.pop_front();
        }
    }

    /// Build context from conversation history
    fn build_context(&self) -> String {
        if self.conversation_history.is_empty() {
            return "New conversation.".to_string();
        }
        
        let recent = self.conversation_history.iter()
            .rev()
            .take(3)
            .rev()
            .map(|turn| format!("User: {}\nKāraṇa: {}", turn.user, turn.assistant))
            .collect::<Vec<_>>()
            .join("\n");
        
        format!("Recent conversation:\n{}", recent)
    }

    /// Learn user preferences from interactions
    fn learn_from_interaction(&mut self, user_input: &str, _response: &str) {
        let lower = user_input.to_lowercase();
        
        // Learn interaction style
        if lower.contains("briefly") || lower.contains("short") || lower.contains("quick") {
            self.user_profile.interaction_style = InteractionStyle::Concise;
        } else if lower.contains("explain") || lower.contains("detail") || lower.contains("more about") {
            self.user_profile.interaction_style = InteractionStyle::Detailed;
        } else if lower.contains("technical") || lower.contains("how does it work") {
            self.user_profile.interaction_style = InteractionStyle::Technical;
        }
        
        // Track common queries
        if !self.user_profile.common_queries.contains(&user_input.to_string()) {
            self.user_profile.common_queries.push(user_input.to_string());
            if self.user_profile.common_queries.len() > 20 {
                self.user_profile.common_queries.remove(0);
            }
        }
    }

    /// Add explicit preference
    pub fn add_preference(&mut self, category: &str, value: &str) {
        self.user_profile.preferences.push(Preference {
            category: category.to_string(),
            value: value.to_string(),
            learned_from: "explicit".to_string(),
        });
    }

    /// Get user profile
    pub fn get_profile(&self) -> &UserProfile {
        &self.user_profile
    }

    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.conversation_history.clear();
    }

    /// Get conversation summary
    pub fn get_summary(&self) -> String {
        if self.conversation_history.is_empty() {
            return "No conversation history.".to_string();
        }
        
        format!(
            "Conversation: {} turns, Style: {:?}, Common topics: {}",
            self.conversation_history.len(),
            self.user_profile.interaction_style,
            self.user_profile.common_queries.len()
        )
    }
}

/// Proactive suggestion engine
pub struct ProactiveEngine {
    ai: Arc<StdMutex<KaranaAI>>,
    dialogue: Arc<StdMutex<DialogueManager>>,
    suggestion_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub type_: SuggestionType,
    pub message: String,
    pub confidence: f32,
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    Reminder,
    LocationBased,
    Habit,
    Optimization,
    Safety,
}

impl ProactiveEngine {
    pub fn new(ai: Arc<StdMutex<KaranaAI>>, dialogue: Arc<StdMutex<DialogueManager>>) -> Self {
        Self {
            ai,
            dialogue,
            suggestion_threshold: 0.7,
        }
    }

    /// Analyze context and generate suggestions
    pub fn analyze_context(&self) -> Result<Vec<Suggestion>> {
        let mut suggestions = Vec::new();
        
        // Time-based suggestions
        if self.is_morning() && !self.has_checked_calendar_today() {
            suggestions.push(Suggestion {
                type_: SuggestionType::Reminder,
                message: "Good morning! Would you like to review today's schedule?".into(),
                confidence: 0.8,
                action: Some("show_calendar".into()),
            });
        }
        
        // Pattern-based from conversation history
        let dialogue = self.dialogue.lock().unwrap();
        let common_queries = &dialogue.get_profile().common_queries;
        
        if common_queries.iter().any(|q| q.contains("weather")) && self.is_morning() {
            suggestions.push(Suggestion {
                type_: SuggestionType::Habit,
                message: "You usually check the weather in the morning. Would you like today's forecast?".into(),
                confidence: 0.75,
                action: Some("show_weather".into()),
            });
        }
        
        // Filter by confidence threshold
        Ok(suggestions.into_iter()
            .filter(|s| s.confidence >= self.suggestion_threshold)
            .collect())
    }

    fn is_morning(&self) -> bool {
        let now = chrono::Local::now();
        now.hour() >= 6 && now.hour() < 12
    }

    fn has_checked_calendar_today(&self) -> bool {
        // Would check actual logs
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialogue_context() {
        let ai = Arc::new(StdMutex::new(KaranaAI::new().unwrap()));
        let mut dialogue = DialogueManager::new(ai);
        
        dialogue.add_to_history("Hello", "Hi there!", None, 0.9);
        let context = dialogue.build_context();
        assert!(context.contains("Hello"));
    }

    #[test]
    fn test_learning_style() {
        let ai = Arc::new(StdMutex::new(KaranaAI::new().unwrap()));
        let mut dialogue = DialogueManager::new(ai);
        
        dialogue.learn_from_interaction("Explain this in detail", "...");
        assert_eq!(dialogue.user_profile.interaction_style, InteractionStyle::Detailed);
    }
}
