//! Workflow Engine
//!
//! Automates multi-step workflows and learns user routines
//! to streamline repetitive tasks.

use super::*;
use super::orchestrator::{RequestRoute, WorkflowInstance};
use std::collections::HashMap;

/// Workflow engine for automating multi-step tasks
pub struct WorkflowEngine {
    /// Available workflows
    workflows: HashMap<String, Workflow>,
    /// Active workflow instances
    active: HashMap<String, WorkflowState>,
    /// Learned workflows from user behavior
    learned: Vec<LearnedWorkflow>,
    /// Whether automation is enabled
    automation_enabled: bool,
    /// Workflow execution statistics
    stats: WorkflowStats,
}

impl WorkflowEngine {
    pub fn new(automation_enabled: bool) -> Self {
        let mut engine = Self {
            workflows: HashMap::new(),
            active: HashMap::new(),
            learned: Vec::new(),
            automation_enabled,
            stats: WorkflowStats::new(),
        };
        engine.setup_default_workflows();
        engine
    }
    
    fn setup_default_workflows(&mut self) {
        // Morning routine workflow
        self.add_workflow(Workflow {
            id: "morning_routine".to_string(),
            name: "Morning Routine".to_string(),
            description: "Daily morning preparation".to_string(),
            trigger: WorkflowTrigger::TimeOfDay(TimeOfDay::Morning),
            steps: vec![
                WorkflowStep {
                    id: "check_weather".to_string(),
                    action: StepAction::Query("weather".to_string()),
                    condition: None,
                    on_success: Some("show_calendar".to_string()),
                    on_failure: Some("show_calendar".to_string()),
                },
                WorkflowStep {
                    id: "show_calendar".to_string(),
                    action: StepAction::ShowWidget("calendar".to_string()),
                    condition: None,
                    on_success: Some("check_notifications".to_string()),
                    on_failure: None,
                },
                WorkflowStep {
                    id: "check_notifications".to_string(),
                    action: StepAction::ShowWidget("notifications".to_string()),
                    condition: None,
                    on_success: None,
                    on_failure: None,
                },
            ],
            requires_confirmation: false,
            cooldown_minutes: 60,
        });
        
        // Meeting preparation workflow
        self.add_workflow(Workflow {
            id: "meeting_prep".to_string(),
            name: "Meeting Preparation".to_string(),
            description: "Prepare for upcoming meetings".to_string(),
            trigger: WorkflowTrigger::Event("meeting_starting".to_string()),
            steps: vec![
                WorkflowStep {
                    id: "mute_notifications".to_string(),
                    action: StepAction::SystemSetting("do_not_disturb".to_string(), "true".to_string()),
                    condition: None,
                    on_success: Some("show_meeting_info".to_string()),
                    on_failure: None,
                },
                WorkflowStep {
                    id: "show_meeting_info".to_string(),
                    action: StepAction::ShowOverlay("meeting_details".to_string()),
                    condition: None,
                    on_success: None,
                    on_failure: None,
                },
            ],
            requires_confirmation: true,
            cooldown_minutes: 5,
        });
        
        // Navigation workflow
        self.add_workflow(Workflow {
            id: "start_navigation".to_string(),
            name: "Navigation Mode".to_string(),
            description: "Set up for navigation".to_string(),
            trigger: WorkflowTrigger::Intent("navigate".to_string()),
            steps: vec![
                WorkflowStep {
                    id: "enable_gps".to_string(),
                    action: StepAction::SystemSetting("gps".to_string(), "high_accuracy".to_string()),
                    condition: None,
                    on_success: Some("show_nav_hud".to_string()),
                    on_failure: None,
                },
                WorkflowStep {
                    id: "show_nav_hud".to_string(),
                    action: StepAction::ShowOverlay("navigation_arrows".to_string()),
                    condition: None,
                    on_success: Some("enable_audio_cues".to_string()),
                    on_failure: None,
                },
                WorkflowStep {
                    id: "enable_audio_cues".to_string(),
                    action: StepAction::SystemSetting("nav_audio".to_string(), "true".to_string()),
                    condition: None,
                    on_success: None,
                    on_failure: None,
                },
            ],
            requires_confirmation: false,
            cooldown_minutes: 0,
        });
        
        // Low battery workflow
        self.add_workflow(Workflow {
            id: "low_battery".to_string(),
            name: "Low Battery Mode".to_string(),
            description: "Conserve battery power".to_string(),
            trigger: WorkflowTrigger::Condition(TriggerCondition::BatteryBelow(20)),
            steps: vec![
                WorkflowStep {
                    id: "reduce_brightness".to_string(),
                    action: StepAction::SystemSetting("brightness".to_string(), "30".to_string()),
                    condition: None,
                    on_success: Some("disable_background".to_string()),
                    on_failure: None,
                },
                WorkflowStep {
                    id: "disable_background".to_string(),
                    action: StepAction::SystemSetting("background_sync".to_string(), "false".to_string()),
                    condition: None,
                    on_success: Some("notify_user".to_string()),
                    on_failure: None,
                },
                WorkflowStep {
                    id: "notify_user".to_string(),
                    action: StepAction::Notification("Low battery mode enabled".to_string()),
                    condition: None,
                    on_success: None,
                    on_failure: None,
                },
            ],
            requires_confirmation: false,
            cooldown_minutes: 30,
        });
    }
    
