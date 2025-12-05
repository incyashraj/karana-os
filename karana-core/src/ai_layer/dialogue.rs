//! Dialogue Management System
//!
//! Manages multi-turn conversations, tracks dialogue state, and handles
//! slot filling across conversation turns.

use std::collections::HashMap;
use std::time::{SystemTime, Duration};

/// Dialogue manager for multi-turn conversations
pub struct DialogueManager {
    /// Current dialogue state
    state: DialogueState,
    /// Conversation history
    history: Vec<DialogueTurn>,
    /// Maximum history size
    max_history: usize,
    /// Dialogue policies
    policies: Vec<DialoguePolicy>,
    /// Active tasks (incomplete actions)
    active_tasks: Vec<ActiveTask>,
    /// Session start time
    session_start: SystemTime,
}

impl DialogueManager {
    pub fn new(max_history: usize) -> Self {
        let mut manager = Self {
            state: DialogueState::default(),
            history: Vec::new(),
            max_history,
            policies: Vec::new(),
            active_tasks: Vec::new(),
            session_start: SystemTime::now(),
        };
        manager.initialize_policies();
        manager
    }
    
    fn initialize_policies(&mut self) {
        // Policy for completing multi-step intents
        self.policies.push(DialoguePolicy {
            name: "slot_filling".to_string(),
            trigger: PolicyTrigger::MissingSlots,
            action: PolicyAction::PromptForSlot,
            priority: 10,
        });
        
        // Policy for confirmations
        self.policies.push(DialoguePolicy {
            name: "confirmation".to_string(),
            trigger: PolicyTrigger::RequiresConfirmation,
            action: PolicyAction::AskConfirmation,
            priority: 5,
        });
        
        // Policy for clarification
        self.policies.push(DialoguePolicy {
            name: "clarification".to_string(),
            trigger: PolicyTrigger::LowConfidence,
            action: PolicyAction::AskClarification,
            priority: 3,
        });
        
        // Policy for error recovery
        self.policies.push(DialoguePolicy {
            name: "error_recovery".to_string(),
            trigger: PolicyTrigger::IntentFailure,
            action: PolicyAction::SuggestAlternative,
            priority: 1,
        });
    }
    
    /// Get current dialogue state
    pub fn current_state(&self) -> &DialogueState {
        &self.state
    }
    
    /// Update dialogue state with new intent and entities
    pub fn update(&mut self, intent: &super::ResolvedIntent, entities: &[super::ExtractedEntity]) {
        // Create turn record
        let turn = DialogueTurn {
            turn_id: self.history.len() as u64,
            timestamp: SystemTime::now(),
            intent: Some(intent.name.clone()),
            entities: entities.iter().map(|e| (e.entity_type.to_string(), e.value.clone())).collect(),
            user_input: None,
            system_response: None,
            confidence: intent.confidence,
        };
        
        // Update state
        self.state.turn_count += 1;
        self.state.last_update = SystemTime::now();
        
        // Track current intent
        if !intent.continues_previous {
            self.state.current_intent = Some(intent.name.clone());
            self.state.filled_slots.clear();
            self.state.expected_slots.clear();
        }
        
        // Fill slots from entities
        for entity in entities {
            self.state.filled_slots.insert(
                entity.slot_name.clone().unwrap_or_else(|| entity.entity_type.to_string()),
                entity.normalized_value.clone()
            );
        }
        
        // Update expected slots
        self.state.expected_slots = intent.missing_slots.clone();
        
        // Update context stack
        self.state.context_stack.push(ContextFrame {
            intent: intent.name.clone(),
            confidence: intent.confidence,
            timestamp: SystemTime::now(),
        });
        
        // Keep stack bounded
        if self.state.context_stack.len() > 5 {
            self.state.context_stack.remove(0);
        }
        
        // Add to history
        self.history.push(turn);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        
        // Check if intent requires multiple steps
        if !intent.missing_slots.is_empty() {
            self.create_or_update_task(intent);
        } else {
            // Mark task complete if we have all slots
            self.complete_task_if_ready(&intent.name);
        }
    }
    
