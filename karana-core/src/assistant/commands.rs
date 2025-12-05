// Command Processing for Kāraṇa OS
// Natural language command parsing and execution

use std::collections::HashMap;

/// Command categories
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandCategory {
    System,
    Navigation,
    Communication,
    Media,
    SmartHome,
    Information,
    Timer,
    Settings,
    Custom,
    Unknown,
}

/// Command intent
#[derive(Debug, Clone)]
pub struct CommandIntent {
    pub category: CommandCategory,
    pub action: String,
    pub entities: HashMap<String, String>,
    pub confidence: f32,
}

/// Command result
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub response_type: ResponseType,
    pub actions: Vec<super::AssistantAction>,
    pub suggestions: Vec<String>,
    pub confidence: f32,
}

impl Default for CommandResult {
    fn default() -> Self {
        Self {
            success: true,
            response_type: ResponseType::Text("Command processed".to_string()),
            actions: Vec::new(),
            suggestions: Vec::new(),
            confidence: 1.0,
        }
    }
}

/// Response type from command processing
#[derive(Debug, Clone)]
pub enum ResponseType {
    Text(String),
    Action(String),
    Query(String),
    Error(String),
    Clarification(String),
}

/// Command context for understanding
#[derive(Debug, Clone, Default)]
pub struct CommandContext {
    pub location: Option<String>,
    pub time_of_day: Option<String>,
    pub recent_apps: Vec<String>,
    pub active_app: Option<String>,
    pub user_preferences: HashMap<String, String>,
}

/// Command pattern for matching
#[derive(Debug, Clone)]
struct CommandPattern {
    pattern: String,
    category: CommandCategory,
    action: String,
    handler_id: String,
}

/// Command handler
pub struct CommandHandler {
    patterns: Vec<CommandPattern>,
    custom_handlers: HashMap<String, String>,
    fallback_enabled: bool,
}

impl CommandHandler {
    pub fn new() -> Self {
        let mut handler = Self {
            patterns: Vec::new(),
            custom_handlers: HashMap::new(),
            fallback_enabled: true,
        };
        handler.register_default_patterns();
        handler
    }

    fn register_default_patterns(&mut self) {
        // System commands
        self.add_pattern("open *", CommandCategory::System, "open_app");
        self.add_pattern("close *", CommandCategory::System, "close_app");
        self.add_pattern("go back", CommandCategory::System, "back");
        self.add_pattern("go home", CommandCategory::System, "home");
        self.add_pattern("take screenshot", CommandCategory::System, "screenshot");
        self.add_pattern("lock", CommandCategory::System, "lock");

        // Navigation
        self.add_pattern("navigate to *", CommandCategory::Navigation, "navigate");
        self.add_pattern("directions to *", CommandCategory::Navigation, "navigate");
        self.add_pattern("where is *", CommandCategory::Navigation, "find_location");
        self.add_pattern("find nearby *", CommandCategory::Navigation, "find_nearby");

        // Communication
        self.add_pattern("call *", CommandCategory::Communication, "call");
        self.add_pattern("send message to *", CommandCategory::Communication, "message");
        self.add_pattern("text *", CommandCategory::Communication, "message");
        self.add_pattern("read messages", CommandCategory::Communication, "read_messages");

        // Media
        self.add_pattern("play *", CommandCategory::Media, "play");
        self.add_pattern("pause", CommandCategory::Media, "pause");
        self.add_pattern("stop", CommandCategory::Media, "stop");
        self.add_pattern("next", CommandCategory::Media, "next");
        self.add_pattern("previous", CommandCategory::Media, "previous");
        self.add_pattern("volume up", CommandCategory::Media, "volume_up");
        self.add_pattern("volume down", CommandCategory::Media, "volume_down");

        // Timer
        self.add_pattern("set timer for *", CommandCategory::Timer, "set_timer");
        self.add_pattern("set alarm for *", CommandCategory::Timer, "set_alarm");
        self.add_pattern("remind me to *", CommandCategory::Timer, "set_reminder");

        // Information
        self.add_pattern("what time is it", CommandCategory::Information, "time");
        self.add_pattern("what's the weather", CommandCategory::Information, "weather");
        self.add_pattern("search for *", CommandCategory::Information, "search");
        self.add_pattern("who is *", CommandCategory::Information, "search_person");
        self.add_pattern("what is *", CommandCategory::Information, "search_thing");

        // Settings
        self.add_pattern("brightness up", CommandCategory::Settings, "brightness_up");
        self.add_pattern("brightness down", CommandCategory::Settings, "brightness_down");
        self.add_pattern("enable *", CommandCategory::Settings, "enable_setting");
        self.add_pattern("disable *", CommandCategory::Settings, "disable_setting");
    }

