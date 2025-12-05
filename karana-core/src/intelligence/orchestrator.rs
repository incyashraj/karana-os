//! Request Orchestrator
//!
//! Coordinates between different system components to handle user requests
//! intelligently and efficiently.

use super::*;
use std::collections::{HashMap, VecDeque};

/// Route information for a request
#[derive(Debug, Clone)]
pub struct RequestRoute {
    pub primary_handler: HandlerType,
    pub secondary_handlers: Vec<HandlerType>,
    pub requires_confirmation: bool,
    pub estimated_time_ms: u64,
    pub resource_requirements: ResourceRequirements,
}

/// Handler types for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandlerType {
    VoiceAssistant,
    AppLauncher,
    SystemSettings,
    Navigation,
    Communication,
    Media,
    Productivity,
    Blockchain,
    AR,
    Custom,
}

/// Resource requirements for handling
#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub cpu_intensive: bool,
    pub network_required: bool,
    pub gpu_required: bool,
    pub permission_required: Vec<String>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_intensive: false,
            network_required: false,
            gpu_required: false,
            permission_required: Vec::new(),
        }
    }
}

/// Request orchestrator coordinates request handling
pub struct RequestOrchestrator {
    /// Pending requests in queue
    pending_requests: VecDeque<OrchestrationTask>,
    /// Active parallel tasks
    active_tasks: HashMap<u64, OrchestrationTask>,
    /// Completed tasks cache
    completed_cache: VecDeque<CompletedTask>,
    /// Task counter
    task_counter: u64,
    /// Max parallel tasks
    max_parallel: usize,
    /// Handler stats
    handler_stats: HashMap<HandlerType, HandlerStats>,
}

/// An orchestration task
#[derive(Debug, Clone)]
pub struct OrchestrationTask {
    pub id: u64,
    pub request: UserRequest,
    pub route: RequestRoute,
    pub workflow: Option<WorkflowInstance>,
    pub predictions: Vec<Prediction>,
    pub decisions: Vec<Decision>,
    pub status: TaskStatus,
    pub started_at: Option<Instant>,
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    WaitingForUser,
    Completed,
    Failed,
    Cancelled,
}

/// Completed task record
#[derive(Debug, Clone)]
pub struct CompletedTask {
    pub id: u64,
    pub duration: Duration,
    pub success: bool,
    pub handler: HandlerType,
}

/// Handler statistics
#[derive(Debug, Clone, Default)]
pub struct HandlerStats {
    pub total_requests: u64,
    pub successful: u64,
    pub average_time_ms: f64,
}

/// A decision made by the decision engine
#[derive(Debug, Clone)]
pub struct Decision {
    pub decision_type: DecisionType,
    pub value: String,
    pub confidence: f32,
    pub reasoning: String,
}

/// Decision types
#[derive(Debug, Clone, PartialEq)]
pub enum DecisionType {
    AppSelection,
    ActionSequence,
    ResponseFormat,
    UiLayout,
    Priority,
    Confirmation,
}

/// Workflow instance reference
#[derive(Debug, Clone)]
pub struct WorkflowInstance {
    pub workflow_id: String,
    pub current_step: usize,
    pub parameters: HashMap<String, String>,
}

impl RequestOrchestrator {
    pub fn new() -> Self {
        Self {
            pending_requests: VecDeque::new(),
            active_tasks: HashMap::new(),
            completed_cache: VecDeque::with_capacity(100),
            task_counter: 0,
            max_parallel: 5,
            handler_stats: HashMap::new(),
        }
    }
    
