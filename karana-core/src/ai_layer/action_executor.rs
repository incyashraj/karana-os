// Kāraṇa OS - Action Executor
// Bridges AI intent to actual OS actions

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::ai_layer::intent::{ResolvedIntent, IntentCategory};
use crate::ai_layer::entities::ExtractedEntity;
use crate::ai_layer::{AiContext, AiAction, ActionPriority};

/// Action executor that converts AI intents to OS actions
pub struct ActionExecutor {
    /// Registered action handlers
    handlers: HashMap<String, Box<dyn ActionHandler + Send + Sync>>,
    /// Action queue for pending actions
    action_queue: Arc<Mutex<Vec<QueuedAction>>>,
    /// Execution history
    history: Vec<ExecutionRecord>,
    /// Max history size
    max_history: usize,
    /// Action policies
    policies: ActionPolicies,
}

/// Trait for action handlers
pub trait ActionHandler: Send + Sync {
    fn execute(&self, action: &ActionRequest) -> ActionResult;
    fn can_execute(&self, action: &ActionRequest, context: &AiContext) -> bool;
    fn get_confirmation_message(&self, action: &ActionRequest) -> Option<String>;
}

/// Request to execute an action
#[derive(Debug, Clone)]
pub struct ActionRequest {
    /// Intent name
    pub intent: String,
    /// Intent category
    pub category: IntentCategory,
    /// Parameters for the action
    pub parameters: HashMap<String, String>,
    /// Extracted entities
    pub entities: Vec<ExtractedEntity>,
    /// Context at time of request
    pub context: AiContext,
    /// Requires confirmation
    pub requires_confirmation: bool,
    /// Priority level
    pub priority: ActionPriority,
}

/// Result of an action execution
#[derive(Debug, Clone)]
pub struct ActionResult {
    /// Whether action succeeded
    pub success: bool,
    /// Result message for user
    pub message: String,
    /// Any data returned
    pub data: Option<ActionData>,
    /// Follow-up actions if any
    pub follow_up: Vec<AiAction>,
    /// Error if failed
    pub error: Option<ActionError>,
}

/// Data returned from an action
#[derive(Debug, Clone)]
pub enum ActionData {
    /// Text response
    Text(String),
    /// Image/Photo reference
    Image { path: String, thumbnail: Option<String> },
    /// List of items
    List(Vec<String>),
    /// Navigation data
    Navigation { route: Vec<(f64, f64)>, eta_minutes: u32 },
    /// Contact info
    Contact { name: String, phone: Option<String>, email: Option<String> },
    /// Media info
    Media { title: String, artist: Option<String>, duration_secs: u32 },
    /// Notification list
    Notifications(Vec<NotificationInfo>),
    /// Weather data
    Weather { temp_c: f32, condition: String, forecast: Vec<String> },
    /// Calendar events
    Events(Vec<CalendarEvent>),
    /// Balance info
    Balance { amount: f64, currency: String },
    /// Generic JSON-like data
    Structured(HashMap<String, String>),
}

/// Notification info
#[derive(Debug, Clone)]
pub struct NotificationInfo {
    pub app: String,
    pub title: String,
    pub body: String,
    pub timestamp: u64,
}

/// Calendar event
#[derive(Debug, Clone)]
pub struct CalendarEvent {
    pub title: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub location: Option<String>,
}

/// Action error
#[derive(Debug, Clone)]
pub struct ActionError {
    pub code: ActionErrorCode,
    pub message: String,
    pub recoverable: bool,
    pub suggestion: Option<String>,
}

/// Error codes
#[derive(Debug, Clone, PartialEq)]
pub enum ActionErrorCode {
    NotFound,
    PermissionDenied,
    NetworkError,
    InvalidParameters,
    ServiceUnavailable,
    Timeout,
    Cancelled,
    Unknown,
}

/// Queued action awaiting execution
#[derive(Debug, Clone)]
pub struct QueuedAction {
    pub id: u64,
    pub request: ActionRequest,
    pub queued_at: std::time::Instant,
    pub status: QueueStatus,
}