    /// Process a follow-up message (for slot filling)
    pub fn process_follow_up(&mut self, _input: &str, entities: &[super::ExtractedEntity]) -> FollowUpResult {
        // Check if we're expecting slots
        if self.state.expected_slots.is_empty() {
            return FollowUpResult::NewIntent;
        }
        
        let mut slots_filled = Vec::new();
        
        // Try to fill expected slots from entities
        for entity in entities {
            let slot_name = entity.slot_name.clone()
                .unwrap_or_else(|| entity.entity_type.to_string());
            
            if self.state.expected_slots.contains(&slot_name) {
                self.state.filled_slots.insert(slot_name.clone(), entity.normalized_value.clone());
                self.state.expected_slots.retain(|s| s != &slot_name);
                slots_filled.push(slot_name);
            }
        }
        
        // Check if we now have all slots
        if self.state.expected_slots.is_empty() {
            // Clone intent before borrowing self mutably
            let intent = self.state.current_intent.clone();
            if let Some(ref i) = intent {
                self.complete_task_if_ready(i);
            }
            FollowUpResult::IntentComplete {
                intent: intent.unwrap_or_default(),
                slots: self.state.filled_slots.clone(),
            }
        } else if !slots_filled.is_empty() {
            FollowUpResult::SlotFilled {
                filled: slots_filled,
                remaining: self.state.expected_slots.clone(),
            }
        } else {
            // Input didn't fill any expected slots
            FollowUpResult::NoSlotsMatched
        }
    }
    
    /// Create or update active task
    fn create_or_update_task(&mut self, intent: &super::ResolvedIntent) {
        // Check if task already exists
        if let Some(task) = self.active_tasks.iter_mut().find(|t| t.intent == intent.name) {
            // Update existing
            task.pending_slots = intent.missing_slots.clone();
            task.last_updated = SystemTime::now();
        } else {
            // Create new
            self.active_tasks.push(ActiveTask {
                task_id: self.active_tasks.len() as u64,
                intent: intent.name.clone(),
                filled_slots: self.state.filled_slots.clone(),
                pending_slots: intent.missing_slots.clone(),
                status: TaskStatus::InProgress,
                created_at: SystemTime::now(),
                last_updated: SystemTime::now(),
                timeout: Duration::from_secs(300), // 5 minute timeout
            });
        }
    }
    
    /// Complete task if all slots are filled
    fn complete_task_if_ready(&mut self, intent: &str) {
        if let Some(task) = self.active_tasks.iter_mut().find(|t| t.intent == intent) {
            if task.pending_slots.is_empty() {
                task.status = TaskStatus::Completed;
                task.filled_slots = self.state.filled_slots.clone();
            }
        }
    }
    
    /// Get the next required slot
    pub fn get_next_required_slot(&self) -> Option<&String> {
        self.state.expected_slots.first()
    }
    
    /// Generate prompt for missing slot
    pub fn get_slot_prompt(&self, slot: &str) -> String {
        match slot {
            "destination" => "Where would you like to go?".to_string(),
            "contact" => "Who would you like to contact?".to_string(),
            "duration" => "How long would you like the timer to be?".to_string(),
            "content" | "message_content" => "What would you like to say?".to_string(),
            "amount" => "How much?".to_string(),
            "recipient" => "Who should I send this to?".to_string(),
            "time" => "When?".to_string(),
            "query" => "What would you like to know?".to_string(),
            "app_name" => "Which app?".to_string(),
            "direction" => "Up or down?".to_string(),
            "label" => "What would you like to call it?".to_string(),
            _ => format!("What {} would you like?", slot),
        }
    }
    
    /// Check if we're in the middle of a multi-turn interaction
    pub fn is_mid_conversation(&self) -> bool {
        !self.state.expected_slots.is_empty() || 
        self.state.context_stack.len() > 1
    }
    
    /// Get recent conversation topics
    pub fn get_recent_topics(&self) -> Vec<String> {
        self.history.iter()
            .rev()
            .take(5)
            .filter_map(|t| t.intent.clone())
            .collect()
    }
    
    /// Check if user might be referring to previous context
    pub fn check_context_reference(&self, input: &str) -> Option<&ContextFrame> {
        let input_lower = input.to_lowercase();
        
        // Check for pronouns that might reference previous context
        let reference_words = ["it", "that", "this", "there", "them", "those"];
        
        if reference_words.iter().any(|w| input_lower.contains(w)) {
            self.state.context_stack.last()
        } else {
            None
        }
    }
    
