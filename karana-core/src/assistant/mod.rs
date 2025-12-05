// Voice Assistant Module for Kāraṇa OS
// Comprehensive voice AI integration for smart glasses

pub mod recognition;
pub mod synthesis;
pub mod commands;
pub mod context;
pub mod conversation;

pub use recognition::*;
pub use synthesis::*;
pub use commands::*;
pub use context::*;
pub use conversation::*;

use std::collections::HashMap;

/// Voice assistant state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssistantState {
    Idle,
    Listening,
    Processing,
    Speaking,
    Error,
}

/// Wake word detection result
#[derive(Debug, Clone)]
pub struct WakeWordResult {
    pub detected: bool,
    pub confidence: f32,
    pub word: String,
    pub timestamp: u64,
}

/// Voice assistant configuration
#[derive(Debug, Clone)]
pub struct AssistantConfig {
    pub wake_words: Vec<String>,
    pub language: String,
    pub voice_id: String,
    pub response_style: ResponseStyle,
    pub listening_timeout_ms: u64,
    pub continuous_mode: bool,
    pub offline_mode: bool,
    pub privacy_mode: bool,
}

impl Default for AssistantConfig {
    fn default() -> Self {
        Self {
            wake_words: vec!["hey karana".to_string(), "ok karana".to_string()],
            language: "en-US".to_string(),
            voice_id: "default".to_string(),
            response_style: ResponseStyle::Concise,
            listening_timeout_ms: 5000,
            continuous_mode: false,
            offline_mode: false,
            privacy_mode: false,
        }
    }
}

/// Response style for the assistant
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponseStyle {
    /// Brief, to-the-point responses
    Concise,
    /// More detailed explanations
    Detailed,
    /// Casual, friendly tone
    Conversational,
    /// Professional tone
    Formal,
}

/// Voice assistant integration
pub struct VoiceAssistant {
    config: AssistantConfig,
    state: AssistantState,
    recognizer: SpeechRecognizer,
    synthesizer: SpeechSynthesizer,
    command_handler: CommandHandler,
    context_manager: ContextManager,
    conversation: ConversationManager,
    last_interaction: u64,
    session_active: bool,
}

impl VoiceAssistant {
    pub fn new() -> Self {
        Self::with_config(AssistantConfig::default())
    }

    pub fn with_config(config: AssistantConfig) -> Self {
        Self {
            recognizer: SpeechRecognizer::new(&config.language),
            synthesizer: SpeechSynthesizer::new(&config.voice_id),
            command_handler: CommandHandler::new(),
            context_manager: ContextManager::new(),
            conversation: ConversationManager::new(),
            config,
            state: AssistantState::Idle,
            last_interaction: 0,
            session_active: false,
        }
    }

    pub fn get_state(&self) -> AssistantState {
        self.state
    }

    pub fn start_listening(&mut self) -> bool {
        if self.state == AssistantState::Idle || self.state == AssistantState::Error {
            self.state = AssistantState::Listening;
            self.session_active = true;
            true
        } else {
            false
        }
    }

    pub fn stop_listening(&mut self) {
        if self.state == AssistantState::Listening {
            self.state = AssistantState::Idle;
        }
    }

    pub fn process_audio(&mut self, samples: &[f32], timestamp: u64) -> Option<String> {
        if self.state != AssistantState::Listening {
            return None;
        }

        self.last_interaction = timestamp;

        // Check for wake word if not in continuous mode
        if !self.config.continuous_mode && !self.session_active {
            if let Some(wake_result) = self.check_wake_word(samples) {
                if wake_result.detected {
                    self.session_active = true;
                }
            }
            return None;
        }

        // Process speech recognition
        if let Some(transcript) = self.recognizer.process(samples) {
            self.state = AssistantState::Processing;
            return Some(transcript);
        }

        None
    }

    fn check_wake_word(&self, _samples: &[f32]) -> Option<WakeWordResult> {
        // Simulated wake word detection
        Some(WakeWordResult {
            detected: false,
            confidence: 0.0,
            word: String::new(),
            timestamp: 0,
        })
    }

    pub fn handle_input(&mut self, text: &str, timestamp: u64) -> AssistantResponse {
        self.last_interaction = timestamp;
        self.state = AssistantState::Processing;

        // Add to conversation history
        self.conversation.add_user_message(text, timestamp);

        // Get context for the command - convert to CommandContext
        let current_ctx = self.context_manager.get_current_context();
        let context = CommandContext {
            location: current_ctx.location.as_ref().map(|l| l.name.clone()),
            time_of_day: Some(current_ctx.time_of_day.clone()),
            recent_apps: Vec::new(),
            active_app: current_ctx.active_app.clone(),
            user_preferences: current_ctx.custom.clone(),
        };

        // Process the command
        let command_result = self.command_handler.process(text, &context);

        // Generate response
        let response = self.generate_response(&command_result);

        // Add assistant response to conversation
        self.conversation.add_assistant_message(&response.text, timestamp);

        // Update state
        self.state = AssistantState::Speaking;

        response
    }