    /// Add a workflow
    pub fn add_workflow(&mut self, workflow: Workflow) {
        self.workflows.insert(workflow.id.clone(), workflow);
    }
    
    /// Find applicable workflow for a request
    pub fn find_applicable(&self, request: &UserRequest, route: &RequestRoute) -> Option<WorkflowInstance> {
        if !self.automation_enabled {
            return None;
        }
        
        for workflow in self.workflows.values() {
            if self.workflow_matches(workflow, request, route) {
                return Some(WorkflowInstance {
                    workflow_id: workflow.id.clone(),
                    current_step: 0,
                    parameters: HashMap::new(),
                });
            }
        }
        
        // Check learned workflows
        for learned in &self.learned {
            if self.learned_workflow_matches(learned, request) {
                return Some(WorkflowInstance {
                    workflow_id: learned.id.clone(),
                    current_step: 0,
                    parameters: HashMap::new(),
                });
            }
        }
        
        None
    }
    
    /// Check if a workflow matches the current request
    fn workflow_matches(&self, workflow: &Workflow, request: &UserRequest, _route: &RequestRoute) -> bool {
        match &workflow.trigger {
            WorkflowTrigger::TimeOfDay(time) => {
                request.context.time_of_day == *time
            }
            WorkflowTrigger::Location(loc) => {
                request.context.location.as_ref() == Some(loc)
            }
            WorkflowTrigger::Intent(intent) => {
                if let RequestInput::Voice(text) | RequestInput::Text(text) = &request.input {
                    text.to_lowercase().contains(intent)
                } else {
                    false
                }
            }
            WorkflowTrigger::Event(event) => {
                if let RequestInput::SystemEvent(e) = &request.input {
                    &e.event_type == event
                } else {
                    false
                }
            }
            WorkflowTrigger::Condition(cond) => {
                match cond {
                    TriggerCondition::BatteryBelow(level) => {
                        request.context.battery_level < *level
                    }
                    TriggerCondition::BatteryAbove(level) => {
                        request.context.battery_level > *level
                    }
                    TriggerCondition::UserState(state) => {
                        &request.context.user_state == state
                    }
                }
            }
            WorkflowTrigger::Sequence(seq) => {
                // Check if recent apps match the sequence
                let recent = &request.context.recent_apps;
                if recent.len() >= seq.len() {
                    let tail = &recent[recent.len() - seq.len()..];
                    tail == seq.as_slice()
                } else {
                    false
                }
            }
        }
    }
    
    /// Check if a learned workflow matches
    fn learned_workflow_matches(&self, learned: &LearnedWorkflow, request: &UserRequest) -> bool {
        if learned.confidence < 0.7 {
            return false;
        }
        
        // Check time pattern
        if let Some(time) = &learned.time_pattern {
            if &request.context.time_of_day != time {
                return false;
            }
        }
        
        // Check location pattern
        if let Some(loc) = &learned.location_pattern {
            if request.context.location.as_ref() != Some(loc) {
                return false;
            }
        }
        
        // Check app sequence
        if let Some(seq) = &learned.app_sequence {
            let recent = &request.context.recent_apps;
            if recent.len() < seq.len() {
                return false;
            }
            let tail = &recent[recent.len() - seq.len()..];
            if tail != seq.as_slice() {
                return false;
            }
        }
        
        true
    }
    
