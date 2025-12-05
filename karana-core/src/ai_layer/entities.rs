//! Entity Extraction System
//!
//! Extracts named entities, values, and slot fillers from user input.

use super::*;
use std::collections::HashMap;

/// Entity extractor
pub struct EntityExtractor {
    /// Named entity patterns
    patterns: Vec<EntityPattern>,
    /// Known entity lists
    known_entities: HashMap<String, Vec<String>>,
    /// Custom extractors
    extractors: Vec<Box<dyn EntityExtractorTrait>>,
}

impl EntityExtractor {
    pub fn new() -> Self {
        let mut extractor = Self {
            patterns: Vec::new(),
            known_entities: HashMap::new(),
            extractors: Vec::new(),
        };
        extractor.initialize_patterns();
        extractor.initialize_known_entities();
        extractor
    }
    
    fn initialize_patterns(&mut self) {
        // Time patterns
        self.patterns.push(EntityPattern {
            entity_type: EntityType::Time,
            patterns: vec![
                r"(\d{1,2}):(\d{2})\s*(am|pm)?".to_string(),
                r"(\d{1,2})\s*(am|pm)".to_string(),
                r"in\s+(\d+)\s+(minute|minutes|min|mins|hour|hours|hr|hrs|second|seconds|sec|secs)".to_string(),
                r"(\d+)\s+(minute|minutes|min|mins|hour|hours|hr|hrs|second|seconds|sec|secs)".to_string(),
                r"(morning|afternoon|evening|night|noon|midnight)".to_string(),
                r"(today|tomorrow|yesterday)".to_string(),
            ],
        });
        
        // Duration patterns
        self.patterns.push(EntityPattern {
            entity_type: EntityType::Duration,
            patterns: vec![
                r"for\s+(\d+)\s+(minute|minutes|min|mins|hour|hours|hr|hrs|second|seconds|sec|secs)".to_string(),
                r"(\d+)\s+(minute|minutes|min|mins|hour|hours|hr|hrs|second|seconds|sec|secs)\s+timer".to_string(),
            ],
        });
        
        // Number patterns
        self.patterns.push(EntityPattern {
            entity_type: EntityType::Number,
            patterns: vec![
                r"\b(\d+(?:\.\d+)?)\b".to_string(),
                r"\b(one|two|three|four|five|six|seven|eight|nine|ten)\b".to_string(),
                r"\b(hundred|thousand|million)\b".to_string(),
            ],
        });
        
        // Currency/Amount patterns
        self.patterns.push(EntityPattern {
            entity_type: EntityType::Amount,
            patterns: vec![
                r"\$(\d+(?:\.\d{2})?)".to_string(),
                r"(\d+(?:\.\d+)?)\s*(dollars|dollar|usd|tokens|eth|btc|sol)".to_string(),
                r"([\d,]+(?:\.\d+)?)\s*(tokens|coins)".to_string(),
            ],
        });
        
        // Email patterns
        self.patterns.push(EntityPattern {
            entity_type: EntityType::Email,
            patterns: vec![
                r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(),
            ],
        });
        
        // Phone patterns
        self.patterns.push(EntityPattern {
            entity_type: EntityType::Phone,
            patterns: vec![
                r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b".to_string(),
                r"\+\d{1,3}[-.]?\d{3}[-.]?\d{3}[-.]?\d{4}".to_string(),
            ],
        });
        
        // URL patterns
        self.patterns.push(EntityPattern {
            entity_type: EntityType::Url,
            patterns: vec![
                r"https?://[^\s]+".to_string(),
                r"www\.[^\s]+".to_string(),
            ],
        });
        
        // Location patterns (simple)
        self.patterns.push(EntityPattern {
            entity_type: EntityType::Location,
            patterns: vec![
                r"to\s+(?:the\s+)?([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)".to_string(),
                r"at\s+(?:the\s+)?([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)".to_string(),
                r"near\s+(?:the\s+)?([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)".to_string(),
            ],
        });
        
        // Percentage patterns
        self.patterns.push(EntityPattern {
            entity_type: EntityType::Percentage,
            patterns: vec![
                r"(\d+(?:\.\d+)?)\s*%".to_string(),
                r"(\d+(?:\.\d+)?)\s*percent".to_string(),
            ],
        });
    }
    
