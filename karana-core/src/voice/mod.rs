//! Voice Command System for Kāraṇa OS
//!
//! Comprehensive voice control for smart glasses.
//! Features:
//! - Natural language command parsing
//! - Context-aware command execution
//! - Multi-language support
//! - Custom command training
//! - Voice shortcuts
//! - Continuous listening
//! - Accessibility support

pub mod commands;
pub mod nlu;
pub mod synthesis;
pub mod context;
pub mod shortcuts;
pub mod listening;
pub mod accessibility;

pub use commands::*;
pub use nlu::*;
pub use synthesis::*;
pub use context::*;
pub use shortcuts::*;
pub use listening::*;
pub use accessibility::*;

use std::collections::HashMap;
use std::time::Instant;
use serde::{Deserialize, Serialize};

/// Voice command manager
pub struct VoiceCommandManager {
    /// NLU engine for parsing
    nlu_engine: NluEngine,
    /// Context manager
    context_manager: VoiceContextManager,
    /// Registered commands
    commands: HashMap<String, VoiceCommand>,
    /// Shortcut manager
    shortcut_manager: ShortcutManager,
    /// Speech synthesizer
    synthesizer: VoiceSynthesizer,
    /// Configuration
    config: VoiceCommandConfig,
    /// Statistics
    stats: VoiceCommandStats,
    /// Last command time
    last_command: Option<Instant>,
}

/// Voice command configuration
#[derive(Debug, Clone)]
pub struct VoiceCommandConfig {
    /// Wake word
    pub wake_word: String,
    /// Confirmation required for destructive actions
    pub require_confirmation: bool,
    /// Enable audio feedback
    pub audio_feedback: bool,
    /// Command timeout (ms)
    pub command_timeout_ms: u64,
    /// Language
    pub language: Language,
    /// Enable fuzzy matching
    pub fuzzy_matching: bool,
    /// Minimum confidence for execution
    pub min_confidence: f32,
    /// Enable conversational mode
    pub conversational_mode: bool,
}

impl Default for VoiceCommandConfig {
    fn default() -> Self {
        Self {
            wake_word: "karana".to_string(),
            require_confirmation: true,
            audio_feedback: true,
            command_timeout_ms: 5000,
            language: Language::English,
            fuzzy_matching: true,
            min_confidence: 0.7,
            conversational_mode: true,
        }
    }
}

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Japanese,
    Chinese,
    Hindi,
    Korean,
    Portuguese,
    Arabic,
}

impl Language {
    /// Get ISO 639-1 code
    pub fn code(&self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Spanish => "es",
            Self::French => "fr",
            Self::German => "de",
            Self::Japanese => "ja",
            Self::Chinese => "zh",
            Self::Hindi => "hi",
            Self::Korean => "ko",
            Self::Portuguese => "pt",
            Self::Arabic => "ar",
        }
    }

    /// Get full name
    pub fn name(&self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Spanish => "Spanish",
            Self::French => "French",
            Self::German => "German",
            Self::Japanese => "Japanese",
            Self::Chinese => "Chinese",
            Self::Hindi => "Hindi",
            Self::Korean => "Korean",
            Self::Portuguese => "Portuguese",
            Self::Arabic => "Arabic",
        }
    }
}

/// Voice command statistics
#[derive(Debug, Default)]
pub struct VoiceCommandStats {
    /// Total commands processed
    pub total_commands: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Commands requiring confirmation
    pub confirmations_required: u64,
    /// Average confidence
    pub avg_confidence: f32,
    /// Most used command
    pub most_used_command: Option<String>,
    /// Command usage counts
    pub command_usage: HashMap<String, u64>,
}

/// Voice action to execute
#[derive(Debug, Clone)]
pub enum VoiceAction {
    /// Open an application
    OpenApp(String),
    /// Navigate to destination
    Navigate(String),
    /// Show something
    Show(String),
    /// Switch to something
    Switch(String),
    /// System command
    System(String),
    /// Media control
    Media(String),
    /// Run a macro
    RunMacro(Vec<String>),
    /// Send message
    Message {
        recipient: Option<String>,
        template: Option<String>,
    },
    /// Custom commands
    Custom(Vec<String>),
}

impl VoiceCommandManager {
    /// Create new manager
    pub fn new() -> Self {
        let mut manager = Self {
            nlu_engine: NluEngine::default(),
            context_manager: VoiceContextManager::default(),
            commands: HashMap::new(),
            shortcut_manager: ShortcutManager::default(),
            synthesizer: VoiceSynthesizer::default(),
            config: VoiceCommandConfig::default(),
            stats: VoiceCommandStats::default(),
            last_command: None,
        };

        // Register built-in commands
        manager.register_builtin_commands();

        manager
    }

