// Kāraṇa OS - Query Understanding
// Deep understanding of user queries beyond simple intent matching

use std::collections::HashMap;
use crate::ai_layer::entities::{EntityExtractor, ExtractedEntity};
use crate::ai_layer::{AiContext, TimeOfDay};

/// Query understanding engine for deeper analysis
pub struct QueryUnderstanding {
    /// Command patterns
    command_patterns: Vec<CommandPattern>,
    /// Modifier recognizers
    modifiers: HashMap<String, ModifierType>,
    /// Urgency indicators
    urgency_indicators: Vec<String>,
    /// Politeness markers
    politeness_markers: Vec<String>,
    /// Negation words
    negations: Vec<String>,
    /// Conditional markers
    conditionals: Vec<String>,
}

/// A command pattern
#[derive(Debug, Clone)]
pub struct CommandPattern {
    /// Pattern keywords
    pub keywords: Vec<String>,
    /// Intent this maps to
    pub intent: String,
    /// Required context
    pub requires_context: Option<RequiredContext>,
}

/// Required context for a command
#[derive(Debug, Clone)]
pub enum RequiredContext {
    Location,
    Contact,
    Time,
    App,
    None,
}

/// Types of modifiers
#[derive(Debug, Clone, PartialEq)]
pub enum ModifierType {
    Urgency,
    Politeness,
    Negation,
    Conditional,
    Quantity,
    Quality,
    Temporal,
    Comparison,
}

/// Result of query understanding
#[derive(Debug, Clone)]
pub struct QueryAnalysis {
    /// Original query
    pub original: String,
    /// Normalized query
    pub normalized: String,
    /// Detected query type
    pub query_type: QueryType,
    /// Key action words
    pub action_words: Vec<String>,
    /// Modifiers found
    pub modifiers: Vec<Modifier>,
    /// Urgency level (0-1)
    pub urgency: f32,
    /// Politeness level (0-1)
    pub politeness: f32,
    /// Is negated
    pub negated: bool,
    /// Is conditional
    pub conditional: bool,
    /// Temporal references
    pub temporal: Option<TemporalReference>,
    /// Implied context
    pub implied_context: Vec<ImpliedContext>,
    /// Semantic components
    pub semantic: SemanticAnalysis,
}

/// Query types
#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    /// Direct command (call mom)
    Command,
    /// Question (what time is it)
    Question,
    /// Request (can you call mom)
    Request,
    /// Statement (I need to call mom)
    Statement,
    /// Acknowledgment (yes, ok)
    Acknowledgment,
    /// Negation (no, don't)
    Negation,
    /// Continuation (also, and)
    Continuation,
}

/// A modifier in the query
#[derive(Debug, Clone)]
pub struct Modifier {
    /// Modifier type
    pub modifier_type: ModifierType,
    /// The modifier word
    pub word: String,
    /// What it modifies
    pub target: Option<String>,
    /// Intensity (0-1)
    pub intensity: f32,
}

/// Temporal reference
#[derive(Debug, Clone)]
pub struct TemporalReference {
    /// Reference type
    pub ref_type: TemporalType,
    /// Specific time if mentioned
    pub specific: Option<String>,
    /// Relative to now
    pub relative: Option<RelativeTime>,
}

/// Temporal reference types
#[derive(Debug, Clone, PartialEq)]
pub enum TemporalType {
    Specific,    // at 3pm
    Relative,    // in 5 minutes
    Duration,    // for 10 minutes
    Deadline,    // by tomorrow
    Recurring,   // every day
}

/// Relative time
#[derive(Debug, Clone)]
pub struct RelativeTime {
    /// Direction (past/future)
    pub direction: TimeDirection,
    /// Amount
    pub amount: Option<u32>,
    /// Unit
    pub unit: Option<String>,
}

/// Time direction
#[derive(Debug, Clone, PartialEq)]
pub enum TimeDirection {
    Past,
    Future,
    Now,
}

