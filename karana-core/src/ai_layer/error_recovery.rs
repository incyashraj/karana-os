// Kāraṇa OS - AI Error Recovery
// Handles errors gracefully and suggests recovery options

use std::collections::HashMap;
use crate::ai_layer::intent::ResolvedIntent;
use crate::ai_layer::{AiContext, AiAction, ActionPriority, NoiseLevel};

/// Helper function to create a ShowHelp action
fn show_help_action() -> AiAction {
    AiAction {
        action_type: "show_help".to_string(),
        parameters: HashMap::new(),
        priority: ActionPriority::Normal,
        requires_confirmation: false,
    }
}

/// Helper function to create an EnableOfflineMode action
fn enable_offline_mode_action() -> AiAction {
    AiAction {
        action_type: "enable_offline_mode".to_string(),
        parameters: HashMap::new(),
        priority: ActionPriority::High,
        requires_confirmation: false,
    }
}

/// Helper function to create an OpenSettings action
fn open_settings_action(setting: &str) -> AiAction {
    let mut params = HashMap::new();
    params.insert("setting".to_string(), setting.to_string());
    AiAction {
        action_type: "open_settings".to_string(),
        parameters: params,
        priority: ActionPriority::Normal,
        requires_confirmation: false,
    }
}

/// Error recovery system that helps users when things go wrong
pub struct ErrorRecovery {
    /// Recovery strategies per error type
    strategies: HashMap<ErrorType, Vec<RecoveryStrategy>>,
    /// Error history for pattern detection
    error_history: Vec<ErrorRecord>,
    /// Maximum errors before escalation
    max_consecutive_errors: u32,
    /// Current consecutive error count
    consecutive_errors: u32,
}

/// Types of errors the AI can encounter
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorType {
    /// User intent unclear
    AmbiguousIntent,
    /// Required slot missing
    MissingSlot,
    /// Entity not found
    UnknownEntity,
    /// Action not possible in current state
    ActionNotFeasible,
    /// External service error
    ServiceError,
    /// Permission denied
    PermissionDenied,
    /// Resource not available
    ResourceUnavailable,
    /// Network error
    NetworkError,
    /// Voice recognition error
    VoiceRecognitionError,
    /// Context lost
    ContextLost,
    /// Timeout
    Timeout,
    /// Unknown error
    Unknown,
}

/// A recovery strategy for handling errors
#[derive(Debug, Clone)]
pub struct RecoveryStrategy {
    /// Strategy name
    pub name: String,
    /// Description
    pub description: String,
    /// Priority (higher = try first)
    pub priority: u8,
    /// Suggested prompts
    pub prompts: Vec<String>,
    /// Alternative actions
    pub alternatives: Vec<AlternativeAction>,
    /// Whether to request clarification
    pub needs_clarification: bool,
}

/// An alternative action to suggest
#[derive(Debug, Clone)]
pub struct AlternativeAction {
    /// Action description
    pub description: String,
    /// The action to take
    pub action: AiAction,
    /// Confidence in this alternative
    pub confidence: f32,
}

/// Record of a past error
#[derive(Debug, Clone)]
struct ErrorRecord {
    /// Error type
    error_type: ErrorType,
    /// Context when error occurred
    intent: Option<String>,
    /// Input that caused error
    input: String,
    /// Recovery attempted
    recovery_attempted: Option<String>,
    /// Whether recovery succeeded
    recovered: bool,
}

/// Result of error recovery
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    /// User-facing message
    pub message: String,
    /// Suggested actions
    pub suggestions: Vec<String>,
    /// Should continue conversation
    pub continue_conversation: bool,
    /// Fallback action if any
    pub fallback_action: Option<AiAction>,
    /// Clarification question if needed
    pub clarification: Option<ClarificationRequest>,
}

/// A clarification request
#[derive(Debug, Clone)]
pub struct ClarificationRequest {
    /// The question to ask
    pub question: String,
    /// What we're trying to clarify
    pub clarifying: String,
    /// Suggested options if applicable
    pub options: Vec<String>,
}