    fn add_pattern(&mut self, pattern: &str, category: CommandCategory, action: &str) {
        self.patterns.push(CommandPattern {
            pattern: pattern.to_string(),
            category,
            action: action.to_string(),
            handler_id: action.to_string(),
        });
    }

    pub fn register_pattern(&mut self, pattern: &str, handler_id: &str) {
        self.patterns.push(CommandPattern {
            pattern: pattern.to_string(),
            category: CommandCategory::Custom,
            action: handler_id.to_string(),
            handler_id: handler_id.to_string(),
        });
    }

    pub fn process(&self, text: &str, context: &CommandContext) -> CommandResult {
        let lower_text = text.to_lowercase();

        // Try to match patterns
        for pattern in &self.patterns {
            if let Some(intent) = self.match_pattern(&lower_text, pattern) {
                return self.execute_intent(&intent, context);
            }
        }

        // Fallback to search/general query
        if self.fallback_enabled {
            CommandResult {
                success: true,
                response_type: ResponseType::Query(text.to_string()),
                actions: vec![super::AssistantAction::Search(text.to_string())],
                suggestions: vec![
                    "Did you mean to search for this?".to_string(),
                ],
                confidence: 0.5,
            }
        } else {
            CommandResult {
                success: false,
                response_type: ResponseType::Error("I didn't understand that command".to_string()),
                actions: Vec::new(),
                suggestions: Vec::new(),
                confidence: 0.0,
            }
        }
    }

    fn match_pattern(&self, text: &str, pattern: &CommandPattern) -> Option<CommandIntent> {
        let pattern_parts: Vec<&str> = pattern.pattern.split_whitespace().collect();
        let text_parts: Vec<&str> = text.split_whitespace().collect();

        if pattern_parts.is_empty() {
            return None;
        }

        let mut entities = HashMap::new();
        let mut text_idx = 0;

        for pattern_part in &pattern_parts {
            if *pattern_part == "*" {
                // Wildcard - capture remaining text
                let remaining: Vec<&str> = text_parts[text_idx..].to_vec();
                if !remaining.is_empty() {
                    entities.insert("target".to_string(), remaining.join(" "));
                }
                return Some(CommandIntent {
                    category: pattern.category,
                    action: pattern.action.clone(),
                    entities,
                    confidence: 0.9,
                });
            } else if text_idx < text_parts.len() && 
                      text_parts[text_idx].to_lowercase() == pattern_part.to_lowercase() {
                text_idx += 1;
            } else {
                return None;
            }
        }

        // Check if we consumed all pattern parts
        if text_idx <= text_parts.len() {
            Some(CommandIntent {
                category: pattern.category,
                action: pattern.action.clone(),
                entities,
                confidence: 0.95,
            })
        } else {
            None
        }
    }