    /// Apply dialogue policies
    pub fn apply_policies(&self, intent: &super::ResolvedIntent) -> Vec<DialogueAction> {
        let mut actions = Vec::new();
        
        for policy in &self.policies {
            let should_trigger = match policy.trigger {
                PolicyTrigger::MissingSlots => !intent.missing_slots.is_empty(),
                PolicyTrigger::RequiresConfirmation => intent.requires_confirmation,
                PolicyTrigger::LowConfidence => intent.confidence < 0.5,
                PolicyTrigger::IntentFailure => !intent.feasible,
                PolicyTrigger::ContextSwitch => !intent.continues_previous && self.is_mid_conversation(),
            };
            
            if should_trigger {
                actions.push(DialogueAction {
                    action_type: policy.action.clone(),
                    priority: policy.priority,
                    parameters: HashMap::new(),
                });
            }
        }
        
        // Sort by priority
        actions.sort_by(|a, b| b.priority.cmp(&a.priority));
        actions
    }
    
    /// Clean up expired tasks
    pub fn cleanup_expired_tasks(&mut self) {
        let now = SystemTime::now();
        
        self.active_tasks.retain(|task| {
            if let Ok(elapsed) = now.duration_since(task.created_at) {
                elapsed < task.timeout || task.status == TaskStatus::Completed
            } else {
                true
            }
        });
    }
    
    /// Reset dialogue state
    pub fn reset(&mut self) {
        self.state = DialogueState::default();
        self.active_tasks.clear();
        // Keep history for context
    }
    
    /// Full reset including history
    pub fn full_reset(&mut self) {
        self.state = DialogueState::default();
        self.history.clear();
        self.active_tasks.clear();
        self.session_start = SystemTime::now();
    }
    
    /// Get conversation summary
    pub fn get_summary(&self) -> ConversationSummary {
        ConversationSummary {
            turn_count: self.state.turn_count,
            topics: self.get_recent_topics(),
            active_tasks: self.active_tasks.iter()
                .filter(|t| t.status == TaskStatus::InProgress)
                .count(),
            session_duration: SystemTime::now()
                .duration_since(self.session_start)
                .unwrap_or_default(),
        }
    }
}

impl Default for DialogueManager {
    fn default() -> Self {
        Self::new(10)
    }
}

/// Dialogue state tracking
#[derive(Debug, Clone)]
pub struct DialogueState {
    /// Number of turns in conversation
    pub turn_count: u64,
    /// Currently active intent
    pub current_intent: Option<String>,
    /// Filled slots in current intent
    pub filled_slots: HashMap<String, String>,
    /// Expected slots still needed
    pub expected_slots: Vec<String>,
    /// Context stack for nested contexts
    pub context_stack: Vec<ContextFrame>,
    /// Last update time
    pub last_update: SystemTime,
    /// Dialogue phase
    pub phase: DialoguePhase,
}

impl Default for DialogueState {
    fn default() -> Self {
        Self {
            turn_count: 0,
            current_intent: None,
            filled_slots: HashMap::new(),
            expected_slots: Vec::new(),
            context_stack: Vec::new(),
            last_update: SystemTime::now(),
            phase: DialoguePhase::Open,
        }
    }
}

/// Dialogue phases
#[derive(Debug, Clone, PartialEq)]
pub enum DialoguePhase {
    /// Open for new intents
    Open,
    /// Collecting slots
    SlotFilling,
    /// Awaiting confirmation
    AwaitingConfirmation,
    /// Awaiting clarification
    AwaitingClarification,
    /// Processing action
    Processing,
}

/// Context frame for nested conversations
#[derive(Debug, Clone)]
pub struct ContextFrame {
    pub intent: String,
    pub confidence: f32,
    pub timestamp: SystemTime,
}

/// Dialogue turn record
#[derive(Debug, Clone)]
pub struct DialogueTurn {
    pub turn_id: u64,
    pub timestamp: SystemTime,
    pub intent: Option<String>,
    pub entities: HashMap<String, String>,
    pub user_input: Option<String>,
    pub system_response: Option<String>,
    pub confidence: f32,
}

/// Active task being tracked
#[derive(Debug, Clone)]
struct ActiveTask {
    task_id: u64,
    intent: String,
    filled_slots: HashMap<String, String>,
    pending_slots: Vec<String>,
    status: TaskStatus,
    created_at: SystemTime,
    last_updated: SystemTime,
    timeout: Duration,
}