/// Queue status
#[derive(Debug, Clone, PartialEq)]
pub enum QueueStatus {
    Pending,
    AwaitingConfirmation,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

/// Execution history record
#[derive(Debug, Clone)]
struct ExecutionRecord {
    intent: String,
    success: bool,
    duration_ms: u64,
    timestamp: std::time::Instant,
}

/// Action execution policies
#[derive(Debug, Clone)]
pub struct ActionPolicies {
    /// Require confirmation for financial actions
    pub confirm_financial: bool,
    /// Require confirmation for communication
    pub confirm_communication: bool,
    /// Max concurrent actions
    pub max_concurrent: usize,
    /// Action timeout in seconds
    pub timeout_secs: u64,
    /// Allow offline actions
    pub allow_offline: bool,
}

impl Default for ActionPolicies {
    fn default() -> Self {
        Self {
            confirm_financial: true,
            confirm_communication: false,
            max_concurrent: 3,
            timeout_secs: 30,
            allow_offline: true,
        }
    }
}

impl ActionExecutor {
    /// Create new action executor
    pub fn new() -> Self {
        let mut executor = Self {
            handlers: HashMap::new(),
            action_queue: Arc::new(Mutex::new(Vec::new())),
            history: Vec::new(),
            max_history: 100,
            policies: ActionPolicies::default(),
        };
        executor.register_default_handlers();
        executor
    }
    
    /// Create with custom policies
    pub fn with_policies(policies: ActionPolicies) -> Self {
        let mut executor = Self::new();
        executor.policies = policies;
        executor
    }
    
    /// Register default action handlers
    fn register_default_handlers(&mut self) {
        // Navigation handler
        self.handlers.insert("navigate".to_string(), Box::new(NavigationHandler));
        self.handlers.insert("show_map".to_string(), Box::new(NavigationHandler));
        self.handlers.insert("find_nearby".to_string(), Box::new(NavigationHandler));
        
        // Communication handler
        self.handlers.insert("call".to_string(), Box::new(CommunicationHandler));
        self.handlers.insert("message".to_string(), Box::new(CommunicationHandler));
        self.handlers.insert("send_email".to_string(), Box::new(CommunicationHandler));
        self.handlers.insert("check_email".to_string(), Box::new(CommunicationHandler));
        self.handlers.insert("show_contact".to_string(), Box::new(CommunicationHandler));
        self.handlers.insert("share".to_string(), Box::new(CommunicationHandler));
        
        // Media handler
        self.handlers.insert("play_media".to_string(), Box::new(MediaHandler));
        self.handlers.insert("pause_media".to_string(), Box::new(MediaHandler));
        self.handlers.insert("skip_track".to_string(), Box::new(MediaHandler));
        self.handlers.insert("volume".to_string(), Box::new(MediaHandler));
        self.handlers.insert("show_gallery".to_string(), Box::new(MediaHandler));
        self.handlers.insert("show_photo".to_string(), Box::new(MediaHandler));
        
        // Camera handler
        self.handlers.insert("take_photo".to_string(), Box::new(CameraHandler));
        self.handlers.insert("record_video".to_string(), Box::new(CameraHandler));
        
        // Productivity handler
        self.handlers.insert("set_timer".to_string(), Box::new(ProductivityHandler));
        self.handlers.insert("set_reminder".to_string(), Box::new(ProductivityHandler));
        self.handlers.insert("show_calendar".to_string(), Box::new(ProductivityHandler));
        self.handlers.insert("create_event".to_string(), Box::new(ProductivityHandler));
        
        // System handler
        self.handlers.insert("launch_app".to_string(), Box::new(SystemHandler));
        self.handlers.insert("close_app".to_string(), Box::new(SystemHandler));
        self.handlers.insert("download".to_string(), Box::new(SystemHandler));
        self.handlers.insert("show_downloads".to_string(), Box::new(SystemHandler));
        self.handlers.insert("show_notifications".to_string(), Box::new(SystemHandler));
        self.handlers.insert("clear_notifications".to_string(), Box::new(SystemHandler));
        self.handlers.insert("battery_status".to_string(), Box::new(SystemHandler));
        self.handlers.insert("help".to_string(), Box::new(SystemHandler));
        self.handlers.insert("cancel".to_string(), Box::new(SystemHandler));
        
        // Settings handler
        self.handlers.insert("adjust_brightness".to_string(), Box::new(SettingsHandler));
        self.handlers.insert("toggle_dnd".to_string(), Box::new(SettingsHandler));
        self.handlers.insert("toggle_wifi".to_string(), Box::new(SettingsHandler));
        self.handlers.insert("toggle_bluetooth".to_string(), Box::new(SettingsHandler));
        
        // Information handler
        self.handlers.insert("question".to_string(), Box::new(InformationHandler));
        self.handlers.insert("search".to_string(), Box::new(InformationHandler));
        self.handlers.insert("weather".to_string(), Box::new(InformationHandler));
        self.handlers.insert("time".to_string(), Box::new(InformationHandler));
        self.handlers.insert("read_aloud".to_string(), Box::new(InformationHandler));
        
        // Vision handler
        self.handlers.insert("identify_object".to_string(), Box::new(VisionHandler));
        self.handlers.insert("translate".to_string(), Box::new(VisionHandler));
        
        // Finance handler
        self.handlers.insert("check_balance".to_string(), Box::new(FinanceHandler));
        self.handlers.insert("transfer".to_string(), Box::new(FinanceHandler));
    }
    