impl ErrorRecovery {
    /// Create new error recovery system
    pub fn new() -> Self {
        let mut recovery = Self {
            strategies: HashMap::new(),
            error_history: Vec::new(),
            max_consecutive_errors: 3,
            consecutive_errors: 0,
        };
        recovery.initialize_strategies();
        recovery
    }
    
    /// Initialize recovery strategies for each error type
    fn initialize_strategies(&mut self) {
        // Ambiguous intent strategies
        self.strategies.insert(ErrorType::AmbiguousIntent, vec![
            RecoveryStrategy {
                name: "clarify_intent".to_string(),
                description: "Ask user to clarify their intent".to_string(),
                priority: 10,
                prompts: vec![
                    "Could you tell me more specifically what you'd like to do?".to_string(),
                    "I want to help, but I'm not sure what you mean. Can you rephrase?".to_string(),
                    "Did you mean one of these?".to_string(),
                ],
                alternatives: vec![],
                needs_clarification: true,
            },
            RecoveryStrategy {
                name: "offer_options".to_string(),
                description: "Offer possible interpretations".to_string(),
                priority: 8,
                prompts: vec![
                    "Here are some things I can do. Which would you like?".to_string(),
                ],
                alternatives: vec![
                    AlternativeAction {
                        description: "Show help".to_string(),
                        action: show_help_action(),
                        confidence: 0.7,
                    },
                ],
                needs_clarification: true,
            },
        ]);
        
        // Missing slot strategies
        self.strategies.insert(ErrorType::MissingSlot, vec![
            RecoveryStrategy {
                name: "request_slot".to_string(),
                description: "Ask for the missing information".to_string(),
                priority: 10,
                prompts: vec![
                    "I need a bit more information.".to_string(),
                    "Who would you like me to contact?".to_string(),
                    "Where would you like to go?".to_string(),
                    "When would you like to set this for?".to_string(),
                ],
                alternatives: vec![],
                needs_clarification: true,
            },
            RecoveryStrategy {
                name: "suggest_recent".to_string(),
                description: "Suggest recent values for the slot".to_string(),
                priority: 8,
                prompts: vec![
                    "Would you like to use a recent contact?".to_string(),
                ],
                alternatives: vec![],
                needs_clarification: true,
            },
        ]);
        
        // Unknown entity strategies
        self.strategies.insert(ErrorType::UnknownEntity, vec![
            RecoveryStrategy {
                name: "clarify_entity".to_string(),
                description: "Ask to clarify the entity".to_string(),
                priority: 10,
                prompts: vec![
                    "I don't recognize that name. Could you spell it or say it differently?".to_string(),
                    "I couldn't find that. Do you mean something else?".to_string(),
                ],
                alternatives: vec![],
                needs_clarification: true,
            },
            RecoveryStrategy {
                name: "search_similar".to_string(),
                description: "Search for similar entities".to_string(),
                priority: 8,
                prompts: vec![
                    "I found some similar options. Is one of these correct?".to_string(),
                ],
                alternatives: vec![],
                needs_clarification: true,
            },
        ]);
        
        // Action not feasible strategies
        self.strategies.insert(ErrorType::ActionNotFeasible, vec![
            RecoveryStrategy {
                name: "explain_limitation".to_string(),
                description: "Explain why action isn't possible".to_string(),
                priority: 10,
                prompts: vec![
                    "I can't do that right now. Here's why:".to_string(),
                    "That's not available at the moment.".to_string(),
                ],
                alternatives: vec![],
                needs_clarification: false,
            },
            RecoveryStrategy {
                name: "suggest_alternative".to_string(),
                description: "Suggest an alternative action".to_string(),
                priority: 8,
                prompts: vec![
                    "Instead, I could:".to_string(),
                    "Would you like me to try something else?".to_string(),
                ],
                alternatives: vec![],
                needs_clarification: false,
            },
        ]);
        
        // Service error strategies
        self.strategies.insert(ErrorType::ServiceError, vec![
            RecoveryStrategy {
                name: "retry".to_string(),
                description: "Retry the operation".to_string(),
                priority: 10,
                prompts: vec![
                    "Something went wrong. Let me try again.".to_string(),
                ],
                alternatives: vec![],
                needs_clarification: false,
            },
            RecoveryStrategy {
                name: "manual_retry".to_string(),
                description: "Ask user to try again".to_string(),
                priority: 5,
                prompts: vec![
                    "There was a service issue. Please try again in a moment.".to_string(),
                ],
                alternatives: vec![],
                needs_clarification: false,
            },
        ]);
        
        // Network error strategies
        self.strategies.insert(ErrorType::NetworkError, vec![
            RecoveryStrategy {
                name: "offline_mode".to_string(),
                description: "Suggest offline alternatives".to_string(),
                priority: 10,
                prompts: vec![
                    "I'm having trouble connecting. Would you like to try something offline?".to_string(),
                ],
                alternatives: vec![
                    AlternativeAction {
                        description: "Use offline features".to_string(),
                        action: enable_offline_mode_action(),
                        confidence: 0.8,
                    },
                ],
                needs_clarification: true,
            },
            RecoveryStrategy {
                name: "queue_for_later".to_string(),
                description: "Queue action for when online".to_string(),
                priority: 8,
                prompts: vec![
                    "I'll do this as soon as you're back online.".to_string(),
                ],
                alternatives: vec![],
                needs_clarification: false,
            },
        ]);
        
        // Voice recognition error strategies
        self.strategies.insert(ErrorType::VoiceRecognitionError, vec![
            RecoveryStrategy {
                name: "ask_repeat".to_string(),
                description: "Ask user to repeat".to_string(),
                priority: 10,
                prompts: vec![
                    "I didn't catch that. Could you say it again?".to_string(),
                    "Sorry, it's a bit noisy. Can you repeat that?".to_string(),
                ],
                alternatives: vec![],
                needs_clarification: true,
            },
            RecoveryStrategy {
                name: "speak_slower".to_string(),
                description: "Ask user to speak slower".to_string(),
                priority: 5,
                prompts: vec![
                    "Could you speak a bit slower?".to_string(),
                ],
                alternatives: vec![],
                needs_clarification: true,
            },
        ]);
        
        // Permission denied strategies
        self.strategies.insert(ErrorType::PermissionDenied, vec![
            RecoveryStrategy {
                name: "explain_permission".to_string(),
                description: "Explain what permission is needed".to_string(),
                priority: 10,
                prompts: vec![
                    "I need permission to do that.".to_string(),
                    "You'll need to grant access in settings.".to_string(),
                ],
                alternatives: vec![
                    AlternativeAction {
                        description: "Open settings".to_string(),
                        action: open_settings_action("permissions"),
                        confidence: 0.8,
                    },
                ],
                needs_clarification: false,
            },
        ]);
        
        // Timeout strategies
        self.strategies.insert(ErrorType::Timeout, vec![
            RecoveryStrategy {
                name: "timeout_retry".to_string(),
                description: "Retry after timeout".to_string(),
                priority: 10,
                prompts: vec![
                    "That took too long. Want me to try again?".to_string(),
                ],
                alternatives: vec![],
                needs_clarification: true,
            },
        ]);
        
        // Unknown error strategies
        self.strategies.insert(ErrorType::Unknown, vec![
            RecoveryStrategy {
                name: "graceful_fail".to_string(),
                description: "Gracefully handle unknown error".to_string(),
                priority: 10,
                prompts: vec![
                    "Something unexpected happened. Let's try something else.".to_string(),
                    "I ran into an issue. What else can I help with?".to_string(),
                ],
                alternatives: vec![
                    AlternativeAction {
                        description: "Show help".to_string(),
                        action: show_help_action(),
                        confidence: 0.5,
                    },
                ],
                needs_clarification: false,
            },
        ]);
    }
    
