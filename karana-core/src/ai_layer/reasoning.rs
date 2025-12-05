//! Reasoning Engine
//!
//! Handles complex queries requiring multi-step reasoning, knowledge synthesis,
//! and intelligent decision making.

use super::*;
use std::collections::HashMap;

/// Reasoning engine for complex queries
pub struct ReasoningEngine {
    /// Reasoning strategies
    strategies: Vec<ReasoningStrategy>,
    /// Knowledge base (simplified)
    knowledge: KnowledgeBase,
    /// Inference rules
    rules: Vec<InferenceRule>,
    /// Reasoning history for context
    history: Vec<ReasoningRecord>,
}

impl ReasoningEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            strategies: Vec::new(),
            knowledge: KnowledgeBase::new(),
            rules: Vec::new(),
            history: Vec::new(),
        };
        engine.initialize_strategies();
        engine.initialize_rules();
        engine
    }
    
    fn initialize_strategies(&mut self) {
        // Chain of thought reasoning
        self.strategies.push(ReasoningStrategy {
            name: "chain_of_thought".to_string(),
            description: "Break down complex questions into steps".to_string(),
            applicable_intents: vec!["question".to_string(), "search".to_string()],
            priority: 10,
        });
        
        // Analogy reasoning
        self.strategies.push(ReasoningStrategy {
            name: "analogy".to_string(),
            description: "Find similar past situations".to_string(),
            applicable_intents: vec!["question".to_string()],
            priority: 5,
        });
        
        // Visual reasoning
        self.strategies.push(ReasoningStrategy {
            name: "visual".to_string(),
            description: "Analyze visual input".to_string(),
            applicable_intents: vec!["identify_object".to_string(), "translate".to_string()],
            priority: 10,
        });
        
        // Knowledge lookup
        self.strategies.push(ReasoningStrategy {
            name: "knowledge_lookup".to_string(),
            description: "Query knowledge base".to_string(),
            applicable_intents: vec!["question".to_string(), "search".to_string()],
            priority: 8,
        });
        
        // Contextual reasoning
        self.strategies.push(ReasoningStrategy {
            name: "contextual".to_string(),
            description: "Use context to infer meaning".to_string(),
            applicable_intents: vec!["question".to_string()],
            priority: 7,
        });
    }
    
    fn initialize_rules(&mut self) {
        // Time-based inference
        self.rules.push(InferenceRule {
            name: "time_greeting".to_string(),
            condition: RuleCondition::TimeOfDay,
            inference: |ctx| {
                Some(format!("Current time context: {:?}", ctx.time_of_day))
            },
        });
        
        // Location-based inference  
        self.rules.push(InferenceRule {
            name: "location_context".to_string(),
            condition: RuleCondition::LocationAvailable,
            inference: |ctx| {
                ctx.location.as_ref().map(|loc| format!("Near {}", loc))
            },
        });
        
        // Movement inference
        self.rules.push(InferenceRule {
            name: "movement".to_string(),
            condition: RuleCondition::UserMoving,
            inference: |ctx| {
                if ctx.environment.is_moving {
                    Some("User is in motion".to_string())
                } else {
                    None
                }
            },
        });
    }
    
    /// Reason about an intent
    pub fn reason(
        &self,
        intent: &ResolvedIntent,
        entities: &[ExtractedEntity],
        context: &AiContext,
    ) -> ReasoningResult {
        // Select applicable strategies
        let strategies: Vec<_> = self.strategies.iter()
            .filter(|s| s.applicable_intents.contains(&intent.name))
            .collect();
        
        if strategies.is_empty() {
            return ReasoningResult::default();
        }
        
        // Apply each strategy
        let mut steps = Vec::new();
        let mut confidence = 0.0;
        let mut answer = None;
        
        for strategy in strategies {
            let result = self.apply_strategy(strategy, intent, entities, context);
            steps.extend(result.reasoning_steps);
            
            if result.confidence > confidence {
                confidence = result.confidence;
                answer = result.answer;
            }
        }
        
        // Apply inference rules
        let inferences = self.apply_rules(context);
        steps.extend(inferences);
        
        ReasoningResult {
            answer,
            confidence,
            sources: vec!["internal_reasoning".to_string()],
            reasoning_steps: steps,
        }
    }
    
    /// Apply a specific reasoning strategy
    fn apply_strategy(
        &self,
        strategy: &ReasoningStrategy,
        intent: &ResolvedIntent,
        entities: &[ExtractedEntity],
        context: &AiContext,
    ) -> ReasoningResult {
        match strategy.name.as_str() {
            "chain_of_thought" => self.chain_of_thought(intent, entities, context),
            "visual" => self.visual_reasoning(intent, entities, context),
            "knowledge_lookup" => self.knowledge_lookup(intent, entities, context),
            "contextual" => self.contextual_reasoning(intent, entities, context),
            "analogy" => self.analogy_reasoning(intent, entities, context),
            _ => ReasoningResult::default(),
        }
    }
    
    /// Chain of thought reasoning
    fn chain_of_thought(
        &self,
        intent: &ResolvedIntent,
        entities: &[ExtractedEntity],
        context: &AiContext,
    ) -> ReasoningResult {
        let mut steps = Vec::new();
        
        // Step 1: Identify the question type
        steps.push(format!("1. Analyzing {} intent", intent.name));
        
        // Step 2: Extract key information
        let key_entities: Vec<_> = entities.iter()
            .map(|e| format!("{}: {}", e.entity_type, e.value))
            .collect();
        steps.push(format!("2. Key information: {:?}", key_entities));
        
        // Step 3: Consider context
        steps.push(format!("3. Context: time={:?}, location={:?}", 
            context.time_of_day,
            context.location
        ));
        
        // Step 4: Generate answer based on entity content
        let answer = if let Some(query_entity) = entities.iter()
            .find(|e| e.entity_type.to_string() == "query") 
        {
            steps.push(format!("4. Processing query: {}", query_entity.value));
            Some(self.generate_answer(&query_entity.value, context))
        } else {
            steps.push("4. No specific query found".to_string());
            None
        };
        
        ReasoningResult {
            answer,
            confidence: 0.7,
            sources: vec!["chain_of_thought".to_string()],
            reasoning_steps: steps,
        }
    }
    
    /// Visual reasoning for object identification
    fn visual_reasoning(
        &self,
        intent: &ResolvedIntent,
        _entities: &[ExtractedEntity],
        context: &AiContext,
    ) -> ReasoningResult {
        let mut steps = Vec::new();
        
        steps.push("1. Capturing visual context".to_string());
        steps.push(format!("2. Lighting conditions: {:?}", context.environment.lighting));
        
        let answer = match intent.name.as_str() {
            "identify_object" => {
                steps.push("3. Running object detection".to_string());
                steps.push("4. Matching against known objects".to_string());
                Some("Object identification requires camera input".to_string())
            }
            "translate" => {
                steps.push("3. Running OCR on visible text".to_string());
                steps.push("4. Translating detected text".to_string());
                Some("Translation requires visible text".to_string())
            }
            _ => None,
        };
        
        ReasoningResult {
            answer,
            confidence: 0.6,
            sources: vec!["visual_analysis".to_string()],
            reasoning_steps: steps,
        }
    }
    
    /// Knowledge base lookup
    fn knowledge_lookup(
        &self,
        _intent: &ResolvedIntent,
        entities: &[ExtractedEntity],
        _context: &AiContext,
    ) -> ReasoningResult {
        let mut steps = Vec::new();
        
        steps.push("1. Searching knowledge base".to_string());
        
        // Look for query in entities
        if let Some(query_entity) = entities.iter()
            .find(|e| e.entity_type.to_string() == "query")
        {
            let query = &query_entity.value;
            steps.push(format!("2. Query: {}", query));
            
            // Check knowledge base
            if let Some(answer) = self.knowledge.lookup(query) {
                steps.push(format!("3. Found in knowledge base"));
                return ReasoningResult {
                    answer: Some(answer),
                    confidence: 0.9,
                    sources: vec!["knowledge_base".to_string()],
                    reasoning_steps: steps,
                };
            } else {
                steps.push("3. Not found in knowledge base".to_string());
            }
        }
        
        ReasoningResult {
            answer: None,
            confidence: 0.3,
            sources: vec!["knowledge_base".to_string()],
            reasoning_steps: steps,
        }
    }
    
    /// Contextual reasoning
    fn contextual_reasoning(
        &self,
        _intent: &ResolvedIntent,
        _entities: &[ExtractedEntity],
        context: &AiContext,
    ) -> ReasoningResult {
        let mut steps = Vec::new();
        let mut contextual_facts: Vec<String> = Vec::new();
        
        steps.push("1. Analyzing context".to_string());
        
        // Time context
        match context.time_of_day {
            TimeOfDay::Morning => contextual_facts.push("It's morning".to_string()),
            TimeOfDay::Afternoon => contextual_facts.push("It's afternoon".to_string()),
            TimeOfDay::Evening => contextual_facts.push("It's evening".to_string()),
            TimeOfDay::Night => contextual_facts.push("It's nighttime".to_string()),
        }
        
        // Location context
        if let Some(loc) = &context.location {
            contextual_facts.push(format!("Near {}", loc));
        }
        
        // Environment context
        if context.environment.is_outdoors {
            contextual_facts.push("User is outdoors".to_string());
        }
        
        if context.environment.is_moving {
            contextual_facts.push("User is moving".to_string());
        }
        
        steps.push(format!("2. Contextual facts: {:?}", contextual_facts));
        
        ReasoningResult {
            answer: None,
            confidence: 0.5,
            sources: vec!["context_analysis".to_string()],
            reasoning_steps: steps,
        }
    }
    
    /// Analogy reasoning
    fn analogy_reasoning(
        &self,
        intent: &ResolvedIntent,
        _entities: &[ExtractedEntity],
        _context: &AiContext,
    ) -> ReasoningResult {
        let mut steps = Vec::new();
        
        steps.push("1. Searching for similar past interactions".to_string());
        
        // Check history for similar intents
        let similar: Vec<_> = self.history.iter()
            .filter(|r| r.intent == intent.name)
            .take(3)
            .collect();
        
        if !similar.is_empty() {
            steps.push(format!("2. Found {} similar interactions", similar.len()));
            
            if let Some(recent) = similar.last() {
                if let Some(answer) = &recent.answer {
                    steps.push("3. Using similar previous answer as reference".to_string());
                    return ReasoningResult {
                        answer: Some(answer.clone()),
                        confidence: 0.6,
                        sources: vec!["history".to_string()],
                        reasoning_steps: steps,
                    };
                }
            }
        } else {
            steps.push("2. No similar interactions found".to_string());
        }
        
        ReasoningResult {
            answer: None,
            confidence: 0.3,
            sources: vec!["history".to_string()],
            reasoning_steps: steps,
        }
    }
    
    /// Apply inference rules
    fn apply_rules(&self, context: &AiContext) -> Vec<String> {
        let mut inferences = Vec::new();
        
        for rule in &self.rules {
            let should_apply = match rule.condition {
                RuleCondition::TimeOfDay => true,
                RuleCondition::LocationAvailable => context.location.is_some(),
                RuleCondition::UserMoving => context.environment.is_moving,
                RuleCondition::LowBattery => context.device_state.battery_level < 20,
                RuleCondition::NoisyEnvironment => matches!(
                    context.environment.noise_level,
                    NoiseLevel::Noisy | NoiseLevel::VeryNoisy
                ),
            };
            
            if should_apply {
                if let Some(inference) = (rule.inference)(context) {
                    inferences.push(format!("Inferred: {}", inference));
                }
            }
        }
        
        inferences
    }
    
    /// Generate answer for common questions
    fn generate_answer(&self, query: &str, context: &AiContext) -> String {
        let query_lower = query.to_lowercase();
        
        // Time-related questions
        if query_lower.contains("time") || query_lower.contains("what time") {
            return format!("It's {:?}", context.time_of_day);
        }
        
        // Weather (simulated)
        if query_lower.contains("weather") {
            return "Current conditions: Clear skies. For detailed forecast, I can open the weather app.".to_string();
        }
        
        // Battery
        if query_lower.contains("battery") {
            return format!("Battery is at {}%", context.device_state.battery_level);
        }
        
        // Location
        if query_lower.contains("where am i") || query_lower.contains("location") {
            if let Some(loc) = &context.location {
                return format!("You're near {}", loc);
            } else {
                return "I don't have your location right now".to_string();
            }
        }
        
        // Default
        "I'd need to search for that information".to_string()
    }
    
    /// Suggest proactive actions
    pub fn suggest_actions(
        &self,
        dialogue_state: &DialogueState,
        context: &AiContext,
    ) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // Time-based suggestions
        match context.time_of_day {
            TimeOfDay::Morning => {
                suggestions.push(Suggestion {
                    text: "Check today's schedule".to_string(),
                    action: "show_calendar".to_string(),
                    confidence: 0.7,
                    reason: "Morning routine".to_string(),
                });
            }
            TimeOfDay::Evening => {
                suggestions.push(Suggestion {
                    text: "Review tomorrow's appointments".to_string(),
                    action: "show_tomorrow".to_string(),
                    confidence: 0.6,
                    reason: "Evening planning".to_string(),
                });
            }
            _ => {}
        }
        
        // Battery-based suggestions
        if context.device_state.battery_level < 20 {
            suggestions.push(Suggestion {
                text: "Enable power saving".to_string(),
                action: "power_save".to_string(),
                confidence: 0.9,
                reason: "Low battery".to_string(),
            });
        }
        
        // Movement-based suggestions
        if context.environment.is_moving {
            suggestions.push(Suggestion {
                text: "Show navigation".to_string(),
                action: "show_nav".to_string(),
                confidence: 0.5,
                reason: "User is moving".to_string(),
            });
        }
        
        // Context-continuation suggestions
        if let Some(intent) = &dialogue_state.current_intent {
            suggestions.push(Suggestion {
                text: format!("Continue with {}", intent),
                action: intent.clone(),
                confidence: 0.4,
                reason: "Active intent".to_string(),
            });
        }
        
        // Sort by confidence
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        suggestions.truncate(3);
        
        suggestions
    }
    
    /// Record reasoning for learning
    pub fn record_reasoning(&mut self, intent: &str, answer: Option<String>, success: bool) {
        self.history.push(ReasoningRecord {
            intent: intent.to_string(),
            answer,
            success,
            timestamp: std::time::SystemTime::now(),
        });
        
        // Keep history bounded
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }
}