    /// Execute a workflow step
    pub fn execute_step(&mut self, workflow_id: &str, step_id: &str) -> StepResult {
        if let Some(workflow) = self.workflows.get(workflow_id) {
            if let Some(step) = workflow.steps.iter().find(|s| s.id == step_id) {
                // Execute the step action
                let success = self.execute_action(&step.action);
                
                self.stats.steps_executed += 1;
                if success {
                    self.stats.successful_steps += 1;
                }
                
                return StepResult {
                    success,
                    next_step: if success { step.on_success.clone() } else { step.on_failure.clone() },
                    message: None,
                };
            }
        }
        
        StepResult {
            success: false,
            next_step: None,
            message: Some("Workflow or step not found".to_string()),
        }
    }
    
    /// Execute a step action
    fn execute_action(&self, action: &StepAction) -> bool {
        match action {
            StepAction::LaunchApp(_app) => {
                // Would launch the app
                true
            }
            StepAction::CloseApp(_app) => {
                // Would close the app
                true
            }
            StepAction::SystemSetting(_key, _value) => {
                // Would change setting
                true
            }
            StepAction::ShowOverlay(_overlay) => {
                // Would show overlay
                true
            }
            StepAction::HideOverlay(_overlay) => {
                // Would hide overlay
                true
            }
            StepAction::ShowWidget(_widget) => {
                // Would show widget
                true
            }
            StepAction::Query(_query) => {
                // Would execute query
                true
            }
            StepAction::Notification(_msg) => {
                // Would show notification
                true
            }
            StepAction::Wait(_ms) => {
                // Would wait
                true
            }
            StepAction::Custom(_name, _params) => {
                // Would execute custom action
                true
            }
        }
    }
    
    /// Learn a workflow from user behavior
    pub fn learn_workflow(&mut self, actions: Vec<LearnedAction>, context: &RequestContext) {
        if actions.len() < 2 {
            return; // Need at least 2 actions for a workflow
        }
        
        let id = format!("learned_{}", self.learned.len() + 1);
        
        // Convert actions to workflow steps
        let steps: Vec<WorkflowStep> = actions.iter().enumerate().map(|(i, action)| {
            WorkflowStep {
                id: format!("step_{}", i),
                action: action.action.clone(),
                condition: None,
                on_success: if i < actions.len() - 1 {
                    Some(format!("step_{}", i + 1))
                } else {
                    None
                },
                on_failure: None,
            }
        }).collect();
        
        let learned = LearnedWorkflow {
            id: id.clone(),
            steps,
            time_pattern: Some(context.time_of_day),
            location_pattern: context.location.clone(),
            app_sequence: if context.recent_apps.len() >= 2 {
                Some(context.recent_apps.clone())
            } else {
                None
            },
            confidence: 0.5,
            occurrences: 1,
        };
        
        self.learned.push(learned);
    }
    
    /// Reinforce a learned workflow
    pub fn reinforce_learned(&mut self, workflow_id: &str) {
        if let Some(learned) = self.learned.iter_mut().find(|w| w.id == workflow_id) {
            learned.occurrences += 1;
            learned.confidence = (learned.confidence + 0.1).min(1.0);
        }
    }
    
    /// Get workflow statistics
    pub fn stats(&self) -> &WorkflowStats {
        &self.stats
    }
    
    /// Get all workflows
    pub fn workflows(&self) -> Vec<&Workflow> {
        self.workflows.values().collect()
    }
    
    /// Get learned workflows
    pub fn learned_workflows(&self) -> &[LearnedWorkflow] {
        &self.learned
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new(true)
    }
}

/// A workflow definition
#[derive(Debug, Clone)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger: WorkflowTrigger,
    pub steps: Vec<WorkflowStep>,
    pub requires_confirmation: bool,
    pub cooldown_minutes: u32,
}

/// Workflow trigger types
#[derive(Debug, Clone)]
pub enum WorkflowTrigger {
    TimeOfDay(TimeOfDay),
    Location(String),
    Intent(String),
    Event(String),
    Condition(TriggerCondition),
    Sequence(Vec<String>), // App sequence
}

/// Trigger conditions
#[derive(Debug, Clone)]
pub enum TriggerCondition {
    BatteryBelow(u8),
    BatteryAbove(u8),
    UserState(UserState),
}

/// A workflow step
#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub id: String,
    pub action: StepAction,
    pub condition: Option<StepCondition>,
    pub on_success: Option<String>,
    pub on_failure: Option<String>,
}

/// Step actions
#[derive(Debug, Clone)]
pub enum StepAction {
    LaunchApp(String),
    CloseApp(String),
    SystemSetting(String, String),
    ShowOverlay(String),
    HideOverlay(String),
    ShowWidget(String),
    Query(String),
    Notification(String),
    Wait(u64),
    Custom(String, HashMap<String, String>),
}