    /// Attempt to recover from an error
    pub fn recover(
        &mut self,
        error_type: ErrorType,
        intent: Option<&ResolvedIntent>,
        context: &AiContext,
        input: &str,
    ) -> RecoveryResult {
        // Track error
        self.consecutive_errors += 1;
        
        // Record error
        self.error_history.push(ErrorRecord {
            error_type: error_type.clone(),
            intent: intent.map(|i| i.name.clone()),
            input: input.to_string(),
            recovery_attempted: None,
            recovered: false,
        });
        
        // Check for escalation
        if self.consecutive_errors >= self.max_consecutive_errors {
            return self.escalate(context);
        }
        
        // Get strategies for this error type
        let strategies = self.strategies.get(&error_type)
            .cloned()
            .unwrap_or_else(|| {
                self.strategies.get(&ErrorType::Unknown).cloned().unwrap_or_default()
            });
        
        // Find best strategy based on context
        let strategy = self.select_strategy(&strategies, context, input);
        
        // Build recovery result
        self.build_recovery_result(error_type, strategy, intent, context, input)
    }
    
    /// Select the best strategy based on context
    fn select_strategy(
        &self,
        strategies: &[RecoveryStrategy],
        context: &AiContext,
        _input: &str,
    ) -> RecoveryStrategy {
        let mut best = strategies.first().cloned().unwrap_or_else(|| {
            RecoveryStrategy {
                name: "default".to_string(),
                description: "Default recovery".to_string(),
                priority: 0,
                prompts: vec!["Something went wrong. Please try again.".to_string()],
                alternatives: vec![],
                needs_clarification: false,
            }
        });
        
        for strategy in strategies {
            // Adjust priority based on context
            let mut adjusted_priority = strategy.priority;
            
            // In noisy environment, prefer non-clarification strategies
            let is_noisy = matches!(context.environment.noise_level, NoiseLevel::Noisy | NoiseLevel::VeryNoisy);
            if is_noisy && strategy.needs_clarification {
                adjusted_priority = adjusted_priority.saturating_sub(3);
            }
            
            // If user moving quickly, prefer quick recoveries
            if context.environment.is_moving && strategy.alternatives.is_empty() {
                adjusted_priority += 1;
            }
            
            if adjusted_priority > best.priority {
                best = strategy.clone();
            }
        }
        
        best
    }
    
