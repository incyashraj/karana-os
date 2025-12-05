//! Natural Language Understanding Engine
//!
//! Provides text preprocessing, intent classification, and semantic understanding.

use super::*;
use std::collections::HashMap;

/// NLU Engine for natural language understanding
pub struct NluEngine {
    /// Confidence threshold
    threshold: f32,
    /// Stop words to filter
    stop_words: Vec<&'static str>,
    /// Keyword mappings for quick intent detection
    keyword_intents: HashMap<&'static str, &'static str>,
    /// Pattern matchers
    patterns: Vec<IntentPattern>,
    /// Semantic similarity cache
    similarity_cache: HashMap<String, Vec<(String, f32)>>,
}

impl NluEngine {
    pub fn new(threshold: f32) -> Self {
        let mut engine = Self {
            threshold,
            stop_words: vec![
                "a", "an", "the", "is", "are", "was", "were", "be", "been", "being",
                "have", "has", "had", "do", "does", "did", "will", "would", "could",
                "should", "may", "might", "must", "shall", "can", "need", "dare",
                "to", "of", "in", "for", "on", "with", "at", "by", "from", "up",
                "about", "into", "through", "during", "before", "after", "above",
                "below", "between", "under", "again", "further", "then", "once",
                "here", "there", "when", "where", "why", "how", "all", "each",
                "few", "more", "most", "other", "some", "such", "no", "nor", "not",
                "only", "own", "same", "so", "than", "too", "very", "just", "also",
                "please", "thanks", "thank", "hey", "hi", "hello", "okay", "ok",
            ],
            keyword_intents: HashMap::new(),
            patterns: Vec::new(),
            similarity_cache: HashMap::new(),
        };
        engine.initialize_keywords();
        engine.initialize_patterns();
        engine
    }
    
