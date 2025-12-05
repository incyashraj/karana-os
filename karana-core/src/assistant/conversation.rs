// Conversation Management for Kāraṇa OS
// Maintains conversation history and context

use std::collections::VecDeque;

/// Conversation turn speaker
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Speaker {
    User,
    Assistant,
    System,
}

/// A single turn in the conversation
#[derive(Debug, Clone)]
pub struct ConversationTurn {
    pub speaker: Speaker,
    pub message: String,
    pub timestamp: u64,
    pub intent: Option<String>,
    pub entities: Vec<(String, String)>,
    pub confidence: Option<f32>,
}

impl ConversationTurn {
    pub fn user(message: &str, timestamp: u64) -> Self {
        Self {
            speaker: Speaker::User,
            message: message.to_string(),
            timestamp,
            intent: None,
            entities: Vec::new(),
            confidence: None,
        }
    }

    pub fn assistant(message: &str, timestamp: u64) -> Self {
        Self {
            speaker: Speaker::Assistant,
            message: message.to_string(),
            timestamp,
            intent: None,
            entities: Vec::new(),
            confidence: None,
        }
    }

    pub fn system(message: &str, timestamp: u64) -> Self {
        Self {
            speaker: Speaker::System,
            message: message.to_string(),
            timestamp,
            intent: None,
            entities: Vec::new(),
            confidence: None,
        }
    }
}

/// Conversation state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConversationState {
    Idle,
    Active,
    WaitingForClarification,
    WaitingForConfirmation,
    Completed,
}

/// Conversation topic
#[derive(Debug, Clone)]
pub struct ConversationTopic {
    pub name: String,
    pub first_mentioned: u64,
    pub last_mentioned: u64,
    pub mention_count: u32,
}

/// Conversation manager
pub struct ConversationManager {
    history: VecDeque<ConversationTurn>,
    max_history: usize,
    state: ConversationState,
    topics: Vec<ConversationTopic>,
    pending_action: Option<String>,
    session_start: u64,
}

