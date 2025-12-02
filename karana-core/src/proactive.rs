//! Proactive Assistant - Generates unprompted, contextual suggestions
//!
//! Combines context awareness, pattern learning, and memory to provide
//! helpful suggestions before the user even asks.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::context::{ContextEngine, ContextSnapshot, TimeOfDay, Location, Activity};
use crate::learning::LearningSystem;
use crate::memory::MemorySystem;

/// Types of proactive suggestions
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionType {
    Routine,        // Based on time patterns ("Good morning! Check balance?")
    Followup,       // Based on recent action ("Also want to stake some?")
    Reminder,       // Time-based reminder ("Your timer is about to expire")
    Warning,        // System warning ("Battery low")
    Opportunity,    // Good opportunity ("Gas fees are low right now")
    Social,         // Social interaction ("You haven't staked in a while")
}

/// A proactive suggestion from the assistant
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub id: u64,
    pub suggestion_type: SuggestionType,
    pub message: String,
    pub action: Option<String>,  // Suggested action if accepted
    pub priority: SuggestionPriority,
    pub expires_at: Option<Instant>,
    pub context: String,  // Why we're suggesting this
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SuggestionPriority {
    Low,
    Medium,
    High,
    Urgent,
}

/// Configuration for proactive behavior
#[derive(Debug, Clone)]
pub struct ProactiveConfig {
    pub enabled: bool,
    pub min_confidence: f32,
    pub cooldown: Duration,
    pub max_suggestions_per_hour: u32,
    pub suggestion_types: Vec<SuggestionType>,
    pub quiet_hours: Option<(u32, u32)>,  // (start_hour, end_hour)
}

impl Default for ProactiveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_confidence: 0.6,
            cooldown: Duration::from_secs(120),
            max_suggestions_per_hour: 5,
            suggestion_types: vec![
                SuggestionType::Routine,
                SuggestionType::Followup,
                SuggestionType::Reminder,
                SuggestionType::Warning,
            ],
            quiet_hours: Some((23, 7)),  // 11pm to 7am
        }
    }
}

/// The proactive assistant that generates suggestions
pub struct ProactiveAssistant {
    config: ProactiveConfig,
    
    // Tracking
    suggestion_counter: u64,
    last_suggestion_time: Option<Instant>,
    suggestions_this_hour: u32,
    hour_start: Instant,
    
    // Declined suggestions (to avoid repeating)
    declined_suggestions: HashMap<String, u32>,
    
    // Pending suggestions
    pending: Vec<Suggestion>,
}

impl ProactiveAssistant {
    pub fn new() -> Self {
        Self::with_config(ProactiveConfig::default())
    }
    
    pub fn with_config(config: ProactiveConfig) -> Self {
        Self {
            config,
            suggestion_counter: 0,
            last_suggestion_time: None,
            suggestions_this_hour: 0,
            hour_start: Instant::now(),
            declined_suggestions: HashMap::new(),
            pending: Vec::new(),
        }
    }
    
    /// Check if we should be quiet right now
    fn is_quiet_hours(&self, hour: u32) -> bool {
        if let Some((start, end)) = self.config.quiet_hours {
            if start < end {
                hour >= start && hour < end
            } else {
                // Wraps around midnight
                hour >= start || hour < end
            }
        } else {
            false
        }
    }
    
    /// Check if we can make a suggestion right now
    fn can_suggest(&self) -> bool {
        if !self.config.enabled {
            return false;
        }
        
        // Check cooldown
        if let Some(last) = self.last_suggestion_time {
            if last.elapsed() < self.config.cooldown {
                return false;
            }
        }
        
        // Check hourly limit
        if self.hour_start.elapsed() > Duration::from_secs(3600) {
            // Reset counter (will be done on next suggestion)
        } else if self.suggestions_this_hour >= self.config.max_suggestions_per_hour {
            return false;
        }
        
        true
    }
    