    fn execute_intent(&self, intent: &CommandIntent, _context: &CommandContext) -> CommandResult {
        let response = match intent.category {
            CommandCategory::Information => {
                match intent.action.as_str() {
                    "time" => ResponseType::Text("It's currently 3:30 PM".to_string()),
                    "weather" => ResponseType::Text("It's sunny and 72°F".to_string()),
                    _ => ResponseType::Query(intent.entities.get("target")
                        .cloned()
                        .unwrap_or_default()),
                }
            }
            CommandCategory::Navigation => {
                let target = intent.entities.get("target").cloned().unwrap_or_default();
                ResponseType::Action(format!("Navigating to {}", target))
            }
            CommandCategory::Communication => {
                let target = intent.entities.get("target").cloned().unwrap_or_default();
                ResponseType::Action(format!("Calling {}", target))
            }
            CommandCategory::Timer => {
                let time = intent.entities.get("target").cloned().unwrap_or_default();
                ResponseType::Action(format!("Setting timer for {}", time))
            }
            CommandCategory::System => {
                ResponseType::Action(format!("Executing: {}", intent.action))
            }
            CommandCategory::Media => {
                ResponseType::Action(format!("Media: {}", intent.action))
            }
            CommandCategory::Settings => {
                ResponseType::Action(format!("Setting: {}", intent.action))
            }
            _ => ResponseType::Text("Command understood".to_string()),
        };

        CommandResult {
            success: true,
            response_type: response,
            actions: Vec::new(),
            suggestions: Vec::new(),
            confidence: intent.confidence,
        }
    }

    pub fn set_fallback_enabled(&mut self, enabled: bool) {
        self.fallback_enabled = enabled;
    }

    pub fn get_patterns(&self) -> &[CommandPattern] {
        &self.patterns
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_handler_creation() {
        let handler = CommandHandler::new();
        assert!(!handler.get_patterns().is_empty());
    }

    #[test]
    fn test_time_command() {
        let handler = CommandHandler::new();
        let context = CommandContext::default();
        
        let result = handler.process("what time is it", &context);
        assert!(result.success);
        if let ResponseType::Text(text) = result.response_type {
            assert!(text.contains("PM") || text.contains("AM") || text.contains(":"));
        }
    }

    #[test]
    fn test_navigation_command() {
        let handler = CommandHandler::new();
        let context = CommandContext::default();
        
        let result = handler.process("navigate to the grocery store", &context);
        assert!(result.success);
        if let ResponseType::Action(action) = result.response_type {
            assert!(action.contains("grocery store"));
        }
    }

    #[test]
    fn test_call_command() {
        let handler = CommandHandler::new();
        let context = CommandContext::default();
        
        let result = handler.process("call mom", &context);
        assert!(result.success);
    }

    #[test]
    fn test_timer_command() {
        let handler = CommandHandler::new();
        let context = CommandContext::default();
        
        let result = handler.process("set timer for 5 minutes", &context);
        assert!(result.success);
        if let ResponseType::Action(action) = result.response_type {
            assert!(action.contains("5 minutes"));
        }
    }

    #[test]
    fn test_unknown_command() {
        let handler = CommandHandler::new();
        let context = CommandContext::default();
        
        let result = handler.process("xyzzy plugh", &context);
        // Should fall back to search
        if let ResponseType::Query(_) = result.response_type {
            assert!(result.success);
        }
    }

    #[test]
    fn test_fallback_disabled() {
        let mut handler = CommandHandler::new();
        handler.set_fallback_enabled(false);
        let context = CommandContext::default();
        
        let result = handler.process("xyzzy plugh", &context);
        assert!(!result.success);
    }

    #[test]
    fn test_custom_pattern() {
        let mut handler = CommandHandler::new();
        handler.register_pattern("launch rockets", "custom_rocket");
        let context = CommandContext::default();
        
        let result = handler.process("launch rockets", &context);
        assert!(result.success);
    }

    #[test]
    fn test_case_insensitivity() {
        let handler = CommandHandler::new();
        let context = CommandContext::default();
        
        let result1 = handler.process("WHAT TIME IS IT", &context);
        let result2 = handler.process("what time is it", &context);
        
        assert_eq!(result1.success, result2.success);
    }

    #[test]
    fn test_media_commands() {
        let handler = CommandHandler::new();
        let context = CommandContext::default();
        
        let pause = handler.process("pause", &context);
        assert!(pause.success);
        
        let play = handler.process("play some music", &context);
        assert!(play.success);
    }
}
