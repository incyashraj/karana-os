// Kāraṇa OS - AI Context Manager
// Manages contextual understanding across interactions

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use crate::ai_layer::{AiContext, TimeOfDay, NoiseLevel, LightingCondition, DeviceState};

/// Context manager that maintains and updates conversation context
pub struct ContextManager {
    /// Conversation history
    conversation_history: VecDeque<ConversationEntry>,
    /// Maximum history entries to keep
    max_history: usize,
    /// Current session context
    session_context: SessionContext,
    /// User preferences learned over time
    user_preferences: UserPreferences,
    /// Short-term memory (recent entities, topics)
    short_term_memory: ShortTermMemory,
    /// Long-term context storage
    long_term_context: LongTermContext,
}

/// Individual conversation entry
#[derive(Debug, Clone)]
pub struct ConversationEntry {
    /// User input text
    pub input: String,
    /// System response
    pub response: String,
    /// Detected intent
    pub intent: Option<String>,
    /// Extracted entities
    pub entities: Vec<(String, String)>,
    /// Timestamp
    pub timestamp: Instant,
    /// Success indicator
    pub successful: bool,
}

/// Session-specific context
#[derive(Debug, Clone, Default)]
pub struct SessionContext {
    /// Session start time
    pub session_start: Option<Instant>,
    /// Active task name
    pub active_task: Option<String>,
    /// Current topic of conversation
    pub current_topic: Option<String>,
    /// Expected next input type
    pub expecting: Option<ExpectedInput>,
    /// Disambiguation context
    pub disambiguation: Option<DisambiguationContext>,
    /// Pending confirmations
    pub pending_confirmations: Vec<PendingConfirmation>,
}

/// Expected input type for context-aware parsing
#[derive(Debug, Clone)]
pub enum ExpectedInput {
    Confirmation,
    Selection(Vec<String>),
    SlotValue(String, String), // slot name, expected type
    FreeText,
    Number,
    Time,
    Location,
}

/// Context for disambiguation
#[derive(Debug, Clone)]
pub struct DisambiguationContext {
    /// Original ambiguous query
    pub original_query: String,
    /// Possible interpretations
    pub options: Vec<DisambiguationOption>,
    /// When disambiguation started
    pub started: Instant,
}

/// A disambiguation option
#[derive(Debug, Clone)]
pub struct DisambiguationOption {
    /// Option label
    pub label: String,
    /// Intent if selected
    pub intent: String,
    /// Associated entities
    pub entities: Vec<(String, String)>,
}

/// Pending confirmation
#[derive(Debug, Clone)]
pub struct PendingConfirmation {
    /// Action to confirm
    pub action: String,
    /// Description for user
    pub description: String,
    /// Confirmation created at
    pub created: Instant,
    /// Timeout duration
    pub timeout: Duration,
}

/// User preferences learned from interactions
#[derive(Debug, Clone, Default)]
pub struct UserPreferences {
    /// Preferred verbosity level (0-2)
    pub verbosity: u8,
    /// Preferred response style
    pub response_style: ResponseStyle,
    /// Frequently used apps
    pub frequent_apps: Vec<(String, u32)>,
    /// Common contacts
    pub frequent_contacts: Vec<(String, u32)>,
    /// Location preferences
    pub location_preferences: HashMap<String, String>,
    /// Time preferences (e.g., preferred reminder times)
    pub time_preferences: HashMap<String, String>,
}

/// Response style preference
#[derive(Debug, Clone, Default)]
pub enum ResponseStyle {
    #[default]
    Balanced,
    Concise,
    Detailed,
    Friendly,
    Professional,
}

/// Short-term memory for recent context
#[derive(Debug, Clone, Default)]
pub struct ShortTermMemory {
    /// Recently mentioned entities
    pub recent_entities: VecDeque<RecentEntity>,
    /// Recent topics
    pub recent_topics: VecDeque<String>,
    /// Referenced items (for "it", "that", etc.)
    pub referents: HashMap<String, String>,
    /// Active pronoun references
    pub pronoun_context: PronounContext,
}

/// A recently mentioned entity
#[derive(Debug, Clone)]
pub struct RecentEntity {
    /// Entity type
    pub entity_type: String,
    /// Entity value
    pub value: String,
    /// When mentioned
    pub mentioned: Instant,
    /// Mention count in session
    pub count: u32,
}

