//! Voice Shortcuts and Macros
//!
//! User-defined voice shortcuts and command macros.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Voice shortcut manager
pub struct ShortcutManager {
    /// User shortcuts
    shortcuts: HashMap<String, VoiceShortcut>,
    /// System shortcuts
    system_shortcuts: HashMap<String, VoiceShortcut>,
    /// Macro recordings
    macros: HashMap<String, VoiceMacro>,
    /// Recording state
    recording: Option<MacroRecording>,
    /// Configuration
    config: ShortcutConfig,
}

impl ShortcutManager {
    /// Create new manager
    pub fn new(config: ShortcutConfig) -> Self {
        let mut manager = Self {
            shortcuts: HashMap::new(),
            system_shortcuts: HashMap::new(),
            macros: HashMap::new(),
            recording: None,
            config,
        };

        manager.register_system_shortcuts();
        manager
    }

    /// Register a shortcut
    pub fn register(&mut self, trigger: &str, shortcut: VoiceShortcut) -> Result<(), ShortcutError> {
        let trigger_lower = trigger.to_lowercase();

        // Check for conflicts
        if self.system_shortcuts.contains_key(&trigger_lower) {
            return Err(ShortcutError::SystemShortcutConflict);
        }

        if self.shortcuts.len() >= self.config.max_shortcuts {
            return Err(ShortcutError::LimitReached);
        }

        self.shortcuts.insert(trigger_lower, shortcut);
        Ok(())
    }

    /// Unregister a shortcut
    pub fn unregister(&mut self, trigger: &str) -> bool {
        self.shortcuts.remove(&trigger.to_lowercase()).is_some()
    }

    /// Find matching shortcut
    pub fn find(&self, phrase: &str) -> Option<&VoiceShortcut> {
        let phrase_lower = phrase.to_lowercase();

        // Check user shortcuts first
        if let Some(shortcut) = self.shortcuts.get(&phrase_lower) {
            return Some(shortcut);
        }

        // Check system shortcuts
        if let Some(shortcut) = self.system_shortcuts.get(&phrase_lower) {
            return Some(shortcut);
        }

        // Check partial matches
        for (trigger, shortcut) in &self.shortcuts {
            if phrase_lower.contains(trigger) || trigger.contains(&phrase_lower) {
                return Some(shortcut);
            }
        }

        None
    }

    /// Start recording a macro
    pub fn start_recording(&mut self, name: &str) -> Result<(), ShortcutError> {
        if self.recording.is_some() {
            return Err(ShortcutError::AlreadyRecording);
        }

        self.recording = Some(MacroRecording {
            name: name.to_string(),
            commands: Vec::new(),
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        });

        Ok(())
    }

    /// Record a command
    pub fn record_command(&mut self, command: &str) {
        if let Some(recording) = &mut self.recording {
            recording.commands.push(RecordedCommand {
                command: command.to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            });
        }
    }

    /// Stop recording and save macro
    pub fn stop_recording(&mut self) -> Result<VoiceMacro, ShortcutError> {
        let recording = self.recording.take()
            .ok_or(ShortcutError::NotRecording)?;

        if recording.commands.is_empty() {
            return Err(ShortcutError::EmptyMacro);
        }

        let macro_def = VoiceMacro {
            name: recording.name.clone(),
            description: String::new(),
            commands: recording.commands.iter()
                .map(|c| c.command.clone())
                .collect(),
            delay_between: self.config.default_macro_delay,
        };

        self.macros.insert(recording.name, macro_def.clone());
        Ok(macro_def)
    }

    /// Cancel recording
    pub fn cancel_recording(&mut self) {
        self.recording = None;
    }

    /// Get macro by name
    pub fn get_macro(&self, name: &str) -> Option<&VoiceMacro> {
        self.macros.get(name)
    }

    /// Delete macro
    pub fn delete_macro(&mut self, name: &str) -> bool {
        self.macros.remove(name).is_some()
    }

    /// List all shortcuts
    pub fn list_shortcuts(&self) -> Vec<ShortcutInfo> {
        self.shortcuts.iter()
            .map(|(trigger, shortcut)| ShortcutInfo {
                trigger: trigger.clone(),
                action: shortcut.description.clone(),
                category: shortcut.category,
                is_system: false,
            })
            .chain(self.system_shortcuts.iter().map(|(trigger, shortcut)| ShortcutInfo {
                trigger: trigger.clone(),
                action: shortcut.description.clone(),
                category: shortcut.category,
                is_system: true,
            }))
            .collect()
    }

    /// List all macros
    pub fn list_macros(&self) -> Vec<MacroInfo> {
        self.macros.iter()
            .map(|(name, macro_def)| MacroInfo {
                name: name.clone(),
                description: macro_def.description.clone(),
                command_count: macro_def.commands.len(),
            })
            .collect()
    }

