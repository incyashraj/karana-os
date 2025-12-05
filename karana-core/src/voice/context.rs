//! Voice Context Management
//!
//! Context-aware voice command processing for smart glasses.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Voice context manager
pub struct VoiceContextManager {
    /// Current context stack
    context_stack: Vec<VoiceContext>,
    /// Global context
    global_context: GlobalContext,
    /// Context history
    history: Vec<ContextSnapshot>,
    /// Active session
    session: Option<ContextSession>,
    /// Configuration
    config: ContextConfig,
}

impl VoiceContextManager {
    /// Create new context manager
    pub fn new(config: ContextConfig) -> Self {
        Self {
            context_stack: vec![VoiceContext::default()],
            global_context: GlobalContext::new(),
            history: Vec::new(),
            session: None,
            config,
        }
    }

    /// Push new context
    pub fn push_context(&mut self, context: VoiceContext) {
        // Save snapshot before pushing
        self.save_snapshot();

        self.context_stack.push(context);

        // Limit stack depth
        while self.context_stack.len() > self.config.max_stack_depth {
            self.context_stack.remove(0);
        }
    }

    /// Pop context
    pub fn pop_context(&mut self) -> Option<VoiceContext> {
        if self.context_stack.len() > 1 {
            let popped = self.context_stack.pop();
            self.save_snapshot();
            popped
        } else {
            None
        }
    }

    /// Get current context
    pub fn current(&self) -> &VoiceContext {
        self.context_stack.last().unwrap()
    }

    /// Get current context mutably
    pub fn current_mut(&mut self) -> &mut VoiceContext {
        self.context_stack.last_mut().unwrap()
    }

    /// Update context from command
    pub fn update_from_command(&mut self, command: &str, entities: &HashMap<String, String>) {
        let current = self.current_mut();

        // Update last command
        current.last_command = Some(command.to_string());

        // Update entities
        for (key, value) in entities {
            current.entities.insert(key.clone(), value.clone());
        }

        // Update timestamp
        current.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Track in history
        self.save_snapshot();
    }

    /// Get context for command resolution
    pub fn resolve_context(&self, command: &str) -> ResolvedContext {
        let current = self.current();

        // Determine scope
        let scope = self.determine_scope(command);

        // Get relevant entities
        let entities = self.get_relevant_entities(&scope);

        // Get active focus
        let focus = self.global_context.focus.clone();

        // Get environmental context
        let environment = self.get_environment_context();

        ResolvedContext {
            scope,
            entities,
            focus,
            environment,
            active_app: current.active_app.clone(),
            pending_action: current.pending_action.clone(),
        }
    }

    /// Determine command scope
    fn determine_scope(&self, command: &str) -> ContextScope {
        let command_lower = command.to_lowercase();

        if command_lower.contains("system") || command_lower.contains("settings") {
            ContextScope::System
        } else if command_lower.contains("here") || command_lower.contains("this") {
            ContextScope::Local
        } else if command_lower.contains("everywhere") || command_lower.contains("all") {
            ContextScope::Global
        } else {
            ContextScope::Current
        }
    }

    /// Get relevant entities for scope
    fn get_relevant_entities(&self, scope: &ContextScope) -> HashMap<String, String> {
        let mut entities = HashMap::new();

        match scope {
            ContextScope::Current | ContextScope::Local => {
                entities.extend(self.current().entities.clone());
            }
            ContextScope::Global => {
                // Include all context stack entities
                for ctx in &self.context_stack {
                    entities.extend(ctx.entities.clone());
                }
            }
            ContextScope::System => {
                entities.insert("scope".to_string(), "system".to_string());
            }
        }

        entities
    }

    /// Get environment context
    fn get_environment_context(&self) -> EnvironmentContext {
        let current = self.current();

        EnvironmentContext {
            location: current.location.clone(),
            time_of_day: self.get_time_of_day(),
            ambient_noise: current.ambient_noise,
            user_activity: current.user_activity.clone(),
        }
    }

    /// Get time of day
    fn get_time_of_day(&self) -> TimeOfDay {
        let hour = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| (d.as_secs() / 3600) % 24)
            .unwrap_or(12);

        match hour {
            5..=11 => TimeOfDay::Morning,
            12..=17 => TimeOfDay::Afternoon,
            18..=21 => TimeOfDay::Evening,
            _ => TimeOfDay::Night,
        }
    }

    /// Save context snapshot
    fn save_snapshot(&mut self) {
        let snapshot = ContextSnapshot {
            context: self.current().clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        self.history.push(snapshot);

        // Limit history size
        while self.history.len() > self.config.max_history {
            self.history.remove(0);
        }
    }

    /// Start new session
    pub fn start_session(&mut self) {
        self.session = Some(ContextSession {
            id: self.generate_session_id(),
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            interactions: 0,
            topics: Vec::new(),
        });
    }

    /// End session
    pub fn end_session(&mut self) {
        self.session = None;
        self.context_stack = vec![VoiceContext::default()];
    }

    /// Record interaction
    pub fn record_interaction(&mut self, topic: Option<&str>) {
        if let Some(session) = &mut self.session {
            session.interactions += 1;
            if let Some(t) = topic {
                if !session.topics.contains(&t.to_string()) {
                    session.topics.push(t.to_string());
                }
            }
        }
    }

    /// Set focus
    pub fn set_focus(&mut self, focus: FocusTarget) {
        self.global_context.focus = Some(focus);
    }

    /// Clear focus
    pub fn clear_focus(&mut self) {
        self.global_context.focus = None;
    }

    /// Generate session ID
    fn generate_session_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("session_{}", nanos)
    }
}