    fn generate_response(&self, result: &CommandResult) -> AssistantResponse {
        let text = match &result.response_type {
            ResponseType::Text(t) => t.clone(),
            ResponseType::Action(action) => format!("Executing: {}", action),
            ResponseType::Query(query) => format!("Looking up: {}", query),
            ResponseType::Error(err) => format!("Sorry, {}. Please try again.", err),
            ResponseType::Clarification(q) => q.clone(),
        };

        AssistantResponse {
            text,
            audio_data: None,
            actions: result.actions.clone(),
            suggestions: result.suggestions.clone(),
            confidence: result.confidence,
        }
    }

    pub fn speak(&mut self, text: &str) -> Option<Vec<f32>> {
        self.state = AssistantState::Speaking;
        let audio = self.synthesizer.synthesize(text);
        self.state = AssistantState::Idle;
        audio
    }

    pub fn cancel(&mut self) {
        self.state = AssistantState::Idle;
        self.session_active = false;
    }

    pub fn is_session_active(&self) -> bool {
        self.session_active
    }

    pub fn set_language(&mut self, language: &str) {
        self.config.language = language.to_string();
        self.recognizer = SpeechRecognizer::new(language);
    }

    pub fn set_voice(&mut self, voice_id: &str) {
        self.config.voice_id = voice_id.to_string();
        self.synthesizer = SpeechSynthesizer::new(voice_id);
    }

    pub fn set_response_style(&mut self, style: ResponseStyle) {
        self.config.response_style = style;
    }

    pub fn add_wake_word(&mut self, word: String) {
        self.config.wake_words.push(word);
    }

    pub fn set_continuous_mode(&mut self, enabled: bool) {
        self.config.continuous_mode = enabled;
        if enabled {
            self.session_active = true;
        }
    }

    pub fn set_privacy_mode(&mut self, enabled: bool) {
        self.config.privacy_mode = enabled;
    }

    pub fn get_conversation_history(&self) -> &[ConversationTurn] {
        self.conversation.get_history()
    }

    pub fn clear_conversation(&mut self) {
        self.conversation.clear();
    }

    pub fn get_config(&self) -> &AssistantConfig {
        &self.config
    }

    pub fn register_custom_command(&mut self, pattern: &str, handler_id: &str) {
        self.command_handler.register_pattern(pattern, handler_id);
    }
}

impl Default for VoiceAssistant {
    fn default() -> Self {
        Self::new()
    }
}

/// Response from the voice assistant
#[derive(Debug, Clone)]
pub struct AssistantResponse {
    pub text: String,
    pub audio_data: Option<Vec<f32>>,
    pub actions: Vec<AssistantAction>,
    pub suggestions: Vec<String>,
    pub confidence: f32,
}

/// Action the assistant can take
#[derive(Debug, Clone)]
pub enum AssistantAction {
    OpenApp(String),
    Navigate(String),
    SetTimer(u64),
    SendMessage { to: String, message: String },
    PlayMedia(String),
    AdjustSetting { setting: String, value: String },
    Search(String),
    Custom { action_type: String, params: HashMap<String, String> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assistant_creation() {
        let assistant = VoiceAssistant::new();
        assert_eq!(assistant.get_state(), AssistantState::Idle);
    }

    #[test]
    fn test_start_listening() {
        let mut assistant = VoiceAssistant::new();
        assert!(assistant.start_listening());
        assert_eq!(assistant.get_state(), AssistantState::Listening);
    }

    #[test]
    fn test_stop_listening() {
        let mut assistant = VoiceAssistant::new();
        assistant.start_listening();
        assistant.stop_listening();
        assert_eq!(assistant.get_state(), AssistantState::Idle);
    }

    #[test]
    fn test_handle_input() {
        let mut assistant = VoiceAssistant::new();
        assistant.start_listening();
        let response = assistant.handle_input("what time is it", 1000);
        assert!(!response.text.is_empty());
    }

    #[test]
    fn test_cancel() {
        let mut assistant = VoiceAssistant::new();
        assistant.start_listening();
        assistant.cancel();
        assert_eq!(assistant.get_state(), AssistantState::Idle);
        assert!(!assistant.is_session_active());
    }

    #[test]
    fn test_language_change() {
        let mut assistant = VoiceAssistant::new();
        assistant.set_language("es-ES");
        assert_eq!(assistant.get_config().language, "es-ES");
    }

    #[test]
    fn test_response_style() {
        let mut assistant = VoiceAssistant::new();
        assistant.set_response_style(ResponseStyle::Detailed);
        assert_eq!(assistant.get_config().response_style, ResponseStyle::Detailed);
    }

    #[test]
    fn test_continuous_mode() {
        let mut assistant = VoiceAssistant::new();
        assistant.set_continuous_mode(true);
        assert!(assistant.get_config().continuous_mode);
        assert!(assistant.is_session_active());
    }

    #[test]
    fn test_conversation_history() {
        let mut assistant = VoiceAssistant::new();
        assistant.start_listening();
        assistant.handle_input("hello", 1000);
        
        let history = assistant.get_conversation_history();
        assert!(!history.is_empty());
    }

    #[test]
    fn test_clear_conversation() {
        let mut assistant = VoiceAssistant::new();
        assistant.start_listening();
        assistant.handle_input("hello", 1000);
        assistant.clear_conversation();
        
        assert!(assistant.get_conversation_history().is_empty());
    }
}
