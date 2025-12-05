//! Smart Router
//!
//! Intelligently routes user requests to the appropriate handlers
//! based on intent classification, context, and learned preferences.

use super::*;
use super::orchestrator::{RequestRoute, HandlerType, ResourceRequirements};
use std::collections::HashMap;

/// Smart router for request routing
pub struct SmartRouter {
    /// Intent classifiers
    classifiers: Vec<IntentClassifier>,
    /// Handler capabilities
    handler_capabilities: HashMap<HandlerType, HandlerCapabilities>,
    /// Routing rules
    rules: Vec<RoutingRule>,
    /// Learned preferences
    preferences: RoutingPreferences,
    /// Routing statistics
    stats: RoutingStats,
}

impl SmartRouter {
    pub fn new() -> Self {
        let mut router = Self {
            classifiers: Vec::new(),
            handler_capabilities: HashMap::new(),
            rules: Vec::new(),
            preferences: RoutingPreferences::new(),
            stats: RoutingStats::new(),
        };
        router.initialize();
        router
    }
    
    fn initialize(&mut self) {
        // Add default classifiers
        self.classifiers = vec![
            IntentClassifier::keyword_based(),
            IntentClassifier::pattern_based(),
        ];
        
        // Set up handler capabilities
        self.setup_handler_capabilities();
        
        // Add default routing rules
        self.setup_default_rules();
    }
    
    fn setup_handler_capabilities(&mut self) {
        self.handler_capabilities.insert(HandlerType::VoiceAssistant, HandlerCapabilities {
            intents: vec!["question".to_string(), "conversation".to_string(), "command".to_string()],
            requires_network: false,
            max_latency_ms: 500,
            supports_streaming: true,
        });
        
        self.handler_capabilities.insert(HandlerType::AppLauncher, HandlerCapabilities {
            intents: vec!["open".to_string(), "launch".to_string(), "start".to_string()],
            requires_network: false,
            max_latency_ms: 100,
            supports_streaming: false,
        });
        
        self.handler_capabilities.insert(HandlerType::Navigation, HandlerCapabilities {
            intents: vec!["navigate".to_string(), "directions".to_string(), "find".to_string(), "locate".to_string()],
            requires_network: true,
            max_latency_ms: 1000,
            supports_streaming: true,
        });
        
        self.handler_capabilities.insert(HandlerType::Communication, HandlerCapabilities {
            intents: vec!["call".to_string(), "message".to_string(), "email".to_string(), "send".to_string()],
            requires_network: true,
            max_latency_ms: 200,
            supports_streaming: false,
        });
        
        self.handler_capabilities.insert(HandlerType::Media, HandlerCapabilities {
            intents: vec!["play".to_string(), "pause".to_string(), "music".to_string(), "video".to_string()],
            requires_network: false,
            max_latency_ms: 100,
            supports_streaming: true,
        });
        
        self.handler_capabilities.insert(HandlerType::SystemSettings, HandlerCapabilities {
            intents: vec!["settings".to_string(), "configure".to_string(), "change".to_string(), "set".to_string()],
            requires_network: false,
            max_latency_ms: 100,
            supports_streaming: false,
        });
        
        self.handler_capabilities.insert(HandlerType::Blockchain, HandlerCapabilities {
            intents: vec!["send".to_string(), "transfer".to_string(), "wallet".to_string(), "balance".to_string(), "stake".to_string()],
            requires_network: true,
            max_latency_ms: 5000,
            supports_streaming: false,
        });
        
        self.handler_capabilities.insert(HandlerType::AR, HandlerCapabilities {
            intents: vec!["show".to_string(), "display".to_string(), "overlay".to_string(), "anchor".to_string()],
            requires_network: false,
            max_latency_ms: 16, // Real-time
            supports_streaming: true,
        });
        
        self.handler_capabilities.insert(HandlerType::Productivity, HandlerCapabilities {
            intents: vec!["note".to_string(), "remind".to_string(), "calendar".to_string(), "timer".to_string(), "todo".to_string()],
            requires_network: false,
            max_latency_ms: 200,
            supports_streaming: false,
        });
    }
    