impl Default for VoiceContextManager {
    fn default() -> Self {
        Self::new(ContextConfig::default())
    }
}

/// Voice context configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum context stack depth
    pub max_stack_depth: usize,
    /// Maximum history entries
    pub max_history: usize,
    /// Context timeout (seconds)
    pub context_timeout: u64,
    /// Enable location context
    pub use_location: bool,
    /// Enable activity context
    pub use_activity: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_stack_depth: 10,
            max_history: 100,
            context_timeout: 300, // 5 minutes
            use_location: true,
            use_activity: true,
        }
    }
}

/// Voice context
#[derive(Debug, Clone, Default)]
pub struct VoiceContext {
    /// Context type
    pub context_type: ContextType,
    /// Active application
    pub active_app: Option<String>,
    /// Last command
    pub last_command: Option<String>,
    /// Pending action requiring confirmation
    pub pending_action: Option<PendingAction>,
    /// Extracted entities
    pub entities: HashMap<String, String>,
    /// Current location
    pub location: Option<LocationContext>,
    /// Ambient noise level
    pub ambient_noise: f32,
    /// User activity
    pub user_activity: Option<UserActivity>,
    /// Last update timestamp
    pub last_update: u64,
}

/// Context type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ContextType {
    #[default]
    General,
    Navigation,
    Media,
    Communication,
    Work,
    Home,
    Driving,
    Exercise,
}

/// Location context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationContext {
    /// Location name
    pub name: Option<String>,
    /// Location type
    pub location_type: LocationType,
    /// Coordinates (latitude, longitude)
    pub coordinates: Option<(f64, f64)>,
    /// Indoor/outdoor
    pub indoor: bool,
}

/// Location type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationType {
    Home,
    Work,
    Vehicle,
    Store,
    Restaurant,
    Outdoors,
    Transit,
    Unknown,
}

/// User activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    /// Activity type
    pub activity_type: ActivityType,
    /// Confidence
    pub confidence: f32,
    /// Duration (seconds)
    pub duration: u64,
}

/// Activity type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    Stationary,
    Walking,
    Running,
    Cycling,
    Driving,
    Unknown,
}

/// Pending action requiring confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAction {
    /// Action type
    pub action: String,
    /// Target
    pub target: Option<String>,
    /// Parameters
    pub params: HashMap<String, String>,
    /// Expiry timestamp
    pub expires: u64,
}

/// Global context
#[derive(Debug, Clone, Default)]
pub struct GlobalContext {
    /// Current focus
    pub focus: Option<FocusTarget>,
    /// User preferences
    pub preferences: UserPreferences,
    /// Active notifications
    pub notifications: Vec<NotificationContext>,
}

impl GlobalContext {
    /// Create new global context
    pub fn new() -> Self {
        Self::default()
    }
}

/// Focus target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusTarget {
    /// Target type
    pub target_type: FocusTargetType,
    /// Target ID
    pub id: String,
    /// Target name
    pub name: String,
}

/// Focus target type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusTargetType {
    Window,
    Element,
    ArObject,
    Notification,
    Menu,
}

/// User preferences for voice
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred response verbosity
    pub verbosity: Verbosity,
    /// Enable confirmations
    pub confirmations: bool,
    /// Audio feedback enabled
    pub audio_feedback: bool,
    /// Haptic feedback enabled
    pub haptic_feedback: bool,
}

/// Verbosity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Verbosity {
    Minimal,
    #[default]
    Normal,
    Verbose,
}

/// Notification context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationContext {
    /// Notification ID
    pub id: String,
    /// App source
    pub app: String,
    /// Title
    pub title: String,
    /// Priority
    pub priority: NotificationPriority,
}

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Context snapshot for history
#[derive(Debug, Clone)]
struct ContextSnapshot {
    context: VoiceContext,
    timestamp: u64,
}

/// Context session
#[derive(Debug, Clone)]
pub struct ContextSession {
    /// Session ID
    pub id: String,
    /// Start time
    pub start_time: u64,
    /// Number of interactions
    pub interactions: usize,
    /// Topics discussed
    pub topics: Vec<String>,
}