/// Implied context from query
#[derive(Debug, Clone)]
pub struct ImpliedContext {
    /// Context type
    pub context_type: String,
    /// Inferred value
    pub value: String,
    /// Confidence
    pub confidence: f32,
}

/// Semantic analysis
#[derive(Debug, Clone, Default)]
pub struct SemanticAnalysis {
    /// Main verb/action
    pub main_action: Option<String>,
    /// Object of action
    pub object: Option<String>,
    /// Subject (usually implicit "I" or "you")
    pub subject: Option<String>,
    /// Indirect object
    pub indirect_object: Option<String>,
    /// Location
    pub location: Option<String>,
    /// Manner (how)
    pub manner: Option<String>,
}

impl QueryUnderstanding {
    /// Create new query understanding engine
    pub fn new() -> Self {
        let mut engine = Self {
            command_patterns: Vec::new(),
            modifiers: HashMap::new(),
            urgency_indicators: Vec::new(),
            politeness_markers: Vec::new(),
            negations: Vec::new(),
            conditionals: Vec::new(),
        };
        engine.initialize();
        engine
    }
    
    /// Initialize patterns and indicators
    fn initialize(&mut self) {
        // Urgency indicators
        self.urgency_indicators = vec![
            "now".to_string(),
            "immediately".to_string(),
            "urgent".to_string(),
            "asap".to_string(),
            "quickly".to_string(),
            "fast".to_string(),
            "hurry".to_string(),
            "right away".to_string(),
            "right now".to_string(),
            "emergency".to_string(),
        ];
        
        // Politeness markers
        self.politeness_markers = vec![
            "please".to_string(),
            "could you".to_string(),
            "would you".to_string(),
            "can you".to_string(),
            "may i".to_string(),
            "if you don't mind".to_string(),
            "kindly".to_string(),
            "thanks".to_string(),
            "thank you".to_string(),
        ];
        
        // Negation words
        self.negations = vec![
            "no".to_string(),
            "not".to_string(),
            "don't".to_string(),
            "doesn't".to_string(),
            "didn't".to_string(),
            "won't".to_string(),
            "wouldn't".to_string(),
            "can't".to_string(),
            "cannot".to_string(),
            "never".to_string(),
            "stop".to_string(),
            "cancel".to_string(),
        ];
        
        // Conditional markers
        self.conditionals = vec![
            "if".to_string(),
            "when".to_string(),
            "unless".to_string(),
            "until".to_string(),
            "while".to_string(),
            "after".to_string(),
            "before".to_string(),
            "once".to_string(),
        ];
        
        // Modifier mappings
        self.modifiers.insert("very".to_string(), ModifierType::Quality);
        self.modifiers.insert("really".to_string(), ModifierType::Quality);
        self.modifiers.insert("extremely".to_string(), ModifierType::Quality);
        self.modifiers.insert("a bit".to_string(), ModifierType::Quantity);
        self.modifiers.insert("little".to_string(), ModifierType::Quantity);
        self.modifiers.insert("more".to_string(), ModifierType::Comparison);
        self.modifiers.insert("less".to_string(), ModifierType::Comparison);
        self.modifiers.insert("louder".to_string(), ModifierType::Comparison);
        self.modifiers.insert("quieter".to_string(), ModifierType::Comparison);
        self.modifiers.insert("brighter".to_string(), ModifierType::Comparison);
        self.modifiers.insert("dimmer".to_string(), ModifierType::Comparison);
        self.modifiers.insert("later".to_string(), ModifierType::Temporal);
        self.modifiers.insert("earlier".to_string(), ModifierType::Temporal);
        self.modifiers.insert("soon".to_string(), ModifierType::Temporal);
    }
    
