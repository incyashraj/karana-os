//! Enhanced AI Layer for Kāraṇa OS
//!
//! Provides intelligent natural language understanding, intent resolution,
//! context management, and response generation for smart glasses.

pub mod nlu;
pub mod intent;
pub mod dialogue;
pub mod response;
pub mod entities;
pub mod reasoning;
pub mod context_manager;
pub mod error_recovery;
pub mod slot_filler;
pub mod query_understanding;

pub use nlu::*;
pub use intent::*;
pub use dialogue::*;
pub use response::*;
pub use entities::*;
pub use reasoning::*;
pub use context_manager::*;
pub use error_recovery::*;
pub use slot_filler::*;
pub use query_understanding::*;

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// AI Layer configuration
#[derive(Debug, Clone)]
pub struct AiLayerConfig {
    /// Confidence threshold for intent matching
    pub intent_confidence_threshold: f32,
    /// Enable multi-turn conversation
    pub multi_turn_enabled: bool,
    /// Maximum conversation history to maintain
    pub max_history_turns: usize,
    /// Enable proactive suggestions
    pub proactive_mode: bool,
    /// Response verbosity level
    pub verbosity: Verbosity,
    /// Enable learning from interactions
    pub learning_enabled: bool,
    /// Timeout for AI processing
    pub processing_timeout_ms: u64,
}

impl Default for AiLayerConfig {
    fn default() -> Self {
        Self {
            intent_confidence_threshold: 0.6,
            multi_turn_enabled: true,
            max_history_turns: 10,
            proactive_mode: true,
            verbosity: Verbosity::Concise,
            learning_enabled: true,
            processing_timeout_ms: 3000,
        }
    }
}

/// Response verbosity levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Verbosity {
    Minimal,
    Concise,
    Normal,
    Detailed,
}

/// The main AI layer that coordinates all AI functionality
pub struct AiLayer {
    config: AiLayerConfig,
    nlu_engine: NluEngine,
    intent_resolver: IntentResolver,
    dialogue_manager: DialogueManager,
    response_generator: ResponseGenerator,
    entity_extractor: EntityExtractor,
    reasoning_engine: ReasoningEngine,
    metrics: AiMetrics,
}

impl AiLayer {
    /// Create new AI layer
    pub fn new() -> Self {
        Self::with_config(AiLayerConfig::default())
    }
    
    /// Create with custom configuration
    pub fn with_config(config: AiLayerConfig) -> Self {
        Self {
            nlu_engine: NluEngine::new(config.intent_confidence_threshold),
            intent_resolver: IntentResolver::new(),
            dialogue_manager: DialogueManager::new(config.max_history_turns),
            response_generator: ResponseGenerator::new(config.verbosity),
            entity_extractor: EntityExtractor::new(),
            reasoning_engine: ReasoningEngine::new(),
            metrics: AiMetrics::new(),
            config,
        }
    }
    
    /// Process user input and generate intelligent response
    pub fn process(&mut self, input: &str, context: &AiContext) -> AiResponse {
        let start = Instant::now();
        
        // 1. Preprocess and normalize input
        let normalized = self.nlu_engine.preprocess(input);
        
        // 2. Extract entities from input
        let entities = self.entity_extractor.extract(&normalized, context);
        
        // 3. Understand intent with NLU
        let nlu_result = self.nlu_engine.understand(&normalized, &entities);
        
        // 4. Resolve intent considering dialogue context
        let resolved_intent = self.intent_resolver.resolve(
            &nlu_result,
            self.dialogue_manager.current_state(),
            context
        );
        
        // 5. Apply reasoning for complex queries
        let reasoning_result = if resolved_intent.requires_reasoning {
            Some(self.reasoning_engine.reason(&resolved_intent, &entities, context))
        } else {
            None
        };
        
        // 6. Update dialogue state
        self.dialogue_manager.update(&resolved_intent, &entities);
        
        // 7. Generate response
        let response = self.response_generator.generate(
            &resolved_intent,
            &entities,
            reasoning_result.as_ref(),
            context
        );
        
        // Track metrics
        self.metrics.record_interaction(start.elapsed(), response.confidence);
        
        response
    }
    