/// Step conditions
#[derive(Debug, Clone)]
pub enum StepCondition {
    AppRunning(String),
    SettingEquals(String, String),
    OverlayVisible(String),
}

/// Workflow state for active workflows
#[derive(Debug, Clone)]
pub struct WorkflowState {
    pub workflow_id: String,
    pub current_step: usize,
    pub started_at: Instant,
    pub parameters: HashMap<String, String>,
}

/// Step execution result
#[derive(Debug, Clone)]
pub struct StepResult {
    pub success: bool,
    pub next_step: Option<String>,
    pub message: Option<String>,
}

/// A learned workflow from user behavior
#[derive(Debug, Clone)]
pub struct LearnedWorkflow {
    pub id: String,
    pub steps: Vec<WorkflowStep>,
    pub time_pattern: Option<TimeOfDay>,
    pub location_pattern: Option<String>,
    pub app_sequence: Option<Vec<String>>,
    pub confidence: f32,
    pub occurrences: u32,
}

/// An action learned from user behavior
#[derive(Debug, Clone)]
pub struct LearnedAction {
    pub action: StepAction,
    pub timestamp: Instant,
}

/// Workflow statistics
#[derive(Debug, Clone, Default)]
pub struct WorkflowStats {
    pub workflows_triggered: u64,
    pub workflows_completed: u64,
    pub steps_executed: u64,
    pub successful_steps: u64,
}

impl WorkflowStats {
    fn new() -> Self {
        Self::default()
    }
    
    pub fn success_rate(&self) -> f32 {
        if self.steps_executed == 0 {
            0.0
        } else {
            self.successful_steps as f32 / self.steps_executed as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workflow_engine_creation() {
        let engine = WorkflowEngine::new(true);
        assert!(!engine.workflows.is_empty());
    }
    
    #[test]
    fn test_default_workflows() {
        let engine = WorkflowEngine::new(true);
        
        assert!(engine.workflows.contains_key("morning_routine"));
        assert!(engine.workflows.contains_key("low_battery"));
    }
    
    #[test]
    fn test_workflow_matching() {
        let engine = WorkflowEngine::new(true);
        
        let request = UserRequest {
            id: 1,
            input: RequestInput::Text("navigate to the store".to_string()),
            context: RequestContext {
                location: None,
                current_app: None,
                recent_apps: vec![],
                time_of_day: TimeOfDay::Afternoon, // Use Afternoon to avoid morning_routine matching
                battery_level: 80,
                is_moving: false,
                ambient_noise: NoiseLevel::Normal,
                user_state: UserState::Active,
            },
            timestamp: Instant::now(),
        };
        
        let route = super::super::orchestrator::RequestRoute {
            primary_handler: super::super::orchestrator::HandlerType::Navigation,
            secondary_handlers: vec![],
            requires_confirmation: false,
            estimated_time_ms: 100,
            resource_requirements: super::super::orchestrator::ResourceRequirements::default(),
        };
        
        let workflow = engine.find_applicable(&request, &route);
        assert!(workflow.is_some());
        assert_eq!(workflow.unwrap().workflow_id, "start_navigation");
    }
    
    #[test]
    fn test_step_execution() {
        let mut engine = WorkflowEngine::new(true);
        
        let result = engine.execute_step("low_battery", "reduce_brightness");
        assert!(result.success);
        assert_eq!(result.next_step, Some("disable_background".to_string()));
    }
    
    #[test]
    fn test_learn_workflow() {
        let mut engine = WorkflowEngine::new(true);
        
        let actions = vec![
            LearnedAction {
                action: StepAction::LaunchApp("email".to_string()),
                timestamp: Instant::now(),
            },
            LearnedAction {
                action: StepAction::LaunchApp("calendar".to_string()),
                timestamp: Instant::now(),
            },
        ];
        
        let context = RequestContext {
            location: Some("office".to_string()),
            current_app: None,
            recent_apps: vec![],
            time_of_day: TimeOfDay::Morning,
            battery_level: 80,
            is_moving: false,
            ambient_noise: NoiseLevel::Normal,
            user_state: UserState::Active,
        };
        
        engine.learn_workflow(actions, &context);
        
        assert_eq!(engine.learned.len(), 1);
        assert_eq!(engine.learned[0].confidence, 0.5);
    }
    
    #[test]
    fn test_workflow_stats() {
        let mut stats = WorkflowStats::new();
        
        stats.steps_executed = 10;
        stats.successful_steps = 8;
        
        assert!((stats.success_rate() - 0.8).abs() < 0.01);
    }
}
