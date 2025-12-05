//! Intelligence System for Kāraṇa OS
//!
//! Core intelligence layer that orchestrates smart decision making,
//! predictive actions, and context-aware app/request handling.

pub mod orchestrator;
pub mod predictor;
pub mod router;
pub mod workflows;
pub mod decisions;
pub mod app_intelligence;

pub use orchestrator::*;
pub use predictor::*;
pub use router::*;
pub use workflows::*;
pub use decisions::*;
pub use app_intelligence::*;

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Intelligence system configuration
#[derive(Debug, Clone)]
pub struct IntelligenceConfig {
    /// Enable predictive features
    pub predictive_enabled: bool,
    /// Confidence threshold for automatic actions
    pub auto_action_threshold: f32,
    /// Maximum parallel predictions
    pub max_parallel_predictions: usize,
    /// Learning rate for adaptive behavior
    pub learning_rate: f32,
    /// Enable workflow automation
    pub workflow_automation: bool,
    /// Privacy mode (local processing only)
    pub privacy_mode: bool,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            predictive_enabled: true,
            auto_action_threshold: 0.85,
            max_parallel_predictions: 3,
            learning_rate: 0.1,
            workflow_automation: true,
            privacy_mode: false,
        }
    }
}

/// Core intelligence system
pub struct IntelligenceSystem {
    config: IntelligenceConfig,
    orchestrator: RequestOrchestrator,
    predictor: ActionPredictor,
    router: SmartRouter,
    workflow_engine: WorkflowEngine,
    decision_engine: DecisionEngine,
    app_intel: AppIntelligence,
    metrics: IntelligenceMetrics,
}

impl IntelligenceSystem {
    /// Create new intelligence system
    pub fn new() -> Self {
        Self::with_config(IntelligenceConfig::default())
    }
    
    /// Create with custom config
    pub fn with_config(config: IntelligenceConfig) -> Self {
        Self {
            orchestrator: RequestOrchestrator::new(),
            predictor: ActionPredictor::new(config.max_parallel_predictions),
            router: SmartRouter::new(),
            workflow_engine: WorkflowEngine::new(config.workflow_automation),
            decision_engine: DecisionEngine::new(config.auto_action_threshold),
            app_intel: AppIntelligence::new(),
            metrics: IntelligenceMetrics::new(),
            config,
        }
    }
    
    /// Process a user request intelligently
    pub fn process_request(&mut self, request: &UserRequest) -> IntelligentResponse {
        let start = Instant::now();
        
        // 1. Classify and route the request
        let route = self.router.route(request);
        
        // 2. Check for applicable workflows
        let workflow = self.workflow_engine.find_applicable(&request, &route);
        
        // 3. Get predictive context
        let predictions = if self.config.predictive_enabled {
            self.predictor.predict_needs(request)
        } else {
            vec![]
        };
        
        // 4. Get app recommendations
        let app_suggestions = self.app_intel.suggest_apps(request);
        
        // 5. Make decisions
        let decisions = self.decision_engine.decide(
            request,
            &route,
            &predictions,
            &app_suggestions
        );
        
        // 6. Orchestrate the response
        let response = self.orchestrator.orchestrate(
            request,
            route,
            workflow,
            predictions,
            decisions
        );
        
        // Track metrics
        self.metrics.record_request(start.elapsed());
        
        response
    }
    
    /// Get predictive suggestions for current context
    pub fn get_predictions(&self) -> Vec<Prediction> {
        self.predictor.get_active_predictions()
    }
    
    /// Learn from user feedback
    pub fn learn_from_feedback(&mut self, feedback: &UserFeedback) {
        self.predictor.update_from_feedback(feedback);
        self.router.update_from_feedback(feedback);
        self.decision_engine.update_from_feedback(feedback);
        self.metrics.record_feedback(feedback);
    }
    
    /// Get current intelligence metrics
    pub fn metrics(&self) -> &IntelligenceMetrics {
        &self.metrics
    }
}

impl Default for IntelligenceSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// A user request to be processed
#[derive(Debug, Clone)]
pub struct UserRequest {
    pub id: u64,
    pub input: RequestInput,
    pub context: RequestContext,
    pub timestamp: Instant,
}

/// Input type for the request
#[derive(Debug, Clone)]
pub enum RequestInput {
    Voice(String),
    Gesture(GestureInput),
    Gaze(GazeInput),
    Touch(TouchInput),
    Text(String),
    SystemEvent(SystemEventInput),
    Combined(Vec<RequestInput>),
}

/// Gesture input data
#[derive(Debug, Clone)]
pub struct GestureInput {
    pub gesture_type: String,
    pub confidence: f32,
    pub hand: Hand,
}

/// Gaze input data
#[derive(Debug, Clone)]
pub struct GazeInput {
    pub target: Option<String>,
    pub dwell_time_ms: u64,
    pub intentional: bool,
}

/// Touch input data
#[derive(Debug, Clone)]
pub struct TouchInput {
    pub action: String,
    pub position: (f32, f32),
}

/// System event input
#[derive(Debug, Clone)]
pub struct SystemEventInput {
    pub event_type: String,
    pub data: HashMap<String, String>,
}

/// Hand enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Hand {
    Left,
    Right,
    Both,
}

/// Context for the request
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub location: Option<String>,
    pub current_app: Option<String>,
    pub recent_apps: Vec<String>,
    pub time_of_day: TimeOfDay,
    pub battery_level: u8,
    pub is_moving: bool,
    pub ambient_noise: NoiseLevel,
    pub user_state: UserState,
}

