//! Conversation Memory - Short and long-term memory for contextual understanding
//!
//! Tracks conversation history, extracts entities, and maintains context
//! across interactions for more natural, contextual responses.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// A single turn in the conversation
#[derive(Debug, Clone)]
pub struct ConversationTurn {
    pub id: u64,
    pub timestamp: u64,
    pub user_input: String,
    pub intent: String,
    pub entities: HashMap<String, String>,
    pub response: String,
    pub success: bool,
}

/// Entity types we can extract and remember
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    Amount,
    Address,
    Duration,
    Time,
    Date,
    Name,
    Percentage,
    Custom(String),
}

impl EntityType {
    pub fn name(&self) -> &str {
        match self {
            EntityType::Amount => "amount",
            EntityType::Address => "address",
            EntityType::Duration => "duration",
            EntityType::Time => "time",
            EntityType::Date => "date",
            EntityType::Name => "name",
            EntityType::Percentage => "percentage",
            EntityType::Custom(s) => s,
        }
    }
}

/// An entity mentioned in conversation
#[derive(Debug, Clone)]
pub struct Entity {
    pub entity_type: EntityType,
    pub value: String,
    pub turn_id: u64,
    pub mentions: u32,
    pub last_mentioned: u64,
}

/// Tracks topic of conversation
#[derive(Debug, Clone, PartialEq)]
pub enum Topic {
    Balance,
    Transactions,
    Staking,
    Camera,
    Timer,
    Settings,
    Help,
    General,
}

impl Topic {
    pub fn from_intent(intent: &str) -> Self {
        match intent {
            i if i.contains("balance") => Topic::Balance,
            i if i.contains("send") || i.contains("transfer") || i.contains("transaction") => Topic::Transactions,
            i if i.contains("stake") || i.contains("unstake") => Topic::Staking,
            i if i.contains("photo") || i.contains("camera") || i.contains("capture") => Topic::Camera,
            i if i.contains("timer") || i.contains("alarm") || i.contains("remind") => Topic::Timer,
            i if i.contains("setting") || i.contains("config") => Topic::Settings,
            i if i.contains("help") || i.contains("how") => Topic::Help,
            _ => Topic::General,
        }
    }
}

/// Short-term working memory for current conversation
#[derive(Debug)]
pub struct WorkingMemory {
    // Recent turns (last 10)
    recent_turns: VecDeque<ConversationTurn>,
    max_turns: usize,
    
    // Current context
    current_topic: Topic,
    topic_depth: u32,  // How many turns on this topic
    
    // Active entities (things mentioned recently)
    active_entities: HashMap<EntityType, Entity>,
    
    // Pending references ("it", "that", "the amount")
    anaphora_map: HashMap<String, String>,
    
    // Session tracking
    session_start: Instant,
    turn_counter: u64,
}

impl WorkingMemory {
    pub fn new() -> Self {
        Self {
            recent_turns: VecDeque::with_capacity(10),
            max_turns: 10,
            current_topic: Topic::General,
            topic_depth: 0,
            active_entities: HashMap::new(),
            anaphora_map: HashMap::new(),
            session_start: Instant::now(),
            turn_counter: 0,
        }
    }
    
    /// Add a new turn to memory
    pub fn add_turn(&mut self, input: &str, intent: &str, entities: HashMap<String, String>, response: &str, success: bool) {
        self.turn_counter += 1;
        
        let turn = ConversationTurn {
            id: self.turn_counter,
            timestamp: current_timestamp(),
            user_input: input.to_string(),
            intent: intent.to_string(),
            entities: entities.clone(),
            response: response.to_string(),
            success,
        };
        
        // Update topic tracking
        let new_topic = Topic::from_intent(intent);
        if new_topic == self.current_topic {
            self.topic_depth += 1;
        } else {
            self.current_topic = new_topic;
            self.topic_depth = 1;
        }
        
        // Extract and store entities
        self.extract_entities(&turn);
        
        // Update anaphora references
        self.update_anaphora(&turn);
        
        // Add to recent turns
        self.recent_turns.push_back(turn);
        if self.recent_turns.len() > self.max_turns {
            self.recent_turns.pop_front();
        }
    }
    
