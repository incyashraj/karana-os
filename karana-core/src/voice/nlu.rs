//! Natural Language Understanding for Voice
//!
//! Intent recognition, entity extraction, and semantic parsing.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// NLU Engine for processing voice input
pub struct NluEngine {
    /// Intent classifiers
    classifiers: Vec<IntentClassifier>,
    /// Entity extractors
    extractors: Vec<EntityExtractor>,
    /// Semantic parser
    parser: SemanticParser,
    /// Context tracker
    context: ConversationContext,
    /// Configuration
    config: NluConfig,
}

impl NluEngine {
    /// Create new NLU engine
    pub fn new(config: NluConfig) -> Self {
        Self {
            classifiers: Self::default_classifiers(),
            extractors: Self::default_extractors(),
            parser: SemanticParser::new(),
            context: ConversationContext::new(),
            config,
        }
    }

    /// Process text and extract meaning
    pub fn process(&mut self, text: &str) -> NluResult {
        // Preprocess text
        let normalized = self.preprocess(text);

        // Classify intent
        let intent = self.classify_intent(&normalized);

        // Extract entities
        let entities = self.extract_entities(&normalized);

        // Parse semantics
        let semantics = self.parser.parse(&normalized, &intent, &entities);

        // Update context
        self.context.update(&intent, &entities);

        NluResult {
            text: text.to_string(),
            normalized,
            intent,
            entities,
            semantics,
            context: self.context.clone(),
        }
    }

    /// Preprocess text
    fn preprocess(&self, text: &str) -> String {
        let mut result = text.to_lowercase();

        // Handle contractions
        result = result.replace("don't", "do not");
        result = result.replace("can't", "cannot");
        result = result.replace("won't", "will not");
        result = result.replace("i'm", "i am");
        result = result.replace("it's", "it is");
        result = result.replace("what's", "what is");
        result = result.replace("that's", "that is");

        // Remove filler words
        let fillers = ["um", "uh", "like", "you know", "basically"];
        for filler in fillers {
            result = result.replace(filler, "");
        }

        // Normalize whitespace
        result = result.split_whitespace().collect::<Vec<_>>().join(" ");

        result
    }

    /// Classify intent
    fn classify_intent(&self, text: &str) -> ClassifiedIntent {
        let mut best_intent = ClassifiedIntent {
            intent: Intent::Unknown,
            confidence: 0.0,
            sub_intent: None,
        };

        for classifier in &self.classifiers {
            let result = classifier.classify(text);
            if result.confidence > best_intent.confidence {
                best_intent = result;
            }
        }

        // Apply context boosting
        if let Some(expected) = &self.context.expected_intent {
            if best_intent.intent == *expected && best_intent.confidence > 0.3 {
                best_intent.confidence = (best_intent.confidence + 0.2).min(1.0);
            }
        }

        best_intent
    }

    /// Extract entities
    fn extract_entities(&self, text: &str) -> Vec<ExtractedEntity> {
        let mut entities = Vec::new();

        for extractor in &self.extractors {
            entities.extend(extractor.extract(text));
        }

        // Deduplicate overlapping entities
        self.dedupe_entities(&mut entities);

        entities
    }

    /// Deduplicate overlapping entities
    fn dedupe_entities(&self, entities: &mut Vec<ExtractedEntity>) {
        entities.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut keep = vec![true; entities.len()];
        for i in 0..entities.len() {
            if !keep[i] { continue; }
            for j in (i + 1)..entities.len() {
                if !keep[j] { continue; }
                // Check overlap
                if entities[i].span.0 < entities[j].span.1 &&
                   entities[j].span.0 < entities[i].span.1 {
                    keep[j] = false;
                }
            }
        }

        let mut i = 0;
        entities.retain(|_| {
            let result = keep[i];
            i += 1;
            result
        });
    }