    /// Analyze a query
    pub fn analyze(&self, query: &str, context: &AiContext) -> QueryAnalysis {
        let normalized = self.normalize(query);
        let words: Vec<&str> = normalized.split_whitespace().collect();
        
        // Detect query type
        let query_type = self.detect_query_type(&normalized, &words);
        
        // Find action words
        let action_words = self.extract_action_words(&words);
        
        // Find modifiers
        let modifiers = self.extract_modifiers(&normalized, &words);
        
        // Calculate urgency
        let urgency = self.calculate_urgency(&normalized);
        
        // Calculate politeness
        let politeness = self.calculate_politeness(&normalized);
        
        // Detect negation
        let negated = self.is_negated(&normalized);
        
        // Detect conditional
        let conditional = self.is_conditional(&normalized);
        
        // Extract temporal references
        let temporal = self.extract_temporal(&normalized, &words);
        
        // Infer implied context
        let implied_context = self.infer_context(&normalized, context);
        
        // Semantic analysis
        let semantic = self.semantic_analysis(&normalized, &words);
        
        QueryAnalysis {
            original: query.to_string(),
            normalized,
            query_type,
            action_words,
            modifiers,
            urgency,
            politeness,
            negated,
            conditional,
            temporal,
            implied_context,
            semantic,
        }
    }
    
    /// Normalize query text
    fn normalize(&self, query: &str) -> String {
        query.to_lowercase()
            .replace("i'm", "i am")
            .replace("i'll", "i will")
            .replace("i'd", "i would")
            .replace("i've", "i have")
            .replace("you're", "you are")
            .replace("you'll", "you will")
            .replace("don't", "do not")
            .replace("doesn't", "does not")
            .replace("can't", "cannot")
            .replace("won't", "will not")
            .replace("wouldn't", "would not")
            .replace("couldn't", "could not")
            .replace("shouldn't", "should not")
    }
    
    /// Detect query type
    fn detect_query_type(&self, query: &str, words: &[&str]) -> QueryType {
        if words.is_empty() {
            return QueryType::Statement;
        }
        
        let first = words[0];
        
        // Check for question
        if query.ends_with('?') || ["what", "who", "where", "when", "why", "how", "is", "are", "can", "could", "will", "would", "do", "does"].contains(&first) {
            return QueryType::Question;
        }
        
        // Check for acknowledgment
        if ["yes", "yeah", "yep", "ok", "okay", "sure", "alright", "right", "correct", "got it"].contains(&first) {
            return QueryType::Acknowledgment;
        }
        
        // Check for negation response
        if ["no", "nope", "nah", "wrong", "incorrect", "not"].contains(&first) {
            return QueryType::Negation;
        }
        
        // Check for continuation
        if ["and", "also", "plus", "then", "next", "another", "additionally"].contains(&first) {
            return QueryType::Continuation;
        }
        
        // Check for request (polite form)
        if query.contains("could you") || query.contains("can you") || 
           query.contains("would you") || query.contains("please") {
            return QueryType::Request;
        }
        
        // Check for statement (I need, I want)
        if query.starts_with("i need") || query.starts_with("i want") || 
           query.starts_with("i am") || query.starts_with("i have") {
            return QueryType::Statement;
        }
        
        // Default to command
        QueryType::Command
    }
    
    /// Extract action words (verbs)
    fn extract_action_words(&self, words: &[&str]) -> Vec<String> {
        let action_verbs = [
            "call", "text", "message", "send", "email",
            "navigate", "go", "drive", "walk", "take",
            "play", "pause", "stop", "skip", "next", "previous",
            "open", "close", "show", "hide", "display",
            "search", "find", "look", "google",
            "set", "create", "make", "add", "remove", "delete",
            "remind", "schedule", "book", "cancel",
            "turn", "switch", "enable", "disable",
            "read", "tell", "say", "speak",
            "take", "capture", "record",
            "translate", "convert",
            "check", "verify", "confirm",
            "help", "assist",
        ];
        
        words.iter()
            .filter(|w| action_verbs.contains(w))
            .map(|w| w.to_string())
            .collect()
    }
    