impl ConversationManager {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(100),
            max_history: 100,
            state: ConversationState::Idle,
            topics: Vec::new(),
            pending_action: None,
            session_start: 0,
        }
    }

    pub fn add_user_message(&mut self, message: &str, timestamp: u64) {
        if self.state == ConversationState::Idle {
            self.state = ConversationState::Active;
            self.session_start = timestamp;
        }

        let turn = ConversationTurn::user(message, timestamp);
        self.add_turn(turn);
        
        // Extract topics
        self.extract_topics(message, timestamp);
    }

    pub fn add_assistant_message(&mut self, message: &str, timestamp: u64) {
        let turn = ConversationTurn::assistant(message, timestamp);
        self.add_turn(turn);
    }

    pub fn add_system_message(&mut self, message: &str, timestamp: u64) {
        let turn = ConversationTurn::system(message, timestamp);
        self.add_turn(turn);
    }

    fn add_turn(&mut self, turn: ConversationTurn) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(turn);
    }

    fn extract_topics(&mut self, message: &str, timestamp: u64) {
        // Simple topic extraction based on keywords
        let keywords = ["weather", "time", "navigation", "call", "message", 
                       "music", "timer", "alarm", "reminder", "news"];
        
        let lower = message.to_lowercase();
        for keyword in keywords {
            if lower.contains(keyword) {
                if let Some(topic) = self.topics.iter_mut().find(|t| t.name == keyword) {
                    topic.last_mentioned = timestamp;
                    topic.mention_count += 1;
                } else {
                    self.topics.push(ConversationTopic {
                        name: keyword.to_string(),
                        first_mentioned: timestamp,
                        last_mentioned: timestamp,
                        mention_count: 1,
                    });
                }
            }
        }
    }

    pub fn get_history(&self) -> &[ConversationTurn] {
        // Convert VecDeque to slice view
        let (front, back) = self.history.as_slices();
        if back.is_empty() {
            front
        } else {
            // This is a simplification - in practice would need to handle both slices
            front
        }
    }

    pub fn get_recent(&self, count: usize) -> Vec<&ConversationTurn> {
        self.history.iter()
            .rev()
            .take(count)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    pub fn get_last_user_message(&self) -> Option<&ConversationTurn> {
        self.history.iter()
            .rev()
            .find(|t| t.speaker == Speaker::User)
    }

    pub fn get_last_assistant_message(&self) -> Option<&ConversationTurn> {
        self.history.iter()
            .rev()
            .find(|t| t.speaker == Speaker::Assistant)
    }

    pub fn get_state(&self) -> ConversationState {
        self.state
    }

    pub fn set_state(&mut self, state: ConversationState) {
        self.state = state;
    }

    pub fn request_clarification(&mut self, question: &str, timestamp: u64) {
        self.add_assistant_message(question, timestamp);
        self.state = ConversationState::WaitingForClarification;
    }

    pub fn request_confirmation(&mut self, action: &str, timestamp: u64) {
        let message = format!("Should I {}?", action);
        self.add_assistant_message(&message, timestamp);
        self.pending_action = Some(action.to_string());
        self.state = ConversationState::WaitingForConfirmation;
    }

    pub fn handle_confirmation(&mut self, confirmed: bool) -> Option<String> {
        if self.state == ConversationState::WaitingForConfirmation {
            self.state = ConversationState::Active;
            if confirmed {
                self.pending_action.take()
            } else {
                self.pending_action = None;
                None
            }
        } else {
            None
        }
    }

    pub fn get_topics(&self) -> &[ConversationTopic] {
        &self.topics
    }

    pub fn get_current_topic(&self) -> Option<&ConversationTopic> {
        self.topics.iter().max_by_key(|t| t.last_mentioned)
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.topics.clear();
        self.state = ConversationState::Idle;
        self.pending_action = None;
    }

    pub fn end_session(&mut self) {
        self.state = ConversationState::Completed;
    }

    pub fn get_session_duration(&self, current_time: u64) -> u64 {
        if self.session_start == 0 {
            0
        } else {
            current_time - self.session_start
        }
    }

    pub fn to_prompt(&self, max_turns: usize) -> String {
        let recent = self.get_recent(max_turns);
        recent.iter()
            .map(|turn| {
                let role = match turn.speaker {
                    Speaker::User => "User",
                    Speaker::Assistant => "Assistant",
                    Speaker::System => "System",
                };
                format!("{}: {}", role, turn.message)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn get_turn_count(&self) -> usize {
        self.history.len()
    }

    pub fn get_user_turn_count(&self) -> usize {
        self.history.iter()
            .filter(|t| t.speaker == Speaker::User)
            .count()
    }
}

impl Default for ConversationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_creation() {
        let conv = ConversationManager::new();
        assert_eq!(conv.get_state(), ConversationState::Idle);
        assert_eq!(conv.get_turn_count(), 0);
    }

    #[test]
    fn test_add_messages() {
        let mut conv = ConversationManager::new();
        conv.add_user_message("Hello", 1000);
        conv.add_assistant_message("Hi there!", 1001);
        
        assert_eq!(conv.get_turn_count(), 2);
        assert_eq!(conv.get_state(), ConversationState::Active);
    }

    #[test]
    fn test_get_last_messages() {
        let mut conv = ConversationManager::new();
        conv.add_user_message("First user", 1000);
        conv.add_assistant_message("First assistant", 1001);
        conv.add_user_message("Second user", 1002);
        
        let last_user = conv.get_last_user_message().unwrap();
        assert_eq!(last_user.message, "Second user");
        
        let last_assistant = conv.get_last_assistant_message().unwrap();
        assert_eq!(last_assistant.message, "First assistant");
    }

    #[test]
    fn test_get_recent() {
        let mut conv = ConversationManager::new();
        for i in 0..10 {
            conv.add_user_message(&format!("Message {}", i), i as u64 * 1000);
        }
        
        let recent = conv.get_recent(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_topic_extraction() {
        let mut conv = ConversationManager::new();
        conv.add_user_message("What's the weather like?", 1000);
        conv.add_user_message("Tell me about the weather tomorrow", 2000);
        
        let topics = conv.get_topics();
        let weather_topic = topics.iter().find(|t| t.name == "weather");
        assert!(weather_topic.is_some());
        assert_eq!(weather_topic.unwrap().mention_count, 2);
    }

    #[test]
    fn test_clarification() {
        let mut conv = ConversationManager::new();
        conv.add_user_message("Call someone", 1000);
        conv.request_clarification("Who would you like to call?", 1001);
        
        assert_eq!(conv.get_state(), ConversationState::WaitingForClarification);
    }

    #[test]
    fn test_confirmation() {
        let mut conv = ConversationManager::new();
        conv.add_user_message("Delete all files", 1000);
        conv.request_confirmation("delete all files", 1001);
        
        assert_eq!(conv.get_state(), ConversationState::WaitingForConfirmation);
        
        let action = conv.handle_confirmation(true);
        assert_eq!(action, Some("delete all files".to_string()));
        assert_eq!(conv.get_state(), ConversationState::Active);
    }

    #[test]
    fn test_confirmation_rejected() {
        let mut conv = ConversationManager::new();
        conv.request_confirmation("delete all files", 1001);
        
        let action = conv.handle_confirmation(false);
        assert!(action.is_none());
    }

    #[test]
    fn test_clear() {
        let mut conv = ConversationManager::new();
        conv.add_user_message("Hello", 1000);
        conv.clear();
        
        assert_eq!(conv.get_turn_count(), 0);
        assert_eq!(conv.get_state(), ConversationState::Idle);
    }

    #[test]
    fn test_to_prompt() {
        let mut conv = ConversationManager::new();
        conv.add_user_message("Hello", 1000);
        conv.add_assistant_message("Hi!", 1001);
        
        let prompt = conv.to_prompt(10);
        assert!(prompt.contains("User: Hello"));
        assert!(prompt.contains("Assistant: Hi!"));
    }

    #[test]
    fn test_session_duration() {
        let mut conv = ConversationManager::new();
        conv.add_user_message("Start", 1000);
        
        let duration = conv.get_session_duration(5000);
        assert_eq!(duration, 4000);
    }

    #[test]
    fn test_user_turn_count() {
        let mut conv = ConversationManager::new();
        conv.add_user_message("1", 1000);
        conv.add_assistant_message("reply", 1001);
        conv.add_user_message("2", 1002);
        conv.add_system_message("system", 1003);
        
        assert_eq!(conv.get_user_turn_count(), 2);
    }

    #[test]
    fn test_current_topic() {
        let mut conv = ConversationManager::new();
        conv.add_user_message("What's the time?", 1000);
        conv.add_user_message("Set a timer", 2000);
        
        let current = conv.get_current_topic().unwrap();
        assert_eq!(current.name, "timer");
    }
}