/// Context scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextScope {
    /// Current context only
    Current,
    /// Local (app/window) context
    Local,
    /// Global (system-wide) context
    Global,
    /// System settings context
    System,
}

/// Resolved context for command execution
#[derive(Debug, Clone)]
pub struct ResolvedContext {
    /// Scope
    pub scope: ContextScope,
    /// Relevant entities
    pub entities: HashMap<String, String>,
    /// Current focus
    pub focus: Option<FocusTarget>,
    /// Environment
    pub environment: EnvironmentContext,
    /// Active application
    pub active_app: Option<String>,
    /// Pending action
    pub pending_action: Option<PendingAction>,
}

/// Environment context
#[derive(Debug, Clone)]
pub struct EnvironmentContext {
    /// Location
    pub location: Option<LocationContext>,
    /// Time of day
    pub time_of_day: TimeOfDay,
    /// Ambient noise level
    pub ambient_noise: f32,
    /// User activity
    pub user_activity: Option<UserActivity>,
}

/// Time of day
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDay {
    Morning,
    Afternoon,
    Evening,
    Night,
}

/// Pronoun resolver for context-aware references
pub struct PronounResolver {
    /// Reference history
    references: Vec<Reference>,
}

impl PronounResolver {
    /// Create new resolver
    pub fn new() -> Self {
        Self {
            references: Vec::new(),
        }
    }

    /// Add reference
    pub fn add_reference(&mut self, reference: Reference) {
        self.references.push(reference);

        // Limit history
        while self.references.len() > 20 {
            self.references.remove(0);
        }
    }

    /// Resolve pronoun
    pub fn resolve(&self, pronoun: &str) -> Option<&Reference> {
        let pronoun_lower = pronoun.to_lowercase();

        match pronoun_lower.as_str() {
            "it" | "this" | "that" => {
                self.references.last()
            }
            "them" | "those" | "these" => {
                self.references.iter().rev()
                    .find(|r| r.is_plural)
            }
            "he" | "him" => {
                self.references.iter().rev()
                    .find(|r| r.gender == Some(Gender::Male))
            }
            "she" | "her" => {
                self.references.iter().rev()
                    .find(|r| r.gender == Some(Gender::Female))
            }
            "there" => {
                self.references.iter().rev()
                    .find(|r| r.ref_type == ReferenceType::Location)
            }
            _ => None,
        }
    }

    /// Clear references
    pub fn clear(&mut self) {
        self.references.clear();
    }
}

impl Default for PronounResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Reference for pronoun resolution
#[derive(Debug, Clone)]
pub struct Reference {
    /// Reference type
    pub ref_type: ReferenceType,
    /// Value
    pub value: String,
    /// Is plural
    pub is_plural: bool,
    /// Gender (for people)
    pub gender: Option<Gender>,
}

/// Reference type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceType {
    Object,
    Person,
    Location,
    Time,
    Application,
}

/// Gender
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
    Neutral,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_manager_creation() {
        let manager = VoiceContextManager::default();
        assert_eq!(manager.context_stack.len(), 1);
    }

    #[test]
    fn test_push_pop_context() {
        let mut manager = VoiceContextManager::default();

        let ctx = VoiceContext {
            context_type: ContextType::Navigation,
            ..Default::default()
        };
        manager.push_context(ctx);
        assert_eq!(manager.context_stack.len(), 2);

        manager.pop_context();
        assert_eq!(manager.context_stack.len(), 1);
    }

    #[test]
    fn test_update_from_command() {
        let mut manager = VoiceContextManager::default();
        let mut entities = HashMap::new();
        entities.insert("target".to_string(), "camera".to_string());

        manager.update_from_command("open camera", &entities);

        assert_eq!(manager.current().last_command, Some("open camera".to_string()));
        assert_eq!(manager.current().entities.get("target"), Some(&"camera".to_string()));
    }

    #[test]
    fn test_resolve_context() {
        let manager = VoiceContextManager::default();
        let resolved = manager.resolve_context("open system settings");
        assert_eq!(resolved.scope, ContextScope::System);
    }

    #[test]
    fn test_session_management() {
        let mut manager = VoiceContextManager::default();
        manager.start_session();
        assert!(manager.session.is_some());

        manager.record_interaction(Some("navigation"));
        assert_eq!(manager.session.as_ref().unwrap().interactions, 1);

        manager.end_session();
        assert!(manager.session.is_none());
    }

    #[test]
    fn test_pronoun_resolver() {
        let mut resolver = PronounResolver::new();

        resolver.add_reference(Reference {
            ref_type: ReferenceType::Object,
            value: "camera".to_string(),
            is_plural: false,
            gender: None,
        });

        let resolved = resolver.resolve("it");
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().value, "camera");
    }

    #[test]
    fn test_time_of_day() {
        let manager = VoiceContextManager::default();
        let _time = manager.get_time_of_day();
        // Just verify it doesn't panic
    }
}