    /// Build recovery result from strategy
    fn build_recovery_result(
        &mut self,
        error_type: ErrorType,
        strategy: RecoveryStrategy,
        intent: Option<&ResolvedIntent>,
        _context: &AiContext,
        _input: &str,
    ) -> RecoveryResult {
        // Pick appropriate prompt
        let message = if let Some(prompt) = strategy.prompts.first() {
            self.customize_prompt(prompt, error_type.clone(), intent)
        } else {
            "Something went wrong. Please try again.".to_string()
        };
        
        // Build suggestions
        let mut suggestions = Vec::new();
        for alt in &strategy.alternatives {
            suggestions.push(alt.description.clone());
        }
        
        // Add contextual suggestions
        if let Some(intent) = intent {
            if !intent.missing_slots.is_empty() {
                suggestions.push("Provide missing information".to_string());
            }
            if intent.alternative.is_some() {
                suggestions.push("Try alternative".to_string());
            }
        }
        
        // Build clarification if needed
        let clarification = if strategy.needs_clarification {
            Some(ClarificationRequest {
                question: message.clone(),
                clarifying: self.get_clarification_target(&error_type, intent),
                options: self.get_clarification_options(&error_type, intent),
            })
        } else {
            None
        };
        
        // Get fallback action
        let fallback_action = strategy.alternatives.first().map(|a| a.action.clone());
        
        // Update recovery attempt in history
        if let Some(last) = self.error_history.last_mut() {
            last.recovery_attempted = Some(strategy.name);
        }
        
        RecoveryResult {
            message,
            suggestions,
            continue_conversation: true,
            fallback_action,
            clarification,
        }
    }
    
