//! Intent parsing types and utilities

use serde::{Serialize, Deserialize};

/// Intent categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IntentCategory {
    Blockchain,
    ArApps,
    Vision,
    Navigation,
    System,
    Reminder,
    Conversation,
}

/// Entities extracted from user input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntities {
    pub amounts: Vec<u64>,
    pub recipients: Vec<String>,
    pub urls: Vec<String>,
    pub times: Vec<String>,
    pub locations: Vec<String>,
    pub app_names: Vec<String>,
}

impl Default for ExtractedEntities {
    fn default() -> Self {
        Self {
            amounts: Vec::new(),
            recipients: Vec::new(),
            urls: Vec::new(),
            times: Vec::new(),
            locations: Vec::new(),
            app_names: Vec::new(),
        }
    }
}

impl ExtractedEntities {
    /// Extract all entities from text
    pub fn extract(text: &str) -> Self {
        let mut entities = Self::default();
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        // Extract amounts (numbers)
        for word in &words {
            if let Ok(num) = word.parse::<u64>() {
                entities.amounts.push(num);
            }
        }
        
        // Extract URLs
        for word in &words {
            if word.contains("http") || word.contains("www.") || 
               word.contains(".com") || word.contains(".org") || word.contains(".io") {
                entities.urls.push(word.to_string());
            }
        }
        
        // Extract recipients (after "to")
        if let Some(idx) = text_lower.find(" to ") {
            let after = &text[idx + 4..];
            if let Some(name) = after.split_whitespace().next() {
                let clean_name = name.trim_matches(|c: char| !c.is_alphanumeric());
                if !clean_name.is_empty() && !["the", "a", "an"].contains(&clean_name.to_lowercase().as_str()) {
                    entities.recipients.push(clean_name.to_string());
                }
            }
        }
        
        // Extract times (minutes, hours, etc.)
        for (i, word) in words.iter().enumerate() {
            let w = word.to_lowercase();
            if w.contains("min") || w.contains("hour") || w.contains("sec") ||
               w.contains("day") || w.contains("week") {
                // Check if there's a number before
                if i > 0 {
                    if let Ok(num) = words[i - 1].parse::<u32>() {
                        entities.times.push(format!("{} {}", num, word));
                    }
                } else {
                    entities.times.push(word.to_string());
                }
            }
        }
        
        // Extract app names
        let known_apps = ["browser", "video", "youtube", "notes", "terminal", 
                          "music", "spotify", "calendar", "mail", "email"];
        for app in known_apps {
            if text_lower.contains(app) {
                entities.app_names.push(app.to_string());
            }
        }
        
        entities
    }
}

/// Confidence scoring for intent matching
#[derive(Debug, Clone)]
pub struct IntentScore {
    pub intent_name: String,
    pub score: f32,
    pub matched_keywords: Vec<String>,
}

impl IntentScore {
    pub fn new(intent_name: impl Into<String>) -> Self {
        Self {
            intent_name: intent_name.into(),
            score: 0.0,
            matched_keywords: Vec::new(),
        }
    }
    
    pub fn add_keyword_match(&mut self, keyword: &str, weight: f32) {
        self.matched_keywords.push(keyword.to_string());
        self.score += weight;
    }
    
    pub fn normalize(&mut self, max_possible_score: f32) {
        if max_possible_score > 0.0 {
            self.score = (self.score / max_possible_score).min(1.0);
        }
    }
}