    /// Extract modifiers
    fn extract_modifiers(&self, query: &str, words: &[&str]) -> Vec<Modifier> {
        let mut modifiers = Vec::new();
        
        for (word, modifier_type) in &self.modifiers {
            if query.contains(word) {
                modifiers.push(Modifier {
                    modifier_type: modifier_type.clone(),
                    word: word.clone(),
                    target: self.find_modifier_target(word, words),
                    intensity: self.modifier_intensity(word),
                });
            }
        }
        
        modifiers
    }
    
    /// Find what a modifier is targeting
    fn find_modifier_target(&self, modifier: &str, words: &[&str]) -> Option<String> {
        if let Some(pos) = words.iter().position(|w| *w == modifier) {
            // Look at next word
            if pos + 1 < words.len() {
                return Some(words[pos + 1].to_string());
            }
        }
        None
    }
    
    /// Get modifier intensity
    fn modifier_intensity(&self, modifier: &str) -> f32 {
        match modifier {
            "extremely" | "very" | "really" => 0.9,
            "quite" | "pretty" => 0.7,
            "somewhat" | "a bit" | "little" => 0.3,
            _ => 0.5,
        }
    }
    
    /// Calculate urgency level
    fn calculate_urgency(&self, query: &str) -> f32 {
        let mut urgency = 0.0;
        
        for indicator in &self.urgency_indicators {
            if query.contains(indicator) {
                urgency += 0.3;
            }
        }
        
        // Exclamation marks increase urgency
        let exclamations = query.chars().filter(|c| *c == '!').count();
        urgency += exclamations as f32 * 0.1;
        
        urgency.min(1.0)
    }
    
    /// Calculate politeness level
    fn calculate_politeness(&self, query: &str) -> f32 {
        let mut politeness: f32 = 0.3; // baseline
        
        for marker in &self.politeness_markers {
            if query.contains(marker) {
                politeness += 0.2;
            }
        }
        
        politeness.min(1.0)
    }
    
    /// Check if query is negated
    fn is_negated(&self, query: &str) -> bool {
        for negation in &self.negations {
            if query.contains(negation) {
                return true;
            }
        }
        false
    }
    
    /// Check if query is conditional
    fn is_conditional(&self, query: &str) -> bool {
        for conditional in &self.conditionals {
            if query.contains(conditional) {
                return true;
            }
        }
        false
    }
    
    /// Extract temporal references
    fn extract_temporal(&self, query: &str, words: &[&str]) -> Option<TemporalReference> {
        // Check for specific time
        if query.contains(':') || query.contains("am") || query.contains("pm") {
            return Some(TemporalReference {
                ref_type: TemporalType::Specific,
                specific: self.extract_time_string(query),
                relative: None,
            });
        }
        
        // Check for relative time
        if query.contains(" in ") {
            if let Some(pos) = words.iter().position(|w| *w == "in") {
                let relative = self.parse_relative_time(&words[pos..], TimeDirection::Future);
                if relative.is_some() {
                    return Some(TemporalReference {
                        ref_type: TemporalType::Relative,
                        specific: None,
                        relative,
                    });
                }
            }
        }
        
        // Check for duration
        if query.contains(" for ") {
            return Some(TemporalReference {
                ref_type: TemporalType::Duration,
                specific: None,
                relative: self.parse_relative_time(words, TimeDirection::Future),
            });
        }
        
        // Check for deadline
        if query.contains(" by ") {
            return Some(TemporalReference {
                ref_type: TemporalType::Deadline,
                specific: self.extract_deadline(query),
                relative: None,
            });
        }
        
        // Check for recurring
        if query.contains("every") || query.contains("daily") || query.contains("weekly") {
            return Some(TemporalReference {
                ref_type: TemporalType::Recurring,
                specific: Some(self.extract_recurring(query)),
                relative: None,
            });
        }
        
        None
    }
    