    fn extract_entities(&mut self, turn: &ConversationTurn) {
        for (key, value) in &turn.entities {
            let entity_type = match key.as_str() {
                "amount" => EntityType::Amount,
                "address" | "recipient" => EntityType::Address,
                "duration" => EntityType::Duration,
                "time" => EntityType::Time,
                "date" => EntityType::Date,
                "name" => EntityType::Name,
                "percentage" => EntityType::Percentage,
                _ => EntityType::Custom(key.clone()),
            };
            
            if let Some(existing) = self.active_entities.get_mut(&entity_type) {
                existing.value = value.clone();
                existing.mentions += 1;
                existing.last_mentioned = turn.timestamp;
            } else {
                self.active_entities.insert(entity_type.clone(), Entity {
                    entity_type,
                    value: value.clone(),
                    turn_id: turn.id,
                    mentions: 1,
                    last_mentioned: turn.timestamp,
                });
            }
        }
    }
    
    fn update_anaphora(&mut self, turn: &ConversationTurn) {
        // Map "it", "that", "the amount" to the most recent relevant entity
        
        // If we discussed an amount, "it" might refer to it
        if let Some(amount) = self.active_entities.get(&EntityType::Amount) {
            self.anaphora_map.insert("it".to_string(), amount.value.clone());
            self.anaphora_map.insert("that amount".to_string(), amount.value.clone());
            self.anaphora_map.insert("the amount".to_string(), amount.value.clone());
        }
        
        // If we discussed an address, "them" or "that address" refers to it
        if let Some(address) = self.active_entities.get(&EntityType::Address) {
            self.anaphora_map.insert("them".to_string(), address.value.clone());
            self.anaphora_map.insert("that address".to_string(), address.value.clone());
            self.anaphora_map.insert("the recipient".to_string(), address.value.clone());
        }
    }
    
    /// Resolve anaphora in user input ("send it to them" -> "send 100 to 0xABC")
    pub fn resolve_anaphora(&self, input: &str) -> String {
        let mut resolved = input.to_string();
        
        for (reference, value) in &self.anaphora_map {
            let patterns = vec![
                format!(" {} ", reference),
                format!(" {}", reference),
                format!("{} ", reference),
            ];
            
            for pattern in patterns {
                if resolved.to_lowercase().contains(&pattern.to_lowercase()) {
                    resolved = resolved.replace(&pattern, &format!(" {} ", value));
                }
            }
        }
        
        resolved.trim().to_string()
    }
    
    /// Get an active entity by type
    pub fn get_entity(&self, entity_type: &EntityType) -> Option<&Entity> {
        self.active_entities.get(entity_type)
    }
    
    /// Get the current topic
    pub fn current_topic(&self) -> &Topic {
        &self.current_topic
    }
    
    /// How deep are we in the current topic?
    pub fn topic_depth(&self) -> u32 {
        self.topic_depth
    }
    
    /// Get last N turns
    pub fn last_turns(&self, n: usize) -> Vec<&ConversationTurn> {
        self.recent_turns.iter().rev().take(n).collect()
    }
    
    /// Get the last turn
    pub fn last_turn(&self) -> Option<&ConversationTurn> {
        self.recent_turns.back()
    }
    
    /// Session duration
    pub fn session_duration(&self) -> Duration {
        self.session_start.elapsed()
    }
    
    /// Summarize current context for AI
    pub fn context_summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str(&format!("Topic: {:?} (depth: {})\n", self.current_topic, self.topic_depth));
        
        if !self.active_entities.is_empty() {
            summary.push_str("Active entities:\n");
            for (_, entity) in &self.active_entities {
                summary.push_str(&format!("  - {}: {}\n", entity.entity_type.name(), entity.value));
            }
        }
        
        if let Some(last) = self.last_turn() {
            summary.push_str(&format!("Last action: {}\n", last.intent));
        }
        
        summary
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Long-term memory for persistent facts
#[derive(Debug, Clone)]
pub struct LongTermMemory {
    // User-stated facts ("I prefer X", "My name is Y")
    user_facts: HashMap<String, String>,
    
    // Learned preferences from observation
    inferred_facts: HashMap<String, (String, f32)>,  // fact -> (value, confidence)
    
    // Important past events
    significant_events: Vec<MemoryEvent>,
    max_events: usize,
}

#[derive(Debug, Clone)]
pub struct MemoryEvent {
    pub timestamp: u64,
    pub description: String,
    pub importance: f32,  // 0.0 to 1.0
}

impl LongTermMemory {
    pub fn new() -> Self {
        Self {
            user_facts: HashMap::new(),
            inferred_facts: HashMap::new(),
            significant_events: Vec::new(),
            max_events: 100,
        }
    }
    