/// Task status
#[derive(Debug, Clone, PartialEq)]
enum TaskStatus {
    InProgress,
    Completed,
    TimedOut,
    Cancelled,
}

/// Result of processing a follow-up
#[derive(Debug, Clone)]
pub enum FollowUpResult {
    /// This is a new intent, not a follow-up
    NewIntent,
    /// Some slots were filled
    SlotFilled {
        filled: Vec<String>,
        remaining: Vec<String>,
    },
    /// All slots filled, intent complete
    IntentComplete {
        intent: String,
        slots: HashMap<String, String>,
    },
    /// Input didn't match expected slots
    NoSlotsMatched,
}

/// Dialogue policy
#[derive(Debug, Clone)]
struct DialoguePolicy {
    name: String,
    trigger: PolicyTrigger,
    action: PolicyAction,
    priority: u8,
}

/// Policy triggers
#[derive(Debug, Clone, PartialEq)]
enum PolicyTrigger {
    MissingSlots,
    RequiresConfirmation,
    LowConfidence,
    IntentFailure,
    ContextSwitch,
}

/// Policy actions
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyAction {
    PromptForSlot,
    AskConfirmation,
    AskClarification,
    SuggestAlternative,
    AbortIntent,
}

/// Dialogue action to take
#[derive(Debug, Clone)]
pub struct DialogueAction {
    pub action_type: PolicyAction,
    pub priority: u8,
    pub parameters: HashMap<String, String>,
}

/// Conversation summary
#[derive(Debug, Clone)]
pub struct ConversationSummary {
    pub turn_count: u64,
    pub topics: Vec<String>,
    pub active_tasks: usize,
    pub session_duration: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_layer::{ResolvedIntent, IntentCategory, ExtractedEntity, EntityType};
    
    #[test]
    fn test_dialogue_manager_creation() {
        let dm = DialogueManager::new(10);
        assert_eq!(dm.state.turn_count, 0);
        assert!(dm.state.current_intent.is_none());
    }
    
    #[test]
    fn test_dialogue_state_update() {
        let mut dm = DialogueManager::new(10);
        
        let intent = ResolvedIntent {
            name: "navigation".to_string(),
            category: IntentCategory::Navigation,
            confidence: 0.9,
            requires_confirmation: false,
            requires_reasoning: false,
            missing_slots: vec!["destination".to_string()],
            feasible: true,
            alternative: None,
            continues_previous: false,
        };
        
        let entities = vec![];
        
        dm.update(&intent, &entities);
        
        assert_eq!(dm.state.turn_count, 1);
        assert_eq!(dm.state.current_intent, Some("navigation".to_string()));
        assert_eq!(dm.state.expected_slots, vec!["destination".to_string()]);
    }
    
    #[test]
    fn test_slot_prompt() {
        let dm = DialogueManager::new(10);
        
        assert!(dm.get_slot_prompt("destination").contains("Where"));
        assert!(dm.get_slot_prompt("contact").contains("Who"));
    }
    
    #[test]
    fn test_follow_up_processing() {
        let mut dm = DialogueManager::new(10);
        
        // Set up state expecting a slot
        dm.state.current_intent = Some("navigation".to_string());
        dm.state.expected_slots = vec!["destination".to_string()];
        
        let entities = vec![ExtractedEntity {
            entity_type: EntityType::Location,
            value: "coffee shop".to_string(),
            normalized_value: "coffee shop".to_string(),
            confidence: 0.9,
            start_pos: 0,
            end_pos: 11,
            slot_name: Some("destination".to_string()),
        }];
        
        let result = dm.process_follow_up("coffee shop", &entities);
        
        match result {
            FollowUpResult::IntentComplete { intent, slots } => {
                assert_eq!(intent, "navigation");
                assert!(slots.contains_key("destination"));
            }
            _ => panic!("Expected IntentComplete"),
        }
    }
    
    #[test]
    fn test_reset() {
        let mut dm = DialogueManager::new(10);
        
        dm.state.turn_count = 5;
        dm.state.current_intent = Some("test".to_string());
        
        dm.reset();
        
        assert_eq!(dm.state.turn_count, 0);
        assert!(dm.state.current_intent.is_none());
    }
    
    #[test]
    fn test_conversation_summary() {
        let dm = DialogueManager::new(10);
        let summary = dm.get_summary();
        
        assert_eq!(summary.turn_count, 0);
        assert_eq!(summary.active_tasks, 0);
    }
}