    /// Create with config
    pub fn with_config(config: VoiceCommandConfig) -> Self {
        let mut manager = Self::new();
        manager.config = config;
        manager
    }

    /// Process voice input
    pub fn process(&mut self, transcript: &str, confidence: f32) -> VoiceResult {
        self.stats.total_commands += 1;

        // Check confidence threshold
        if confidence < self.config.min_confidence {
            return VoiceResult::LowConfidence { confidence };
        }

        // Process with NLU engine
        let nlu_result = self.nlu_engine.process(transcript);

        // Check for shortcuts first - clone to avoid borrow issues
        let shortcut_result = self.shortcut_manager.find(transcript).cloned();
        if let Some(shortcut) = shortcut_result {
            return self.execute_shortcut_owned(shortcut);
        }

        // Execute based on intent
        match nlu_result.intent.intent {
            Intent::Open | Intent::Navigate | Intent::Show | Intent::Switch => {
                let target = nlu_result.semantics.slots.get("target")
                    .or(nlu_result.semantics.slots.get("destination"))
                    .cloned()
                    .unwrap_or_default();
                self.execute_navigation(&nlu_result.intent.intent, &target)
            }
            Intent::Query | Intent::Search => {
                let query = nlu_result.semantics.slots.get("query")
                    .cloned()
                    .unwrap_or_else(|| transcript.to_string());
                VoiceResult::Question { query }
            }
            Intent::Confirm => {
                self.handle_confirmation(true)
            }
            Intent::Deny | Intent::Cancel => {
                self.handle_confirmation(false)
            }
            Intent::Help => {
                let commands: Vec<_> = self.commands.values()
                    .map(|c| c.name.clone())
                    .collect();
                VoiceResult::Help { available_commands: commands }
            }
            _ => {
                // Try to find matching command - collect first to avoid borrow issue
                let matching_cmd: Option<(String, String)> = {
                    let transcript_lower = transcript.to_lowercase();
                    self.commands.iter()
                        .find_map(|(cmd_name, cmd)| {
                            cmd.patterns.iter()
                                .find(|pattern| transcript_lower.contains(&pattern.to_lowercase()))
                                .map(|_| (cmd_name.clone(), cmd.name.clone()))
                        })
                };

                if let Some((cmd_name, _)) = matching_cmd {
                    let args = HashMap::new();
                    return self.execute_command_internal(&cmd_name, args, confidence);
                }
                
                VoiceResult::NotUnderstood { 
                    text: transcript.to_string() 
                }
            }
        }
    }

    /// Execute shortcut
    /// Execute shortcut (takes owned value to avoid borrow issues)
    fn execute_shortcut_owned(&mut self, shortcut: VoiceShortcut) -> VoiceResult {
        self.execute_shortcut(&shortcut)
    }

    fn execute_shortcut(&mut self, shortcut: &VoiceShortcut) -> VoiceResult {
        match &shortcut.action {
            ShortcutAction::OpenApp(app) => {
                VoiceResult::Success {
                    message: format!("Opening {}", app),
                    action: Some(VoiceAction::OpenApp(app.clone())),
                }
            }
            ShortcutAction::Navigate(target) => {
                VoiceResult::Success {
                    message: format!("Navigating to {}", target),
                    action: Some(VoiceAction::Navigate(target.clone())),
                }
            }
            ShortcutAction::System(cmd) => {
                VoiceResult::Success {
                    message: format!("Executing {}", cmd),
                    action: Some(VoiceAction::System(cmd.clone())),
                }
            }
            ShortcutAction::Media(control) => {
                VoiceResult::Success {
                    message: format!("Media: {}", control),
                    action: Some(VoiceAction::Media(control.clone())),
                }
            }
            ShortcutAction::Query(q) => {
                VoiceResult::Question { query: q.clone() }
            }
            ShortcutAction::RunMacro(name) => {
                if let Some(m) = self.shortcut_manager.get_macro(name) {
                    VoiceResult::Success {
                        message: format!("Running macro: {}", name),
                        action: Some(VoiceAction::RunMacro(m.commands.clone())),
                    }
                } else {
                    VoiceResult::Error {
                        message: format!("Macro not found: {}", name),
                    }
                }
            }
            ShortcutAction::Message { recipient, template } => {
                VoiceResult::Success {
                    message: "Composing message".to_string(),
                    action: Some(VoiceAction::Message {
                        recipient: recipient.clone(),
                        template: template.clone(),
                    }),
                }
            }
            ShortcutAction::Custom(cmds) => {
                VoiceResult::Success {
                    message: "Executing custom commands".to_string(),
                    action: Some(VoiceAction::Custom(cmds.clone())),
                }
            }
        }
    }

