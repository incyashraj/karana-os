//! Decision Engine
//!
//! Makes intelligent decisions about how to respond to user requests
//! based on context, preferences, and learned behavior.

use super::*;
use super::orchestrator::{RequestRoute, Decision, DecisionType};
use std::collections::HashMap;

/// Decision engine for intelligent response generation
pub struct DecisionEngine {
    /// Confidence threshold for automatic decisions
    auto_threshold: f32,
    /// Decision rules
    rules: Vec<DecisionRule>,
    /// User preferences
    preferences: DecisionPreferences,
    /// Decision history for learning
    history: Vec<DecisionRecord>,
    /// Statistics
    stats: DecisionStats,
}

impl DecisionEngine {
    pub fn new(auto_threshold: f32) -> Self {
        let mut engine = Self {
            auto_threshold,
            rules: Vec::new(),
            preferences: DecisionPreferences::new(),
            history: Vec::new(),
            stats: DecisionStats::new(),
        };
        engine.setup_default_rules();
        engine
    }
    
    fn setup_default_rules(&mut self) {
        // App selection rules
        self.rules.push(DecisionRule {
            decision_type: DecisionType::AppSelection,
            condition: RuleCondition::InputContains(vec!["browser".to_string(), "web".to_string(), "search".to_string()]),
            value: "browser".to_string(),
            confidence: 0.9,
            reasoning: "User wants to browse the web".to_string(),
        });
        
        self.rules.push(DecisionRule {
            decision_type: DecisionType::AppSelection,
            condition: RuleCondition::InputContains(vec!["email".to_string(), "mail".to_string()]),
            value: "email".to_string(),
            confidence: 0.9,
            reasoning: "User wants email".to_string(),
        });
        
        self.rules.push(DecisionRule {
            decision_type: DecisionType::AppSelection,
            condition: RuleCondition::InputContains(vec!["camera".to_string(), "photo".to_string(), "picture".to_string()]),
            value: "camera".to_string(),
            confidence: 0.9,
            reasoning: "User wants to take a photo".to_string(),
        });
        
        self.rules.push(DecisionRule {
            decision_type: DecisionType::AppSelection,
            condition: RuleCondition::InputContains(vec!["music".to_string(), "play".to_string(), "song".to_string()]),
            value: "music".to_string(),
            confidence: 0.8,
            reasoning: "User wants music".to_string(),
        });
        
        // UI layout rules
        self.rules.push(DecisionRule {
            decision_type: DecisionType::UiLayout,
            condition: RuleCondition::ContextMatch(ContextCondition::IsMoving),
            value: "minimal".to_string(),
            confidence: 0.8,
            reasoning: "Minimal UI while moving".to_string(),
        });
        
        self.rules.push(DecisionRule {
            decision_type: DecisionType::UiLayout,
            condition: RuleCondition::ContextMatch(ContextCondition::LowBattery),
            value: "power_save".to_string(),
            confidence: 0.85,
            reasoning: "Power saving layout on low battery".to_string(),
        });
        
        // Response format rules
        self.rules.push(DecisionRule {
            decision_type: DecisionType::ResponseFormat,
            condition: RuleCondition::ContextMatch(ContextCondition::Noisy),
            value: "visual".to_string(),
            confidence: 0.9,
            reasoning: "Visual response in noisy environment".to_string(),
        });
        
        self.rules.push(DecisionRule {
            decision_type: DecisionType::ResponseFormat,
            condition: RuleCondition::ContextMatch(ContextCondition::Moving),
            value: "audio".to_string(),
            confidence: 0.85,
            reasoning: "Audio response while moving".to_string(),
        });
        
        // Confirmation rules
        self.rules.push(DecisionRule {
            decision_type: DecisionType::Confirmation,
            condition: RuleCondition::InputContains(vec!["send".to_string(), "pay".to_string(), "transfer".to_string()]),
            value: "required".to_string(),
            confidence: 1.0,
            reasoning: "Financial actions require confirmation".to_string(),
        });
        
        self.rules.push(DecisionRule {
            decision_type: DecisionType::Confirmation,
            condition: RuleCondition::InputContains(vec!["delete".to_string(), "remove".to_string(), "clear".to_string()]),
            value: "required".to_string(),
            confidence: 0.95,
            reasoning: "Destructive actions require confirmation".to_string(),
        });
    }
    