impl Default for ReasoningEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Reasoning strategy
#[derive(Debug, Clone)]
struct ReasoningStrategy {
    name: String,
    description: String,
    applicable_intents: Vec<String>,
    priority: u8,
}

/// Inference rule
struct InferenceRule {
    name: String,
    condition: RuleCondition,
    inference: fn(&AiContext) -> Option<String>,
}

/// Rule conditions
#[derive(Debug, Clone)]
enum RuleCondition {
    TimeOfDay,
    LocationAvailable,
    UserMoving,
    LowBattery,
    NoisyEnvironment,
}

/// Reasoning record for history
#[derive(Debug, Clone)]
struct ReasoningRecord {
    intent: String,
    answer: Option<String>,
    success: bool,
    timestamp: std::time::SystemTime,
}

/// Simple knowledge base
struct KnowledgeBase {
    facts: HashMap<String, String>,
}

impl KnowledgeBase {
    fn new() -> Self {
        let mut kb = Self {
            facts: HashMap::new(),
        };
        kb.initialize();
        kb
    }
    
    fn initialize(&mut self) {
        // Basic facts
        self.facts.insert("capital of france".to_string(), "Paris is the capital of France".to_string());
        self.facts.insert("paris".to_string(), "Paris is the capital of France".to_string());
        
        self.facts.insert("speed of light".to_string(), "The speed of light is approximately 299,792 km/s".to_string());
        
        self.facts.insert("water boiling point".to_string(), "Water boils at 100°C (212°F) at sea level".to_string());
        
        self.facts.insert("earth moon distance".to_string(), "The Moon is about 384,400 km from Earth".to_string());
        
        // Karana OS facts
        self.facts.insert("karana".to_string(), "Kāraṇa OS is an intelligent operating system for smart glasses".to_string());
        
        self.facts.insert("help".to_string(), "You can ask me to navigate, make calls, send messages, play music, take photos, set reminders, check your wallet, and more".to_string());
    }
    