    fn initialize_keywords(&mut self) {
        // Navigation
        self.keyword_intents.insert("navigate", "navigation");
        self.keyword_intents.insert("directions", "navigation");
        self.keyword_intents.insert("route", "navigation");
        self.keyword_intents.insert("map", "navigation");
        self.keyword_intents.insert("go to", "navigation");
        self.keyword_intents.insert("take me", "navigation");
        self.keyword_intents.insert("how do i get", "navigation");
        
        // Communication
        self.keyword_intents.insert("call", "call");
        self.keyword_intents.insert("phone", "call");
        self.keyword_intents.insert("dial", "call");
        self.keyword_intents.insert("message", "message");
        self.keyword_intents.insert("text", "message");
        self.keyword_intents.insert("send message", "message");
        self.keyword_intents.insert("email", "email");
        self.keyword_intents.insert("mail", "email");
        
        // Media
        self.keyword_intents.insert("play", "play_media");
        self.keyword_intents.insert("music", "play_media");
        self.keyword_intents.insert("song", "play_media");
        self.keyword_intents.insert("podcast", "play_media");
        self.keyword_intents.insert("pause", "pause_media");
        self.keyword_intents.insert("stop", "stop_media");
        self.keyword_intents.insert("skip", "skip_media");
        self.keyword_intents.insert("next", "next_media");
        self.keyword_intents.insert("previous", "previous_media");
        self.keyword_intents.insert("volume", "volume");
        
        // Camera
        self.keyword_intents.insert("photo", "take_photo");
        self.keyword_intents.insert("picture", "take_photo");
        self.keyword_intents.insert("camera", "take_photo");
        self.keyword_intents.insert("capture", "take_photo");
        self.keyword_intents.insert("record", "record_video");
        self.keyword_intents.insert("video", "record_video");
        
        // Apps
        self.keyword_intents.insert("open", "launch_app");
        self.keyword_intents.insert("launch", "launch_app");
        self.keyword_intents.insert("start", "launch_app");
        self.keyword_intents.insert("close", "close_app");
        self.keyword_intents.insert("quit", "close_app");
        self.keyword_intents.insert("exit", "close_app");
        
        // Timer/Reminder
        self.keyword_intents.insert("timer", "set_timer");
        self.keyword_intents.insert("alarm", "set_alarm");
        self.keyword_intents.insert("remind", "set_reminder");
        self.keyword_intents.insert("reminder", "set_reminder");
        self.keyword_intents.insert("schedule", "schedule");
        self.keyword_intents.insert("calendar", "calendar");
        
        // Settings
        self.keyword_intents.insert("brightness", "adjust_brightness");
        self.keyword_intents.insert("dim", "adjust_brightness");
        self.keyword_intents.insert("brighter", "adjust_brightness");
        self.keyword_intents.insert("wifi", "wifi_settings");
        self.keyword_intents.insert("bluetooth", "bluetooth_settings");
        self.keyword_intents.insert("settings", "open_settings");
        
        // Wallet/Blockchain
        self.keyword_intents.insert("balance", "check_balance");
        self.keyword_intents.insert("wallet", "open_wallet");
        self.keyword_intents.insert("transfer", "transfer");
        self.keyword_intents.insert("send tokens", "transfer");
        self.keyword_intents.insert("stake", "stake");
        self.keyword_intents.insert("staking", "stake");
        
        // Search/Query
        self.keyword_intents.insert("search", "search");
        self.keyword_intents.insert("find", "search");
        self.keyword_intents.insert("look up", "search");
        self.keyword_intents.insert("what is", "question");
        self.keyword_intents.insert("who is", "question");
        self.keyword_intents.insert("where is", "question");
        self.keyword_intents.insert("when is", "question");
        self.keyword_intents.insert("how to", "question");
        self.keyword_intents.insert("why", "question");
        
        // Vision/AR
        self.keyword_intents.insert("identify", "identify_object");
        self.keyword_intents.insert("what am i looking at", "identify_object");
        self.keyword_intents.insert("scan", "scan");
        self.keyword_intents.insert("translate", "translate");
        self.keyword_intents.insert("read", "read_text");
        
        // Notifications
        self.keyword_intents.insert("notifications", "show_notifications");
        self.keyword_intents.insert("alerts", "show_notifications");
        self.keyword_intents.insert("do not disturb", "dnd_mode");
        self.keyword_intents.insert("silent", "silent_mode");
        self.keyword_intents.insert("mute", "mute");
    }
    
    fn initialize_patterns(&mut self) {
        // Time patterns
        self.patterns.push(IntentPattern {
            pattern_type: PatternType::Time,
            keywords: vec!["in", "minutes", "hours", "seconds", "at", "o'clock", "am", "pm"],
            intent: "time_related".to_string(),
        });
        
        // Contact patterns
        self.patterns.push(IntentPattern {
            pattern_type: PatternType::Contact,
            keywords: vec!["to", "from", "with"],
            intent: "contact_related".to_string(),
        });
        
        // Location patterns
        self.patterns.push(IntentPattern {
            pattern_type: PatternType::Location,
            keywords: vec!["to", "from", "at", "near", "around", "nearby"],
            intent: "location_related".to_string(),
        });
        
        // Quantity patterns
        self.patterns.push(IntentPattern {
            pattern_type: PatternType::Quantity,
            keywords: vec!["times", "percent", "degrees", "steps"],
            intent: "quantity_related".to_string(),
        });
    }
    
    /// Preprocess and normalize text
    pub fn preprocess(&self, input: &str) -> String {
        let mut text = input.to_lowercase();
        
        // Remove extra whitespace
        text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        
        // Handle contractions
        text = text.replace("i'm", "i am");
        text = text.replace("i'll", "i will");
        text = text.replace("i've", "i have");
        text = text.replace("i'd", "i would");
        text = text.replace("don't", "do not");
        text = text.replace("doesn't", "does not");
        text = text.replace("didn't", "did not");
        text = text.replace("won't", "will not");
        text = text.replace("wouldn't", "would not");
        text = text.replace("couldn't", "could not");
        text = text.replace("shouldn't", "should not");
        text = text.replace("can't", "cannot");
        text = text.replace("let's", "let us");
        text = text.replace("what's", "what is");
        text = text.replace("where's", "where is");
        text = text.replace("who's", "who is");
        text = text.replace("it's", "it is");
        text = text.replace("that's", "that is");
        text = text.replace("there's", "there is");
        
        // Remove punctuation except for specific cases
        text = text.chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '\'' || *c == '-' || *c == '@' || *c == '.')
            .collect();
        