    /// Execute an intent
    pub fn execute(&mut self, intent: &ResolvedIntent, entities: &[ExtractedEntity], context: &AiContext) -> ActionResult {
        let start = std::time::Instant::now();
        
        // Build action request
        let request = self.build_request(intent, entities, context);
        
        // Check if confirmation needed
        if self.needs_confirmation(&request) {
            return ActionResult {
                success: false,
                message: self.get_confirmation_message(&request),
                data: None,
                follow_up: vec![],
                error: None,
            };
        }
        
        // Find handler
        let result = if let Some(handler) = self.handlers.get(&intent.name) {
            // Check if action can execute
            if !handler.can_execute(&request, context) {
                ActionResult {
                    success: false,
                    message: "This action isn't available right now.".to_string(),
                    data: None,
                    follow_up: vec![],
                    error: Some(ActionError {
                        code: ActionErrorCode::ServiceUnavailable,
                        message: "Action not available in current context".to_string(),
                        recoverable: true,
                        suggestion: Some("Try again later or check your connection".to_string()),
                    }),
                }
            } else {
                handler.execute(&request)
            }
        } else {
            // No specific handler, use generic response
            self.generic_execute(&request)
        };
        
        // Record execution
        self.history.push(ExecutionRecord {
            intent: intent.name.clone(),
            success: result.success,
            duration_ms: start.elapsed().as_millis() as u64,
            timestamp: start,
        });
        
        // Trim history
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        
        result
    }
    
    /// Build action request from intent
    fn build_request(&self, intent: &ResolvedIntent, entities: &[ExtractedEntity], context: &AiContext) -> ActionRequest {
        let mut parameters = HashMap::new();
        
        // Convert entities to parameters
        for entity in entities {
            if let Some(slot_name) = &entity.slot_name {
                parameters.insert(slot_name.clone(), entity.normalized_value.clone());
            } else {
                let key = format!("{:?}", entity.entity_type).to_lowercase();
                parameters.insert(key, entity.normalized_value.clone());
            }
        }
        
        ActionRequest {
            intent: intent.name.clone(),
            category: intent.category.clone(),
            parameters,
            entities: entities.to_vec(),
            context: context.clone(),
            requires_confirmation: intent.requires_confirmation,
            priority: if intent.requires_confirmation {
                ActionPriority::Normal
            } else {
                ActionPriority::High
            },
        }
    }
    
    /// Check if action needs confirmation
    fn needs_confirmation(&self, request: &ActionRequest) -> bool {
        if request.requires_confirmation {
            return true;
        }
        
        match request.category {
            IntentCategory::Finance => self.policies.confirm_financial,
            IntentCategory::Communication => {
                self.policies.confirm_communication && 
                (request.intent == "send_email" || request.intent == "message")
            }
            _ => false,
        }
    }
    
    /// Get confirmation message
    fn get_confirmation_message(&self, request: &ActionRequest) -> String {
        if let Some(handler) = self.handlers.get(&request.intent) {
            if let Some(msg) = handler.get_confirmation_message(request) {
                return msg;
            }
        }
        
        // Default confirmation messages
        match request.intent.as_str() {
            "transfer" => {
                let amount = request.parameters.get("amount").map(|s| s.as_str()).unwrap_or("some");
                let recipient = request.parameters.get("recipient").map(|s| s.as_str()).unwrap_or("someone");
                format!("Send {} to {}?", amount, recipient)
            }
            "message" | "send_email" => {
                let contact = request.parameters.get("contact")
                    .or_else(|| request.parameters.get("recipient"))
                    .map(|s| s.as_str())
                    .unwrap_or("them");
                format!("Send message to {}?", contact)
            }
            "create_event" => {
                let title = request.parameters.get("title").map(|s| s.as_str()).unwrap_or("this event");
                format!("Create {}?", title)
            }
            _ => format!("Proceed with {}?", request.intent.replace('_', " ")),
        }
    }
    
    /// Execute confirmed action
    pub fn execute_confirmed(&mut self, request: &ActionRequest) -> ActionResult {
        if let Some(handler) = self.handlers.get(&request.intent) {
            handler.execute(request)
        } else {
            self.generic_execute(request)
        }
    }
    
    /// Generic execution for unhandled intents
    fn generic_execute(&self, request: &ActionRequest) -> ActionResult {
        ActionResult {
            success: true,
            message: format!("Processing {}...", request.intent.replace('_', " ")),
            data: None,
            follow_up: vec![],
            error: None,
        }
    }
    