    /// Generate suggestions based on current state
    pub fn generate_suggestions(
        &mut self,
        context: &ContextEngine,
        learning: &LearningSystem,
        memory: &MemorySystem,
    ) -> Vec<Suggestion> {
        if !self.can_suggest() {
            return Vec::new();
        }
        
        let snapshot = context.snapshot();
        let hour = match snapshot.time_of_day {
            TimeOfDay::EarlyMorning => 6,
            TimeOfDay::Morning => 9,
            TimeOfDay::Afternoon => 14,
            TimeOfDay::Evening => 18,
            TimeOfDay::Night => 22,
            TimeOfDay::LateNight => 2,
        };
        
        if self.is_quiet_hours(hour) {
            return Vec::new();
        }
        
        let mut suggestions = Vec::new();
        
        // 1. Routine suggestions (time-based patterns)
        if self.config.suggestion_types.contains(&SuggestionType::Routine) {
            if let Some(routine) = self.generate_routine_suggestion(context, learning, &snapshot) {
                suggestions.push(routine);
            }
        }
        
        // 2. Follow-up suggestions (based on recent actions)
        if self.config.suggestion_types.contains(&SuggestionType::Followup) {
            if let Some(followup) = self.generate_followup_suggestion(memory, &snapshot) {
                suggestions.push(followup);
            }
        }
        
        // 3. Warning suggestions (system state)
        if self.config.suggestion_types.contains(&SuggestionType::Warning) {
            if let Some(warning) = self.generate_warning_suggestion(&snapshot) {
                suggestions.push(warning);
            }
        }
        
        // 4. Social suggestions (engagement)
        if self.config.suggestion_types.contains(&SuggestionType::Social) {
            if let Some(social) = self.generate_social_suggestion(context, learning) {
                suggestions.push(social);
            }
        }
        
        // Filter out recently declined
        suggestions.retain(|s| {
            if let Some(action) = &s.action {
                let decline_count = self.declined_suggestions.get(action).unwrap_or(&0);
                *decline_count < 3  // Stop suggesting after 3 declines
            } else {
                true
            }
        });
        
        // Sort by priority
        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Take top suggestion only (to avoid overwhelming)
        suggestions.truncate(1);
        
        if !suggestions.is_empty() {
            self.record_suggestion_made();
        }
        
        suggestions
    }
    
    fn generate_routine_suggestion(
        &mut self,
        context: &ContextEngine,
        learning: &LearningSystem,
        snapshot: &ContextSnapshot,
    ) -> Option<Suggestion> {
        // Get predictions from context engine
        let predictions = context.predict_next_actions(3);
        
        if let Some((action, score)) = predictions.first() {
            if *score > 10.0 {  // High confidence threshold
                self.suggestion_counter += 1;
                
                let message = format!(
                    "Good {}! Based on your patterns, would you like to {}?",
                    snapshot.time_of_day.description(),
                    humanize_action(action)
                );
                
                return Some(Suggestion {
                    id: self.suggestion_counter,
                    suggestion_type: SuggestionType::Routine,
                    message,
                    action: Some(action.clone()),
                    priority: SuggestionPriority::Low,
                    expires_at: Some(Instant::now() + Duration::from_secs(300)),
                    context: format!("Pattern score: {:.1}", score),
                });
            }
        }
        
        // Morning greeting with balance check
        if snapshot.time_of_day == TimeOfDay::Morning && snapshot.session_duration < Duration::from_secs(60) {
            self.suggestion_counter += 1;
            
            return Some(Suggestion {
                id: self.suggestion_counter,
                suggestion_type: SuggestionType::Routine,
                message: "Good morning! Would you like a quick overview of your balance and pending transactions?".to_string(),
                action: Some("check_balance".to_string()),
                priority: SuggestionPriority::Low,
                expires_at: Some(Instant::now() + Duration::from_secs(120)),
                context: "Morning session start".to_string(),
            });
        }
        
        None
    }
    
    fn generate_followup_suggestion(
        &mut self,
        memory: &MemorySystem,
        _snapshot: &ContextSnapshot,
    ) -> Option<Suggestion> {
        // Check if there's a natural followup to suggest
        if let Some(followup_msg) = memory.suggest_followup() {
            self.suggestion_counter += 1;
            
            let topic = memory.working.current_topic();
            let action = match topic {
                crate::memory::Topic::Balance => Some("check_transactions".to_string()),
                crate::memory::Topic::Transactions => Some("setup_recurring".to_string()),
                crate::memory::Topic::Staking => Some("check_rewards".to_string()),
                _ => None,
            };
            
            return Some(Suggestion {
                id: self.suggestion_counter,
                suggestion_type: SuggestionType::Followup,
                message: followup_msg,
                action,
                priority: SuggestionPriority::Low,
                expires_at: Some(Instant::now() + Duration::from_secs(60)),
                context: format!("Topic depth: {}", memory.working.topic_depth()),
            });
        }
        
        None
    }
    