        text
    }
    
    /// Main understanding function
    pub fn understand(&self, text: &str, _entities: &[ExtractedEntity]) -> NluResult {
        let tokens = self.tokenize(text);
        
        // Try keyword-based intent matching first
        if let Some((intent, confidence)) = self.match_keywords(&tokens) {
            let sentiment = self.analyze_sentiment(&tokens);
            return NluResult {
                primary_intent: Some(intent.to_string()),
                confidence,
                secondary_intents: vec![],
                tokens,
                sentiment,
                question_type: self.detect_question_type(text),
                requires_clarification: confidence < self.threshold,
            };
        }
        
        // Try pattern matching
        let pattern_matches = self.match_patterns(&tokens);
        
        // Combine results
        let primary_intent = pattern_matches.first().map(|(i, _)| i.clone());
        let confidence = pattern_matches.first().map(|(_, c)| *c).unwrap_or(0.0);
        let sentiment = self.analyze_sentiment(&tokens);
        
        NluResult {
            primary_intent,
            confidence,
            secondary_intents: pattern_matches.iter().skip(1).map(|(i, _)| i.clone()).collect(),
            tokens,
            sentiment,
            question_type: self.detect_question_type(text),
            requires_clarification: confidence < self.threshold,
        }
    }
    
    /// Tokenize text
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }
    
    /// Match keywords to intents
    fn match_keywords(&self, tokens: &[String]) -> Option<(&str, f32)> {
        let text = tokens.join(" ");
        let mut best_match: Option<(&str, f32, usize)> = None;
        
        for (keyword, intent) in &self.keyword_intents {
            if text.contains(keyword) {
                let keyword_len = keyword.split_whitespace().count();
                let confidence = 0.7 + (keyword_len as f32 * 0.1).min(0.25);
                
                match &best_match {
                    None => best_match = Some((intent, confidence, keyword_len)),
                    Some((_, _, len)) if keyword_len > *len => {
                        best_match = Some((intent, confidence, keyword_len));
                    }
                    _ => {}
                }
            }
        }
        
        best_match.map(|(intent, conf, _)| (intent, conf))
    }
    
    /// Match patterns in text
    fn match_patterns(&self, tokens: &[String]) -> Vec<(String, f32)> {
        let mut matches = Vec::new();
        
        for pattern in &self.patterns {
            let match_count = pattern.keywords.iter()
                .filter(|kw| tokens.iter().any(|t| t.contains(*kw)))
                .count();
            
            if match_count > 0 {
                let confidence = (match_count as f32 / pattern.keywords.len() as f32) * 0.6;
                matches.push((pattern.intent.clone(), confidence));
            }
        }
        
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        matches
    }
    
    /// Analyze sentiment
    fn analyze_sentiment(&self, tokens: &[String]) -> Sentiment {
        let positive_words = ["great", "good", "nice", "awesome", "excellent", "amazing", 
                             "wonderful", "fantastic", "love", "like", "happy", "pleased"];
        let negative_words = ["bad", "terrible", "awful", "horrible", "hate", "dislike",
                             "angry", "annoyed", "frustrated", "wrong", "error", "problem"];
        
        let mut positive_count = 0;
        let mut negative_count = 0;
        
        for token in tokens {
            if positive_words.contains(&token.as_str()) {
                positive_count += 1;
            }
            if negative_words.contains(&token.as_str()) {
                negative_count += 1;
            }
        }
        
        if positive_count > negative_count {
            Sentiment::Positive
        } else if negative_count > positive_count {
            Sentiment::Negative
        } else {
            Sentiment::Neutral
        }
    }
    
    /// Detect question type
    fn detect_question_type(&self, text: &str) -> Option<QuestionType> {
        let text_lower = text.to_lowercase();
        
        if text_lower.starts_with("what") || text_lower.contains("what is") || text_lower.contains("what are") {
            Some(QuestionType::What)
        } else if text_lower.starts_with("who") || text_lower.contains("who is") {
            Some(QuestionType::Who)
        } else if text_lower.starts_with("where") || text_lower.contains("where is") {
            Some(QuestionType::Where)
        } else if text_lower.starts_with("when") || text_lower.contains("when is") {
            Some(QuestionType::When)
        } else if text_lower.starts_with("why") {
            Some(QuestionType::Why)
        } else if text_lower.starts_with("how") {
            if text_lower.contains("how much") || text_lower.contains("how many") {
                Some(QuestionType::HowMuch)
            } else {
                Some(QuestionType::How)
            }
        } else if text_lower.starts_with("can") || text_lower.starts_with("could") || 
                  text_lower.starts_with("would") || text_lower.starts_with("will") {
            Some(QuestionType::YesNo)
        } else if text.contains("?") {
            Some(QuestionType::General)
        } else {
            None
        }
    }
}