    /// Default intent classifiers
    fn default_classifiers() -> Vec<IntentClassifier> {
        vec![
            IntentClassifier::pattern_based("navigation", vec![
                ("go to *", Intent::Navigate),
                ("open *", Intent::Open),
                ("show *", Intent::Show),
                ("switch to *", Intent::Switch),
                ("navigate to *", Intent::Navigate),
                ("take me to *", Intent::Navigate),
            ]),
            IntentClassifier::pattern_based("control", vec![
                ("play *", Intent::Play),
                ("pause", Intent::Pause),
                ("stop", Intent::Stop),
                ("next", Intent::Next),
                ("previous", Intent::Previous),
                ("volume *", Intent::Volume),
                ("mute", Intent::Mute),
                ("unmute", Intent::Unmute),
            ]),
            IntentClassifier::pattern_based("query", vec![
                ("what is *", Intent::Query),
                ("who is *", Intent::Query),
                ("where is *", Intent::Query),
                ("when is *", Intent::Query),
                ("how *", Intent::Query),
                ("search for *", Intent::Search),
                ("find *", Intent::Search),
                ("look up *", Intent::Search),
            ]),
            IntentClassifier::pattern_based("communication", vec![
                ("call *", Intent::Call),
                ("message *", Intent::Message),
                ("send * to *", Intent::Send),
                ("reply *", Intent::Reply),
                ("text *", Intent::Message),
                ("email *", Intent::Email),
            ]),
            IntentClassifier::pattern_based("system", vec![
                ("turn on *", Intent::TurnOn),
                ("turn off *", Intent::TurnOff),
                ("enable *", Intent::Enable),
                ("disable *", Intent::Disable),
                ("settings", Intent::Settings),
                ("help", Intent::Help),
                ("cancel", Intent::Cancel),
                ("undo", Intent::Undo),
            ]),
            IntentClassifier::pattern_based("confirmation", vec![
                ("yes", Intent::Confirm),
                ("no", Intent::Deny),
                ("okay", Intent::Confirm),
                ("sure", Intent::Confirm),
                ("cancel", Intent::Cancel),
                ("confirm", Intent::Confirm),
                ("accept", Intent::Confirm),
                ("reject", Intent::Deny),
            ]),
        ]
    }

    /// Default entity extractors
    fn default_extractors() -> Vec<EntityExtractor> {
        vec![
            EntityExtractor::regex("time", r"\b(\d{1,2}:\d{2}(?:\s*[ap]m)?)\b"),
            EntityExtractor::regex("date", r"\b(\d{1,2}/\d{1,2}(?:/\d{2,4})?)\b"),
            EntityExtractor::regex("number", r"\b(\d+(?:\.\d+)?)\b"),
            EntityExtractor::keyword("app", vec![
                "camera", "photos", "messages", "phone", "browser",
                "settings", "calendar", "maps", "music", "notes",
            ]),
            EntityExtractor::keyword("direction", vec![
                "up", "down", "left", "right", "forward", "back",
                "top", "bottom", "north", "south", "east", "west",
            ]),
            EntityExtractor::keyword("action", vec![
                "open", "close", "start", "stop", "create", "delete",
                "save", "load", "send", "receive", "move", "copy",
            ]),
        ]
    }
}

impl Default for NluEngine {
    fn default() -> Self {
        Self::new(NluConfig::default())
    }
}

/// NLU configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NluConfig {
    /// Minimum confidence threshold
    pub min_confidence: f32,
    /// Enable context tracking
    pub use_context: bool,
    /// Context window size
    pub context_window: usize,
    /// Language
    pub language: String,
}

impl Default for NluConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            use_context: true,
            context_window: 5,
            language: "en".to_string(),
        }
    }
}

/// NLU processing result
#[derive(Debug, Clone)]
pub struct NluResult {
    /// Original text
    pub text: String,
    /// Normalized text
    pub normalized: String,
    /// Classified intent
    pub intent: ClassifiedIntent,
    /// Extracted entities
    pub entities: Vec<ExtractedEntity>,
    /// Semantic representation
    pub semantics: SemanticFrame,
    /// Conversation context
    pub context: ConversationContext,
}

/// Classified intent
#[derive(Debug, Clone)]
pub struct ClassifiedIntent {
    /// Intent type
    pub intent: Intent,
    /// Confidence score
    pub confidence: f32,
    /// Sub-intent (if any)
    pub sub_intent: Option<String>,
}