    /// Queue an action for later execution
    pub fn queue_action(&self, request: ActionRequest) -> u64 {
        let mut queue = self.action_queue.lock().unwrap();
        let id = queue.len() as u64 + 1;
        queue.push(QueuedAction {
            id,
            request,
            queued_at: std::time::Instant::now(),
            status: QueueStatus::Pending,
        });
        id
    }
    
    /// Cancel a queued action
    pub fn cancel_queued(&self, id: u64) -> bool {
        let mut queue = self.action_queue.lock().unwrap();
        if let Some(action) = queue.iter_mut().find(|a| a.id == id) {
            if action.status == QueueStatus::Pending || action.status == QueueStatus::AwaitingConfirmation {
                action.status = QueueStatus::Cancelled;
                return true;
            }
        }
        false
    }
    
    /// Get execution success rate
    pub fn success_rate(&self) -> f32 {
        if self.history.is_empty() {
            return 1.0;
        }
        
        let successful = self.history.iter().filter(|r| r.success).count();
        successful as f32 / self.history.len() as f32
    }
    
    /// Get average execution time
    pub fn avg_execution_time_ms(&self) -> u64 {
        if self.history.is_empty() {
            return 0;
        }
        
        let total: u64 = self.history.iter().map(|r| r.duration_ms).sum();
        total / self.history.len() as u64
    }
}

impl Default for ActionExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// Default handler implementations

struct NavigationHandler;
impl ActionHandler for NavigationHandler {
    fn execute(&self, action: &ActionRequest) -> ActionResult {
        let destination = action.parameters.get("destination")
            .or_else(|| action.parameters.get("location"))
            .map(|s| s.as_str())
            .unwrap_or("destination");
        
        match action.intent.as_str() {
            "navigate" => ActionResult {
                success: true,
                message: format!("Starting navigation to {}", destination),
                data: Some(ActionData::Navigation {
                    route: vec![(0.0, 0.0), (1.0, 1.0)], // Placeholder
                    eta_minutes: 15,
                }),
                follow_up: vec![],
                error: None,
            },
            "show_map" => ActionResult {
                success: true,
                message: "Showing map".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
            "find_nearby" => {
                let place_type = action.parameters.get("place_type").map(|s| s.as_str()).unwrap_or("places");
                ActionResult {
                    success: true,
                    message: format!("Finding {} nearby", place_type),
                    data: Some(ActionData::List(vec![
                        format!("{} 1 - 0.2 mi", place_type),
                        format!("{} 2 - 0.5 mi", place_type),
                    ])),
                    follow_up: vec![],
                    error: None,
                }
            }
            _ => ActionResult {
                success: true,
                message: "Navigation action completed".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
        }
    }
    
    fn can_execute(&self, _action: &ActionRequest, context: &AiContext) -> bool {
        context.device_state.gps_enabled
    }
    
    fn get_confirmation_message(&self, _action: &ActionRequest) -> Option<String> {
        None
    }
}

struct CommunicationHandler;
impl ActionHandler for CommunicationHandler {
    fn execute(&self, action: &ActionRequest) -> ActionResult {
        let contact = action.parameters.get("contact")
            .or_else(|| action.parameters.get("recipient"))
            .map(|s| s.as_str())
            .unwrap_or("contact");
        
        match action.intent.as_str() {
            "call" => ActionResult {
                success: true,
                message: format!("Calling {}", contact),
                data: None,
                follow_up: vec![],
                error: None,
            },
            "message" => {
                let content = action.parameters.get("content")
                    .or_else(|| action.parameters.get("message_content"))
                    .map(|s| s.as_str())
                    .unwrap_or("your message");
                ActionResult {
                    success: true,
                    message: format!("Sending message to {}: \"{}\"", contact, content),
                    data: None,
                    follow_up: vec![],
                    error: None,
                }
            },
            "send_email" => ActionResult {
                success: true,
                message: format!("Composing email to {}", contact),
                data: None,
                follow_up: vec![],
                error: None,
            },
            "check_email" => ActionResult {
                success: true,
                message: "You have 3 new emails".to_string(),
                data: Some(ActionData::List(vec![
                    "From: John - Meeting tomorrow".to_string(),
                    "From: Mom - How are you?".to_string(),
                    "From: Work - Weekly report".to_string(),
                ])),
                follow_up: vec![],
                error: None,
            },
            "show_contact" => ActionResult {
                success: true,
                message: format!("Contact: {}", contact),
                data: Some(ActionData::Contact {
                    name: contact.to_string(),
                    phone: Some("+1-555-0123".to_string()),
                    email: Some(format!("{}@email.com", contact.to_lowercase())),
                }),
                follow_up: vec![],
                error: None,
            },
            "share" => ActionResult {
                success: true,
                message: "Sharing...".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
            _ => ActionResult {
                success: true,
                message: "Communication action completed".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
        }
    }
    
    fn can_execute(&self, _action: &ActionRequest, _context: &AiContext) -> bool {
        true
    }
    
    fn get_confirmation_message(&self, action: &ActionRequest) -> Option<String> {
        if action.intent == "message" || action.intent == "send_email" {
            let contact = action.parameters.get("contact").map(|s| s.as_str()).unwrap_or("them");
            Some(format!("Send to {}?", contact))
        } else {
            None
        }
    }
}

struct MediaHandler;
impl ActionHandler for MediaHandler {
    fn execute(&self, action: &ActionRequest) -> ActionResult {
        match action.intent.as_str() {
            "play_media" => {
                let query = action.parameters.get("media_name")
                    .or_else(|| action.parameters.get("query"))
                    .map(|s| s.as_str());
                let msg = if let Some(q) = query {
                    format!("Playing {}", q)
                } else {
                    "Resuming playback".to_string()
                };
                ActionResult {
                    success: true,
                    message: msg,
                    data: Some(ActionData::Media {
                        title: query.unwrap_or("Now Playing").to_string(),
                        artist: Some("Artist".to_string()),
                        duration_secs: 240,
                    }),
                    follow_up: vec![],
                    error: None,
                }
            },
            "pause_media" => ActionResult {
                success: true,
                message: "Paused".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
            "skip_track" => {
                let direction = action.parameters.get("direction").map(|s| s.as_str()).unwrap_or("next");
                ActionResult {
                    success: true,
                    message: format!("Playing {} track", direction),
                    data: None,
                    follow_up: vec![],
                    error: None,
                }
            },
            "volume" => {
                let direction = action.parameters.get("direction").map(|s| s.as_str()).unwrap_or("adjusted");
                ActionResult {
                    success: true,
                    message: format!("Volume {}", direction),
                    data: None,
                    follow_up: vec![],
                    error: None,
                }
            },
            "show_gallery" => ActionResult {
                success: true,
                message: "Opening gallery".to_string(),
                data: Some(ActionData::List(vec![
                    "Photo 1 - Today".to_string(),
                    "Photo 2 - Today".to_string(),
                    "Photo 3 - Yesterday".to_string(),
                ])),
                follow_up: vec![],
                error: None,
            },
            "show_photo" => {
                let query = action.parameters.get("query")
                    .or_else(|| action.parameters.get("date"))
                    .map(|s| s.as_str())
                    .unwrap_or("recent");
                ActionResult {
                    success: true,
                    message: format!("Showing {} photos", query),
                    data: Some(ActionData::Image {
                        path: "/photos/recent.jpg".to_string(),
                        thumbnail: Some("/photos/recent_thumb.jpg".to_string()),
                    }),
                    follow_up: vec![],
                    error: None,
                }
            },
            _ => ActionResult {
                success: true,
                message: "Media action completed".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
        }
    }
    
    fn can_execute(&self, _action: &ActionRequest, _context: &AiContext) -> bool {
        true
    }
    
    fn get_confirmation_message(&self, _action: &ActionRequest) -> Option<String> {
        None
    }
}

struct CameraHandler;
impl ActionHandler for CameraHandler {
    fn execute(&self, action: &ActionRequest) -> ActionResult {
        match action.intent.as_str() {
            "take_photo" => ActionResult {
                success: true,
                message: "Photo taken!".to_string(),
                data: Some(ActionData::Image {
                    path: "/photos/new_photo.jpg".to_string(),
                    thumbnail: Some("/photos/new_photo_thumb.jpg".to_string()),
                }),
                follow_up: vec![],
                error: None,
            },
            "record_video" => ActionResult {
                success: true,
                message: "Recording started".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
            _ => ActionResult {
                success: true,
                message: "Camera action completed".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
        }
    }
    
    fn can_execute(&self, _action: &ActionRequest, context: &AiContext) -> bool {
        context.device_state.camera_available
    }
    
    fn get_confirmation_message(&self, _action: &ActionRequest) -> Option<String> {
        None
    }
}

struct ProductivityHandler;
impl ActionHandler for ProductivityHandler {
    fn execute(&self, action: &ActionRequest) -> ActionResult {
        match action.intent.as_str() {
            "set_timer" => {
                let duration = action.parameters.get("duration").map(|s| s.as_str()).unwrap_or("5 minutes");
                ActionResult {
                    success: true,
                    message: format!("Timer set for {}", duration),
                    data: None,
                    follow_up: vec![],
                    error: None,
                }
            },
            "set_reminder" => {
                let content = action.parameters.get("content").map(|s| s.as_str()).unwrap_or("reminder");
                let time = action.parameters.get("time").map(|s| s.as_str()).unwrap_or("later");
                ActionResult {
                    success: true,
                    message: format!("I'll remind you to {} at {}", content, time),
                    data: None,
                    follow_up: vec![],
                    error: None,
                }
            },
            "show_calendar" => ActionResult {
                success: true,
                message: "Here's your schedule".to_string(),
                data: Some(ActionData::Events(vec![
                    CalendarEvent {
                        title: "Team Meeting".to_string(),
                        start_time: "10:00 AM".to_string(),
                        end_time: Some("11:00 AM".to_string()),
                        location: Some("Conference Room".to_string()),
                    },
                    CalendarEvent {
                        title: "Lunch".to_string(),
                        start_time: "12:30 PM".to_string(),
                        end_time: None,
                        location: None,
                    },
                ])),
                follow_up: vec![],
                error: None,
            },
            "create_event" => {
                let title = action.parameters.get("title").map(|s| s.as_str()).unwrap_or("event");
                ActionResult {
                    success: true,
                    message: format!("Created: {}", title),
                    data: None,
                    follow_up: vec![],
                    error: None,
                }
            },
            _ => ActionResult {
                success: true,
                message: "Productivity action completed".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
        }
    }
    
    fn can_execute(&self, _action: &ActionRequest, _context: &AiContext) -> bool {
        true
    }
    
    fn get_confirmation_message(&self, action: &ActionRequest) -> Option<String> {
        if action.intent == "create_event" {
            let title = action.parameters.get("title").map(|s| s.as_str()).unwrap_or("this event");
            Some(format!("Create {}?", title))
        } else {
            None
        }
    }
}

struct SystemHandler;
impl ActionHandler for SystemHandler {
    fn execute(&self, action: &ActionRequest) -> ActionResult {
        match action.intent.as_str() {
            "launch_app" => {
                let app = action.parameters.get("app_name").map(|s| s.as_str()).unwrap_or("app");
                ActionResult {
                    success: true,
                    message: format!("Opening {}", app),
                    data: None,
                    follow_up: vec![],
                    error: None,
                }
            },
            "close_app" => ActionResult {
                success: true,
                message: "App closed".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
            "download" => {
                let item = action.parameters.get("item").map(|s| s.as_str()).unwrap_or("file");
                ActionResult {
                    success: true,
                    message: format!("Downloading {}", item),
                    data: None,
                    follow_up: vec![],
                    error: None,
                }
            },
            "show_downloads" => ActionResult {
                success: true,
                message: "Your downloads".to_string(),
                data: Some(ActionData::List(vec![
                    "Document.pdf - Complete".to_string(),
                    "Photo.jpg - Complete".to_string(),
                ])),
                follow_up: vec![],
                error: None,
            },
            "show_notifications" => ActionResult {
                success: true,
                message: "You have 5 notifications".to_string(),
                data: Some(ActionData::Notifications(vec![
                    NotificationInfo {
                        app: "Messages".to_string(),
                        title: "New message".to_string(),
                        body: "Hey, are you free?".to_string(),
                        timestamp: 1234567890,
                    },
                    NotificationInfo {
                        app: "Calendar".to_string(),
                        title: "Reminder".to_string(),
                        body: "Meeting in 15 minutes".to_string(),
                        timestamp: 1234567800,
                    },
                ])),
                follow_up: vec![],
                error: None,
            },
            "clear_notifications" => ActionResult {
                success: true,
                message: "Notifications cleared".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
            "battery_status" => {
                let level = action.context.device_state.battery_level;
                let charging = if action.context.device_state.is_charging { ", charging" } else { "" };
                ActionResult {
                    success: true,
                    message: format!("Battery at {}%{}", level, charging),
                    data: Some(ActionData::Structured({
                        let mut map = HashMap::new();
                        map.insert("level".to_string(), level.to_string());
                        map.insert("charging".to_string(), action.context.device_state.is_charging.to_string());
                        map
                    })),
                    follow_up: vec![],
                    error: None,
                }
            },
            "help" => ActionResult {
                success: true,
                message: "I can help you with: calling, messaging, navigation, playing music, taking photos, setting reminders, checking your wallet, searching the web, and more. Just ask!".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
            "cancel" => ActionResult {
                success: true,
                message: "Cancelled".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
            _ => ActionResult {
                success: true,
                message: "System action completed".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
        }
    }
    
    fn can_execute(&self, _action: &ActionRequest, _context: &AiContext) -> bool {
        true
    }
    
    fn get_confirmation_message(&self, _action: &ActionRequest) -> Option<String> {
        None
    }
}

struct SettingsHandler;
impl ActionHandler for SettingsHandler {
    fn execute(&self, action: &ActionRequest) -> ActionResult {
        match action.intent.as_str() {
            "adjust_brightness" => {
                let direction = action.parameters.get("direction").map(|s| s.as_str()).unwrap_or("adjusted");
                ActionResult {
                    success: true,
                    message: format!("Brightness {}", direction),
                    data: None,
                    follow_up: vec![],
                    error: None,
                }
            },
            "toggle_dnd" => ActionResult {
                success: true,
                message: "Do Not Disturb enabled".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
            "toggle_wifi" => {
                let state = action.parameters.get("state").map(|s| s.as_str()).unwrap_or("toggled");
                ActionResult {
                    success: true,
                    message: format!("WiFi {}", state),
                    data: None,
                    follow_up: vec![],
                    error: None,
                }
            },
            "toggle_bluetooth" => {
                let state = action.parameters.get("state").map(|s| s.as_str()).unwrap_or("toggled");
                ActionResult {
                    success: true,
                    message: format!("Bluetooth {}", state),
                    data: None,
                    follow_up: vec![],
                    error: None,
                }
            },
            _ => ActionResult {
                success: true,
                message: "Settings updated".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
        }
    }
    
    fn can_execute(&self, _action: &ActionRequest, _context: &AiContext) -> bool {
        true
    }
    
    fn get_confirmation_message(&self, _action: &ActionRequest) -> Option<String> {
        None
    }
}

struct InformationHandler;
impl ActionHandler for InformationHandler {
    fn execute(&self, action: &ActionRequest) -> ActionResult {
        match action.intent.as_str() {
            "question" | "search" => {
                let query = action.parameters.get("query").map(|s| s.as_str()).unwrap_or("your question");
                ActionResult {
                    success: true,
                    message: format!("Here's what I found about {}", query),
                    data: Some(ActionData::Text("Search results would appear here".to_string())),
                    follow_up: vec![],
                    error: None,
                }
            },
            "weather" => {
                let location = action.parameters.get("location").map(|s| s.as_str()).unwrap_or("your location");
                ActionResult {
                    success: true,
                    message: format!("Weather for {}: 72°F, Sunny", location),
                    data: Some(ActionData::Weather {
                        temp_c: 22.0,
                        condition: "Sunny".to_string(),
                        forecast: vec!["Tomorrow: 75°F".to_string(), "Saturday: 68°F".to_string()],
                    }),
                    follow_up: vec![],
                    error: None,
                }
            },
            "time" => {
                let now = chrono::Local::now();
                ActionResult {
                    success: true,
                    message: format!("It's {}", now.format("%I:%M %p")),
                    data: None,
                    follow_up: vec![],
                    error: None,
                }
            },
            "read_aloud" => ActionResult {
                success: true,
                message: "Reading content...".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
            _ => ActionResult {
                success: true,
                message: "Information retrieved".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
        }
    }
    
    fn can_execute(&self, action: &ActionRequest, context: &AiContext) -> bool {
        // Most info queries need network except time
        if action.intent == "time" {
            return true;
        }
        context.device_state.wifi_connected
    }
    
    fn get_confirmation_message(&self, _action: &ActionRequest) -> Option<String> {
        None
    }
}

struct VisionHandler;
impl ActionHandler for VisionHandler {
    fn execute(&self, action: &ActionRequest) -> ActionResult {
        match action.intent.as_str() {
            "identify_object" => ActionResult {
                success: true,
                message: "Analyzing what you're looking at...".to_string(),
                data: Some(ActionData::Text("Object identification result".to_string())),
                follow_up: vec![],
                error: None,
            },
            "translate" => {
                let target = action.parameters.get("target_language").map(|s| s.as_str()).unwrap_or("English");
                ActionResult {
                    success: true,
                    message: format!("Translating to {}...", target),
                    data: Some(ActionData::Text("Translation result".to_string())),
                    follow_up: vec![],
                    error: None,
                }
            },
            _ => ActionResult {
                success: true,
                message: "Vision action completed".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
        }
    }
    
    fn can_execute(&self, _action: &ActionRequest, context: &AiContext) -> bool {
        context.device_state.camera_available
    }
    
    fn get_confirmation_message(&self, _action: &ActionRequest) -> Option<String> {
        None
    }
}

struct FinanceHandler;
impl ActionHandler for FinanceHandler {
    fn execute(&self, action: &ActionRequest) -> ActionResult {
        match action.intent.as_str() {
            "check_balance" => {
                let currency = action.parameters.get("currency").map(|s| s.as_str()).unwrap_or("SOL");
                ActionResult {
                    success: true,
                    message: format!("Your {} balance: 10.5", currency),
                    data: Some(ActionData::Balance {
                        amount: 10.5,
                        currency: currency.to_string(),
                    }),
                    follow_up: vec![],
                    error: None,
                }
            },
            "transfer" => {
                let amount = action.parameters.get("amount").map(|s| s.as_str()).unwrap_or("0");
                let recipient = action.parameters.get("recipient").map(|s| s.as_str()).unwrap_or("recipient");
                ActionResult {
                    success: true,
                    message: format!("Sent {} to {}", amount, recipient),
                    data: None,
                    follow_up: vec![],
                    error: None,
                }
            },
            _ => ActionResult {
                success: true,
                message: "Finance action completed".to_string(),
                data: None,
                follow_up: vec![],
                error: None,
            },
        }
    }
    
    fn can_execute(&self, _action: &ActionRequest, context: &AiContext) -> bool {
        context.device_state.wifi_connected
    }
    
    fn get_confirmation_message(&self, action: &ActionRequest) -> Option<String> {
        if action.intent == "transfer" {
            let amount = action.parameters.get("amount").map(|s| s.as_str()).unwrap_or("funds");
            let recipient = action.parameters.get("recipient").map(|s| s.as_str()).unwrap_or("recipient");
            Some(format!("Confirm: Send {} to {}?", amount, recipient))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_action_executor_creation() {
        let executor = ActionExecutor::new();
        assert!(!executor.handlers.is_empty());
    }
    
    #[test]
    fn test_execute_call() {
        let mut executor = ActionExecutor::new();
        let context = AiContext::default();
        
        let intent = ResolvedIntent {
            name: "call".to_string(),
            category: IntentCategory::Communication,
            confidence: 0.9,
            requires_confirmation: false,
            requires_reasoning: false,
            missing_slots: vec![],
            feasible: true,
            alternative: None,
            continues_previous: false,
        };
        
        let entities = vec![ExtractedEntity {
            entity_type: crate::ai_layer::entities::EntityType::Contact,
            value: "Mom".to_string(),
            normalized_value: "Mom".to_string(),
            confidence: 0.9,
            start_pos: 5,
            end_pos: 8,
            slot_name: Some("contact".to_string()),
        }];
        
        let result = executor.execute(&intent, &entities, &context);
        assert!(result.success);
        assert!(result.message.contains("Mom"));
    }
    
    #[test]
    fn test_execute_navigate() {
        let mut executor = ActionExecutor::new();
        let mut context = AiContext::default();
        context.device_state.gps_enabled = true;
        
        let intent = ResolvedIntent {
            name: "navigate".to_string(),
            category: IntentCategory::Navigation,
            confidence: 0.9,
            requires_confirmation: false,
            requires_reasoning: false,
            missing_slots: vec![],
            feasible: true,
            alternative: None,
            continues_previous: false,
        };
        
        let entities = vec![ExtractedEntity {
            entity_type: crate::ai_layer::entities::EntityType::Location,
            value: "home".to_string(),
            normalized_value: "home".to_string(),
            confidence: 0.9,
            start_pos: 12,
            end_pos: 16,
            slot_name: Some("destination".to_string()),
        }];
        
        let result = executor.execute(&intent, &entities, &context);
        assert!(result.success);
        assert!(result.data.is_some());
    }
    
    #[test]
    fn test_confirmation_required() {
        let mut executor = ActionExecutor::new();
        let context = AiContext::default();
        
        let intent = ResolvedIntent {
            name: "transfer".to_string(),
            category: IntentCategory::Finance,
            confidence: 0.9,
            requires_confirmation: true,
            requires_reasoning: false,
            missing_slots: vec![],
            feasible: true,
            alternative: None,
            continues_previous: false,
        };
        
        let entities = vec![];
        
        let result = executor.execute(&intent, &entities, &context);
        // Should not succeed immediately due to confirmation
        assert!(!result.success || result.message.contains("?"));
    }
    
    #[test]
    fn test_success_rate() {
        let mut executor = ActionExecutor::new();
        let context = AiContext::default();
        
        // Execute a few actions
        let intent = ResolvedIntent {
            name: "help".to_string(),
            category: IntentCategory::System,
            confidence: 0.9,
            requires_confirmation: false,
            requires_reasoning: false,
            missing_slots: vec![],
            feasible: true,
            alternative: None,
            continues_previous: false,
        };
        
        executor.execute(&intent, &[], &context);
        executor.execute(&intent, &[], &context);
        
        assert!(executor.success_rate() > 0.0);
    }
}