    /// Extract time string from query
    fn extract_time_string(&self, query: &str) -> Option<String> {
        let words: Vec<&str> = query.split_whitespace().collect();
        
        for word in &words {
            // Check for HH:MM format
            if word.contains(':') {
                let parts: Vec<&str> = word.split(':').collect();
                if parts.len() == 2 {
                    if parts[0].chars().all(|c| c.is_ascii_digit()) {
                        return Some(word.to_string());
                    }
                }
            }
            // Check for Xam/Xpm format
            if word.ends_with("am") || word.ends_with("pm") {
                return Some(word.to_string());
            }
        }
        
        // Check for number followed by am/pm
        for (i, word) in words.iter().enumerate() {
            if word.chars().all(|c| c.is_ascii_digit()) {
                if i + 1 < words.len() {
                    let next = words[i + 1];
                    if next == "am" || next == "pm" {
                        return Some(format!("{} {}", word, next));
                    }
                }
            }
        }
        
        None
    }
    
    /// Parse relative time from words
    fn parse_relative_time(&self, words: &[&str], direction: TimeDirection) -> Option<RelativeTime> {
        let time_units = ["second", "seconds", "minute", "minutes", "min", "mins", 
                         "hour", "hours", "hr", "hrs", "day", "days", "week", "weeks"];
        
        for (i, word) in words.iter().enumerate() {
            // Found a number
            if word.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(amount) = word.parse::<u32>() {
                    // Check next word for unit
                    if i + 1 < words.len() && time_units.contains(&words[i + 1]) {
                        return Some(RelativeTime {
                            direction,
                            amount: Some(amount),
                            unit: Some(words[i + 1].to_string()),
                        });
                    }
                }
            }
        }
        