    /// Customize prompt based on error and intent
    fn customize_prompt(
        &self,
        base_prompt: &str,
        error_type: ErrorType,
        intent: Option<&ResolvedIntent>,
    ) -> String {
        let mut prompt = base_prompt.to_string();
        
        // Customize based on error type
        match error_type {
            ErrorType::MissingSlot => {
                if let Some(intent) = intent {
                    if let Some(slot) = intent.missing_slots.first() {
                        prompt = match slot.as_str() {
                            "contact" => "Who would you like me to contact?".to_string(),
                            "location" | "destination" => "Where would you like to go?".to_string(),
                            "time" => "What time would you like?".to_string(),
                            "message" => "What would you like the message to say?".to_string(),
                            "query" => "What would you like to search for?".to_string(),
                            _ => format!("What {} would you like?", slot),
                        };
                    }
                }
            }
            ErrorType::UnknownEntity => {
                if let Some(intent) = intent {
                    prompt = format!("I couldn't find that for {}. Could you try a different name?", intent.name);
                }
            }
            _ => {}
        }
        
        prompt
    }
    
    /// Get what we're trying to clarify
    fn get_clarification_target(&self, error_type: &ErrorType, intent: Option<&ResolvedIntent>) -> String {
        match error_type {
            ErrorType::MissingSlot => {
                intent.and_then(|i| i.missing_slots.first().cloned())
                    .unwrap_or_else(|| "information".to_string())
            }
            ErrorType::AmbiguousIntent => "intent".to_string(),
            ErrorType::UnknownEntity => "entity".to_string(),
            _ => "request".to_string(),
        }
    }
    
