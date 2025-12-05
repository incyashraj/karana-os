//! Voice Command Definitions
//!
//! Structures for defining and executing voice commands.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Voice command definition
pub struct VoiceCommand {
    /// Command name/identifier
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Trigger patterns (supports wildcards with *)
    pub patterns: Vec<String>,
    /// Command handler
    pub handler: Box<dyn Fn(HashMap<String, String>) -> CommandResult + Send + Sync>,
    /// Whether command is destructive (requires confirmation)
    pub destructive: bool,
    /// Command category
    pub category: CommandCategory,
}

/// Command categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandCategory {
    /// Navigation commands
    Navigation,
    /// Window management
    Window,
    /// System controls
    System,
    /// Application commands
    Application,
    /// Media controls
    Media,
    /// Communication
    Communication,
    /// Accessibility
    Accessibility,
    /// Custom/user-defined
    Custom,
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Result message
    pub message: String,
    /// Output data (if any)
    pub data: Option<CommandData>,
    /// Error details (if failed)
    pub error: Option<String>,
}

impl CommandResult {
    /// Create success result
    pub fn success(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: None,
            error: None,
        }
    }

    /// Create success with data
    pub fn success_with_data(message: &str, data: CommandData) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
            error: None,
        }
    }

    /// Create failure result
    pub fn failure(error: &str) -> Self {
        Self {
            success: false,
            message: String::new(),
            data: None,
            error: Some(error.to_string()),
        }
    }
}

/// Command output data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandData {
    /// Text data
    Text(String),
    /// Numeric data
    Number(f64),
    /// Boolean data
    Boolean(bool),
    /// List of items
    List(Vec<String>),
    /// Key-value pairs
    Map(HashMap<String, String>),
    /// JSON data
    Json(String),
}

/// Parsed command from voice input
#[derive(Debug)]
pub enum ParsedCommand {
    /// Recognized command
    Command {
        name: String,
        args: HashMap<String, String>,
        confidence: f32,
    },
    /// Question/query
    Question {
        query: String,
    },
    /// Shortcut invocation
    Shortcut {
        name: String,
    },
    /// Confirmation response
    Confirmation {
        confirmed: bool,
    },
    /// Continuation of previous command
    Continuation {
        context: String,
    },
    /// Unknown/unrecognized
    Unknown {
        text: String,
    },
}

/// Command builder for easy registration
pub struct CommandBuilder {
    name: String,
    description: String,
    patterns: Vec<String>,
    destructive: bool,
    category: CommandCategory,
}

impl CommandBuilder {
    /// Create new builder
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            patterns: Vec::new(),
            destructive: false,
            category: CommandCategory::Custom,
        }
    }

    /// Set description
    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Add pattern
    pub fn pattern(mut self, pattern: &str) -> Self {
        self.patterns.push(pattern.to_string());
        self
    }

    /// Add multiple patterns
    pub fn patterns(mut self, patterns: &[&str]) -> Self {
        self.patterns.extend(patterns.iter().map(|s| s.to_string()));
        self
    }

    /// Mark as destructive
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    /// Set category
    pub fn category(mut self, category: CommandCategory) -> Self {
        self.category = category;
        self
    }

    /// Build command with handler
    pub fn build<F>(self, handler: F) -> VoiceCommand
    where
        F: Fn(HashMap<String, String>) -> CommandResult + Send + Sync + 'static,
    {
        VoiceCommand {
            name: self.name,
            description: self.description,
            patterns: self.patterns,
            handler: Box::new(handler),
            destructive: self.destructive,
            category: self.category,
        }
    }
}

/// Intent extracted from voice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceIntent {
    /// Intent type
    pub intent_type: IntentType,
    /// Entities extracted
    pub entities: HashMap<String, Entity>,
    /// Raw text
    pub text: String,
    /// Confidence
    pub confidence: f32,
}

/// Intent types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentType {
    /// Command intent
    Command,
    /// Query/question
    Query,
    /// Navigation
    Navigate,
    /// Control media
    MediaControl,
    /// System action
    SystemAction,
    /// Communication
    Communicate,
    /// Confirmation
    Confirm,
    /// Denial
    Deny,
    /// Greeting
    Greeting,
    /// Unknown
    Unknown,
}

/// Entity in intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Entity type
    pub entity_type: EntityType,
    /// Entity value
    pub value: String,
    /// Confidence
    pub confidence: f32,
    /// Start position in text
    pub start: usize,
    /// End position in text
    pub end: usize,
}

/// Entity types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    /// Application name
    AppName,
    /// Contact name
    Contact,
    /// Location
    Location,
    /// Time
    Time,
    /// Date
    Date,
    /// Number
    Number,
    /// Direction
    Direction,
    /// Action
    Action,
    /// Object
    Object,
    /// Unknown
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result_success() {
        let result = CommandResult::success("Done");
        assert!(result.success);
        assert_eq!(result.message, "Done");
    }

    #[test]
    fn test_command_result_failure() {
        let result = CommandResult::failure("Error");
        assert!(!result.success);
        assert_eq!(result.error.unwrap(), "Error");
    }

    #[test]
    fn test_command_builder() {
        let command = CommandBuilder::new("test")
            .description("Test command")
            .patterns(&["test", "try"])
            .category(CommandCategory::Custom)
            .build(|_| CommandResult::success("OK"));

        assert_eq!(command.name, "test");
        assert_eq!(command.patterns.len(), 2);
    }

    #[test]
    fn test_command_categories() {
        let nav = CommandCategory::Navigation;
        let sys = CommandCategory::System;
        assert_ne!(nav, sys);
    }
}