    fn initialize_known_entities(&mut self) {
        // Known apps
        self.known_entities.insert("app".to_string(), vec![
            "calendar".to_string(),
            "camera".to_string(),
            "music".to_string(),
            "photos".to_string(),
            "settings".to_string(),
            "wallet".to_string(),
            "navigation".to_string(),
            "browser".to_string(),
            "notes".to_string(),
            "weather".to_string(),
            "timer".to_string(),
            "clock".to_string(),
            "calculator".to_string(),
            "translator".to_string(),
        ]);
        
        // Known commands/directions
        self.known_entities.insert("direction".to_string(), vec![
            "up".to_string(),
            "down".to_string(),
            "higher".to_string(),
            "lower".to_string(),
            "louder".to_string(),
            "softer".to_string(),
            "quieter".to_string(),
            "brighter".to_string(),
            "dimmer".to_string(),
            "more".to_string(),
            "less".to_string(),
            "max".to_string(),
            "min".to_string(),
            "full".to_string(),
            "mute".to_string(),
        ]);
        
        // Known travel modes
        self.known_entities.insert("travel_mode".to_string(), vec![
            "walking".to_string(),
            "driving".to_string(),
            "transit".to_string(),
            "biking".to_string(),
            "cycling".to_string(),
        ]);
        
        // Known media types
        self.known_entities.insert("media_type".to_string(), vec![
            "music".to_string(),
            "song".to_string(),
            "playlist".to_string(),
            "album".to_string(),
            "podcast".to_string(),
            "audiobook".to_string(),
            "radio".to_string(),
        ]);
        
        // Known languages
        self.known_entities.insert("language".to_string(), vec![
            "english".to_string(),
            "spanish".to_string(),
            "french".to_string(),
            "german".to_string(),
            "italian".to_string(),
            "portuguese".to_string(),
            "chinese".to_string(),
            "japanese".to_string(),
            "korean".to_string(),
            "russian".to_string(),
            "arabic".to_string(),
            "hindi".to_string(),
        ]);
    }
    
    /// Extract entities from text
    pub fn extract(&self, text: &str, _context: &AiContext) -> Vec<ExtractedEntity> {
        let mut entities = Vec::new();
        let text_lower = text.to_lowercase();
        
        // Simple keyword-based extraction for time words
        for word in ["morning", "afternoon", "evening", "night", "noon", "midnight", "today", "tomorrow", "yesterday"] {
            if text_lower.contains(word) {
                if let Some(pos) = text_lower.find(word) {
                    entities.push(ExtractedEntity {
                        entity_type: EntityType::Time,
                        value: word.to_string(),
                        normalized_value: self.normalize_time(word),
                        confidence: 0.9,
                        start_pos: pos,
                        end_pos: pos + word.len(),
                        slot_name: Some("time".to_string()),
                    });
                }
            }
        }
        
        // Extract time patterns (e.g., "3:30 pm", "10:15 am")
        self.extract_time_patterns(text, &text_lower, &mut entities);
        
        // Extract numbers (simple digit sequences)
        let mut current_num = String::new();
        let mut num_start = 0;
        for (i, c) in text.chars().enumerate() {
            if c.is_ascii_digit() || (c == '.' && !current_num.is_empty()) {
                if current_num.is_empty() {
                    num_start = i;
                }
                current_num.push(c);
            } else if !current_num.is_empty() {
                entities.push(ExtractedEntity {
                    entity_type: EntityType::Number,
                    value: current_num.clone(),
                    normalized_value: current_num.clone(),
                    confidence: 0.85,
                    start_pos: num_start,
                    end_pos: i,
                    slot_name: None,
                });
                current_num.clear();
            }
        }
        if !current_num.is_empty() {
            entities.push(ExtractedEntity {
                entity_type: EntityType::Number,
                value: current_num.clone(),
                normalized_value: current_num,
                confidence: 0.85,
                start_pos: num_start,
                end_pos: text.len(),
                slot_name: None,
            });
        }
        
        // Check for duration keywords after numbers
        for dur in ["minute", "minutes", "min", "mins", "hour", "hours", "hr", "hrs", "second", "seconds", "sec", "secs"] {
            if text_lower.contains(dur) {
                // Check if there's a number before duration
                if let Some(pos) = text_lower.find(dur) {
                    let before = &text_lower[..pos].trim();
                    if let Some(last_word) = before.split_whitespace().last() {
                        if last_word.chars().all(|c| c.is_ascii_digit()) {
                            let dur_str = format!("{} {}", last_word, dur);
                            entities.push(ExtractedEntity {
                                entity_type: EntityType::Duration,
                                value: dur_str.clone(),
                                normalized_value: self.normalize_duration(&dur_str),
                                confidence: 0.9,
                                start_pos: pos.saturating_sub(last_word.len() + 1),
                                end_pos: pos + dur.len(),
                                slot_name: Some("duration".to_string()),
                            });
                        }
                    }
                }
            }
        }
        
        // Known entity extraction
        for (category, values) in &self.known_entities {
            for value in values {
                if text_lower.contains(value) {
                    let entity_type = match category.as_str() {
                        "app" => EntityType::App,
                        "direction" => EntityType::Direction,
                        "travel_mode" => EntityType::TravelMode,
                        "media_type" => EntityType::MediaType,
                        "language" => EntityType::Language,
                        _ => EntityType::Other,
                    };
                    
                    entities.push(ExtractedEntity {
                        entity_type,
                        value: value.clone(),
                        normalized_value: value.clone(),
                        confidence: 0.85,
                        start_pos: text_lower.find(value).unwrap_or(0),
                        end_pos: text_lower.find(value).unwrap_or(0) + value.len(),
                        slot_name: Some(category.clone()),
                    });
                }
            }
        }
        
        // Extract contact names (simple approach)
        entities.extend(self.extract_contacts_simple(text));
        
        // Deduplicate overlapping entities
        self.deduplicate_entities(&mut entities);
        
        entities
    }
    
