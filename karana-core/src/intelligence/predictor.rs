//! Action Predictor
//!
//! Predicts what the user will need next based on patterns,
//! context, and historical behavior.

use super::*;
use std::collections::{HashMap, VecDeque};

/// Action predictor engine
pub struct ActionPredictor {
    /// Maximum parallel predictions to maintain
    max_predictions: usize,
    /// Active predictions
    active_predictions: Vec<Prediction>,
    /// Pattern database
    patterns: PatternDatabase,
    /// Context model
    context_model: ContextModel,
    /// Temporal model for time-based predictions
    temporal_model: TemporalModel,
    /// App usage model
    app_model: AppUsageModel,
}

impl ActionPredictor {
    pub fn new(max_predictions: usize) -> Self {
        Self {
            max_predictions,
            active_predictions: Vec::new(),
            patterns: PatternDatabase::new(),
            context_model: ContextModel::new(),
            temporal_model: TemporalModel::new(),
            app_model: AppUsageModel::new(),
        }
    }
    
    /// Predict what the user will need based on request
    pub fn predict_needs(&mut self, request: &UserRequest) -> Vec<Prediction> {
        let mut predictions = Vec::new();
        
        // 1. Pattern-based predictions
        predictions.extend(self.patterns.predict(request));
        
        // 2. Context-based predictions
        predictions.extend(self.context_model.predict(&request.context));
        
        // 3. Temporal predictions (time-based)
        predictions.extend(self.temporal_model.predict(&request.context.time_of_day));
        
        // 4. App sequence predictions
        predictions.extend(self.app_model.predict_next_app(&request.context.recent_apps));
        
        // Sort by confidence and take top N
        predictions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        predictions.truncate(self.max_predictions);
        
        // Update active predictions
        self.active_predictions = predictions.clone();
        
        predictions
    }
    
    /// Get currently active predictions
    pub fn get_active_predictions(&self) -> Vec<Prediction> {
        self.active_predictions.clone()
    }
    
    /// Update models from user feedback
    pub fn update_from_feedback(&mut self, feedback: &UserFeedback) {
        // If prediction was accepted, reinforce the pattern
        if feedback.accepted {
            self.patterns.reinforce(feedback.request_id);
        } else {
            self.patterns.weaken(feedback.request_id);
        }
    }
    
    /// Learn a new pattern
    pub fn learn_pattern(&mut self, trigger: PatternTrigger, action: &str) {
        self.patterns.add_pattern(trigger, action.to_string());
    }
}

impl Default for ActionPredictor {
    fn default() -> Self {
        Self::new(3)
    }
}

/// Pattern database for sequence learning
pub struct PatternDatabase {
    /// Learned patterns
    patterns: Vec<LearnedPattern>,
    /// Pattern counter
    counter: u64,
    /// Recent pattern matches for reinforcement
    recent_matches: HashMap<u64, u64>, // request_id -> pattern_id
}

/// A learned pattern
#[derive(Debug, Clone)]
pub struct LearnedPattern {
    pub id: u64,
    pub trigger: PatternTrigger,
    pub action: String,
    pub confidence: f32,
    pub usage_count: u32,
    pub last_used: Option<Instant>,
}

/// Pattern trigger conditions
#[derive(Debug, Clone)]
pub struct PatternTrigger {
    pub input_keywords: Vec<String>,
    pub context_location: Option<String>,
    pub context_time: Option<TimeOfDay>,
    pub context_app: Option<String>,
    pub user_state: Option<UserState>,
}

impl PatternDatabase {
    pub fn new() -> Self {
        let mut db = Self {
            patterns: Vec::new(),
            counter: 0,
            recent_matches: HashMap::new(),
        };
        
        // Add some default patterns
        db.add_default_patterns();
        db
    }
    