    /// Register system shortcuts
    fn register_system_shortcuts(&mut self) {
        let system = vec![
            ("go back", ShortcutAction::Navigate("back".to_string())),
            ("go home", ShortcutAction::Navigate("home".to_string())),
            ("take screenshot", ShortcutAction::System("screenshot".to_string())),
            ("show notifications", ShortcutAction::System("notifications".to_string())),
            ("show recent", ShortcutAction::System("recents".to_string())),
            ("lock screen", ShortcutAction::System("lock".to_string())),
            ("volume up", ShortcutAction::Media("volume_up".to_string())),
            ("volume down", ShortcutAction::Media("volume_down".to_string())),
            ("play music", ShortcutAction::Media("play".to_string())),
            ("pause music", ShortcutAction::Media("pause".to_string())),
            ("next track", ShortcutAction::Media("next".to_string())),
            ("previous track", ShortcutAction::Media("previous".to_string())),
            ("what time is it", ShortcutAction::Query("time".to_string())),
            ("show battery", ShortcutAction::Query("battery".to_string())),
        ];

        for (trigger, action) in system {
            self.system_shortcuts.insert(trigger.to_string(), VoiceShortcut {
                action,
                description: trigger.to_string(),
                enabled: true,
                category: ShortcutCategory::System,
            });
        }
    }

    /// Is currently recording
    pub fn is_recording(&self) -> bool {
        self.recording.is_some()
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new(ShortcutConfig::default())
    }
}

/// Shortcut configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    /// Maximum user shortcuts
    pub max_shortcuts: usize,
    /// Maximum macros
    pub max_macros: usize,
    /// Default delay between macro commands (ms)
    pub default_macro_delay: u32,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            max_shortcuts: 100,
            max_macros: 50,
            default_macro_delay: 500,
        }
    }
}

/// Voice shortcut definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceShortcut {
    /// Action to perform
    pub action: ShortcutAction,
    /// Description
    pub description: String,
    /// Is enabled
    pub enabled: bool,
    /// Category
    pub category: ShortcutCategory,
}

/// Shortcut action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShortcutAction {
    /// Open application
    OpenApp(String),
    /// Navigate to location
    Navigate(String),
    /// Execute system command
    System(String),
    /// Media control
    Media(String),
    /// Query information
    Query(String),
    /// Run macro
    RunMacro(String),
    /// Send message
    Message {
        recipient: Option<String>,
        template: Option<String>,
    },
    /// Custom command sequence
    Custom(Vec<String>),
}

/// Shortcut category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShortcutCategory {
    System,
    Navigation,
    Media,
    Communication,
    Productivity,
    Custom,
}

/// Shortcut info for listing
#[derive(Debug, Clone)]
pub struct ShortcutInfo {
    /// Trigger phrase
    pub trigger: String,
    /// Action description
    pub action: String,
    /// Category
    pub category: ShortcutCategory,
    /// Is system shortcut
    pub is_system: bool,
}

/// Voice macro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceMacro {
    /// Macro name
    pub name: String,
    /// Description
    pub description: String,
    /// Commands to execute
    pub commands: Vec<String>,
    /// Delay between commands (ms)
    pub delay_between: u32,
}

/// Macro recording state
struct MacroRecording {
    /// Name
    name: String,
    /// Recorded commands
    commands: Vec<RecordedCommand>,
    /// Start time
    start_time: u64,
}

/// Recorded command
struct RecordedCommand {
    /// Command text
    command: String,
    /// Timestamp
    timestamp: u64,
}

/// Macro info for listing
#[derive(Debug, Clone)]
pub struct MacroInfo {
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Number of commands
    pub command_count: usize,
}

/// Shortcut errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutError {
    /// Conflicts with system shortcut
    SystemShortcutConflict,
    /// Maximum shortcuts reached
    LimitReached,
    /// Already recording
    AlreadyRecording,
    /// Not recording
    NotRecording,
    /// Empty macro
    EmptyMacro,
    /// Invalid trigger
    InvalidTrigger,
}

impl std::fmt::Display for ShortcutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortcutError::SystemShortcutConflict => write!(f, "Conflicts with system shortcut"),
            ShortcutError::LimitReached => write!(f, "Maximum shortcuts reached"),
            ShortcutError::AlreadyRecording => write!(f, "Already recording a macro"),
            ShortcutError::NotRecording => write!(f, "Not currently recording"),
            ShortcutError::EmptyMacro => write!(f, "Cannot save empty macro"),
            ShortcutError::InvalidTrigger => write!(f, "Invalid trigger phrase"),
        }
    }
}

impl std::error::Error for ShortcutError {}

/// Shortcut builder
pub struct ShortcutBuilder {
    action: Option<ShortcutAction>,
    description: String,
    category: ShortcutCategory,
}