    fn setup_default_rules(&mut self) {
        // High priority rules
        self.rules.push(RoutingRule {
            priority: 100,
            condition: RuleCondition::KeywordMatch(vec!["emergency".to_string(), "help".to_string(), "urgent".to_string()]),
            handler: HandlerType::Communication,
            requires_confirmation: false,
        });
        
        // App launching
        self.rules.push(RoutingRule {
            priority: 80,
            condition: RuleCondition::KeywordMatch(vec!["open".to_string(), "launch".to_string(), "start".to_string()]),
            handler: HandlerType::AppLauncher,
            requires_confirmation: false,
        });
        
        // Navigation
        self.rules.push(RoutingRule {
            priority: 70,
            condition: RuleCondition::KeywordMatch(vec!["navigate".to_string(), "directions".to_string(), "how do i get".to_string()]),
            handler: HandlerType::Navigation,
            requires_confirmation: false,
        });
        
        // Financial (requires confirmation)
        self.rules.push(RoutingRule {
            priority: 60,
            condition: RuleCondition::KeywordMatch(vec!["send".to_string(), "transfer".to_string(), "pay".to_string()]),
            handler: HandlerType::Blockchain,
            requires_confirmation: true,
        });
        
        // Communication
        self.rules.push(RoutingRule {
            priority: 50,
            condition: RuleCondition::KeywordMatch(vec!["call".to_string(), "message".to_string(), "text".to_string()]),
            handler: HandlerType::Communication,
            requires_confirmation: false,
        });
        
        // Media
        self.rules.push(RoutingRule {
            priority: 40,
            condition: RuleCondition::KeywordMatch(vec!["play".to_string(), "music".to_string(), "song".to_string()]),
            handler: HandlerType::Media,
            requires_confirmation: false,
        });
    }
    
    /// Route a request to the appropriate handler
    pub fn route(&mut self, request: &UserRequest) -> RequestRoute {
        // 1. Classify the intent
        let intents = self.classify_intent(request);
        
        // 2. Check rules
        if let Some(route) = self.apply_rules(request, &intents) {
            self.stats.record_route(&route.primary_handler);
            return route;
        }
        
        // 3. Score handlers based on capabilities
        let scores = self.score_handlers(&intents, &request.context);
        
        // 4. Select best handler
        let primary = scores.iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(h, _)| *h)
            .unwrap_or(HandlerType::VoiceAssistant);
        
        // 5. Find secondary handlers
        let secondary: Vec<HandlerType> = scores.iter()
            .filter(|(h, s)| *h != primary && *s > 0.3)
            .map(|(h, _)| *h)
            .take(2)
            .collect();
        
        // 6. Build resource requirements
        let requirements = self.build_requirements(&primary, &secondary);
        
        let route = RequestRoute {
            primary_handler: primary,
            secondary_handlers: secondary,
            requires_confirmation: self.requires_confirmation(&primary, request),
            estimated_time_ms: self.estimate_time(&primary),
            resource_requirements: requirements,
        };
        