    /// Make decisions for a request
    pub fn decide(
        &mut self,
        request: &UserRequest,
        _route: &RequestRoute,
        predictions: &[Prediction],
        app_suggestions: &[AppSuggestion],
    ) -> Vec<Decision> {
        let mut decisions = Vec::new();
        
        // Extract text from request
        let text = match &request.input {
            RequestInput::Text(t) | RequestInput::Voice(t) => t.clone(),
            _ => String::new(),
        };
        
        // Apply rules
        for rule in &self.rules {
            if self.rule_matches(rule, &text, &request.context) {
                decisions.push(Decision {
                    decision_type: rule.decision_type.clone(),
                    value: rule.value.clone(),
                    confidence: rule.confidence,
                    reasoning: rule.reasoning.clone(),
                });
            }
        }
        
        // Add app selection from suggestions if not already decided
        if !decisions.iter().any(|d| d.decision_type == DecisionType::AppSelection) {
            if let Some(best_app) = app_suggestions.first() {
                decisions.push(Decision {
                    decision_type: DecisionType::AppSelection,
                    value: best_app.app_id.clone(),
                    confidence: best_app.score,
                    reasoning: best_app.reason.clone(),
                });
            }
        }
        
        // Add predictions-based decisions
        for pred in predictions.iter().filter(|p| p.confidence > 0.7) {
            if let PredictionType::NextAction = pred.prediction_type {
                decisions.push(Decision {
                    decision_type: DecisionType::ActionSequence,
                    value: pred.target.clone(),
                    confidence: pred.confidence * 0.8, // Reduce for being predictive
                    reasoning: format!("Predicted: {}", pred.reason),
                });
            }
        }
        
        // Apply user preferences
        self.apply_preferences(&mut decisions);
        
        // Record decisions
        self.record_decisions(request.id, &decisions);
        
        decisions
    }
    
    /// Check if a rule matches
    fn rule_matches(&self, rule: &DecisionRule, text: &str, context: &RequestContext) -> bool {
        match &rule.condition {
            RuleCondition::InputContains(keywords) => {
                let text_lower = text.to_lowercase();
                keywords.iter().any(|kw| text_lower.contains(kw))
            }
            RuleCondition::ContextMatch(cond) => {
                match cond {
                    ContextCondition::IsMoving => context.is_moving,
                    ContextCondition::Moving => context.is_moving,
                    ContextCondition::LowBattery => context.battery_level < 20,
                    ContextCondition::Noisy => matches!(context.ambient_noise, NoiseLevel::Noisy | NoiseLevel::VeryNoisy),
                    ContextCondition::TimeOfDay(time) => &context.time_of_day == time,
                    ContextCondition::Location(loc) => context.location.as_ref() == Some(loc),
                    ContextCondition::UserState(state) => &context.user_state == state,
                }
            }
            RuleCondition::Combined(conditions) => {
                conditions.iter().all(|c| {
                    let temp_rule = DecisionRule {
                        decision_type: rule.decision_type.clone(),
                        condition: c.clone(),
                        value: String::new(),
                        confidence: 0.0,
                        reasoning: String::new(),
                    };
                    self.rule_matches(&temp_rule, text, context)
                })
            }
        }
    }
    
    /// Apply user preferences to decisions
    fn apply_preferences(&self, decisions: &mut Vec<Decision>) {
        for decision in decisions.iter_mut() {
            // Adjust confidence based on preferences
            if let Some(pref) = self.preferences.get(&decision.decision_type, &decision.value) {
                decision.confidence *= pref;
            }
        }
        
        // Sort by confidence
        decisions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    }
    
    /// Record decisions for learning
    fn record_decisions(&mut self, request_id: u64, decisions: &[Decision]) {
        self.history.push(DecisionRecord {
            request_id,
            decisions: decisions.to_vec(),
            timestamp: Instant::now(),
            feedback: None,
        });
        
        // Keep history bounded
        if self.history.len() > 1000 {
            self.history.remove(0);
        }
        
        self.stats.decisions_made += decisions.len() as u64;
    }
    