/// Intent types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Intent {
    // Navigation
    Navigate,
    Open,
    Show,
    Switch,
    GoBack,
    GoHome,

    // Media control
    Play,
    Pause,
    Stop,
    Next,
    Previous,
    Volume,
    Mute,
    Unmute,

    // Queries
    Query,
    Search,

    // Communication
    Call,
    Message,
    Send,
    Reply,
    Email,

    // System
    TurnOn,
    TurnOff,
    Enable,
    Disable,
    Settings,
    Help,

    // Confirmation
    Confirm,
    Deny,
    Cancel,
    Undo,

    // Unknown
    Unknown,
}

/// Extracted entity
#[derive(Debug, Clone)]
pub struct ExtractedEntity {
    /// Entity type
    pub entity_type: String,
    /// Entity value
    pub value: String,
    /// Span in text (start, end)
    pub span: (usize, usize),
    /// Confidence
    pub confidence: f32,
    /// Resolved value (if different)
    pub resolved: Option<String>,
}

/// Intent classifier
pub struct IntentClassifier {
    name: String,
    patterns: Vec<(String, Intent)>,
}

impl IntentClassifier {
    /// Create pattern-based classifier
    pub fn pattern_based(name: &str, patterns: Vec<(&str, Intent)>) -> Self {
        Self {
            name: name.to_string(),
            patterns: patterns.into_iter()
                .map(|(p, i)| (p.to_string(), i))
                .collect(),
        }
    }

    /// Classify text
    pub fn classify(&self, text: &str) -> ClassifiedIntent {
        let mut best_match = ClassifiedIntent {
            intent: Intent::Unknown,
            confidence: 0.0,
            sub_intent: None,
        };

        for (pattern, intent) in &self.patterns {
            if let Some(confidence) = self.match_pattern(text, pattern) {
                if confidence > best_match.confidence {
                    best_match = ClassifiedIntent {
                        intent: *intent,
                        confidence,
                        sub_intent: Some(self.name.clone()),
                    };
                }
            }
        }

        best_match
    }

    /// Match pattern against text
    fn match_pattern(&self, text: &str, pattern: &str) -> Option<f32> {
        let pattern_parts: Vec<&str> = pattern.split('*').collect();

        if pattern_parts.len() == 1 {
            // Exact match
            if text == pattern || text.contains(pattern) {
                return Some(0.9);
            }
        } else if pattern_parts.len() == 2 {
            // Wildcard match
            let prefix = pattern_parts[0].trim();
            let suffix = pattern_parts[1].trim();

            if !prefix.is_empty() && text.starts_with(prefix) {
                if suffix.is_empty() || text.ends_with(suffix) {
                    return Some(0.8);
                }
            }
        }

        None
    }
}

/// Entity extractor
pub struct EntityExtractor {
    name: String,
    extractor_type: ExtractorType,
}

enum ExtractorType {
    Regex(regex::Regex),
    Keywords(Vec<String>),
}

impl EntityExtractor {
    /// Create regex extractor
    pub fn regex(name: &str, pattern: &str) -> Self {
        Self {
            name: name.to_string(),
            extractor_type: ExtractorType::Regex(
                regex::Regex::new(pattern).unwrap_or_else(|_| regex::Regex::new(".").unwrap())
            ),
        }
    }

    /// Create keyword extractor
    pub fn keyword(name: &str, keywords: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            extractor_type: ExtractorType::Keywords(
                keywords.into_iter().map(String::from).collect()
            ),
        }
    }

    /// Extract entities from text
    pub fn extract(&self, text: &str) -> Vec<ExtractedEntity> {
        match &self.extractor_type {
            ExtractorType::Regex(re) => {
                re.find_iter(text).map(|m| ExtractedEntity {
                    entity_type: self.name.clone(),
                    value: m.as_str().to_string(),
                    span: (m.start(), m.end()),
                    confidence: 0.9,
                    resolved: None,
                }).collect()
            }
            ExtractorType::Keywords(keywords) => {
                let text_lower = text.to_lowercase();
                keywords.iter().filter_map(|kw| {
                    if let Some(pos) = text_lower.find(kw) {
                        Some(ExtractedEntity {
                            entity_type: self.name.clone(),
                            value: kw.clone(),
                            span: (pos, pos + kw.len()),
                            confidence: 0.85,
                            resolved: None,
                        })
                    } else {
                        None
                    }
                }).collect()
            }
        }
    }
}