/// Context for resolving pronouns
#[derive(Debug, Clone, Default)]
pub struct PronounContext {
    /// Last mentioned person
    pub last_person: Option<String>,
    /// Last mentioned place
    pub last_place: Option<String>,
    /// Last mentioned thing/object
    pub last_thing: Option<String>,
    /// Last mentioned time
    pub last_time: Option<String>,
    /// Last mentioned action
    pub last_action: Option<String>,
}

/// Long-term context (persists across sessions)
#[derive(Debug, Clone, Default)]
pub struct LongTermContext {
    /// Interaction count
    pub total_interactions: u64,
    /// Common patterns
    pub usage_patterns: Vec<UsagePattern>,
    /// Learned facts about user
    pub user_facts: HashMap<String, String>,
    /// Saved shortcuts
    pub shortcuts: HashMap<String, String>,
}

/// Usage pattern
#[derive(Debug, Clone)]
pub struct UsagePattern {
    /// Pattern description
    pub pattern: String,
    /// When pattern occurs
    pub context: PatternContext,
    /// Associated action
    pub action: String,
    /// Confidence in pattern
    pub confidence: f32,
}

/// When a pattern typically occurs
#[derive(Debug, Clone)]
pub enum PatternContext {
    TimeOfDay(TimeOfDay),
    DayOfWeek(u8),
    Location(String),
    AfterAction(String),
    Always,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new() -> Self {
        Self {
            conversation_history: VecDeque::with_capacity(100),
            max_history: 100,
            session_context: SessionContext::default(),
            user_preferences: UserPreferences::default(),
            short_term_memory: ShortTermMemory::default(),
            long_term_context: LongTermContext::default(),
        }
    }
    
    /// Start a new session
    pub fn start_session(&mut self) {
        self.session_context = SessionContext {
            session_start: Some(Instant::now()),
            ..Default::default()
        };
        self.short_term_memory = ShortTermMemory::default();
    }
    
    /// Add a conversation entry
    pub fn add_entry(&mut self, entry: ConversationEntry) {
        // Update short-term memory with entities
        for (entity_type, value) in &entry.entities {
            self.update_recent_entity(entity_type, value);
            self.update_pronoun_context(entity_type, value);
        }
        
        // Update topic if intent changed
        if let Some(intent) = &entry.intent {
            self.session_context.current_topic = Some(intent.clone());
        }
        
        // Track success for learning
        if entry.successful {
            self.long_term_context.total_interactions += 1;
        }
        
        // Add to history
        self.conversation_history.push_back(entry);
        while self.conversation_history.len() > self.max_history {
            self.conversation_history.pop_front();
        }
    }
    
    /// Update recent entity tracking
    fn update_recent_entity(&mut self, entity_type: &str, value: &str) {
        // Check if entity already exists
        let mut found = false;
        for recent in &mut self.short_term_memory.recent_entities {
            if recent.entity_type == entity_type && recent.value == value {
                recent.count += 1;
                recent.mentioned = Instant::now();
                found = true;
                break;
            }
        }
        
        if !found {
            self.short_term_memory.recent_entities.push_back(RecentEntity {
                entity_type: entity_type.to_string(),
                value: value.to_string(),
                mentioned: Instant::now(),
                count: 1,
            });
        }
        
        // Keep bounded
        while self.short_term_memory.recent_entities.len() > 20 {
            self.short_term_memory.recent_entities.pop_front();
        }
    }
    
    /// Update pronoun context based on entity type
    fn update_pronoun_context(&mut self, entity_type: &str, value: &str) {
        let pc = &mut self.short_term_memory.pronoun_context;
        
        match entity_type.to_lowercase().as_str() {
            "contact" | "person" | "name" => {
                pc.last_person = Some(value.to_string());
            }
            "location" | "place" | "address" => {
                pc.last_place = Some(value.to_string());
            }
            "time" | "duration" | "date" => {
                pc.last_time = Some(value.to_string());
            }
            "app" | "item" | "object" => {
                pc.last_thing = Some(value.to_string());
            }
            _ => {}
        }
    }
    