    /// Process with streaming response (for long answers)
    pub fn process_streaming<F>(&mut self, input: &str, context: &AiContext, mut callback: F) 
    where F: FnMut(&str)
    {
        let response = self.process(input, context);
        
        // Stream response in chunks for better UX
        for chunk in response.text.split(". ") {
            callback(chunk);
        }
    }
    
    /// Get follow-up suggestions based on current context
    pub fn get_suggestions(&self, context: &AiContext) -> Vec<Suggestion> {
        if !self.config.proactive_mode {
            return vec![];
        }
        
        self.reasoning_engine.suggest_actions(
            self.dialogue_manager.current_state(),
            context
        )
    }
    
    /// Learn from user feedback
    pub fn learn(&mut self, feedback: &AiFeedback) {
        if !self.config.learning_enabled {
            return;
        }
        
        self.intent_resolver.learn_from_feedback(feedback);
        self.response_generator.learn_from_feedback(feedback);
        self.metrics.record_feedback(feedback);
    }
    
    /// Reset conversation state
    pub fn reset_conversation(&mut self) {
        self.dialogue_manager.reset();
    }
    
    /// Get AI metrics
    pub fn metrics(&self) -> &AiMetrics {
        &self.metrics
    }
}

impl Default for AiLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Context provided to AI for processing
#[derive(Debug, Clone)]
pub struct AiContext {
    /// User ID
    pub user_id: Option<String>,
    /// Current location
    pub location: Option<String>,
    /// Time of day
    pub time_of_day: TimeOfDay,
    /// Current app/screen
    pub current_app: Option<String>,
    /// Recent actions
    pub recent_actions: Vec<String>,
    /// User preferences
    pub preferences: HashMap<String, String>,
    /// Environmental context
    pub environment: EnvironmentContext,
    /// Device state
    pub device_state: DeviceState,
}

impl Default for AiContext {
    fn default() -> Self {
        Self {
            user_id: None,
            location: None,
            time_of_day: TimeOfDay::Afternoon,
            current_app: None,
            recent_actions: vec![],
            preferences: HashMap::new(),
            environment: EnvironmentContext::default(),
            device_state: DeviceState::default(),
        }
    }
}

/// Time of day for context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeOfDay {
    Morning,
    Afternoon,
    Evening,
    Night,
}

/// Environmental context
#[derive(Debug, Clone, Default)]
pub struct EnvironmentContext {
    pub noise_level: NoiseLevel,
    pub lighting: LightingCondition,
    pub is_outdoors: bool,
    pub is_moving: bool,
    pub nearby_people: u32,
}

/// Noise levels
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum NoiseLevel {
    Quiet,
    #[default]
    Normal,
    Noisy,
    VeryNoisy,
}

/// Lighting conditions
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LightingCondition {
    Dark,
    Dim,
    #[default]
    Normal,
    Bright,
    VeryBright,
}

/// Device state
#[derive(Debug, Clone)]
pub struct DeviceState {
    pub battery_level: u8,
    pub is_charging: bool,
    pub wifi_connected: bool,
    pub bluetooth_connected: bool,
    pub gps_enabled: bool,
    pub camera_available: bool,
    pub microphone_available: bool,
}

impl Default for DeviceState {
    fn default() -> Self {
        Self {
            battery_level: 80,
            is_charging: false,
            wifi_connected: true,
            bluetooth_connected: true,
            gps_enabled: true,
            camera_available: true,
            microphone_available: true,
        }
    }
}

/// AI response structure
#[derive(Debug, Clone)]
pub struct AiResponse {
    /// Response text
    pub text: String,
    /// Confidence score
    pub confidence: f32,
    /// Resolved intent
    pub intent: Option<ResolvedIntent>,
    /// Extracted entities
    pub entities: Vec<ExtractedEntity>,
    /// Actions to execute
    pub actions: Vec<AiAction>,
    /// Follow-up suggestions
    pub suggestions: Vec<Suggestion>,
    /// Response type
    pub response_type: ResponseType,
    /// Whether clarification is needed
    pub needs_clarification: bool,
    /// Clarification question if needed
    pub clarification_question: Option<String>,
}

/// Response types
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseType {
    Informational,
    Confirmation,
    ActionResult,
    Clarification,
    Error,
    Suggestion,
    Greeting,
}