    fn generate_warning_suggestion(&mut self, snapshot: &ContextSnapshot) -> Option<Suggestion> {
        // Battery warning
        if snapshot.environment.battery_level < 15 && !snapshot.environment.is_charging {
            self.suggestion_counter += 1;
            
            return Some(Suggestion {
                id: self.suggestion_counter,
                suggestion_type: SuggestionType::Warning,
                message: format!(
                    "⚠️ Battery at {}%. Consider connecting to power soon.",
                    snapshot.environment.battery_level
                ),
                action: Some("enable_power_saving".to_string()),
                priority: SuggestionPriority::High,
                expires_at: None,  // Warnings don't expire
                context: "Low battery".to_string(),
            });
        }
        
        // Network warning
        if !snapshot.environment.wifi_connected {
            self.suggestion_counter += 1;
            
            return Some(Suggestion {
                id: self.suggestion_counter,
                suggestion_type: SuggestionType::Warning,
                message: "📡 No WiFi connection. Blockchain sync may be delayed.".to_string(),
                action: None,
                priority: SuggestionPriority::Medium,
                expires_at: None,
                context: "No WiFi".to_string(),
            });
        }
        
        None
    }
    
    fn generate_social_suggestion(
        &mut self,
        context: &ContextEngine,
        _learning: &LearningSystem,
    ) -> Option<Suggestion> {
        let snapshot = context.snapshot();
        
        // If user has been idle for a while, offer help
        if snapshot.session_duration > Duration::from_secs(600) && snapshot.last_actions.is_empty() {
            self.suggestion_counter += 1;
            
            return Some(Suggestion {
                id: self.suggestion_counter,
                suggestion_type: SuggestionType::Social,
                message: "Need any help? Just say 'help' to see what I can do.".to_string(),
                action: Some("show_help".to_string()),
                priority: SuggestionPriority::Low,
                expires_at: Some(Instant::now() + Duration::from_secs(300)),
                context: "User seems idle".to_string(),
            });
        }
        
        None
    }
    
    fn record_suggestion_made(&mut self) {
        self.last_suggestion_time = Some(Instant::now());
        
        // Reset hourly counter if needed
        if self.hour_start.elapsed() > Duration::from_secs(3600) {
            self.hour_start = Instant::now();
            self.suggestions_this_hour = 0;
        }
        
        self.suggestions_this_hour += 1;
    }
    
    /// Record that user accepted a suggestion
    pub fn accepted(&mut self, suggestion_id: u64) {
        // Remove from pending
        self.pending.retain(|s| s.id != suggestion_id);
    }
    
    /// Record that user declined a suggestion
    pub fn declined(&mut self, suggestion_id: u64, action: Option<&str>) {
        self.pending.retain(|s| s.id != suggestion_id);
        
        if let Some(action) = action {
            *self.declined_suggestions.entry(action.to_string()).or_insert(0) += 1;
        }
    }
    
    /// Get pending suggestions
    pub fn pending_suggestions(&self) -> &[Suggestion] {
        &self.pending
    }
    
    /// Clear expired suggestions
    pub fn clear_expired(&mut self) {
        let now = Instant::now();
        self.pending.retain(|s| {
            if let Some(expires) = s.expires_at {
                expires > now
            } else {
                true
            }
        });
    }
    
    /// Update config
    pub fn configure(&mut self, config: ProactiveConfig) {
        self.config = config;
    }
    
    /// Enable/disable proactive suggestions
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }
    
    /// Stats
    pub fn stats(&self) -> ProactiveStats {
        ProactiveStats {
            total_suggestions: self.suggestion_counter,
            suggestions_this_hour: self.suggestions_this_hour,
            declined_actions: self.declined_suggestions.len(),
            pending_count: self.pending.len(),
            enabled: self.config.enabled,
        }
    }
}

impl Default for ProactiveAssistant {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about proactive behavior
#[derive(Debug, Clone)]
pub struct ProactiveStats {
    pub total_suggestions: u64,
    pub suggestions_this_hour: u32,
    pub declined_actions: usize,
    pub pending_count: usize,
    pub enabled: bool,
}

/// Convert action ID to human-readable description
fn humanize_action(action: &str) -> &str {
    match action {
        "check_balance" => "check your balance",
        "stake_tokens" => "stake some tokens",
        "unstake_tokens" => "unstake tokens",
        "send_tokens" => "send tokens",
        "capture_photo" => "take a photo",
        "set_timer" => "set a timer",
        "show_notifications" => "check notifications",
        "check_transactions" => "view recent transactions",
        "check_rewards" => "see your staking rewards",
        "enable_power_saving" => "enable power saving mode",
        _ => action,
    }
}

/// Intelligent assistant that combines all systems
pub struct IntelligentAssistant {
    pub context: ContextEngine,
    pub learning: LearningSystem,
    pub memory: MemorySystem,
    pub proactive: ProactiveAssistant,
}

impl IntelligentAssistant {
    pub fn new() -> Self {
        Self {
            context: ContextEngine::new(),
            learning: LearningSystem::new(),
            memory: MemorySystem::new(),
            proactive: ProactiveAssistant::new(),
        }
    }
    