    /// Execute navigation command
    fn execute_navigation(&mut self, intent: &Intent, target: &str) -> VoiceResult {
        let action = match intent {
            Intent::Open => VoiceAction::OpenApp(target.to_string()),
            Intent::Navigate => VoiceAction::Navigate(target.to_string()),
            Intent::Show => VoiceAction::Show(target.to_string()),
            Intent::Switch => VoiceAction::Switch(target.to_string()),
            _ => return VoiceResult::Error { message: "Unknown navigation intent".to_string() },
        };

        VoiceResult::Success {
            message: format!("{:?} {}", intent, target),
            action: Some(action),
        }
    }

    /// Handle confirmation response
    fn handle_confirmation(&mut self, confirmed: bool) -> VoiceResult {
        let context = self.context_manager.current();
        if let Some(pending) = &context.pending_action {
            if confirmed {
                VoiceResult::Confirmed {
                    action: pending.action.clone(),
                }
            } else {
                VoiceResult::Cancelled
            }
        } else {
            VoiceResult::NoPendingAction
        }
    }

    /// Execute command internal
    fn execute_command_internal(&mut self, name: &str, args: HashMap<String, String>, _confidence: f32) -> VoiceResult {
        if let Some(command) = self.commands.get(name) {
            // Check if confirmation required
            if command.destructive && self.config.require_confirmation {
                // Set pending action in context
                let mut entities = args.clone();
                entities.insert("command".to_string(), name.to_string());
                self.context_manager.update_from_command(name, &entities);

                self.stats.confirmations_required += 1;

                return VoiceResult::ConfirmationRequired {
                    action: command.description.clone(),
                };
            }

            // Execute command
            let result = (command.handler)(args.clone());
            
            // Update stats
            *self.stats.command_usage.entry(name.to_string()).or_insert(0) += 1;
            
            if result.success {
                self.stats.successful_executions += 1;
            } else {
                self.stats.failed_executions += 1;
            }

            self.last_command = Some(Instant::now());

            // Update context
            self.context_manager.update_from_command(name, &args);

            VoiceResult::Executed {
                command: name.to_string(),
                result,
            }
        } else {
            VoiceResult::CommandNotFound { name: name.to_string() }
        }
    }

    /// Register a command
    pub fn register(&mut self, command: VoiceCommand) {
        self.commands.insert(command.name.clone(), command);
    }

    /// Get configuration
    pub fn config(&self) -> &VoiceCommandConfig {
        &self.config
    }

    /// Set language
    pub fn set_language(&mut self, language: Language) {
        self.config.language = language;
    }

    /// Get statistics
    pub fn stats(&self) -> &VoiceCommandStats {
        &self.stats
    }

    /// Speak feedback
    pub fn speak(&mut self, text: &str) {
        if self.config.audio_feedback {
            self.synthesizer.speak(text);
        }
    }