    /// Get clarification options
    fn get_clarification_options(&self, error_type: &ErrorType, intent: Option<&ResolvedIntent>) -> Vec<String> {
        match error_type {
            ErrorType::AmbiguousIntent => {
                vec![
                    "Make a call".to_string(),
                    "Send a message".to_string(),
                    "Navigate somewhere".to_string(),
                    "Play music".to_string(),
                    "Something else".to_string(),
                ]
            }
            ErrorType::MissingSlot => {
                if let Some(intent) = intent {
                    if let Some(slot) = intent.missing_slots.first() {
                        return match slot.as_str() {
                            "contact" => vec!["Mom".to_string(), "Dad".to_string(), "Boss".to_string()],
                            _ => vec![],
                        };
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }
    
    /// Escalate when too many errors
    fn escalate(&mut self, _context: &AiContext) -> RecoveryResult {
        // Reset error counter
        self.consecutive_errors = 0;
        
        RecoveryResult {
            message: "I'm having trouble understanding. Let me show you what I can help with.".to_string(),
            suggestions: vec![
                "See available commands".to_string(),
                "Start over".to_string(),
                "Get help".to_string(),
            ],
            continue_conversation: true,
            fallback_action: Some(show_help_action()),
            clarification: None,
        }
    }
    
    /// Mark recovery as successful
    pub fn recovery_succeeded(&mut self) {
        self.consecutive_errors = 0;
        if let Some(last) = self.error_history.last_mut() {
            last.recovered = true;
        }
    }
    
    /// Get error rate for monitoring
    pub fn get_error_rate(&self, window: usize) -> f32 {
        let recent = self.error_history.iter().rev().take(window);
        let total = recent.clone().count();
        
        if total == 0 {
            return 0.0;
        }
        
        let recovered = recent.filter(|e| e.recovered).count();
        (total - recovered) as f32 / total as f32
    }
    
    /// Get most common error type
    pub fn get_common_error(&self) -> Option<ErrorType> {
        let mut counts: HashMap<ErrorType, u32> = HashMap::new();
        
        for record in &self.error_history {
            *counts.entry(record.error_type.clone()).or_insert(0) += 1;
        }
        
        counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(error_type, _)| error_type)
    }
    
    /// Detect error from failed intent
    pub fn detect_error(intent: &ResolvedIntent, context: &AiContext) -> ErrorType {
        // Check for missing slots
        if !intent.missing_slots.is_empty() {
            return ErrorType::MissingSlot;
        }
        
        // Check for low confidence
        if intent.confidence < 0.3 {
            return ErrorType::AmbiguousIntent;
        }
        
        // Check feasibility
        if !intent.feasible {
            // Check specific reasons
            if !context.device_state.wifi_connected {
                return ErrorType::NetworkError;
            }
            return ErrorType::ActionNotFeasible;
        }
        
        ErrorType::Unknown
    }
}

impl Default for ErrorRecovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_layer::intent::IntentCategory;
    
    #[test]
    fn test_error_recovery_creation() {
        let recovery = ErrorRecovery::new();
        assert!(!recovery.strategies.is_empty());
    }
    
    #[test]
    fn test_ambiguous_intent_recovery() {
        let mut recovery = ErrorRecovery::new();
        let context = AiContext::default();
        
        let result = recovery.recover(ErrorType::AmbiguousIntent, None, &context, "do something");
        
        assert!(result.continue_conversation);
        assert!(result.clarification.is_some());
    }
    
    #[test]
    fn test_missing_slot_recovery() {
        let mut recovery = ErrorRecovery::new();
        let context = AiContext::default();
        
        let intent = ResolvedIntent {
            name: "call".to_string(),
            category: IntentCategory::Communication,
            confidence: 0.9,
            requires_confirmation: false,
            requires_reasoning: false,
            missing_slots: vec!["contact".to_string()],
            feasible: true,
            alternative: None,
            continues_previous: false,
        };
        
        let result = recovery.recover(ErrorType::MissingSlot, Some(&intent), &context, "call");
        
        assert!(result.message.contains("contact") || result.message.contains("Who"));
    }
    
    #[test]
    fn test_escalation() {
        let mut recovery = ErrorRecovery::new();
        let context = AiContext::default();
        
        // Trigger errors up to escalation threshold
        // max_consecutive_errors is 3, so the 3rd error triggers escalation
        recovery.recover(ErrorType::Unknown, None, &context, "test");
        recovery.recover(ErrorType::Unknown, None, &context, "test");
        let result = recovery.recover(ErrorType::Unknown, None, &context, "test");
        
        // Should escalate to help on 3rd error
        assert!(result.message.contains("trouble"));
    }
    
    #[test]
    fn test_recovery_success_tracking() {
        let mut recovery = ErrorRecovery::new();
        let context = AiContext::default();
        
        recovery.recover(ErrorType::AmbiguousIntent, None, &context, "test");
        assert_eq!(recovery.consecutive_errors, 1);
        
        recovery.recovery_succeeded();
        assert_eq!(recovery.consecutive_errors, 0);
    }
    
    #[test]
    fn test_error_detection() {
        let context = AiContext::default();
        
        let intent_missing_slot = ResolvedIntent {
            name: "call".to_string(),
            category: IntentCategory::Communication,
            confidence: 0.9,
            requires_confirmation: false,
            requires_reasoning: false,
            missing_slots: vec!["contact".to_string()],
            feasible: true,
            alternative: None,
            continues_previous: false,
        };
        
        let error = ErrorRecovery::detect_error(&intent_missing_slot, &context);
        assert_eq!(error, ErrorType::MissingSlot);
        
        let intent_low_conf = ResolvedIntent {
            name: "unknown".to_string(),
            category: IntentCategory::Information,
            confidence: 0.2,
            requires_confirmation: false,
            requires_reasoning: false,
            missing_slots: vec![],
            feasible: true,
            alternative: None,
            continues_previous: false,
        };
        
        let error = ErrorRecovery::detect_error(&intent_low_conf, &context);
        assert_eq!(error, ErrorType::AmbiguousIntent);
    }
    
    #[test]
    fn test_error_rate() {
        let mut recovery = ErrorRecovery::new();
        let context = AiContext::default();
        
        // Generate some errors
        recovery.recover(ErrorType::Unknown, None, &context, "test1");
        recovery.recovery_succeeded();
        recovery.recover(ErrorType::Unknown, None, &context, "test2");
        // No recovery
        
        let rate = recovery.get_error_rate(10);
        assert!(rate > 0.0 && rate < 1.0);
    }
}