    /// Update from user feedback
    pub fn update_from_feedback(&mut self, feedback: &UserFeedback) {
        // Find the decision record
        if let Some(record) = self.history.iter_mut()
            .find(|r| r.request_id == feedback.request_id) {
            record.feedback = Some(feedback.accepted);
            
            if feedback.accepted {
                self.stats.accepted += 1;
                // Reinforce preferences
                for decision in &record.decisions {
                    self.preferences.reinforce(&decision.decision_type, &decision.value);
                }
            } else {
                self.stats.rejected += 1;
                // Weaken preferences
                for decision in &record.decisions {
                    self.preferences.weaken(&decision.decision_type, &decision.value);
                }
            }
        }
    }
    
    /// Add a custom rule
    pub fn add_rule(&mut self, rule: DecisionRule) {
        self.rules.push(rule);
    }
    
    /// Get statistics
    pub fn stats(&self) -> &DecisionStats {
        &self.stats
    }
    
    /// Check if decision should be automatic
    pub fn should_auto_execute(&self, decisions: &[Decision]) -> bool {
        if decisions.is_empty() {
            return false;
        }
        
        // All decisions must be above threshold
        decisions.iter().all(|d| d.confidence >= self.auto_threshold)
    }
}

impl Default for DecisionEngine {
    fn default() -> Self {
        Self::new(0.85)
    }
}

/// A decision rule
#[derive(Debug, Clone)]
pub struct DecisionRule {
    pub decision_type: DecisionType,
    pub condition: RuleCondition,
    pub value: String,
    pub confidence: f32,
    pub reasoning: String,
}

/// Rule conditions
#[derive(Debug, Clone)]
pub enum RuleCondition {
    InputContains(Vec<String>),
    ContextMatch(ContextCondition),
    Combined(Vec<RuleCondition>),
}

/// Context conditions
#[derive(Debug, Clone, PartialEq)]
pub enum ContextCondition {
    IsMoving,
    Moving,
    LowBattery,
    Noisy,
    TimeOfDay(TimeOfDay),
    Location(String),
    UserState(UserState),
}

/// Decision preferences for personalization
pub struct DecisionPreferences {
    /// Type -> Value -> Weight
    weights: HashMap<String, HashMap<String, f32>>,
}

impl DecisionPreferences {
    fn new() -> Self {
        Self {
            weights: HashMap::new(),
        }
    }
    
    fn get(&self, decision_type: &DecisionType, value: &str) -> Option<f32> {
        let type_key = format!("{:?}", decision_type);
        self.weights.get(&type_key)
            .and_then(|inner| inner.get(value))
            .copied()
    }
    
    fn reinforce(&mut self, decision_type: &DecisionType, value: &str) {
        let type_key = format!("{:?}", decision_type);
        let inner = self.weights.entry(type_key).or_default();
        let weight = inner.entry(value.to_string()).or_insert(1.0);
        *weight = (*weight + 0.05).min(1.5);
    }
    
    fn weaken(&mut self, decision_type: &DecisionType, value: &str) {
        let type_key = format!("{:?}", decision_type);
        let inner = self.weights.entry(type_key).or_default();
        let weight = inner.entry(value.to_string()).or_insert(1.0);
        *weight = (*weight - 0.05).max(0.5);
    }
}

/// Decision record for history
#[derive(Debug, Clone)]
pub struct DecisionRecord {
    pub request_id: u64,
    pub decisions: Vec<Decision>,
    pub timestamp: Instant,
    pub feedback: Option<bool>,
}

/// Decision statistics
#[derive(Debug, Clone, Default)]
pub struct DecisionStats {
    pub decisions_made: u64,
    pub accepted: u64,
    pub rejected: u64,
}

impl DecisionStats {
    fn new() -> Self {
        Self::default()
    }
    
    pub fn acceptance_rate(&self) -> f32 {
        let total = self.accepted + self.rejected;
        if total == 0 {
            0.0
        } else {
            self.accepted as f32 / total as f32
        }
    }
}