/// Semantic parser
pub struct SemanticParser {
    frames: HashMap<Intent, FrameTemplate>,
}

impl SemanticParser {
    /// Create new parser
    pub fn new() -> Self {
        let mut frames = HashMap::new();

        // Define frame templates
        frames.insert(Intent::Navigate, FrameTemplate {
            slots: vec!["destination".to_string()],
            required: vec!["destination".to_string()],
        });
        frames.insert(Intent::Open, FrameTemplate {
            slots: vec!["target".to_string()],
            required: vec!["target".to_string()],
        });
        frames.insert(Intent::Call, FrameTemplate {
            slots: vec!["contact".to_string(), "method".to_string()],
            required: vec!["contact".to_string()],
        });
        frames.insert(Intent::Message, FrameTemplate {
            slots: vec!["recipient".to_string(), "content".to_string()],
            required: vec!["recipient".to_string()],
        });

        Self { frames }
    }

    /// Parse into semantic frame
    pub fn parse(&self, text: &str, intent: &ClassifiedIntent, entities: &[ExtractedEntity]) -> SemanticFrame {
        let mut slots = HashMap::new();

        // Fill slots from entities
        for entity in entities {
            slots.insert(entity.entity_type.clone(), entity.value.clone());
        }

        // Extract additional slots from text
        if let Some(template) = self.frames.get(&intent.intent) {
            for slot in &template.required {
                if !slots.contains_key(slot) {
                    // Try to extract from remaining text
                    if let Some(value) = self.extract_slot_value(text, slot, entities) {
                        slots.insert(slot.clone(), value);
                    }
                }
            }
        }

        let complete = self.is_complete(intent.intent, &slots);
        SemanticFrame {
            intent: intent.intent,
            slots,
            complete,
        }
    }

    /// Extract slot value from text
    fn extract_slot_value(&self, text: &str, slot: &str, entities: &[ExtractedEntity]) -> Option<String> {
        // Remove entity spans from consideration
        let mut mask = vec![true; text.len()];
        for entity in entities {
            for i in entity.span.0..entity.span.1.min(text.len()) {
                mask[i] = false;
            }
        }

        // Get remaining text after known patterns
        let words: Vec<&str> = text.split_whitespace().collect();
        let action_words = ["open", "go", "navigate", "show", "call", "message", "find"];

        for (i, word) in words.iter().enumerate() {
            if action_words.contains(&word.to_lowercase().as_str()) {
                if i + 1 < words.len() {
                    let remaining: String = words[i + 1..].join(" ");
                    if !remaining.is_empty() {
                        return Some(remaining);
                    }
                }
            }
        }

        None
    }

    /// Check if frame is complete
    fn is_complete(&self, intent: Intent, slots: &HashMap<String, String>) -> bool {
        if let Some(template) = self.frames.get(&intent) {
            template.required.iter().all(|r| slots.contains_key(r))
        } else {
            true // No template = always complete
        }
    }
}

impl Default for SemanticParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Frame template
struct FrameTemplate {
    slots: Vec<String>,
    required: Vec<String>,
}

/// Semantic frame
#[derive(Debug, Clone)]
pub struct SemanticFrame {
    /// Intent
    pub intent: Intent,
    /// Filled slots
    pub slots: HashMap<String, String>,
    /// Whether all required slots are filled
    pub complete: bool,
}

/// Conversation context
#[derive(Debug, Clone, Default)]
pub struct ConversationContext {
    /// Recent intents
    pub recent_intents: Vec<Intent>,
    /// Expected next intent
    pub expected_intent: Option<Intent>,
    /// Active topic
    pub topic: Option<String>,
    /// Pending slots to fill
    pub pending_slots: Vec<String>,
    /// Turn count
    pub turn_count: usize,
}