    /// Orchestrate handling of a request
    pub fn orchestrate(
        &mut self,
        request: &UserRequest,
        route: RequestRoute,
        workflow: Option<WorkflowInstance>,
        predictions: Vec<Prediction>,
        decisions: Vec<Decision>,
    ) -> IntelligentResponse {
        self.task_counter += 1;
        let task_id = self.task_counter;
        
        // Create orchestration task
        let task = OrchestrationTask {
            id: task_id,
            request: request.clone(),
            route: route.clone(),
            workflow: workflow.clone(),
            predictions: predictions.clone(),
            decisions: decisions.clone(),
            status: TaskStatus::Running,
            started_at: Some(Instant::now()),
        };
        
        self.active_tasks.insert(task_id, task);
        
        // Build the response based on route and decisions
        let primary_action = self.build_primary_action(&route, &decisions);
        let secondary_actions = self.build_secondary_actions(&route, &predictions);
        let ui_updates = self.build_ui_updates(&route, &decisions, &predictions);
        
        // Calculate confidence
        let confidence = self.calculate_confidence(&route, &decisions);
        
        // Build explanation if confidence is borderline
        let explanation = if confidence < 0.9 {
            Some(self.build_explanation(&route, &decisions))
        } else {
            None
        };
        
        // Mark task completed
        self.complete_task(task_id, route.primary_handler);
        
        IntelligentResponse {
            primary_action,
            secondary_actions,
            ui_updates,
            predictions,
            confidence,
            explanation,
        }
    }
    
    /// Build the primary action
    fn build_primary_action(&self, route: &RequestRoute, decisions: &[Decision]) -> Action {
        let action_type = match route.primary_handler {
            HandlerType::VoiceAssistant => ActionType::Speak,
            HandlerType::AppLauncher => ActionType::LaunchApp,
            HandlerType::SystemSettings => ActionType::SystemSetting,
            HandlerType::Navigation => ActionType::Navigate,
            HandlerType::Media => ActionType::LaunchApp,
            HandlerType::AR => ActionType::ShowOverlay,
            _ => ActionType::Query,
        };
        
        // Get target from decisions
        let target = decisions.iter()
            .find(|d| d.decision_type == DecisionType::AppSelection)
            .map(|d| d.value.clone())
            .unwrap_or_else(|| "default".to_string());
        
        Action {
            action_type,
            target,
            parameters: HashMap::new(),
            priority: if route.requires_confirmation {
                ActionPriority::Normal
            } else {
                ActionPriority::High
            },
        }
    }
    
    /// Build secondary actions from predictions
    fn build_secondary_actions(&self, route: &RequestRoute, predictions: &[Prediction]) -> Vec<Action> {
        let mut actions = Vec::new();
        
        // Add actions for secondary handlers
        for handler in &route.secondary_handlers {
            let action = Action {
                action_type: match handler {
                    HandlerType::AR => ActionType::ShowOverlay,
                    HandlerType::Media => ActionType::Custom("prepare_media".to_string()),
                    _ => ActionType::Query,
                },
                target: format!("{:?}", handler).to_lowercase(),
                parameters: HashMap::new(),
                priority: ActionPriority::Low,
            };
            actions.push(action);
        }
        
        // Add actions from predictions
        for pred in predictions.iter().filter(|p| p.confidence > 0.7) {
            if let PredictionType::NextAction = pred.prediction_type {
                actions.push(Action {
                    action_type: ActionType::Custom("predicted".to_string()),
                    target: pred.target.clone(),
                    parameters: HashMap::new(),
                    priority: ActionPriority::Low,
                });
            }
        }
        
        actions
    }
    
    /// Build UI updates
    fn build_ui_updates(
        &self,
        route: &RequestRoute,
        decisions: &[Decision],
        predictions: &[Prediction],
    ) -> Vec<UiUpdate> {
        let mut updates = Vec::new();
        
        // Add UI layout decision
        if let Some(layout_decision) = decisions.iter()
            .find(|d| d.decision_type == DecisionType::UiLayout) {
            updates.push(UiUpdate {
                update_type: UiUpdateType::RefreshWidget,
                data: [("layout".to_string(), layout_decision.value.clone())]
                    .into_iter().collect(),
            });
        }
        
        // Show suggestions from predictions
        for pred in predictions.iter().filter(|p| p.confidence > 0.6) {
            updates.push(UiUpdate {
                update_type: UiUpdateType::ShowSuggestion,
                data: [
                    ("target".to_string(), pred.target.clone()),
                    ("reason".to_string(), pred.reason.clone()),
                ].into_iter().collect(),
            });
        }
        
        // If requires confirmation, show quick action
        if route.requires_confirmation {
            updates.push(UiUpdate {
                update_type: UiUpdateType::ShowQuickAction,
                data: [("action".to_string(), "confirm".to_string())]
                    .into_iter().collect(),
            });
        }
        
        updates
    }
    