    fn lookup(&self, query: &str) -> Option<String> {
        let query_lower = query.to_lowercase();
        
        // Direct match
        if let Some(answer) = self.facts.get(&query_lower) {
            return Some(answer.clone());
        }
        
        // Partial match - check if query is contained in key or vice versa
        for (key, value) in &self.facts {
            if query_lower.contains(key) || key.contains(&query_lower) {
                return Some(value.clone());
            }
        }
        
        // Word overlap match - check if most query words match key words
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        for (key, value) in &self.facts {
            let key_words: Vec<&str> = key.split_whitespace().collect();
            let matching_words = query_words.iter()
                .filter(|qw| key_words.iter().any(|kw| kw == *qw || kw.contains(*qw) || qw.contains(kw)))
                .count();
            
            // If most query words match, consider it a hit
            if matching_words > 0 && matching_words >= query_words.len() / 2 {
                return Some(value.clone());
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reasoning_engine_creation() {
        let engine = ReasoningEngine::new();
        assert!(!engine.strategies.is_empty());
    }
    
    #[test]
    fn test_knowledge_base_lookup() {
        let kb = KnowledgeBase::new();
        
        let result = kb.lookup("capital of france");
        assert!(result.is_some());
        assert!(result.unwrap().contains("Paris"));
    }
    
    #[test]
    fn test_partial_knowledge_lookup() {
        let kb = KnowledgeBase::new();
        
        let result = kb.lookup("france capital");
        assert!(result.is_some());
    }
    
    #[test]
    fn test_reason_question_intent() {
        let engine = ReasoningEngine::new();
        let context = AiContext::default();
        
        let intent = ResolvedIntent {
            name: "question".to_string(),
            category: IntentCategory::Information,
            confidence: 0.9,
            requires_confirmation: false,
            requires_reasoning: true,
            missing_slots: vec![],
            feasible: true,
            alternative: None,
            continues_previous: false,
        };
        
        let result = engine.reason(&intent, &[], &context);
        
        // Should have reasoning steps
        assert!(!result.reasoning_steps.is_empty());
    }
    
    #[test]
    fn test_suggest_actions() {
        let engine = ReasoningEngine::new();
        let state = DialogueState::default();
        let mut context = AiContext::default();
        context.time_of_day = TimeOfDay::Morning;
        
        let suggestions = engine.suggest_actions(&state, &context);
        
        assert!(!suggestions.is_empty());
    }
    
    #[test]
    fn test_low_battery_suggestion() {
        let engine = ReasoningEngine::new();
        let state = DialogueState::default();
        let mut context = AiContext::default();
        context.device_state.battery_level = 15;
        
        let suggestions = engine.suggest_actions(&state, &context);
        
        assert!(suggestions.iter().any(|s| s.action == "power_save"));
    }
    
    #[test]
    fn test_generate_answer() {
        let engine = ReasoningEngine::new();
        let context = AiContext::default();
        
        let answer = engine.generate_answer("what's the battery level", &context);
        assert!(answer.contains("Battery"));
    }
}