/// An action for the system to execute
#[derive(Debug, Clone)]
pub struct AiAction {
    pub action_type: String,
    pub parameters: HashMap<String, String>,
    pub priority: ActionPriority,
    pub requires_confirmation: bool,
}

/// Action priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionPriority {
    Low,
    Normal,
    High,
    Immediate,
}

/// Follow-up suggestion
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub text: String,
    pub action: String,
    pub confidence: f32,
    pub reason: String,
}

/// User feedback for learning
#[derive(Debug, Clone)]
pub struct AiFeedback {
    pub interaction_id: u64,
    pub accepted: bool,
    pub rating: Option<u8>,
    pub correction: Option<String>,
    pub selected_suggestion: Option<usize>,
}

/// AI metrics tracking
#[derive(Debug, Clone)]
pub struct AiMetrics {
    pub total_interactions: u64,
    pub successful_interactions: u64,
    pub average_confidence: f32,
    pub average_response_time_ms: f64,
    pub feedback_positive: u64,
    pub feedback_negative: u64,
    response_times: Vec<Duration>,
    confidences: Vec<f32>,
}

impl AiMetrics {
    pub fn new() -> Self {
        Self {
            total_interactions: 0,
            successful_interactions: 0,
            average_confidence: 0.0,
            average_response_time_ms: 0.0,
            feedback_positive: 0,
            feedback_negative: 0,
            response_times: Vec::new(),
            confidences: Vec::new(),
        }
    }
    
    pub fn record_interaction(&mut self, duration: Duration, confidence: f32) {
        self.total_interactions += 1;
        if confidence > 0.6 {
            self.successful_interactions += 1;
        }
        
        self.response_times.push(duration);
        self.confidences.push(confidence);
        
        // Keep bounded
        if self.response_times.len() > 1000 {
            self.response_times.remove(0);
            self.confidences.remove(0);
        }
        
        // Update averages
        self.average_response_time_ms = self.response_times.iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .sum::<f64>() / self.response_times.len() as f64;
        
        self.average_confidence = self.confidences.iter().sum::<f32>() 
            / self.confidences.len() as f32;
    }
    
    pub fn record_feedback(&mut self, feedback: &AiFeedback) {
        if feedback.accepted {
            self.feedback_positive += 1;
        } else {
            self.feedback_negative += 1;
        }
    }
    
    pub fn success_rate(&self) -> f32 {
        if self.total_interactions == 0 {
            0.0
        } else {
            self.successful_interactions as f32 / self.total_interactions as f32
        }
    }
    
    pub fn satisfaction_rate(&self) -> f32 {
        let total = self.feedback_positive + self.feedback_negative;
        if total == 0 {
            0.0
        } else {
            self.feedback_positive as f32 / total as f32
        }
    }
}

impl Default for AiMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ai_layer_creation() {
        let ai = AiLayer::new();
        assert_eq!(ai.config.intent_confidence_threshold, 0.6);
    }
    
    #[test]
    fn test_ai_context_default() {
        let ctx = AiContext::default();
        assert_eq!(ctx.time_of_day, TimeOfDay::Afternoon);
    }
    
    #[test]
    fn test_ai_metrics() {
        let mut metrics = AiMetrics::new();
        
        metrics.record_interaction(Duration::from_millis(100), 0.8);
        metrics.record_interaction(Duration::from_millis(50), 0.9);
        
        assert_eq!(metrics.total_interactions, 2);
        assert_eq!(metrics.successful_interactions, 2);
        assert!((metrics.average_confidence - 0.85).abs() < 0.01);
    }
    
    #[test]
    fn test_ai_feedback() {
        let mut metrics = AiMetrics::new();
        
        metrics.record_feedback(&AiFeedback {
            interaction_id: 1,
            accepted: true,
            rating: Some(8),
            correction: None,
            selected_suggestion: None,
        });
        
        metrics.record_feedback(&AiFeedback {
            interaction_id: 2,
            accepted: false,
            rating: Some(3),
            correction: Some("I meant something else".to_string()),
            selected_suggestion: None,
        });
        
        assert_eq!(metrics.feedback_positive, 1);
        assert_eq!(metrics.feedback_negative, 1);
        assert!((metrics.satisfaction_rate() - 0.5).abs() < 0.01);
    }
}