    /// Store a fact the user explicitly stated
    pub fn store_user_fact(&mut self, key: &str, value: &str) {
        self.user_facts.insert(key.to_string(), value.to_string());
    }
    
    /// Store an inferred fact with confidence
    pub fn store_inferred(&mut self, key: &str, value: &str, confidence: f32) {
        if let Some((_, existing_conf)) = self.inferred_facts.get(key) {
            if confidence > *existing_conf {
                self.inferred_facts.insert(key.to_string(), (value.to_string(), confidence));
            }
        } else {
            self.inferred_facts.insert(key.to_string(), (value.to_string(), confidence));
        }
    }
    
    /// Retrieve a fact (user facts take precedence)
    pub fn get_fact(&self, key: &str) -> Option<String> {
        if let Some(value) = self.user_facts.get(key) {
            return Some(value.clone());
        }
        if let Some((value, conf)) = self.inferred_facts.get(key) {
            if *conf > 0.6 {
                return Some(value.clone());
            }
        }
        None
    }
    
    /// Record a significant event
    pub fn record_event(&mut self, description: &str, importance: f32) {
        self.significant_events.push(MemoryEvent {
            timestamp: current_timestamp(),
            description: description.to_string(),
            importance: importance.clamp(0.0, 1.0),
        });
        
        // Keep only most important events
        if self.significant_events.len() > self.max_events {
            self.significant_events.sort_by(|a, b| {
                b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal)
            });
            self.significant_events.truncate(self.max_events);
        }
    }
    
    /// Get recent significant events
    pub fn recent_events(&self, limit: usize) -> Vec<&MemoryEvent> {
        let mut events: Vec<_> = self.significant_events.iter().collect();
        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        events.truncate(limit);
        events
    }
    
    /// All stored facts
    pub fn all_facts(&self) -> Vec<(&str, &str)> {
        self.user_facts.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect()
    }
}

impl Default for LongTermMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// The complete memory system
pub struct MemorySystem {
    pub working: WorkingMemory,
    pub long_term: LongTermMemory,
}

impl MemorySystem {
    pub fn new() -> Self {
        Self {
            working: WorkingMemory::new(),
            long_term: LongTermMemory::new(),
        }
    }
    
    /// Process input with full memory context
    pub fn process_input(&mut self, input: &str) -> ProcessedInput {
        // Resolve anaphora
        let resolved = self.working.resolve_anaphora(input);
        
        // Check for fact-setting patterns
        let facts = self.extract_facts(input);
        for (key, value) in &facts {
            self.long_term.store_user_fact(key, value);
        }
        
        // Get relevant context
        let context = self.build_context();
        
        ProcessedInput {
            original: input.to_string(),
            resolved,
            context,
            extracted_facts: facts,
        }
    }
    
    fn extract_facts(&self, input: &str) -> Vec<(String, String)> {
        let mut facts = Vec::new();
        let lower = input.to_lowercase();
        
        // "My name is X"
        if let Some(pos) = lower.find("my name is ") {
            let rest = &input[pos + 11..];
            if let Some(name) = rest.split_whitespace().next() {
                facts.push(("user_name".to_string(), name.to_string()));
            }
        }
        
        // "I prefer X"
        if let Some(pos) = lower.find("i prefer ") {
            let rest = &input[pos + 9..];
            facts.push(("user_preference".to_string(), rest.trim().to_string()));
        }
        
        // "Call me X"
        if let Some(pos) = lower.find("call me ") {
            let rest = &input[pos + 8..];
            if let Some(name) = rest.split_whitespace().next() {
                facts.push(("user_name".to_string(), name.to_string()));
            }
        }
        
        facts
    }
    