/// App suggestion for decision making
#[derive(Debug, Clone)]
pub struct AppSuggestion {
    pub app_id: String,
    pub score: f32,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decision_engine_creation() {
        let engine = DecisionEngine::new(0.85);
        assert!(!engine.rules.is_empty());
    }
    
    #[test]
    fn test_rule_matching_input() {
        let engine = DecisionEngine::new(0.85);
        
        let rule = DecisionRule {
            decision_type: DecisionType::AppSelection,
            condition: RuleCondition::InputContains(vec!["browser".to_string()]),
            value: "browser".to_string(),
            confidence: 0.9,
            reasoning: "Test".to_string(),
        };
        
        let context = RequestContext {
            location: None,
            current_app: None,
            recent_apps: vec![],
            time_of_day: TimeOfDay::Morning,
            battery_level: 80,
            is_moving: false,
            ambient_noise: NoiseLevel::Normal,
            user_state: UserState::Active,
        };
        
        assert!(engine.rule_matches(&rule, "open the browser", &context));
        assert!(!engine.rule_matches(&rule, "open email", &context));
    }
    
    #[test]
    fn test_rule_matching_context() {
        let engine = DecisionEngine::new(0.85);
        
        let rule = DecisionRule {
            decision_type: DecisionType::UiLayout,
            condition: RuleCondition::ContextMatch(ContextCondition::LowBattery),
            value: "power_save".to_string(),
            confidence: 0.85,
            reasoning: "Test".to_string(),
        };
        
        let low_battery_context = RequestContext {
            location: None,
            current_app: None,
            recent_apps: vec![],
            time_of_day: TimeOfDay::Morning,
            battery_level: 15,
            is_moving: false,
            ambient_noise: NoiseLevel::Normal,
            user_state: UserState::Active,
        };
        
        let normal_context = RequestContext {
            battery_level: 80,
            ..low_battery_context.clone()
        };
        
        assert!(engine.rule_matches(&rule, "", &low_battery_context));
        assert!(!engine.rule_matches(&rule, "", &normal_context));
    }
    
    #[test]
    fn test_make_decisions() {
        let mut engine = DecisionEngine::new(0.85);
        
        let request = UserRequest {
            id: 1,
            input: RequestInput::Voice("open the browser".to_string()),
            context: RequestContext {
                location: None,
                current_app: None,
                recent_apps: vec![],
                time_of_day: TimeOfDay::Morning,
                battery_level: 80,
                is_moving: false,
                ambient_noise: NoiseLevel::Normal,
                user_state: UserState::Active,
            },
            timestamp: Instant::now(),
        };
        
        let route = super::super::orchestrator::RequestRoute {
            primary_handler: super::super::orchestrator::HandlerType::AppLauncher,
            secondary_handlers: vec![],
            requires_confirmation: false,
            estimated_time_ms: 100,
            resource_requirements: super::super::orchestrator::ResourceRequirements::default(),
        };
        
        let decisions = engine.decide(&request, &route, &[], &[]);
        
        assert!(!decisions.is_empty());
        assert!(decisions.iter().any(|d| d.decision_type == DecisionType::AppSelection));
    }
    
    #[test]
    fn test_auto_execute_threshold() {
        let engine = DecisionEngine::new(0.85);
        
        let high_conf_decisions = vec![
            Decision {
                decision_type: DecisionType::AppSelection,
                value: "browser".to_string(),
                confidence: 0.9,
                reasoning: "Test".to_string(),
            }
        ];
        
        let low_conf_decisions = vec![
            Decision {
                decision_type: DecisionType::AppSelection,
                value: "browser".to_string(),
                confidence: 0.7,
                reasoning: "Test".to_string(),
            }
        ];
        
        assert!(engine.should_auto_execute(&high_conf_decisions));
        assert!(!engine.should_auto_execute(&low_conf_decisions));
    }
    
    #[test]
    fn test_decision_stats() {
        let mut stats = DecisionStats::new();
        
        stats.accepted = 8;
        stats.rejected = 2;
        
        assert!((stats.acceptance_rate() - 0.8).abs() < 0.01);
    }
}