    /// Process user input with full intelligence
    pub fn process(&mut self, input: &str) -> ProcessedQuery {
        // 1. Process through memory (resolve anaphora, extract facts)
        let processed_input = self.memory.process_input(input);
        
        // 2. Check if learning system knows this phrase
        let learned_action = self.learning.enhance_input(&processed_input.resolved);
        
        // 3. Build full context
        let context_snapshot = self.context.snapshot();
        
        ProcessedQuery {
            original_input: input.to_string(),
            resolved_input: processed_input.resolved,
            learned_action,
            context: context_snapshot,
            personalization: self.memory.personalized_greeting(),
            extracted_facts: processed_input.extracted_facts,
        }
    }
    
    /// Record completion of an action
    pub fn record_action(&mut self, input: &str, action: &str, entities: HashMap<String, String>, response: &str, success: bool) {
        self.context.record_action(action);
        self.learning.record_command(input, action, success);
        self.memory.record_turn(input, action, entities, response, success);
    }
    
    /// Get proactive suggestions
    pub fn get_suggestions(&mut self) -> Vec<Suggestion> {
        self.proactive.generate_suggestions(&self.context, &self.learning, &self.memory)
    }
    
    /// Describe what the assistant has learned
    pub fn describe_intelligence(&self) -> String {
        let mut desc = String::new();
        
        desc.push_str("🧠 Intelligence Report\n");
        desc.push_str("═════════════════════\n\n");
        
        // Context learning
        desc.push_str(&self.context.describe_learning());
        desc.push_str("\n\n");
        
        // User profile
        desc.push_str(&self.learning.profile().describe());
        
        // Memory stats
        let turns = self.memory.working.last_turns(0).len();
        let facts = self.memory.long_term.all_facts().len();
        desc.push_str(&format!(
            "\n💭 Memory: {} conversation turns, {} stored facts\n",
            turns, facts
        ));
        
        // Proactive stats
        let proactive = self.proactive.stats();
        desc.push_str(&format!(
            "🤖 Proactive: {} total suggestions ({} declined patterns)\n",
            proactive.total_suggestions, proactive.declined_actions
        ));
        
        desc
    }
}

impl Default for IntelligentAssistant {
    fn default() -> Self {
        Self::new()
    }
}

/// Fully processed query with intelligence
#[derive(Debug, Clone)]
pub struct ProcessedQuery {
    pub original_input: String,
    pub resolved_input: String,
    pub learned_action: Option<String>,
    pub context: ContextSnapshot,
    pub personalization: String,
    pub extracted_facts: Vec<(String, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_proactive_assistant_creation() {
        let assistant = ProactiveAssistant::new();
        let stats = assistant.stats();
        
        assert_eq!(stats.total_suggestions, 0);
        assert!(stats.enabled);
    }
    
    #[test]
    fn test_suggestion_priority_ordering() {
        assert!(SuggestionPriority::Urgent > SuggestionPriority::High);
        assert!(SuggestionPriority::High > SuggestionPriority::Medium);
        assert!(SuggestionPriority::Medium > SuggestionPriority::Low);
    }
    
    #[test]
    fn test_intelligent_assistant() {
        let mut assistant = IntelligentAssistant::new();
        
        // Process first query
        let processed = assistant.process("check my balance");
        assert_eq!(processed.original_input, "check my balance");
        
        // Record the action
        assistant.record_action(
            "check my balance",
            "check_balance",
            HashMap::new(),
            "Your balance is 1000 KARA",
            true
        );
        
        // Check that it learned
        let report = assistant.describe_intelligence();
        assert!(report.contains("Intelligence Report"));
    }
    
    #[test]
    fn test_humanize_action() {
        assert_eq!(humanize_action("check_balance"), "check your balance");
        assert_eq!(humanize_action("stake_tokens"), "stake some tokens");
        assert_eq!(humanize_action("unknown_action"), "unknown_action");
    }
    
    #[test]
    fn test_quiet_hours() {
        let config = ProactiveConfig {
            quiet_hours: Some((22, 7)),  // 10pm to 7am
            ..Default::default()
        };
        
        let assistant = ProactiveAssistant::with_config(config);
        
        assert!(assistant.is_quiet_hours(23));
        assert!(assistant.is_quiet_hours(3));
        assert!(!assistant.is_quiet_hours(12));
        assert!(!assistant.is_quiet_hours(20));
    }
    
    #[test]
    fn test_declined_suggestion_tracking() {
        let mut assistant = ProactiveAssistant::new();
        
        assistant.declined(1, Some("check_balance"));
        assistant.declined(2, Some("check_balance"));
        assistant.declined(3, Some("check_balance"));
        
        // Should have 3 declines for check_balance
        assert_eq!(assistant.declined_suggestions.get("check_balance"), Some(&3));
    }
}