        None
    }
    
    /// Extract deadline reference
    fn extract_deadline(&self, query: &str) -> Option<String> {
        let deadlines = ["tomorrow", "tonight", "end of day", "eod", "morning", "afternoon", "evening"];
        
        for deadline in &deadlines {
            if query.contains(deadline) {
                return Some(deadline.to_string());
            }
        }
        
        None
    }
    
    /// Extract recurring pattern
    fn extract_recurring(&self, query: &str) -> String {
        if query.contains("every day") || query.contains("daily") {
            return "daily".to_string();
        }
        if query.contains("every week") || query.contains("weekly") {
            return "weekly".to_string();
        }
        if query.contains("every month") || query.contains("monthly") {
            return "monthly".to_string();
        }
        if query.contains("every hour") || query.contains("hourly") {
            return "hourly".to_string();
        }
        
        "custom".to_string()
    }
    
    /// Infer context from query
    fn infer_context(&self, query: &str, context: &AiContext) -> Vec<ImpliedContext> {
        let mut implied = Vec::new();
        
        // Infer "home" location
        if query.contains("home") && !query.contains("at home") {
            implied.push(ImpliedContext {
                context_type: "destination".to_string(),
                value: "home".to_string(),
                confidence: 0.9,
            });
        }
        
        // Infer "work" location
        if query.contains("work") && !query.contains("at work") {
            implied.push(ImpliedContext {
                context_type: "destination".to_string(),
                value: "work".to_string(),
                confidence: 0.9,
            });
        }
        
        // Infer current activity context
        if context.environment.is_moving {
            implied.push(ImpliedContext {
                context_type: "activity".to_string(),
                value: "moving".to_string(),
                confidence: 0.8,
            });
        }
        
        // Infer time-based suggestions
        match &context.time_of_day {
            TimeOfDay::Morning => {
                if query.contains("coffee") || query.contains("breakfast") {
                    implied.push(ImpliedContext {
                        context_type: "location_type".to_string(),
                        value: "cafe".to_string(),
                        confidence: 0.6,
                    });
                }
            }
            TimeOfDay::Evening | TimeOfDay::Night => {
                if query.contains("dinner") || query.contains("eat") {
                    implied.push(ImpliedContext {
                        context_type: "location_type".to_string(),
                        value: "restaurant".to_string(),
                        confidence: 0.6,
                    });
                }
            }
            _ => {}
        }
        
        implied
    }
    
    /// Perform semantic analysis
    fn semantic_analysis(&self, query: &str, words: &[&str]) -> SemanticAnalysis {
        let mut semantic = SemanticAnalysis::default();
        
        // Find main action
        let action_verbs = self.extract_action_words(words);
        if let Some(action) = action_verbs.first() {
            semantic.main_action = Some(action.clone());
            
            // Find object after action
            if let Some(pos) = words.iter().position(|w| *w == action.as_str()) {
                if pos + 1 < words.len() {
                    // Get remaining words as potential object
                    let object_words: Vec<&str> = words[pos + 1..].iter()
                        .take_while(|w| !["to", "at", "in", "on", "for", "with"].contains(w))
                        .cloned()
                        .collect();
                    
                    if !object_words.is_empty() {
                        semantic.object = Some(object_words.join(" "));
                    }
                }
            }
        }
        
        // Find indirect object (to X)
        if let Some(pos) = words.iter().position(|w| *w == "to") {
            if pos + 1 < words.len() {
                semantic.indirect_object = Some(words[pos + 1].to_string());
            }
        }
        
        // Find location (at X, in X)
        for prep in ["at", "in", "near"] {
            if let Some(pos) = words.iter().position(|w| *w == prep) {
                if pos + 1 < words.len() {
                    semantic.location = Some(words[pos + 1].to_string());
                    break;
                }
            }
        }
        
        // Find manner (how - with, by)
        for prep in ["with", "by", "using"] {
            if let Some(pos) = words.iter().position(|w| *w == prep) {
                if pos + 1 < words.len() {
                    semantic.manner = Some(words[pos + 1].to_string());
                    break;
                }
            }
        }
        
        // Infer subject
        if query.starts_with("i ") {
            semantic.subject = Some("user".to_string());
        } else {
            semantic.subject = Some("you".to_string()); // assistant
        }
        
        semantic
    }
    
    /// Check if query is a simple yes/no answer
    pub fn is_simple_response(&self, query: &str) -> Option<bool> {
        let normalized = self.normalize(query);
        let trimmed = normalized.trim();
        
        let positive = ["yes", "yeah", "yep", "yup", "sure", "ok", "okay", "alright", "absolutely", "definitely", "of course", "please", "do it", "go ahead"];
        let negative = ["no", "nope", "nah", "don't", "cancel", "never mind", "nevermind", "stop", "forget it"];
        
        if positive.contains(&trimmed) {
            return Some(true);
        }
        if negative.contains(&trimmed) {
            return Some(false);
        }
        
        None
    }
    
    /// Check if query is asking for help
    pub fn is_help_request(&self, query: &str) -> bool {
        let normalized = self.normalize(query);
        
        let help_patterns = [
            "help", "what can you do", "what do you do", 
            "how do i", "how to", "what are", "show me",
            "i need help", "assist", "guide",
        ];
        
        for pattern in &help_patterns {
            if normalized.contains(pattern) {
                return true;
            }
        }
        
        false
    }
    
    /// Extract numbers from query
    pub fn extract_numbers(&self, query: &str) -> Vec<f64> {
        let mut numbers = Vec::new();
        
        let mut current = String::new();
        for c in query.chars() {
            if c.is_ascii_digit() || c == '.' {
                current.push(c);
            } else if !current.is_empty() {
                if let Ok(n) = current.parse::<f64>() {
                    numbers.push(n);
                }
                current.clear();
            }
        }
        
        if !current.is_empty() {
            if let Ok(n) = current.parse::<f64>() {
                numbers.push(n);
            }
        }
        
        numbers
    }
    
    /// Get selection from numbered options
    pub fn get_selection(&self, query: &str, max_options: usize) -> Option<usize> {
        // Check for ordinal words
        let ordinals = ["first", "second", "third", "fourth", "fifth"];
        let normalized = self.normalize(query);
        
        for (i, ordinal) in ordinals.iter().enumerate() {
            if normalized.contains(ordinal) && i < max_options {
                return Some(i);
            }
        }
        
        // Check for numbers
        let numbers = self.extract_numbers(query);
        if let Some(n) = numbers.first() {
            let idx = *n as usize;
            if idx >= 1 && idx <= max_options {
                return Some(idx - 1); // Convert to 0-indexed
            }
        }
        
        None
    }
}