    /// Extract contact names from text (simple approach without regex)
    fn extract_contacts_simple(&self, text: &str) -> Vec<ExtractedEntity> {
        let mut contacts = Vec::new();
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        // Look for contact patterns like "call X" or "message X"
        let trigger_words = ["call", "message", "text", "email"];
        
        for (i, word) in words.iter().enumerate() {
            if trigger_words.contains(&word.to_lowercase().as_str()) && i + 1 < words.len() {
                let name = words[i + 1];
                // Filter out common non-name words
                if !["the", "my", "a", "an", "me", "home", "work"].contains(&name.to_lowercase().as_str()) {
                    if let Some(pos) = text_lower.find(&name.to_lowercase()) {
                        contacts.push(ExtractedEntity {
                            entity_type: EntityType::Contact,
                            value: name.to_string(),
                            normalized_value: self.normalize_contact_name(name),
                            confidence: 0.7,
                            start_pos: pos,
                            end_pos: pos + name.len(),
                            slot_name: Some("contact".to_string()),
                        });
                    }
                }
            }
        }
        
        contacts
    }
    
    /// Extract time patterns like "3:30 pm", "10:15 am", "3pm"
    fn extract_time_patterns(&self, _text: &str, text_lower: &str, entities: &mut Vec<ExtractedEntity>) {
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            // Check for patterns with colon (e.g., "3:30")
            if word.contains(':') {
                let parts: Vec<&str> = word.split(':').collect();
                if parts.len() == 2 {
                    if let (Ok(hour), Ok(min)) = (parts[0].parse::<u32>(), parts[1].chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>()) {
                        if hour <= 23 && min <= 59 {
                            // Check if next word is am/pm
                            let mut time_str = word.to_string();
                            let mut end_word_str = word.to_string();
                            if i + 1 < words.len() {
                                let next = words[i + 1];
                                if next == "am" || next == "pm" || next == "a.m." || next == "p.m." {
                                    time_str = format!("{} {}", word, next);
                                    end_word_str = next.to_string();
                                }
                            }
                            // Check for am/pm attached to number
                            if word.ends_with("am") || word.ends_with("pm") {
                                time_str = word.to_string();
                            }
                            
                            if let Some(pos) = text_lower.find(*word) {
                                let end_pos = if let Some(epos) = text_lower.find(&end_word_str) {
                                    epos + end_word_str.len()
                                } else {
                                    pos + word.len()
                                };
                                
                                entities.push(ExtractedEntity {
                                    entity_type: EntityType::Time,
                                    value: time_str.clone(),
                                    normalized_value: self.normalize_clock_time(&time_str),
                                    confidence: 0.95,
                                    start_pos: pos,
                                    end_pos,
                                    slot_name: Some("time".to_string()),
                                });
                            }
                        }
                    }
                }
            }
            // Check for simple hour with am/pm (e.g., "3pm", "3 pm")
            else if word.ends_with("am") || word.ends_with("pm") {
                let num_part: String = word.chars().take_while(|c| c.is_ascii_digit()).collect();
                if !num_part.is_empty() {
                    if let Ok(hour) = num_part.parse::<u32>() {
                        if hour >= 1 && hour <= 12 {
                            if let Some(pos) = text_lower.find(word) {
                                entities.push(ExtractedEntity {
                                    entity_type: EntityType::Time,
                                    value: word.to_string(),
                                    normalized_value: self.normalize_clock_time(word),
                                    confidence: 0.9,
                                    start_pos: pos,
                                    end_pos: pos + word.len(),
                                    slot_name: Some("time".to_string()),
                                });
                            }
                        }
                    }
                }
            }
            // Check for number followed by am/pm
            else if word.chars().all(|c| c.is_ascii_digit()) && i + 1 < words.len() {
                let next = words[i + 1];
                if next == "am" || next == "pm" || next == "a.m." || next == "p.m." {
                    if let Ok(hour) = word.parse::<u32>() {
                        if hour >= 1 && hour <= 12 {
                            let time_str = format!("{} {}", word, next);
                            if let Some(pos) = text_lower.find(word) {
                                entities.push(ExtractedEntity {
                                    entity_type: EntityType::Time,
                                    value: time_str.clone(),
                                    normalized_value: self.normalize_clock_time(&time_str),
                                    confidence: 0.9,
                                    start_pos: pos,
                                    end_pos: pos + word.len() + 1 + next.len(),
                                    slot_name: Some("time".to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Normalize clock time to 24-hour format
    fn normalize_clock_time(&self, time_str: &str) -> String {
        let lower = time_str.to_lowercase();
        let is_pm = lower.contains("pm") || lower.contains("p.m.");
        let is_am = lower.contains("am") || lower.contains("a.m.");
        
        // Extract hour and minute
        let clean: String = lower.chars()
            .filter(|c| c.is_ascii_digit() || *c == ':')
            .collect();
        
        if clean.contains(':') {
            let parts: Vec<&str> = clean.split(':').collect();
            if parts.len() == 2 {
                if let (Ok(mut hour), Ok(min)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    if is_pm && hour != 12 {
                        hour += 12;
                    } else if is_am && hour == 12 {
                        hour = 0;
                    }
                    return format!("{:02}:{:02}", hour, min);
                }
            }
        } else if let Ok(mut hour) = clean.parse::<u32>() {
            if is_pm && hour != 12 {
                hour += 12;
            } else if is_am && hour == 12 {
                hour = 0;
            }
            return format!("{:02}:00", hour);
        }
        
        time_str.to_string()
    }
    
    /// Normalize entity value
    fn normalize_entity(&self, value: &str, entity_type: &EntityType) -> String {
        match entity_type {
            EntityType::Time => self.normalize_time(value),
            EntityType::Duration => self.normalize_duration(value),
            EntityType::Number => self.normalize_number(value),
            EntityType::Amount => self.normalize_amount(value),
            _ => value.to_string(),
        }
    }
    
    /// Normalize time expressions
    fn normalize_time(&self, value: &str) -> String {
        let value_lower = value.to_lowercase();
        
        // Convert word times to approximate hours
        match value_lower.as_str() {
            "morning" => "08:00".to_string(),
            "noon" => "12:00".to_string(),
            "afternoon" => "14:00".to_string(),
            "evening" => "18:00".to_string(),
            "night" => "21:00".to_string(),
            "midnight" => "00:00".to_string(),
            _ => value.to_string(),
        }
    }
    
    /// Normalize duration expressions
    fn normalize_duration(&self, value: &str) -> String {
        let value_lower = value.to_lowercase();
        let parts: Vec<&str> = value_lower.split_whitespace().collect();
        
        // Simple parsing: look for number followed by unit
        if parts.len() >= 2 {
            if let Ok(num) = parts[0].parse::<i32>() {
                let unit = parts[1];
                let seconds = match unit {
                    "second" | "seconds" | "sec" | "secs" => num,
                    "minute" | "minutes" | "min" | "mins" => num * 60,
                    "hour" | "hours" | "hr" | "hrs" => num * 3600,
                    _ => num,
                };
                return format!("{}s", seconds);
            }
        }
        
        value.to_string()
    }
    
    /// Normalize number expressions
    fn normalize_number(&self, value: &str) -> String {
        let value_lower = value.to_lowercase();
        
        // Convert word numbers
        match value_lower.as_str() {
            "one" => "1".to_string(),
            "two" => "2".to_string(),
            "three" => "3".to_string(),
            "four" => "4".to_string(),
            "five" => "5".to_string(),
            "six" => "6".to_string(),
            "seven" => "7".to_string(),
            "eight" => "8".to_string(),
            "nine" => "9".to_string(),
            "ten" => "10".to_string(),
            _ => value.replace(",", ""),
        }
    }
    
    /// Normalize amount expressions
    fn normalize_amount(&self, value: &str) -> String {
        // Remove currency symbols and normalize
        let cleaned = value.replace("$", "").replace(",", "");
        
        // Extract numeric portion (digits and decimal point)
        let number: String = cleaned
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        
        if number.is_empty() {
            cleaned
        } else {
            number
        }
    }
    
    /// Normalize contact name
    fn normalize_contact_name(&self, name: &str) -> String {
        // Capitalize first letter of each word
        name.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Remove overlapping entities, keeping highest confidence
    fn deduplicate_entities(&self, entities: &mut Vec<ExtractedEntity>) {
        entities.sort_by(|a, b| {
            a.start_pos.cmp(&b.start_pos)
                .then(b.confidence.partial_cmp(&a.confidence).unwrap())
        });
        
        let mut result = Vec::new();
        let mut last_end = 0;
        
        for entity in entities.drain(..) {
            if entity.start_pos >= last_end {
                last_end = entity.end_pos;
                result.push(entity);
            }
        }
        
        *entities = result;
    }
}

impl Default for EntityExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Entity pattern definition
#[derive(Debug, Clone)]
struct EntityPattern {
    entity_type: EntityType,
    patterns: Vec<String>,
}

/// Extracted entity
#[derive(Debug, Clone)]
pub struct ExtractedEntity {
    pub entity_type: EntityType,
    pub value: String,
    pub normalized_value: String,
    pub confidence: f32,
    pub start_pos: usize,
    pub end_pos: usize,
    pub slot_name: Option<String>,
}

/// Entity types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Time,
    Duration,
    Number,
    Amount,
    Email,
    Phone,
    Url,
    Location,
    Contact,
    App,
    Direction,
    TravelMode,
    MediaType,
    Language,
    Percentage,
    Other,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Time => "time",
            EntityType::Duration => "duration",
            EntityType::Number => "number",
            EntityType::Amount => "amount",
            EntityType::Email => "email",
            EntityType::Phone => "phone",
            EntityType::Url => "url",
            EntityType::Location => "location",
            EntityType::Contact => "contact",
            EntityType::App => "app",
            EntityType::Direction => "direction",
            EntityType::TravelMode => "travel_mode",
            EntityType::MediaType => "media_type",
            EntityType::Language => "language",
            EntityType::Percentage => "percentage",
            EntityType::Other => "other",
        }
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Trait for custom entity extractors
pub trait EntityExtractorTrait {
    fn extract(&self, text: &str, context: &AiContext) -> Vec<ExtractedEntity>;
    fn entity_types(&self) -> Vec<EntityType>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_extractor_creation() {
        let extractor = EntityExtractor::new();
        assert!(!extractor.patterns.is_empty());
        assert!(!extractor.known_entities.is_empty());
    }
    
    #[test]
    fn test_time_extraction() {
        let extractor = EntityExtractor::new();
        let context = AiContext::default();
        
        let entities = extractor.extract("remind me at 3:30 pm", &context);
        assert!(entities.iter().any(|e| e.entity_type == EntityType::Time));
    }
    
    #[test]
    fn test_duration_extraction() {
        let extractor = EntityExtractor::new();
        let context = AiContext::default();
        
        let entities = extractor.extract("set timer for 5 minutes", &context);
        assert!(entities.iter().any(|e| e.entity_type == EntityType::Duration));
    }
    
    #[test]
    fn test_app_extraction() {
        let extractor = EntityExtractor::new();
        let context = AiContext::default();
        
        let entities = extractor.extract("open the calendar", &context);
        assert!(entities.iter().any(|e| e.entity_type == EntityType::App && e.value == "calendar"));
    }
    
    #[test]
    fn test_normalize_time() {
        let extractor = EntityExtractor::new();
        
        assert_eq!(extractor.normalize_time("morning"), "08:00");
        assert_eq!(extractor.normalize_time("noon"), "12:00");
    }
    
    #[test]
    fn test_normalize_duration() {
        let extractor = EntityExtractor::new();
        
        assert_eq!(extractor.normalize_duration("5 minutes"), "300s");
        assert_eq!(extractor.normalize_duration("1 hour"), "3600s");
    }
}