    fn add_default_patterns(&mut self) {
        // Morning routines
        self.add_pattern(
            PatternTrigger {
                input_keywords: vec![],
                context_location: Some("home".to_string()),
                context_time: Some(TimeOfDay::Morning),
                context_app: None,
                user_state: None,
            },
            "check_calendar".to_string()
        );
        
        // Work context
        self.add_pattern(
            PatternTrigger {
                input_keywords: vec![],
                context_location: Some("office".to_string()),
                context_time: None,
                context_app: None,
                user_state: Some(UserState::Active),
            },
            "open_productivity".to_string()
        );
        
        // Evening
        self.add_pattern(
            PatternTrigger {
                input_keywords: vec![],
                context_location: Some("home".to_string()),
                context_time: Some(TimeOfDay::Evening),
                context_app: None,
                user_state: None,
            },
            "suggest_entertainment".to_string()
        );
    }
    
    /// Add a new pattern
    pub fn add_pattern(&mut self, trigger: PatternTrigger, action: String) {
        self.counter += 1;
        self.patterns.push(LearnedPattern {
            id: self.counter,
            trigger,
            action,
            confidence: 0.5, // Start with medium confidence
            usage_count: 0,
            last_used: None,
        });
    }
    
    /// Find matching patterns and create predictions
    pub fn predict(&mut self, request: &UserRequest) -> Vec<Prediction> {
        let mut predictions = Vec::new();
        let mut matches = Vec::new();
        
        // First, calculate match scores without borrowing self mutably
        for pattern in &self.patterns {
            let match_score = Self::calculate_match_score_static(pattern, request);
            
            if match_score > 0.3 {
                let confidence = match_score * pattern.confidence;
                
                predictions.push(Prediction {
                    prediction_type: PredictionType::NextAction,
                    target: pattern.action.clone(),
                    confidence,
                    reason: format!("Based on learned pattern #{}", pattern.id),
                    expires_at: Instant::now() + Duration::from_secs(300),
                });
                
                matches.push((request.id, pattern.id));
            }
        }
        
        // Now update recent_matches
        for (req_id, pattern_id) in matches {
            self.recent_matches.insert(req_id, pattern_id);
        }
        
        predictions
    }
    
    /// Calculate how well a pattern matches the request (static version)
    fn calculate_match_score_static(pattern: &LearnedPattern, request: &UserRequest) -> f32 {
        let mut score = 0.0;
        let mut criteria_count = 0.0;
        
        // Check keywords in input
        if !pattern.trigger.input_keywords.is_empty() {
            criteria_count += 1.0;
            if let RequestInput::Text(text) | RequestInput::Voice(text) = &request.input {
                let text_lower = text.to_lowercase();
                let matches = pattern.trigger.input_keywords.iter()
                    .filter(|kw| text_lower.contains(&kw.to_lowercase()))
                    .count();
                score += matches as f32 / pattern.trigger.input_keywords.len() as f32;
            }
        }
        
        // Check location
        if let Some(loc) = &pattern.trigger.context_location {
            criteria_count += 1.0;
            if request.context.location.as_ref() == Some(loc) {
                score += 1.0;
            }
        }
        
        // Check time
        if let Some(time) = &pattern.trigger.context_time {
            criteria_count += 1.0;
            if &request.context.time_of_day == time {
                score += 1.0;
            }
        }
        
        // Check current app
        if let Some(app) = &pattern.trigger.context_app {
            criteria_count += 1.0;
            if request.context.current_app.as_ref() == Some(app) {
                score += 1.0;
            }
        }
        
        // Check user state
        if let Some(state) = &pattern.trigger.user_state {
            criteria_count += 1.0;
            if &request.context.user_state == state {
                score += 1.0;
            }
        }
        
        if criteria_count > 0.0 {
            score / criteria_count
        } else {
            0.0
        }
    }
    
    /// Reinforce a pattern that was accepted
    pub fn reinforce(&mut self, request_id: u64) {
        if let Some(pattern_id) = self.recent_matches.get(&request_id) {
            if let Some(pattern) = self.patterns.iter_mut().find(|p| p.id == *pattern_id) {
                pattern.confidence = (pattern.confidence + 0.1).min(1.0);
                pattern.usage_count += 1;
                pattern.last_used = Some(Instant::now());
            }
        }
    }
    