    fn build_context(&self) -> ConversationContext {
        ConversationContext {
            topic: self.working.current_topic().clone(),
            topic_depth: self.working.topic_depth(),
            active_entities: self.working.active_entities
                .iter()
                .map(|(t, e)| (t.name().to_string(), e.value.clone()))
                .collect(),
            user_name: self.long_term.get_fact("user_name"),
            relevant_facts: self.long_term.all_facts()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
    
    /// Record a completed turn
    pub fn record_turn(&mut self, input: &str, intent: &str, entities: HashMap<String, String>, response: &str, success: bool) {
        self.working.add_turn(input, intent, entities, response, success);
        
        // Record significant events
        if !success {
            self.long_term.record_event(
                &format!("Failed to understand: {}", input),
                0.3,
            );
        }
    }
    
    /// Get personalized greeting using memory
    pub fn personalized_greeting(&self) -> String {
        if let Some(name) = self.long_term.get_fact("user_name") {
            format!("Hello, {}!", name)
        } else {
            "Hello!".to_string()
        }
    }
    
    /// Suggest follow-up based on context
    pub fn suggest_followup(&self) -> Option<String> {
        let topic = self.working.current_topic();
        let depth = self.working.topic_depth();
        
        // If we've been on a topic for a while, suggest related actions
        if depth >= 2 {
            return match topic {
                Topic::Balance => Some("Would you like to see transaction history too?".to_string()),
                Topic::Transactions => Some("Want to set up a recurring transfer?".to_string()),
                Topic::Staking => Some("Would you like to see staking rewards?".to_string()),
                Topic::Camera => Some("Want to review recent photos?".to_string()),
                Topic::Timer => Some("Need to set another timer?".to_string()),
                _ => None,
            };
        }
        
        None
    }
}

impl Default for MemorySystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Processed input with memory context
#[derive(Debug, Clone)]
pub struct ProcessedInput {
    pub original: String,
    pub resolved: String,
    pub context: ConversationContext,
    pub extracted_facts: Vec<(String, String)>,
}

/// Context built from memory
#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub topic: Topic,
    pub topic_depth: u32,
    pub active_entities: HashMap<String, String>,
    pub user_name: Option<String>,
    pub relevant_facts: Vec<(String, String)>,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_working_memory_turn() {
        let mut memory = WorkingMemory::new();
        
        let mut entities = HashMap::new();
        entities.insert("amount".to_string(), "100".to_string());
        
        memory.add_turn(
            "check my balance",
            "check_balance",
            entities,
            "Your balance is 1000 KARA",
            true
        );
        
        assert_eq!(memory.topic_depth(), 1);
        assert!(matches!(memory.current_topic(), Topic::Balance));
    }
    
    #[test]
    fn test_anaphora_resolution() {
        let mut memory = WorkingMemory::new();
        
        let mut entities = HashMap::new();
        entities.insert("amount".to_string(), "100".to_string());
        
        memory.add_turn("send 100 KARA", "send_tokens", entities, "Sent!", true);
        
        // Now "it" should resolve to 100
        let resolved = memory.resolve_anaphora("send it again");
        assert!(resolved.contains("100"));
    }
    
    #[test]
    fn test_long_term_memory() {
        let mut memory = LongTermMemory::new();
        
        memory.store_user_fact("user_name", "Alice");
        memory.store_inferred("preferred_amount", "50", 0.8);
        
        assert_eq!(memory.get_fact("user_name"), Some("Alice".to_string()));
        assert_eq!(memory.get_fact("preferred_amount"), Some("50".to_string()));
    }
    
    #[test]
    fn test_fact_extraction() {
        let mut system = MemorySystem::new();
        
        let processed = system.process_input("My name is Bob and I prefer dark mode");
        
        assert!(processed.extracted_facts.iter().any(|(k, v)| k == "user_name" && v == "Bob"));
    }
    
    #[test]
    fn test_topic_tracking() {
        let mut memory = WorkingMemory::new();
        
        memory.add_turn("check balance", "check_balance", HashMap::new(), "1000 KARA", true);
        assert_eq!(memory.topic_depth(), 1);
        
        memory.add_turn("what about staking balance", "check_staking_balance", HashMap::new(), "500 KARA", true);
        assert_eq!(memory.topic_depth(), 2);  // Still balance-related
        
        memory.add_turn("take a photo", "capture_photo", HashMap::new(), "Photo taken", true);
        assert_eq!(memory.topic_depth(), 1);  // New topic
    }
    
    #[test]
    fn test_personalized_greeting() {
        let mut system = MemorySystem::new();
        
        // Before learning name
        assert_eq!(system.personalized_greeting(), "Hello!");
        
        // Teach it our name
        system.long_term.store_user_fact("user_name", "Alice");
        
        assert_eq!(system.personalized_greeting(), "Hello, Alice!");
    }
}