    /// Resolve a pronoun to its referent
    pub fn resolve_pronoun(&self, pronoun: &str) -> Option<String> {
        let pc = &self.short_term_memory.pronoun_context;
        
        match pronoun.to_lowercase().as_str() {
            "he" | "him" | "she" | "her" | "they" | "them" => {
                pc.last_person.clone()
            }
            "it" | "this" | "that" => {
                // Could be thing, place, or time - use most recent
                pc.last_thing.clone()
                    .or_else(|| pc.last_place.clone())
                    .or_else(|| pc.last_time.clone())
            }
            "there" | "here" => {
                pc.last_place.clone()
            }
            "then" | "that time" => {
                pc.last_time.clone()
            }
            _ => None,
        }
    }
    
    /// Get most recent entity of a type
    pub fn get_recent_entity(&self, entity_type: &str) -> Option<String> {
        self.short_term_memory.recent_entities.iter()
            .rev()
            .find(|e| e.entity_type == entity_type)
            .map(|e| e.value.clone())
    }
    
    /// Check if we're expecting a specific input
    pub fn is_expecting(&self) -> Option<&ExpectedInput> {
        self.session_context.expecting.as_ref()
    }
    
    /// Set expected input for next turn
    pub fn expect(&mut self, input_type: ExpectedInput) {
        self.session_context.expecting = Some(input_type);
    }
    
    /// Clear expected input
    pub fn clear_expectation(&mut self) {
        self.session_context.expecting = None;
    }
    
    /// Start disambiguation
    pub fn start_disambiguation(&mut self, query: &str, options: Vec<DisambiguationOption>) {
        self.session_context.disambiguation = Some(DisambiguationContext {
            original_query: query.to_string(),
            options,
            started: Instant::now(),
        });
        self.session_context.expecting = Some(ExpectedInput::Selection(
            self.session_context.disambiguation.as_ref()
                .map(|d| d.options.iter().map(|o| o.label.clone()).collect())
                .unwrap_or_default()
        ));
    }
    
    /// Resolve disambiguation with user selection
    pub fn resolve_disambiguation(&mut self, selection: usize) -> Option<(&str, Vec<(String, String)>)> {
        if let Some(disambiguation) = &self.session_context.disambiguation {
            if selection < disambiguation.options.len() {
                let option = &disambiguation.options[selection];
                return Some((&option.intent, option.entities.clone()));
            }
        }
        None
    }
    
    /// Clear disambiguation
    pub fn clear_disambiguation(&mut self) {
        self.session_context.disambiguation = None;
        self.session_context.expecting = None;
    }
    
    /// Add a pending confirmation
    pub fn add_confirmation(&mut self, action: &str, description: &str, timeout: Duration) {
        self.session_context.pending_confirmations.push(PendingConfirmation {
            action: action.to_string(),
            description: description.to_string(),
            created: Instant::now(),
            timeout,
        });
        self.session_context.expecting = Some(ExpectedInput::Confirmation);
    }
    
    /// Process confirmation response
    pub fn process_confirmation(&mut self, confirmed: bool) -> Option<String> {
        if let Some(confirmation) = self.session_context.pending_confirmations.pop() {
            self.session_context.expecting = None;
            if confirmed {
                return Some(confirmation.action);
            }
        }
        None
    }
    
    /// Check and remove expired confirmations
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.session_context.pending_confirmations.retain(|c| {
            now.duration_since(c.created) < c.timeout
        });
        