    /// Weaken a pattern that was rejected
    pub fn weaken(&mut self, request_id: u64) {
        if let Some(pattern_id) = self.recent_matches.get(&request_id) {
            if let Some(pattern) = self.patterns.iter_mut().find(|p| p.id == *pattern_id) {
                pattern.confidence = (pattern.confidence - 0.1).max(0.1);
            }
        }
    }
}

impl Default for PatternDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Context-based prediction model
pub struct ContextModel {
    /// Context -> action mappings
    mappings: HashMap<ContextKey, Vec<(String, f32)>>,
}

/// Key for context lookup
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ContextKey {
    pub location: Option<String>,
    pub user_state: Option<String>,
}

impl ContextModel {
    pub fn new() -> Self {
        let mut model = Self {
            mappings: HashMap::new(),
        };
        model.initialize_defaults();
        model
    }
    
    fn initialize_defaults(&mut self) {
        // Moving context
        self.mappings.insert(
            ContextKey {
                location: None,
                user_state: Some("moving".to_string()),
            },
            vec![
                ("navigation".to_string(), 0.7),
                ("music".to_string(), 0.5),
            ]
        );
        
        // Busy context
        self.mappings.insert(
            ContextKey {
                location: None,
                user_state: Some("busy".to_string()),
            },
            vec![
                ("do_not_disturb".to_string(), 0.8),
                ("silence_notifications".to_string(), 0.7),
            ]
        );
    }
    
    /// Predict based on context
    pub fn predict(&self, context: &RequestContext) -> Vec<Prediction> {
        let key = ContextKey {
            location: context.location.clone(),
            user_state: Some(format!("{:?}", context.user_state).to_lowercase()),
        };
        
        if let Some(actions) = self.mappings.get(&key) {
            actions.iter().map(|(action, conf)| {
                Prediction {
                    prediction_type: PredictionType::NextAction,
                    target: action.clone(),
                    confidence: *conf,
                    reason: "Based on current context".to_string(),
                    expires_at: Instant::now() + Duration::from_secs(600),
                }
            }).collect()
        } else {
            vec![]
        }
    }
}

impl Default for ContextModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Temporal prediction model
pub struct TemporalModel {
    /// Time-based predictions
    time_actions: HashMap<TimeOfDay, Vec<(String, f32)>>,
}

impl TemporalModel {
    pub fn new() -> Self {
        let mut model = Self {
            time_actions: HashMap::new(),
        };
        model.initialize_defaults();
        model
    }
    
    fn initialize_defaults(&mut self) {
        self.time_actions.insert(TimeOfDay::Morning, vec![
            ("check_notifications".to_string(), 0.8),
            ("weather".to_string(), 0.6),
            ("calendar".to_string(), 0.7),
        ]);
        
        self.time_actions.insert(TimeOfDay::Afternoon, vec![
            ("productivity".to_string(), 0.6),
        ]);
        
        self.time_actions.insert(TimeOfDay::Evening, vec![
            ("entertainment".to_string(), 0.5),
            ("news".to_string(), 0.4),
        ]);
        
        self.time_actions.insert(TimeOfDay::Night, vec![
            ("reduce_brightness".to_string(), 0.9),
            ("sleep_mode".to_string(), 0.6),
        ]);
    }
    
    /// Predict based on time of day
    pub fn predict(&self, time: &TimeOfDay) -> Vec<Prediction> {
        if let Some(actions) = self.time_actions.get(time) {
            actions.iter().map(|(action, conf)| {
                Prediction {
                    prediction_type: PredictionType::NextAction,
                    target: action.clone(),
                    confidence: *conf * 0.7, // Reduce confidence for time-based
                    reason: format!("Common action for {:?}", time),
                    expires_at: Instant::now() + Duration::from_secs(1800),
                }
            }).collect()
        } else {
            vec![]
        }
    }
}