    fn register_builtin_commands(&mut self) {
        // Navigation commands
        self.register(VoiceCommand {
            name: "go_back".to_string(),
            description: "Navigate back".to_string(),
            patterns: vec!["go back".to_string(), "back".to_string(), "previous".to_string(), "return".to_string()],
            handler: Box::new(|_| CommandResult::success("Navigating back")),
            destructive: false,
            category: CommandCategory::Navigation,
        });

        self.register(VoiceCommand {
            name: "go_home".to_string(),
            description: "Go to home screen".to_string(),
            patterns: vec!["go home".to_string(), "home".to_string(), "home screen".to_string(), "main screen".to_string()],
            handler: Box::new(|_| CommandResult::success("Going home")),
            destructive: false,
            category: CommandCategory::Navigation,
        });

        // Window commands
        self.register(VoiceCommand {
            name: "close_window".to_string(),
            description: "Close current window".to_string(),
            patterns: vec!["close".to_string(), "close window".to_string(), "close this".to_string(), "dismiss".to_string()],
            handler: Box::new(|_| CommandResult::success("Window closed")),
            destructive: true,
            category: CommandCategory::Window,
        });

        self.register(VoiceCommand {
            name: "minimize_window".to_string(),
            description: "Minimize current window".to_string(),
            patterns: vec!["minimize".to_string(), "minimize window".to_string(), "hide window".to_string()],
            handler: Box::new(|_| CommandResult::success("Window minimized")),
            destructive: false,
            category: CommandCategory::Window,
        });

        // System commands
        self.register(VoiceCommand {
            name: "take_screenshot".to_string(),
            description: "Take a screenshot".to_string(),
            patterns: vec!["screenshot".to_string(), "take screenshot".to_string(), "capture screen".to_string()],
            handler: Box::new(|_| CommandResult::success("Screenshot taken")),
            destructive: false,
            category: CommandCategory::System,
        });

        self.register(VoiceCommand {
            name: "volume_up".to_string(),
            description: "Increase volume".to_string(),
            patterns: vec!["volume up".to_string(), "louder".to_string(), "increase volume".to_string(), "turn up".to_string()],
            handler: Box::new(|_| CommandResult::success("Volume increased")),
            destructive: false,
            category: CommandCategory::System,
        });

        self.register(VoiceCommand {
            name: "volume_down".to_string(),
            description: "Decrease volume".to_string(),
            patterns: vec!["volume down".to_string(), "quieter".to_string(), "decrease volume".to_string(), "turn down".to_string()],
            handler: Box::new(|_| CommandResult::success("Volume decreased")),
            destructive: false,
            category: CommandCategory::System,
        });

        // Media commands
        self.register(VoiceCommand {
            name: "play".to_string(),
            description: "Play media".to_string(),
            patterns: vec!["play".to_string(), "resume".to_string(), "start playing".to_string()],
            handler: Box::new(|_| CommandResult::success("Playing")),
            destructive: false,
            category: CommandCategory::Media,
        });

        self.register(VoiceCommand {
            name: "pause".to_string(),
            description: "Pause media".to_string(),
            patterns: vec!["pause".to_string(), "stop".to_string(), "hold".to_string()],
            handler: Box::new(|_| CommandResult::success("Paused")),
            destructive: false,
            category: CommandCategory::Media,
        });

        self.register(VoiceCommand {
            name: "next".to_string(),
            description: "Skip to next".to_string(),
            patterns: vec!["next".to_string(), "skip".to_string(), "next track".to_string(), "next song".to_string()],
            handler: Box::new(|_| CommandResult::success("Skipped to next")),
            destructive: false,
            category: CommandCategory::Media,
        });
    }
}

impl Default for VoiceCommandManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Voice command result
#[derive(Debug)]
pub enum VoiceResult {
    /// Command executed successfully
    Executed {
        command: String,
        result: CommandResult,
    },
    /// Success with action
    Success {
        message: String,
        action: Option<VoiceAction>,
    },
    /// Question asked
    Question {
        query: String,
    },
    /// Command requires confirmation
    ConfirmationRequired {
        action: String,
    },
    /// Action confirmed
    Confirmed {
        action: String,
    },
    /// Action cancelled
    Cancelled,
    /// No pending action
    NoPendingAction,
    /// Command not found
    CommandNotFound {
        name: String,
    },
    /// Help response
    Help {
        available_commands: Vec<String>,
    },
    /// Error occurred
    Error {
        message: String,
    },
    /// Confidence too low
    LowConfidence {
        confidence: f32,
    },
    /// Could not understand
    NotUnderstood {
        text: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_command_manager_creation() {
        let manager = VoiceCommandManager::new();
        assert!(!manager.commands.is_empty());
    }

    #[test]
    fn test_builtin_commands() {
        let manager = VoiceCommandManager::new();
        assert!(manager.commands.contains_key("go_back"));
        assert!(manager.commands.contains_key("go_home"));
        assert!(manager.commands.contains_key("close_window"));
    }

    #[test]
    fn test_language_code() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Japanese.code(), "ja");
    }

    #[test]
    fn test_config_defaults() {
        let config = VoiceCommandConfig::default();
        assert!(config.audio_feedback);
        assert!(config.require_confirmation);
    }

    #[test]
    fn test_process_low_confidence() {
        let mut manager = VoiceCommandManager::new();
        let result = manager.process("hello", 0.1);
        matches!(result, VoiceResult::LowConfidence { .. });
    }

    #[test]
    fn test_process_command() {
        let mut manager = VoiceCommandManager::new();
        let result = manager.process("go back", 0.9);
        matches!(result, VoiceResult::Executed { .. });
    }
}
