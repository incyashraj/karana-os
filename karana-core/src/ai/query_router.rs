// Kāraṇa OS - Intelligent Query Router
// Routes queries to optimal handlers based on intent classification

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use regex::Regex;

// Note: EmbeddingGenerator is in oracle module, so we'll use a simpler approach
// For now, we use pattern matching. Future: integrate with oracle embeddings.

/// Query intent types for intelligent routing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryIntent {
    /// Mathematical computation (deterministic)
    Computational(MathQuery),
    
    /// Static factual knowledge (Wikipedia, encyclopedia)
    FactualStatic(StaticQuery),
    
    /// Current/live information (weather, news, prices)
    FactualCurrent(LiveQuery),
    
    /// Personal user data (calendar, files, preferences)
    Personal(UserQuery),
    
    /// Conversational/chit-chat
    Conversational(DialogueQuery),
    
    /// Multi-step reasoning required
    MultiHop(ChainQuery),
    
    /// Blockchain operations
    Blockchain(ChainCommand),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MathQuery {
    pub expression: String,
    pub operation_type: MathOperation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MathOperation {
    BasicArithmetic,
    Percentage,
    UnitConversion,
    DateCalculation,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticQuery {
    pub topic: String,
    pub query_type: StaticQueryType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StaticQueryType {
    Definition,      // "What is X?"
    Biography,       // "Who is X?"
    Historical,      // "When did X happen?"
    Geographic,      // "Where is X?"
    General,         // Other factual
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveQuery {
    pub query_type: LiveQueryType,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiveQueryType {
    Weather,
    News,
    Price,
    Traffic,
    Sports,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserQuery {
    pub query_type: UserQueryType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserQueryType {
    Calendar,
    Files,
    Preferences,
    Memory,
    Contacts,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogueQuery {
    pub sentiment: DialogueSentiment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DialogueSentiment {
    Greeting,
    Farewell,
    Gratitude,
    Affirmation,
    Negation,
    Neutral,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChainQuery {
    pub sub_questions: Vec<String>,
    pub complexity: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChainCommand {
    pub command_type: String,
}

/// Routing decision with handler specification
#[derive(Debug, Clone)]
pub enum RouteDecision {
    /// Direct deterministic handler (no AI needed)
    Direct(DirectHandler),
    
    /// Single tool call
    SingleTool(ToolName, f32),  // Tool name + confidence
    
    /// Multi-step ReAct reasoning chain
    ReActChain(Vec<ToolName>),
    
    /// Conversational dialogue manager
    Conversational,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectHandler {
    Calculator,
    BalanceCheck,
    SystemStatus,
    UnitConverter,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolName {
    WebSearch,
    Wikipedia,
    Weather,
    Calculator,
    BlockchainQuery,
    FileSearch,
    MemoryRecall,
    Calendar,
    Maps,
    News,
}

/// Intelligent query router
pub struct QueryRouter {
    confidence_threshold: f32,
    
    // Regex patterns for quick classification
    math_pattern: Regex,
    percentage_pattern: Regex,
    date_pattern: Regex,
    weather_pattern: Regex,
    blockchain_pattern: Regex,
    personal_pattern: Regex,
}

impl QueryRouter {
    pub fn new() -> Result<Self> {
        Ok(Self {
            confidence_threshold: 0.75,
            math_pattern: Regex::new(r"(?i)\d+\s*[\+\-\*/×÷]\s*\d+|calculate|compute|what'?s?\s+\d+").unwrap(),
            percentage_pattern: Regex::new(r"(?i)\d+%|\d+\s*percent|tip\s+(?:on|of)|discount").unwrap(),
            date_pattern: Regex::new(r"(?i)days?\s+(?:ago|from|until|before|after)|weeks?\s+|months?\s+|years?\s+").unwrap(),
            weather_pattern: Regex::new(r"(?i)weather|temperature|rain|snow|forecast|umbrella|sunny|cloudy").unwrap(),
            blockchain_pattern: Regex::new(r"(?i)balance|send|transfer|stake|vote|proposal|token|wallet|transaction").unwrap(),
            personal_pattern: Regex::new(r"(?i)\bmy\s+|\bi\s+|\bme\s+|mine\b").unwrap(),
        })
    }
    
    /// Initialize pattern embeddings (call once at startup)
    /// Note: Embedding functionality deferred to future integration with oracle embeddings
    pub fn initialize_embeddings(&mut self) -> Result<()> {
        // Placeholder for future embedding initialization
        Ok(())
    }
    
    /// Main routing function - determines optimal handler for query
    pub fn route(&self, query: &str) -> Result<RouteDecision> {
        log::debug!("[QueryRouter] Routing query: {}", query);
        
        // 1. Check for deterministic handlers (fastest)
        if let Some(handler) = self.check_deterministic(query)? {
            log::info!("[QueryRouter] → Direct handler: {:?}", handler);
            return Ok(RouteDecision::Direct(handler));
        }
        
        // 2. Classify intent
        let intent = self.classify_intent(query)?;
        log::debug!("[QueryRouter] Intent: {:?}", intent);
        
        // 3. Route based on intent
        match intent {
            QueryIntent::Computational(_) => {
                Ok(RouteDecision::SingleTool(ToolName::Calculator, 1.0))
            }
            
            QueryIntent::FactualStatic(static_query) => {
                // Static facts → Wikipedia first, then web search
                Ok(RouteDecision::SingleTool(ToolName::Wikipedia, 0.9))
            }
            
            QueryIntent::FactualCurrent(live_query) => {
                let tool = match live_query.query_type {
                    LiveQueryType::Weather => ToolName::Weather,
                    LiveQueryType::News => ToolName::News,
                    LiveQueryType::Traffic => ToolName::Maps,
                    _ => ToolName::WebSearch,
                };
                Ok(RouteDecision::SingleTool(tool, 0.85))
            }
            
            QueryIntent::Personal(user_query) => {
                let tool = match user_query.query_type {
                    UserQueryType::Calendar => ToolName::Calendar,
                    UserQueryType::Files => ToolName::FileSearch,
                    UserQueryType::Memory => ToolName::MemoryRecall,
                    _ => ToolName::MemoryRecall,
                };
                Ok(RouteDecision::SingleTool(tool, 0.8))
            }
            
            QueryIntent::Blockchain(_) => {
                Ok(RouteDecision::SingleTool(ToolName::BlockchainQuery, 0.95))
            }
            
            QueryIntent::MultiHop(chain_query) => {
                // Complex multi-step reasoning
                let tools = self.predict_tool_sequence(&chain_query)?;
                Ok(RouteDecision::ReActChain(tools))
            }
            
            QueryIntent::Conversational(_) => {
                Ok(RouteDecision::Conversational)
            }
        }
    }
    
    /// Check for deterministic handlers (no AI needed)
    fn check_deterministic(&self, query: &str) -> Result<Option<DirectHandler>> {
        let lower = query.to_lowercase();
        
        // Balance check patterns
        if lower.contains("balance") || lower.contains("how much") && lower.contains("token") {
            return Ok(Some(DirectHandler::BalanceCheck));
        }
        
        // System status
        if lower.contains("system") && (lower.contains("status") || lower.contains("health")) {
            return Ok(Some(DirectHandler::SystemStatus));
        }
        
        // Simple math (handled by calculator)
        if self.math_pattern.is_match(&lower) {
            return Ok(Some(DirectHandler::Calculator));
        }
        
        Ok(None)
    }
    
    /// Classify query into intent category
    fn classify_intent(&self, query: &str) -> Result<QueryIntent> {
        let lower = query.to_lowercase();
        
        // 1. Check math patterns (highest priority for deterministic queries)
        if self.math_pattern.is_match(&lower) || self.percentage_pattern.is_match(&lower) {
            return Ok(QueryIntent::Computational(MathQuery {
                expression: query.to_string(),
                operation_type: if self.percentage_pattern.is_match(&lower) {
                    MathOperation::Percentage
                } else if self.date_pattern.is_match(&lower) {
                    MathOperation::DateCalculation
                } else {
                    MathOperation::BasicArithmetic
                },
            }));
        }
        
        // 2. Check temporal markers for live queries
        if lower.contains("today") || lower.contains("now") || lower.contains("current") 
            || lower.contains("latest") || lower.contains("recent") {
            
            if self.weather_pattern.is_match(&lower) {
                return Ok(QueryIntent::FactualCurrent(LiveQuery {
                    query_type: LiveQueryType::Weather,
                    location: self.extract_location(query),
                }));
            }
            
            if lower.contains("news") {
                return Ok(QueryIntent::FactualCurrent(LiveQuery {
                    query_type: LiveQueryType::News,
                    location: None,
                }));
            }
            
            // Default to web search for current info
            return Ok(QueryIntent::FactualCurrent(LiveQuery {
                query_type: LiveQueryType::Other,
                location: None,
            }));
        }
        
        // 3. Check weather-specific patterns
        if self.weather_pattern.is_match(&lower) {
            return Ok(QueryIntent::FactualCurrent(LiveQuery {
                query_type: LiveQueryType::Weather,
                location: self.extract_location(query),
            }));
        }
        
        // 4. Check personal markers
        if self.personal_pattern.is_match(&lower) {
            let query_type = if lower.contains("calendar") || lower.contains("meeting") || lower.contains("appointment") {
                UserQueryType::Calendar
            } else if lower.contains("file") || lower.contains("document") {
                UserQueryType::Files
            } else if lower.contains("remember") || lower.contains("recall") {
                UserQueryType::Memory
            } else {
                UserQueryType::Preferences
            };
            
            return Ok(QueryIntent::Personal(UserQuery { query_type }));
        }
        
        // 5. Check blockchain patterns
        if self.blockchain_pattern.is_match(&lower) {
            return Ok(QueryIntent::Blockchain(ChainCommand {
                command_type: "query".to_string(),
            }));
        }
        
        // 6. Check for multi-hop questions (multiple question words)
        let question_words = ["who", "what", "when", "where", "why", "how"];
        let question_count = question_words.iter()
            .filter(|&&w| lower.contains(w))
            .count();
        
        if question_count >= 2 {
            return Ok(QueryIntent::MultiHop(ChainQuery {
                sub_questions: vec![query.to_string()],
                complexity: question_count,
            }));
        }
        
        // 7. Check for conversational patterns
        if self.is_conversational(&lower) {
            let sentiment = if lower.contains("hello") || lower.contains("hi ") || lower.contains("hey") {
                DialogueSentiment::Greeting
            } else if lower.contains("bye") || lower.contains("goodbye") {
                DialogueSentiment::Farewell
            } else if lower.contains("thank") {
                DialogueSentiment::Gratitude
            } else if lower.starts_with("yes") || lower.starts_with("yeah") || lower.starts_with("sure") {
                DialogueSentiment::Affirmation
            } else if lower.starts_with("no ") || lower.starts_with("nope") {
                DialogueSentiment::Negation
            } else {
                DialogueSentiment::Neutral
            };
            
            return Ok(QueryIntent::Conversational(DialogueQuery { sentiment }));
        }
        
        // 8. Check query structure for static vs current
        if self.is_definition_query(&lower) {
            return Ok(QueryIntent::FactualStatic(StaticQuery {
                topic: query.to_string(),
                query_type: StaticQueryType::Definition,
            }));
        }
        
        if lower.starts_with("who is") || lower.starts_with("who was") {
            return Ok(QueryIntent::FactualStatic(StaticQuery {
                topic: query.to_string(),
                query_type: StaticQueryType::Biography,
            }));
        }
        
        // 9. Default to static factual for most questions
        Ok(QueryIntent::FactualStatic(StaticQuery {
            topic: query.to_string(),
            query_type: StaticQueryType::General,
        }))
    }
    
    /// Extract location from query
    fn extract_location(&self, query: &str) -> Option<String> {
        let lower = query.to_lowercase();
        
        // Check for "in [location]"
        if let Some(pos) = lower.find(" in ") {
            let after = &query[pos + 4..];
            if let Some(end) = after.find(|c: char| c == '?' || c == '.' || c == ',') {
                return Some(after[..end].trim().to_string());
            }
            return Some(after.trim().to_string());
        }
        
        // Check for "at [location]"
        if let Some(pos) = lower.find(" at ") {
            let after = &query[pos + 4..];
            if let Some(end) = after.find(|c: char| c == '?' || c == '.' || c == ',') {
                return Some(after[..end].trim().to_string());
            }
            return Some(after.trim().to_string());
        }
        
        None
    }
    
    /// Check if query is conversational
    fn is_conversational(&self, query: &str) -> bool {
        let conversational_markers = [
            "hello", "hi ", "hey ", "bye", "goodbye", "thank",
            "yes", "no", "yeah", "nope", "sure", "okay", "ok",
            "please", "sorry",
        ];
        
        conversational_markers.iter().any(|&m| query.contains(m))
    }
    
    /// Check if query is asking for definition
    fn is_definition_query(&self, query: &str) -> bool {
        query.starts_with("what is") 
            || query.starts_with("what's") 
            || query.starts_with("define ")
            || query.contains("meaning of")
            || query.contains("definition of")
    }
    
    /// Predict sequence of tools needed for multi-hop query
    fn predict_tool_sequence(&self, chain_query: &ChainQuery) -> Result<Vec<ToolName>> {
        // Start with web search as default for complex queries
        let mut tools = vec![ToolName::WebSearch];
        
        // Add Wikipedia for factual components
        if chain_query.complexity >= 2 {
            tools.push(ToolName::Wikipedia);
        }
        
        Ok(tools)
    }
}

impl Default for QueryRouter {
    fn default() -> Self {
        Self::new().expect("Failed to create QueryRouter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_math_classification() {
        let router = QueryRouter::new().unwrap();
        
        let intent = router.classify_intent("What's 15 + 25?").unwrap();
        assert!(matches!(intent, QueryIntent::Computational(_)));
        
        let intent = router.classify_intent("Calculate 42 * 7").unwrap();
        assert!(matches!(intent, QueryIntent::Computational(_)));
    }
    
    #[test]
    fn test_weather_classification() {
        let router = QueryRouter::new().unwrap();
        
        let intent = router.classify_intent("What's the weather today?").unwrap();
        assert!(matches!(intent, QueryIntent::FactualCurrent(_)));
        
        let intent = router.classify_intent("Should I carry an umbrella?").unwrap();
        assert!(matches!(intent, QueryIntent::FactualCurrent(_)));
    }
    
    #[test]
    fn test_static_factual() {
        let router = QueryRouter::new().unwrap();
        
        let intent = router.classify_intent("What is democracy?").unwrap();
        assert!(matches!(intent, QueryIntent::FactualStatic(_)));
        
        let intent = router.classify_intent("Who was Einstein?").unwrap();
        assert!(matches!(intent, QueryIntent::FactualStatic(_)));
    }
    
    #[test]
    fn test_blockchain_classification() {
        let router = QueryRouter::new().unwrap();
        
        let intent = router.classify_intent("Check my balance").unwrap();
        assert!(matches!(intent, QueryIntent::Blockchain(_)));
        
        let intent = router.classify_intent("Send 10 tokens to Alice").unwrap();
        assert!(matches!(intent, QueryIntent::Blockchain(_)));
    }
    
    #[test]
    fn test_personal_classification() {
        let router = QueryRouter::new().unwrap();
        
        let intent = router.classify_intent("Show my calendar").unwrap();
        assert!(matches!(intent, QueryIntent::Personal(_)));
        
        let intent = router.classify_intent("What files do I have?").unwrap();
        assert!(matches!(intent, QueryIntent::Personal(_)));
    }
    
    #[test]
    fn test_route_decision() {
        let router = QueryRouter::new().unwrap();
        
        let decision = router.route("What's 10 + 20?").unwrap();
        assert!(matches!(decision, RouteDecision::Direct(DirectHandler::Calculator)));
        
        let decision = router.route("What's the weather?").unwrap();
        assert!(matches!(decision, RouteDecision::SingleTool(ToolName::Weather, _)));
    }
    
    #[test]
    fn test_location_extraction() {
        let router = QueryRouter::new().unwrap();
        
        assert_eq!(
            router.extract_location("Weather in London"),
            Some("London".to_string())
        );
        
        assert_eq!(
            router.extract_location("Temperature at New York?"),
            Some("New York".to_string())
        );
    }
}