impl Default for NluEngine {
    fn default() -> Self {
        Self::new(0.6)
    }
}

/// NLU result
#[derive(Debug, Clone)]
pub struct NluResult {
    pub primary_intent: Option<String>,
    pub confidence: f32,
    pub secondary_intents: Vec<String>,
    pub tokens: Vec<String>,
    pub sentiment: Sentiment,
    pub question_type: Option<QuestionType>,
    pub requires_clarification: bool,
}

/// Sentiment enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sentiment {
    Positive,
    Neutral,
    Negative,
}

/// Question types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuestionType {
    What,
    Who,
    Where,
    When,
    Why,
    How,
    HowMuch,
    YesNo,
    General,
}

/// Intent pattern for matching
#[derive(Debug, Clone)]
struct IntentPattern {
    pattern_type: PatternType,
    keywords: Vec<&'static str>,
    intent: String,
}

/// Pattern types
#[derive(Debug, Clone, Copy, PartialEq)]
enum PatternType {
    Time,
    Contact,
    Location,
    Quantity,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nlu_preprocess() {
        let nlu = NluEngine::new(0.6);
        
        assert_eq!(nlu.preprocess("I'm going"), "i am going");
        assert_eq!(nlu.preprocess("don't   stop"), "do not stop");
    }
    
    #[test]
    fn test_keyword_matching() {
        let nlu = NluEngine::new(0.6);
        let tokens = vec!["navigate".to_string(), "to".to_string(), "store".to_string()];
        
        let result = nlu.match_keywords(&tokens);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, "navigation");
    }
    
    #[test]
    fn test_nlu_understand() {
        let nlu = NluEngine::new(0.6);
        let result = nlu.understand("call mom", &[]);
        
        assert!(result.primary_intent.is_some());
        assert_eq!(result.primary_intent.unwrap(), "call");
    }
    
    #[test]
    fn test_question_detection() {
        let nlu = NluEngine::new(0.6);
        
        assert_eq!(nlu.detect_question_type("what is the weather"), Some(QuestionType::What));
        assert_eq!(nlu.detect_question_type("where is the store"), Some(QuestionType::Where));
        assert_eq!(nlu.detect_question_type("how do I get there"), Some(QuestionType::How));
    }
    
    #[test]
    fn test_sentiment_analysis() {
        let nlu = NluEngine::new(0.6);
        
        let positive = nlu.analyze_sentiment(&["great".to_string(), "job".to_string()]);
        assert_eq!(positive, Sentiment::Positive);
        
        let negative = nlu.analyze_sentiment(&["terrible".to_string(), "weather".to_string()]);
        assert_eq!(negative, Sentiment::Negative);
    }
}