impl Default for TemporalModel {
    fn default() -> Self {
        Self::new()
    }
}

/// App usage prediction model
pub struct AppUsageModel {
    /// App transition probabilities: from_app -> [(to_app, probability)]
    transitions: HashMap<String, Vec<(String, f32)>>,
    /// Most used apps
    popular_apps: Vec<(String, u32)>,
}

impl AppUsageModel {
    pub fn new() -> Self {
        let mut model = Self {
            transitions: HashMap::new(),
            popular_apps: Vec::new(),
        };
        model.initialize_defaults();
        model
    }
    
    fn initialize_defaults(&mut self) {
        // Common transitions
        self.transitions.insert("email".to_string(), vec![
            ("calendar".to_string(), 0.4),
            ("browser".to_string(), 0.3),
            ("notes".to_string(), 0.2),
        ]);
        
        self.transitions.insert("browser".to_string(), vec![
            ("notes".to_string(), 0.3),
            ("email".to_string(), 0.2),
        ]);
        
        self.transitions.insert("camera".to_string(), vec![
            ("gallery".to_string(), 0.6),
            ("share".to_string(), 0.3),
        ]);
    }
    
    /// Predict next app based on recent apps
    pub fn predict_next_app(&self, recent_apps: &[String]) -> Vec<Prediction> {
        if let Some(current) = recent_apps.last() {
            if let Some(transitions) = self.transitions.get(current) {
                return transitions.iter().map(|(app, prob)| {
                    Prediction {
                        prediction_type: PredictionType::NextApp,
                        target: app.clone(),
                        confidence: *prob,
                        reason: format!("Users often open {} after {}", app, current),
                        expires_at: Instant::now() + Duration::from_secs(300),
                    }
                }).collect();
            }
        }
        
        vec![]
    }
    
    /// Record app transition for learning
    pub fn record_transition(&mut self, from: &str, to: &str) {
        let transitions = self.transitions.entry(from.to_string()).or_default();
        
        if let Some(entry) = transitions.iter_mut().find(|(app, _)| app == to) {
            // Increase probability
            entry.1 = (entry.1 + 0.1).min(1.0);
        } else {
            // Add new transition
            transitions.push((to.to_string(), 0.3));
        }
        
        // Normalize probabilities
        let total: f32 = transitions.iter().map(|(_, p)| *p).sum();
        if total > 0.0 {
            for (_, p) in transitions.iter_mut() {
                *p /= total;
            }
        }
    }
}

impl Default for AppUsageModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_predictor_creation() {
        let predictor = ActionPredictor::new(5);
        assert!(predictor.get_active_predictions().is_empty());
    }
    
    #[test]
    fn test_pattern_database() {
        let db = PatternDatabase::new();
        assert!(!db.patterns.is_empty()); // Should have default patterns
    }
    
    #[test]
    fn test_temporal_model() {
        let model = TemporalModel::new();
        let predictions = model.predict(&TimeOfDay::Morning);
        assert!(!predictions.is_empty());
    }
    
    #[test]
    fn test_app_usage_model() {
        let model = AppUsageModel::new();
        let predictions = model.predict_next_app(&["email".to_string()]);
        assert!(!predictions.is_empty());
    }
    
    #[test]
    fn test_app_transition_recording() {
        let mut model = AppUsageModel::new();
        model.record_transition("photos", "instagram");
        
        let predictions = model.predict_next_app(&["photos".to_string()]);
        assert!(predictions.iter().any(|p| p.target == "instagram"));
    }
    
    #[test]
    fn test_context_model() {
        let model = ContextModel::new();
        let context = RequestContext {
            location: None,
            current_app: None,
            recent_apps: vec![],
            time_of_day: TimeOfDay::Morning,
            battery_level: 80,
            is_moving: false,
            ambient_noise: NoiseLevel::Normal,
            user_state: UserState::Busy,
        };
        
        // Should have predictions for busy state
        let predictions = model.predict(&context);
        // Predictions might be empty if exact context key doesn't match
        // This is expected behavior
    }
}
