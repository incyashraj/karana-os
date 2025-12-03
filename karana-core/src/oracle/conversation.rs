//! Conversation history tracking

use serde::{Serialize, Deserialize};

/// A single turn in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: String,      // "user", "assistant", or "system"
    pub content: String,
    pub timestamp: u64,
}

impl ConversationTurn {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
    
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
    
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
}

/// Conversation memory with sliding window
#[derive(Debug, Clone, Default)]
pub struct ConversationMemory {
    turns: Vec<ConversationTurn>,
    max_turns: usize,
}

impl ConversationMemory {
    pub fn new(max_turns: usize) -> Self {
        Self {
            turns: Vec::new(),
            max_turns,
        }
    }
    
    pub fn add(&mut self, turn: ConversationTurn) {
        self.turns.push(turn);
        // Keep only last N turns
        if self.turns.len() > self.max_turns {
            self.turns.remove(0);
        }
    }
    
    pub fn get_context_string(&self) -> String {
        self.turns
            .iter()
            .map(|t| format!("{}: {}", t.role, t.content))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    pub fn last_user_message(&self) -> Option<&ConversationTurn> {
        self.turns.iter().rev().find(|t| t.role == "user")
    }
    
    pub fn clear(&mut self) {
        self.turns.clear();
    }
}