        if self.session_context.pending_confirmations.is_empty() {
            if matches!(self.session_context.expecting, Some(ExpectedInput::Confirmation)) {
                self.session_context.expecting = None;
            }
        }
    }
    
    /// Track app usage for preferences
    pub fn track_app_usage(&mut self, app: &str) {
        let mut found = false;
        for (a, count) in &mut self.user_preferences.frequent_apps {
            if a == app {
                *count += 1;
                found = true;
                break;
            }
        }
        if !found {
            self.user_preferences.frequent_apps.push((app.to_string(), 1));
        }
        
        // Sort by frequency
        self.user_preferences.frequent_apps.sort_by(|a, b| b.1.cmp(&a.1));
        self.user_preferences.frequent_apps.truncate(10);
    }
    
    /// Track contact usage for preferences
    pub fn track_contact_usage(&mut self, contact: &str) {
        let mut found = false;
        for (c, count) in &mut self.user_preferences.frequent_contacts {
            if c == contact {
                *count += 1;
                found = true;
                break;
            }
        }
        if !found {
            self.user_preferences.frequent_contacts.push((contact.to_string(), 1));
        }
        
        // Sort by frequency
        self.user_preferences.frequent_contacts.sort_by(|a, b| b.1.cmp(&a.1));
        self.user_preferences.frequent_contacts.truncate(20);
    }
    
    /// Get suggested contacts (most frequent)
    pub fn get_suggested_contacts(&self, limit: usize) -> Vec<String> {
        self.user_preferences.frequent_contacts.iter()
            .take(limit)
            .map(|(c, _)| c.clone())
            .collect()
    }
    
    /// Get suggested apps (most frequent)
    pub fn get_suggested_apps(&self, limit: usize) -> Vec<String> {
        self.user_preferences.frequent_apps.iter()
            .take(limit)
            .map(|(a, _)| a.clone())
            .collect()
    }
    
    /// Build AI context from current state
    pub fn build_ai_context(&self, base_context: &AiContext) -> AiContext {
        let context = base_context.clone();
        
        // Note: AiContext conversation_turns and last_intent
        // are tracked internally by the ContextManager
        // rather than stored on AiContext itself
        
        context
    }
    
    /// Get conversation summary for context
    pub fn get_conversation_summary(&self, turns: usize) -> Vec<(String, String)> {
        self.conversation_history.iter()
            .rev()
            .take(turns)
            .map(|e| (e.input.clone(), e.response.clone()))
            .collect()
    }
    
    /// Check if this is a follow-up to previous
    pub fn is_follow_up(&self, input: &str) -> bool {
        let input_lower = input.to_lowercase();
        
        // Check for explicit follow-up patterns
        let follow_up_patterns = [
            "and also", "also", "another", "what about",
            "how about", "and", "but", "instead",
            "actually", "wait", "oh", "sorry",
            "yes", "no", "yeah", "nope", "ok", "okay",
        ];
        
        for pattern in &follow_up_patterns {
            if input_lower.starts_with(pattern) {
                return true;
            }
        }
        
        // Check for pronouns that need context
        let context_pronouns = ["it", "that", "this", "him", "her", "them", "there"];
        for pronoun in &context_pronouns {
            if input_lower.contains(pronoun) && self.resolve_pronoun(pronoun).is_some() {
                return true;
            }
        }
        
        // Check for missing subject (verb-only)
        let verb_starters = ["call", "text", "send", "open", "play", "stop", "show", "tell"];
        for verb in &verb_starters {
            if input_lower.starts_with(verb) && !input_lower.contains(' ') {
                return true;
            }
        }
        
        false
    }
    
    /// Set a user shortcut
    pub fn set_shortcut(&mut self, phrase: &str, expansion: &str) {
        self.long_term_context.shortcuts.insert(phrase.to_lowercase(), expansion.to_string());
    }
    
    /// Get shortcut expansion
    pub fn get_shortcut(&self, phrase: &str) -> Option<&String> {
        self.long_term_context.shortcuts.get(&phrase.to_lowercase())
    }
    
    /// Expand shortcuts in input
    pub fn expand_shortcuts(&self, input: &str) -> String {
        let input_lower = input.to_lowercase();
        
        for (shortcut, expansion) in &self.long_term_context.shortcuts {
            if input_lower == *shortcut {
                return expansion.clone();
            }
        }
        
        input.to_string()
    }
    
    /// Learn from successful interaction
    pub fn learn_from_success(&mut self, intent: &str, context: &AiContext) {
        // Learn time-based patterns
        let tod = &context.time_of_day;
        let pattern = UsagePattern {
            pattern: format!("{} at {:?}", intent, tod),
            context: PatternContext::TimeOfDay(tod.clone()),
            action: intent.to_string(),
            confidence: 0.5,
        };
        
        // Check if pattern exists
        let mut found = false;
        for existing in &mut self.long_term_context.usage_patterns {
            if existing.action == pattern.action && 
               matches!(&existing.context, PatternContext::TimeOfDay(t) if t == tod) {
                existing.confidence = (existing.confidence + 0.1).min(1.0);
                found = true;
                break;
            }
        }
        
        if !found {
            self.long_term_context.usage_patterns.push(pattern);
        }
        
        // Keep patterns bounded
        self.long_term_context.usage_patterns.truncate(50);
    }
    
    /// Get suggestions based on learned patterns
    pub fn get_pattern_suggestions(&self, context: &AiContext) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        for pattern in &self.long_term_context.usage_patterns {
            if pattern.confidence > 0.7 {
                let matches = match &pattern.context {
                    PatternContext::TimeOfDay(tod) => {
                        context.time_of_day == *tod
                    }
                    PatternContext::Always => true,
                    _ => false,
                };
                
                if matches {
                    suggestions.push(pattern.action.clone());
                }
            }
        }
        
        suggestions
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_manager_creation() {
        let manager = ContextManager::new();
        assert!(manager.conversation_history.is_empty());
    }
    
    #[test]
    fn test_session_start() {
        let mut manager = ContextManager::new();
        manager.start_session();
        assert!(manager.session_context.session_start.is_some());
    }
    
    #[test]
    fn test_add_entry() {
        let mut manager = ContextManager::new();
        manager.start_session();
        
        let entry = ConversationEntry {
            input: "call mom".to_string(),
            response: "Calling Mom".to_string(),
            intent: Some("call".to_string()),
            entities: vec![("contact".to_string(), "Mom".to_string())],
            timestamp: Instant::now(),
            successful: true,
        };
        
        manager.add_entry(entry);
        assert_eq!(manager.conversation_history.len(), 1);
    }
    
    #[test]
    fn test_pronoun_resolution() {
        let mut manager = ContextManager::new();
        manager.start_session();
        
        let entry = ConversationEntry {
            input: "call John".to_string(),
            response: "Calling John".to_string(),
            intent: Some("call".to_string()),
            entities: vec![("contact".to_string(), "John".to_string())],
            timestamp: Instant::now(),
            successful: true,
        };
        
        manager.add_entry(entry);
        
        let resolved = manager.resolve_pronoun("him");
        assert_eq!(resolved, Some("John".to_string()));
    }
    
    #[test]
    fn test_recent_entity() {
        let mut manager = ContextManager::new();
        manager.start_session();
        
        manager.update_recent_entity("contact", "Alice");
        manager.update_recent_entity("contact", "Bob");
        
        let recent = manager.get_recent_entity("contact");
        assert_eq!(recent, Some("Bob".to_string()));
    }
    
    #[test]
    fn test_disambiguation() {
        let mut manager = ContextManager::new();
        manager.start_session();
        
        let options = vec![
            DisambiguationOption {
                label: "John Smith".to_string(),
                intent: "call".to_string(),
                entities: vec![("contact".to_string(), "John Smith".to_string())],
            },
            DisambiguationOption {
                label: "John Doe".to_string(),
                intent: "call".to_string(),
                entities: vec![("contact".to_string(), "John Doe".to_string())],
            },
        ];
        
        manager.start_disambiguation("call john", options);
        
        let resolved = manager.resolve_disambiguation(0);
        assert!(resolved.is_some());
        let (intent, entities) = resolved.unwrap();
        assert_eq!(intent, "call");
        assert_eq!(entities[0].1, "John Smith");
    }
    
    #[test]
    fn test_confirmation() {
        let mut manager = ContextManager::new();
        manager.start_session();
        
        manager.add_confirmation("send_message", "Send message to John?", Duration::from_secs(30));
        
        assert!(matches!(manager.is_expecting(), Some(ExpectedInput::Confirmation)));
        
        let action = manager.process_confirmation(true);
        assert_eq!(action, Some("send_message".to_string()));
    }
    
    #[test]
    fn test_app_tracking() {
        let mut manager = ContextManager::new();
        
        manager.track_app_usage("music");
        manager.track_app_usage("camera");
        manager.track_app_usage("music");
        manager.track_app_usage("music");
        
        let suggested = manager.get_suggested_apps(2);
        assert_eq!(suggested[0], "music");
    }
    
    #[test]
    fn test_shortcuts() {
        let mut manager = ContextManager::new();
        
        manager.set_shortcut("gohome", "navigate to home");
        
        let expanded = manager.expand_shortcuts("gohome");
        assert_eq!(expanded, "navigate to home");
    }
    
    #[test]
    fn test_follow_up_detection() {
        let mut manager = ContextManager::new();
        manager.start_session();
        
        // Add context
        let entry = ConversationEntry {
            input: "call John".to_string(),
            response: "Calling John".to_string(),
            intent: Some("call".to_string()),
            entities: vec![("contact".to_string(), "John".to_string())],
            timestamp: Instant::now(),
            successful: true,
        };
        manager.add_entry(entry);
        
        // Check follow-up detection
        assert!(manager.is_follow_up("text him instead"));
        assert!(manager.is_follow_up("and also send a message"));
    }
}