impl ConversationContext {
    /// Create new context
    pub fn new() -> Self {
        Self::default()
    }

    /// Update context
    pub fn update(&mut self, intent: &ClassifiedIntent, _entities: &[ExtractedEntity]) {
        self.turn_count += 1;

        // Track recent intents
        self.recent_intents.push(intent.intent);
        if self.recent_intents.len() > 5 {
            self.recent_intents.remove(0);
        }

        // Set expected intent for confirmations
        if matches!(intent.intent, Intent::Call | Intent::Message | Intent::Send) {
            self.expected_intent = Some(Intent::Confirm);
        } else {
            self.expected_intent = None;
        }
    }

    /// Clear context
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

// Minimal regex module for pattern matching
mod regex {
    pub struct Regex {
        pattern: String,
    }

    impl Regex {
        pub fn new(pattern: &str) -> Result<Self, ()> {
            Ok(Self { pattern: pattern.to_string() })
        }

        pub fn find_iter<'a>(&'a self, text: &'a str) -> FindIter<'a> {
            FindIter { text, pattern: &self.pattern, pos: 0 }
        }
    }

    pub struct FindIter<'a> {
        text: &'a str,
        pattern: &'a str,
        pos: usize,
    }

    impl<'a> Iterator for FindIter<'a> {
        type Item = Match<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            // Simple digit sequence matching for numbers
            if self.pattern.contains(r"\d") {
                let bytes = self.text.as_bytes();
                while self.pos < bytes.len() {
                    if bytes[self.pos].is_ascii_digit() {
                        let start = self.pos;
                        while self.pos < bytes.len() && 
                              (bytes[self.pos].is_ascii_digit() || 
                               bytes[self.pos] == b'.' || 
                               bytes[self.pos] == b':' ||
                               bytes[self.pos] == b'/') {
                            self.pos += 1;
                        }
                        return Some(Match {
                            text: &self.text[start..self.pos],
                            start,
                            end: self.pos,
                        });
                    }
                    self.pos += 1;
                }
            }
            None
        }
    }

    pub struct Match<'a> {
        text: &'a str,
        start: usize,
        end: usize,
    }

    impl<'a> Match<'a> {
        pub fn as_str(&self) -> &'a str {
            self.text
        }

        pub fn start(&self) -> usize {
            self.start
        }

        pub fn end(&self) -> usize {
            self.end
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nlu_engine_creation() {
        let engine = NluEngine::default();
        assert!(!engine.classifiers.is_empty());
    }

    #[test]
    fn test_preprocess() {
        let engine = NluEngine::default();
        let result = engine.preprocess("I'm   going   to the store");
        assert_eq!(result, "i am going to the store");
    }

    #[test]
    fn test_intent_classification() {
        let mut engine = NluEngine::default();
        let result = engine.process("open camera");
        assert_eq!(result.intent.intent, Intent::Open);
    }

    #[test]
    fn test_entity_extraction() {
        let mut engine = NluEngine::default();
        let result = engine.process("set timer for 5 minutes");
        assert!(!result.entities.is_empty());
    }

    #[test]
    fn test_conversation_context() {
        let mut context = ConversationContext::new();
        assert_eq!(context.turn_count, 0);
        
        let intent = ClassifiedIntent {
            intent: Intent::Call,
            confidence: 0.9,
            sub_intent: None,
        };
        context.update(&intent, &[]);
        
        assert_eq!(context.turn_count, 1);
        assert_eq!(context.expected_intent, Some(Intent::Confirm));
    }

    #[test]
    fn test_semantic_parser() {
        let parser = SemanticParser::new();
        let intent = ClassifiedIntent {
            intent: Intent::Open,
            confidence: 0.9,
            sub_intent: None,
        };
        let entities = vec![ExtractedEntity {
            entity_type: "app".to_string(),
            value: "camera".to_string(),
            span: (5, 11),
            confidence: 0.9,
            resolved: None,
        }];
        
        let frame = parser.parse("open camera", &intent, &entities);
        assert_eq!(frame.intent, Intent::Open);
    }
}