        self.stats.record_route(&route.primary_handler);
        route
    }
    
    /// Classify the intent of a request
    fn classify_intent(&self, request: &UserRequest) -> Vec<(String, f32)> {
        let text = match &request.input {
            RequestInput::Text(t) | RequestInput::Voice(t) => t.clone(),
            _ => String::new(),
        };
        
        let mut intents = Vec::new();
        
        for classifier in &self.classifiers {
            intents.extend(classifier.classify(&text));
        }
        
        // Deduplicate and keep highest confidence
        let mut seen: HashMap<String, f32> = HashMap::new();
        for (intent, conf) in intents {
            seen.entry(intent)
                .and_modify(|c| *c = c.max(conf))
                .or_insert(conf);
        }
        
        seen.into_iter().collect()
    }
    
    /// Apply routing rules
    fn apply_rules(&self, request: &UserRequest, intents: &[(String, f32)]) -> Option<RequestRoute> {
        let text = match &request.input {
            RequestInput::Text(t) | RequestInput::Voice(t) => t.to_lowercase(),
            _ => String::new(),
        };
        
        // Sort rules by priority
        let mut sorted_rules: Vec<_> = self.rules.iter().collect();
        sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        for rule in sorted_rules {
            if rule.matches(&text, intents) {
                return Some(RequestRoute {
                    primary_handler: rule.handler,
                    secondary_handlers: vec![],
                    requires_confirmation: rule.requires_confirmation,
                    estimated_time_ms: self.estimate_time(&rule.handler),
                    resource_requirements: ResourceRequirements::default(),
                });
            }
        }
        
        None
    }
    
    /// Score handlers based on intents and context
    fn score_handlers(&self, intents: &[(String, f32)], context: &RequestContext) -> Vec<(HandlerType, f32)> {
        let mut scores: Vec<(HandlerType, f32)> = Vec::new();
        
        for (handler, caps) in &self.handler_capabilities {
            let mut score = 0.0;
            
            // Score based on intent match
            for (intent, conf) in intents {
                if caps.intents.iter().any(|i| intent.contains(i) || i.contains(intent)) {
                    score += conf;
                }
            }
            
            // Adjust for context
            if context.battery_level < 20 && caps.requires_network {
                score *= 0.5; // Penalize network ops on low battery
            }
            
            // Apply preferences
            score *= self.preferences.get_preference(handler);
            
            scores.push((*handler, score));
        }
        
        scores
    }
    
    /// Build resource requirements
    fn build_requirements(&self, primary: &HandlerType, secondary: &[HandlerType]) -> ResourceRequirements {
        let mut req = ResourceRequirements::default();
        
        if let Some(caps) = self.handler_capabilities.get(primary) {
            req.network_required = caps.requires_network;
        }
        
        // Check if any handler needs GPU
        if *primary == HandlerType::AR || secondary.contains(&HandlerType::AR) {
            req.gpu_required = true;
        }
        
        req
    }
    
    /// Check if confirmation is required
    fn requires_confirmation(&self, handler: &HandlerType, request: &UserRequest) -> bool {
        match handler {
            HandlerType::Blockchain => true, // Always confirm financial
            HandlerType::Communication => {
                // Confirm if sending to unknown contact
                if let RequestInput::Voice(text) | RequestInput::Text(text) = &request.input {
                    text.contains("send") || text.contains("call")
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    /// Estimate handling time
    fn estimate_time(&self, handler: &HandlerType) -> u64 {
        self.handler_capabilities.get(handler)
            .map(|c| c.max_latency_ms)
            .unwrap_or(500)
    }
    
    /// Update from user feedback
    pub fn update_from_feedback(&mut self, feedback: &UserFeedback) {
        // Update preferences based on feedback
        if feedback.accepted {
            // Would need to track which handler was used for this request
            // For now, just update stats
            self.stats.successful_routes += 1;
        }
    }
    
    /// Get routing statistics
    pub fn stats(&self) -> &RoutingStats {
        &self.stats
    }
}

impl Default for SmartRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler capabilities
#[derive(Debug, Clone)]
pub struct HandlerCapabilities {
    pub intents: Vec<String>,
    pub requires_network: bool,
    pub max_latency_ms: u64,
    pub supports_streaming: bool,
}

/// Routing rule
#[derive(Debug, Clone)]
pub struct RoutingRule {
    pub priority: u32,
    pub condition: RuleCondition,
    pub handler: HandlerType,
    pub requires_confirmation: bool,
}

impl RoutingRule {
    fn matches(&self, text: &str, _intents: &[(String, f32)]) -> bool {
        match &self.condition {
            RuleCondition::KeywordMatch(keywords) => {
                keywords.iter().any(|kw| text.contains(kw))
            }
            RuleCondition::IntentMatch(intent, min_conf) => {
                _intents.iter().any(|(i, c)| i == intent && c >= min_conf)
            }
            RuleCondition::ContextMatch(ctx) => {
                // Would need context to check
                false
            }
            RuleCondition::Combined(conditions) => {
                conditions.iter().all(|c| {
                    let rule = RoutingRule {
                        priority: 0,
                        condition: c.clone(),
                        handler: self.handler,
                        requires_confirmation: false,
                    };
                    rule.matches(text, _intents)
                })
            }
        }
    }
}

/// Rule condition
#[derive(Debug, Clone)]
pub enum RuleCondition {
    KeywordMatch(Vec<String>),
    IntentMatch(String, f32),
    ContextMatch(String),
    Combined(Vec<RuleCondition>),
}

/// Intent classifier
pub struct IntentClassifier {
    classification_type: ClassificationType,
    keywords: HashMap<String, Vec<String>>,
}

enum ClassificationType {
    Keyword,
    Pattern,
    ML,
}

impl IntentClassifier {
    fn keyword_based() -> Self {
        let mut keywords = HashMap::new();
        
        keywords.insert("open".to_string(), vec!["open".to_string(), "launch".to_string(), "start".to_string(), "run".to_string()]);
        keywords.insert("navigate".to_string(), vec!["navigate".to_string(), "directions".to_string(), "route".to_string(), "go to".to_string()]);
        keywords.insert("play".to_string(), vec!["play".to_string(), "music".to_string(), "song".to_string(), "video".to_string()]);
        keywords.insert("send".to_string(), vec!["send".to_string(), "transfer".to_string(), "pay".to_string()]);
        keywords.insert("call".to_string(), vec!["call".to_string(), "phone".to_string(), "dial".to_string()]);
        keywords.insert("message".to_string(), vec!["message".to_string(), "text".to_string(), "sms".to_string()]);
        keywords.insert("settings".to_string(), vec!["settings".to_string(), "configure".to_string(), "preferences".to_string()]);
        keywords.insert("question".to_string(), vec!["what".to_string(), "who".to_string(), "when".to_string(), "where".to_string(), "why".to_string(), "how".to_string()]);
        
        Self {
            classification_type: ClassificationType::Keyword,
            keywords,
        }
    }
    
    fn pattern_based() -> Self {
        Self {
            classification_type: ClassificationType::Pattern,
            keywords: HashMap::new(),
        }
    }
    
    fn classify(&self, text: &str) -> Vec<(String, f32)> {
        let text_lower = text.to_lowercase();
        let mut results = Vec::new();
        
        match self.classification_type {
            ClassificationType::Keyword => {
                for (intent, kws) in &self.keywords {
                    let matches = kws.iter()
                        .filter(|kw| text_lower.contains(&kw.to_lowercase()))
                        .count();
                    
                    if matches > 0 {
                        let confidence = (matches as f32 / kws.len() as f32).min(1.0);
                        results.push((intent.clone(), confidence));
                    }
                }
            }
            ClassificationType::Pattern => {
                // Pattern-based classification
                if text_lower.contains("?") {
                    results.push(("question".to_string(), 0.6));
                }
                if text_lower.starts_with("please") || text_lower.starts_with("can you") {
                    results.push(("command".to_string(), 0.5));
                }
            }
            ClassificationType::ML => {
                // Would use ML model
            }
        }
        
        results
    }
}

/// User routing preferences
pub struct RoutingPreferences {
    handler_weights: HashMap<HandlerType, f32>,
}

impl RoutingPreferences {
    fn new() -> Self {
        let mut weights = HashMap::new();
        
        // Default equal weights
        for handler in [
            HandlerType::VoiceAssistant,
            HandlerType::AppLauncher,
            HandlerType::Navigation,
            HandlerType::Communication,
            HandlerType::Media,
            HandlerType::SystemSettings,
            HandlerType::Blockchain,
            HandlerType::AR,
            HandlerType::Productivity,
        ] {
            weights.insert(handler, 1.0);
        }
        
        Self { handler_weights: weights }
    }
    
    fn get_preference(&self, handler: &HandlerType) -> f32 {
        *self.handler_weights.get(handler).unwrap_or(&1.0)
    }
    
    fn update_preference(&mut self, handler: &HandlerType, delta: f32) {
        if let Some(weight) = self.handler_weights.get_mut(handler) {
            *weight = (*weight + delta).clamp(0.1, 2.0);
        }
    }
}

/// Routing statistics
#[derive(Debug, Clone, Default)]
pub struct RoutingStats {
    pub total_routes: u64,
    pub successful_routes: u64,
    pub routes_by_handler: HashMap<HandlerType, u64>,
}

impl RoutingStats {
    fn new() -> Self {
        Self::default()
    }
    
    fn record_route(&mut self, handler: &HandlerType) {
        self.total_routes += 1;
        *self.routes_by_handler.entry(*handler).or_insert(0) += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_router_creation() {
        let router = SmartRouter::new();
        assert!(!router.handler_capabilities.is_empty());
    }
    
    #[test]
    fn test_intent_classification() {
        let classifier = IntentClassifier::keyword_based();
        
        let results = classifier.classify("open the browser");
        assert!(results.iter().any(|(i, _)| i == "open"));
        
        let results = classifier.classify("play some music");
        assert!(results.iter().any(|(i, _)| i == "play"));
    }
    
    #[test]
    fn test_routing() {
        let mut router = SmartRouter::new();
        
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
        
        let route = router.route(&request);
        assert_eq!(route.primary_handler, HandlerType::AppLauncher);
    }
    
    #[test]
    fn test_financial_requires_confirmation() {
        let mut router = SmartRouter::new();
        
        let request = UserRequest {
            id: 1,
            input: RequestInput::Voice("send 100 dollars to alice".to_string()),
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
        
        let route = router.route(&request);
        assert!(route.requires_confirmation);
    }
    
    #[test]
    fn test_routing_stats() {
        let mut stats = RoutingStats::new();
        
        stats.record_route(&HandlerType::AppLauncher);
        stats.record_route(&HandlerType::AppLauncher);
        stats.record_route(&HandlerType::Navigation);
        
        assert_eq!(stats.total_routes, 3);
        assert_eq!(*stats.routes_by_handler.get(&HandlerType::AppLauncher).unwrap(), 2);
    }
}