impl Default for QueryUnderstanding {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_understanding_creation() {
        let qu = QueryUnderstanding::new();
        assert!(!qu.urgency_indicators.is_empty());
    }
    
    #[test]
    fn test_query_type_detection() {
        let qu = QueryUnderstanding::new();
        let context = AiContext::default();
        
        let analysis = qu.analyze("call mom", &context);
        assert_eq!(analysis.query_type, QueryType::Command);
        
        let analysis = qu.analyze("what time is it?", &context);
        assert_eq!(analysis.query_type, QueryType::Question);
        
        let analysis = qu.analyze("yes", &context);
        assert_eq!(analysis.query_type, QueryType::Acknowledgment);
    }
    
    #[test]
    fn test_urgency_detection() {
        let qu = QueryUnderstanding::new();
        let context = AiContext::default();
        
        let analysis = qu.analyze("call mom now please!", &context);
        assert!(analysis.urgency > 0.0);
    }
    
    #[test]
    fn test_politeness_detection() {
        let qu = QueryUnderstanding::new();
        let context = AiContext::default();
        
        let analysis = qu.analyze("could you please call mom", &context);
        assert!(analysis.politeness > 0.5);
    }
    
    #[test]
    fn test_negation_detection() {
        let qu = QueryUnderstanding::new();
        let context = AiContext::default();
        
        let analysis = qu.analyze("don't call mom", &context);
        assert!(analysis.negated);
    }
    
    #[test]
    fn test_temporal_extraction() {
        let qu = QueryUnderstanding::new();
        let context = AiContext::default();
        
        let analysis = qu.analyze("remind me in 5 minutes", &context);
        assert!(analysis.temporal.is_some());
        
        let temporal = analysis.temporal.unwrap();
        assert_eq!(temporal.ref_type, TemporalType::Relative);
    }
    
    #[test]
    fn test_simple_response() {
        let qu = QueryUnderstanding::new();
        
        assert_eq!(qu.is_simple_response("yes"), Some(true));
        assert_eq!(qu.is_simple_response("no"), Some(false));
        assert_eq!(qu.is_simple_response("call mom"), None);
    }
    
    #[test]
    fn test_help_request() {
        let qu = QueryUnderstanding::new();
        
        assert!(qu.is_help_request("what can you do"));
        assert!(qu.is_help_request("help"));
        assert!(!qu.is_help_request("call mom"));
    }
    
    #[test]
    fn test_number_extraction() {
        let qu = QueryUnderstanding::new();
        
        let numbers = qu.extract_numbers("set timer for 5 minutes");
        assert_eq!(numbers, vec![5.0]);
        
        let numbers = qu.extract_numbers("call 555-1234");
        assert_eq!(numbers.len(), 2);
    }
    
    #[test]
    fn test_selection() {
        let qu = QueryUnderstanding::new();
        
        assert_eq!(qu.get_selection("the second one", 3), Some(1));
        assert_eq!(qu.get_selection("number 2", 3), Some(1));
        assert_eq!(qu.get_selection("1", 3), Some(0));
    }
    
    #[test]
    fn test_semantic_analysis() {
        let qu = QueryUnderstanding::new();
        let context = AiContext::default();
        
        let analysis = qu.analyze("send message to john", &context);
        
        assert_eq!(analysis.semantic.main_action, Some("send".to_string()));
        assert!(analysis.semantic.indirect_object.is_some());
    }
}