impl ShortcutBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            action: None,
            description: String::new(),
            category: ShortcutCategory::Custom,
        }
    }

    /// Set action to open app
    pub fn open_app(mut self, app: &str) -> Self {
        self.action = Some(ShortcutAction::OpenApp(app.to_string()));
        self
    }

    /// Set action to navigate
    pub fn navigate(mut self, target: &str) -> Self {
        self.action = Some(ShortcutAction::Navigate(target.to_string()));
        self
    }

    /// Set action to system command
    pub fn system(mut self, command: &str) -> Self {
        self.action = Some(ShortcutAction::System(command.to_string()));
        self
    }

    /// Set action to media control
    pub fn media(mut self, control: &str) -> Self {
        self.action = Some(ShortcutAction::Media(control.to_string()));
        self
    }

    /// Set action to run macro
    pub fn run_macro(mut self, name: &str) -> Self {
        self.action = Some(ShortcutAction::RunMacro(name.to_string()));
        self
    }

    /// Set action to custom commands
    pub fn custom(mut self, commands: Vec<String>) -> Self {
        self.action = Some(ShortcutAction::Custom(commands));
        self
    }

    /// Set description
    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Set category
    pub fn category(mut self, category: ShortcutCategory) -> Self {
        self.category = category;
        self
    }

    /// Build shortcut
    pub fn build(self) -> Result<VoiceShortcut, ShortcutError> {
        let action = self.action.ok_or(ShortcutError::InvalidTrigger)?;

        Ok(VoiceShortcut {
            action,
            description: self.description,
            enabled: true,
            category: self.category,
        })
    }
}

impl Default for ShortcutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro executor
pub struct MacroExecutor {
    /// Current execution
    current: Option<MacroExecution>,
}

impl MacroExecutor {
    /// Create new executor
    pub fn new() -> Self {
        Self { current: None }
    }

    /// Start macro execution
    pub fn start(&mut self, macro_def: &VoiceMacro) {
        self.current = Some(MacroExecution {
            name: macro_def.name.clone(),
            commands: macro_def.commands.clone(),
            current_index: 0,
            delay: macro_def.delay_between,
            paused: false,
        });
    }

    /// Get next command
    pub fn next_command(&mut self) -> Option<String> {
        if let Some(execution) = &mut self.current {
            if execution.paused {
                return None;
            }

            if execution.current_index < execution.commands.len() {
                let cmd = execution.commands[execution.current_index].clone();
                execution.current_index += 1;
                return Some(cmd);
            } else {
                self.current = None;
            }
        }
        None
    }

    /// Pause execution
    pub fn pause(&mut self) {
        if let Some(execution) = &mut self.current {
            execution.paused = true;
        }
    }

    /// Resume execution
    pub fn resume(&mut self) {
        if let Some(execution) = &mut self.current {
            execution.paused = false;
        }
    }

    /// Stop execution
    pub fn stop(&mut self) {
        self.current = None;
    }

    /// Is executing
    pub fn is_executing(&self) -> bool {
        self.current.is_some()
    }

    /// Get progress
    pub fn progress(&self) -> Option<(usize, usize)> {
        self.current.as_ref().map(|e| (e.current_index, e.commands.len()))
    }
}

impl Default for MacroExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro execution state
struct MacroExecution {
    name: String,
    commands: Vec<String>,
    current_index: usize,
    delay: u32,
    paused: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_manager() {
        let manager = ShortcutManager::default();
        assert!(!manager.system_shortcuts.is_empty());
    }

    #[test]
    fn test_register_shortcut() {
        let mut manager = ShortcutManager::default();

        let shortcut = ShortcutBuilder::new()
            .open_app("camera")
            .description("Open camera")
            .build()
            .unwrap();

        assert!(manager.register("open camera", shortcut).is_ok());
        assert!(manager.find("open camera").is_some());
    }

    #[test]
    fn test_system_shortcut_conflict() {
        let mut manager = ShortcutManager::default();

        let shortcut = ShortcutBuilder::new()
            .open_app("test")
            .build()
            .unwrap();

        // "go back" is a system shortcut
        let result = manager.register("go back", shortcut);
        assert_eq!(result, Err(ShortcutError::SystemShortcutConflict));
    }

    #[test]
    fn test_macro_recording() {
        let mut manager = ShortcutManager::default();

        assert!(manager.start_recording("test_macro").is_ok());
        assert!(manager.is_recording());

        manager.record_command("open camera");
        manager.record_command("take photo");

        let macro_def = manager.stop_recording().unwrap();
        assert_eq!(macro_def.commands.len(), 2);
    }

    #[test]
    fn test_macro_executor() {
        let mut executor = MacroExecutor::new();

        let macro_def = VoiceMacro {
            name: "test".to_string(),
            description: String::new(),
            commands: vec!["cmd1".to_string(), "cmd2".to_string()],
            delay_between: 100,
        };

        executor.start(&macro_def);
        assert!(executor.is_executing());

        assert_eq!(executor.next_command(), Some("cmd1".to_string()));
        assert_eq!(executor.next_command(), Some("cmd2".to_string()));
        assert_eq!(executor.next_command(), None);
        assert!(!executor.is_executing());
    }

    #[test]
    fn test_shortcut_builder() {
        let shortcut = ShortcutBuilder::new()
            .navigate("settings")
            .description("Open settings")
            .category(ShortcutCategory::Navigation)
            .build()
            .unwrap();

        assert_eq!(shortcut.category, ShortcutCategory::Navigation);
    }

    #[test]
    fn test_list_shortcuts() {
        let manager = ShortcutManager::default();
        let shortcuts = manager.list_shortcuts();
        assert!(!shortcuts.is_empty());
    }
}