    /// Calculate overall confidence
    fn calculate_confidence(&self, route: &RequestRoute, decisions: &[Decision]) -> f32 {
        if decisions.is_empty() {
            return 0.5;
        }
        
        let avg_confidence: f32 = decisions.iter()
            .map(|d| d.confidence)
            .sum::<f32>() / decisions.len() as f32;
        
        // Reduce confidence if requires confirmation
        if route.requires_confirmation {
            avg_confidence * 0.9
        } else {
            avg_confidence
        }
    }
    
    /// Build explanation for the response
    fn build_explanation(&self, route: &RequestRoute, decisions: &[Decision]) -> String {
        let mut explanations = Vec::new();
        
        explanations.push(format!("Routing to {:?}", route.primary_handler));
        
        for decision in decisions {
            if decision.confidence > 0.5 {
                explanations.push(format!("{}: {}", 
                    match decision.decision_type {
                        DecisionType::AppSelection => "Selected app",
                        DecisionType::ActionSequence => "Action",
                        DecisionType::ResponseFormat => "Format",
                        DecisionType::UiLayout => "Layout",
                        DecisionType::Priority => "Priority",
                        DecisionType::Confirmation => "Confirmation",
                    },
                    decision.reasoning.clone()
                ));
            }
        }
        
        explanations.join(". ")
    }
    
    /// Mark a task as completed
    fn complete_task(&mut self, task_id: u64, handler: HandlerType) {
        if let Some(task) = self.active_tasks.remove(&task_id) {
            let duration = task.started_at
                .map(|s| s.elapsed())
                .unwrap_or_default();
            
            // Update handler stats
            let stats = self.handler_stats.entry(handler).or_default();
            stats.total_requests += 1;
            stats.successful += 1;
            let n = stats.total_requests as f64;
            stats.average_time_ms = 
                stats.average_time_ms * (n - 1.0) / n + duration.as_millis() as f64 / n;
            
            // Cache completed task
            self.completed_cache.push_back(CompletedTask {
                id: task_id,
                duration,
                success: true,
                handler,
            });
            
            // Keep cache bounded
            while self.completed_cache.len() > 100 {
                self.completed_cache.pop_front();
            }
        }
    }
    
    /// Get handler statistics
    pub fn handler_stats(&self) -> &HashMap<HandlerType, HandlerStats> {
        &self.handler_stats
    }
    
    /// Get active task count
    pub fn active_count(&self) -> usize {
        self.active_tasks.len()
    }
    
    /// Get pending count
    pub fn pending_count(&self) -> usize {
        self.pending_requests.len()
    }
}

impl Default for RequestOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = RequestOrchestrator::new();
        assert_eq!(orchestrator.active_count(), 0);
        assert_eq!(orchestrator.pending_count(), 0);
    }
    
    #[test]
    fn test_resource_requirements() {
        let req = ResourceRequirements::default();
        assert!(!req.cpu_intensive);
        assert!(!req.network_required);
    }
    
    #[test]
    fn test_handler_stats() {
        let mut orchestrator = RequestOrchestrator::new();
        
        let route = RequestRoute {
            primary_handler: HandlerType::AppLauncher,
            secondary_handlers: vec![],
            requires_confirmation: false,
            estimated_time_ms: 100,
            resource_requirements: ResourceRequirements::default(),
        };
        
        let request = UserRequest {
            id: 1,
            input: RequestInput::Text("open browser".to_string()),
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
        
        let _response = orchestrator.orchestrate(
            &request,
            route,
            None,
            vec![],
            vec![Decision {
                decision_type: DecisionType::AppSelection,
                value: "browser".to_string(),
                confidence: 0.9,
                reasoning: "User requested browser".to_string(),
            }]
        );
        
        assert!(orchestrator.handler_stats().contains_key(&HandlerType::AppLauncher));
    }
}