/// Time of day classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeOfDay {
    Morning,
    Afternoon,
    Evening,
    Night,
}

/// Ambient noise level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoiseLevel {
    Quiet,
    Normal,
    Noisy,
    VeryNoisy,
}

/// User state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserState {
    Idle,
    Active,
    Busy,
    DoNotDisturb,
}

/// Intelligent response
#[derive(Debug, Clone)]
pub struct IntelligentResponse {
    pub primary_action: Action,
    pub secondary_actions: Vec<Action>,
    pub ui_updates: Vec<UiUpdate>,
    pub predictions: Vec<Prediction>,
    pub confidence: f32,
    pub explanation: Option<String>,
}

/// An action to take
#[derive(Debug, Clone)]
pub struct Action {
    pub action_type: ActionType,
    pub target: String,
    pub parameters: HashMap<String, String>,
    pub priority: ActionPriority,
}

/// Action types
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    LaunchApp,
    CloseApp,
    Navigate,
    ShowOverlay,
    HideOverlay,
    Speak,
    Haptic,
    SystemSetting,
    Workflow,
    Query,
    Custom(String),
}

/// Action priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionPriority {
    Low,
    Normal,
    High,
    Immediate,
}

/// UI update
#[derive(Debug, Clone)]
pub struct UiUpdate {
    pub update_type: UiUpdateType,
    pub data: HashMap<String, String>,
}

/// UI update types
#[derive(Debug, Clone, PartialEq)]
pub enum UiUpdateType {
    ShowSuggestion,
    HideSuggestion,
    HighlightElement,
    ShowQuickAction,
    UpdateStatus,
    RefreshWidget,
}

/// A prediction for what the user might want
#[derive(Debug, Clone)]
pub struct Prediction {
    pub prediction_type: PredictionType,
    pub target: String,
    pub confidence: f32,
    pub reason: String,
    pub expires_at: Instant,
}

/// Prediction types
#[derive(Debug, Clone, PartialEq)]
pub enum PredictionType {
    NextApp,
    NextAction,
    UpcomingReminder,
    InformationNeed,
    EnvironmentChange,
}

/// User feedback for learning
#[derive(Debug, Clone)]
pub struct UserFeedback {
    pub request_id: u64,
    pub accepted: bool,
    pub rating: Option<u8>,
    pub correction: Option<String>,
    pub timestamp: Instant,
}

/// Intelligence metrics
#[derive(Debug, Clone)]
pub struct IntelligenceMetrics {
    pub requests_processed: u64,
    pub successful_predictions: u64,
    pub total_predictions: u64,
    pub average_response_time_ms: f64,
    pub user_satisfaction: f32,
    response_times: Vec<Duration>,
}

impl IntelligenceMetrics {
    pub fn new() -> Self {
        Self {
            requests_processed: 0,
            successful_predictions: 0,
            total_predictions: 0,
            average_response_time_ms: 0.0,
            user_satisfaction: 0.5,
            response_times: Vec::new(),
        }
    }
    
    pub fn record_request(&mut self, duration: Duration) {
        self.requests_processed += 1;
        self.response_times.push(duration);
        
        // Keep last 1000 samples
        if self.response_times.len() > 1000 {
            self.response_times.remove(0);
        }
        
        // Update average
        let total_ms: f64 = self.response_times.iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .sum();
        self.average_response_time_ms = total_ms / self.response_times.len() as f64;
    }
    
    pub fn record_feedback(&mut self, feedback: &UserFeedback) {
        if feedback.accepted {
            self.successful_predictions += 1;
        }
        self.total_predictions += 1;
        
        // Update satisfaction
        if let Some(rating) = feedback.rating {
            let new_rating = rating as f32 / 10.0;
            self.user_satisfaction = 0.9 * self.user_satisfaction + 0.1 * new_rating;
        }
    }
    
    pub fn prediction_accuracy(&self) -> f32 {
        if self.total_predictions == 0 {
            0.0
        } else {
            self.successful_predictions as f32 / self.total_predictions as f32
        }
    }
}

impl Default for IntelligenceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_intelligence_config() {
        let config = IntelligenceConfig::default();
        assert!(config.predictive_enabled);
        assert_eq!(config.auto_action_threshold, 0.85);
    }
    
    #[test]
    fn test_intelligence_metrics() {
        let mut metrics = IntelligenceMetrics::new();
        
        metrics.record_request(Duration::from_millis(50));
        metrics.record_request(Duration::from_millis(100));
        
        assert_eq!(metrics.requests_processed, 2);
        assert!((metrics.average_response_time_ms - 75.0).abs() < 1.0);
    }
    
    #[test]
    fn test_prediction_accuracy() {
        let mut metrics = IntelligenceMetrics::new();
        
        for i in 0..10 {
            metrics.record_feedback(&UserFeedback {
                request_id: i,
                accepted: i < 7, // 70% acceptance
                rating: None,
                correction: None,
                timestamp: Instant::now(),
            });
        }
        
        assert!((metrics.prediction_accuracy() - 0.7).abs() < 0.01);
    }
    
    #[test]
    fn test_action_priority_ordering() {
        assert!(ActionPriority::Low < ActionPriority::Normal);
        assert!(ActionPriority::High < ActionPriority::Immediate);
    }
}
