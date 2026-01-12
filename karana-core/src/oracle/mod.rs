//! Kāraṇa OS Oracle - Intelligent AI Assistant
//! 
//! A sophisticated natural language AI that:
//! - Understands context from vision, conversation history, and user state
//! - Executes blockchain transactions seamlessly
//! - Controls AR apps and OS functions
//! - Provides helpful, accurate responses
//!
//! ## Oracle Veil Architecture (v1.4 - Universal Agentic)
//! 
//! The Oracle Veil is the SOLE user interface for Kāraṇa OS. There are no panels,
//! no buttons, no cluttered UI—only the Oracle's whispers. All user interactions
//! flow through the OracleVeil which:
//! 
//! 1. Receives multimodal input via `MultimodalSense` (voice, gaze, touch)
//! 2. Processes intents through the AI engine with semantic understanding
//! 3. Generates ZK proofs for any action affecting state
//! 4. Sends commands to the Monad via typed channels
//! 5. Returns minimal AR output via `MinimalManifest` (whispers, haptics)
//!
//! ## Phase 54: Universal Agentic Oracle
//! - Multi-step reasoning with tool chaining
//! - Long-term memory and feedback loops
//! - 200+ use case coverage (OS, knowledge, apps, creative)

// Phase 54: Universal Oracle integration
pub mod universal;

// Oracle Tool Bridge - connects intents to actual execution
pub mod tool_bridge;

// Legacy modules (for backwards compatibility)
mod intent;
mod context;
mod actions;
mod conversation;

// Oracle Veil v1.1 modules
pub mod command;
pub mod veil;
pub mod sense;
pub mod manifest;
pub mod use_cases;
pub mod tab_commands;
pub mod embeddings; // Phase 42: Vector embeddings
pub mod swarm_knowledge; // Phase 42: P2P knowledge sharing
pub mod knowledge_manager; // Phase 43: User knowledge CRUD
pub mod knowledge_graph; // Phase 44: Knowledge graph visualization
pub mod knowledge_sync; // Phase 45: Cross-device sync
pub mod web_search; // Phase 3: Real-time web search
pub mod knowledge_base; // Phase 3: Offline Wikipedia indexing
pub mod cache; // Phase 3 & 5: Search result caching

// Legacy exports
pub use intent::*;
pub use context::*;
pub use actions::*;
pub use conversation::*;

// Oracle Veil v1.1 exports
pub use command::{
    OracleCommand, CommandResult, CommandData,
    OracleChannels, MonadChannels, COMMAND_BUFFER_SIZE,
    TransactionPayload, ChainQuery, HapticPattern as CommandHapticPattern, AROverlay, WhisperStyle,
};
pub use veil::OracleVeil;
pub use sense::MultimodalSense;
pub use manifest::MinimalManifest;
pub use use_cases::UseCaseDispatcher;
pub use universal::{UniversalOracle, UniversalResponse, ResponseSource, QueryContext};  // Phase 41

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Oracle Intent Types - All actions the Oracle can understand and execute
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OracleIntent {
    // Blockchain Actions
    Transfer { amount: u64, recipient: String, memo: Option<String> },
    CheckBalance,
    TransactionHistory,
    StakeTokens { amount: u64 },
    VoteProposal { proposal_id: String, vote: bool },
    
    // AI & Vision
    AnalyzeVision,
    ExplainObject { context: String },
    TranslateText { text: String, target_lang: String },
    
    // AR App Control
    OpenApp { app_type: String },
    CloseApp { app_id: Option<String> },
    PlayVideo { query: Option<String>, url: Option<String> },
    OpenBrowser { url: Option<String> },
    TakeNote { content: Option<String> },
    SetReminder { message: String, duration: String },
    PlayMusic { query: Option<String> },
    
    // Navigation & AR
    Navigate { destination: String },
    ShowDirections,
    PinLocation,
    
    // OS Control
    SystemStatus,
    AdjustBrightness { level: u8 },
    ToggleNightMode,
    EnablePrivacyMode,
    
    // General Conversation
    Conversation { response: String },
    Clarify { question: String },
    Help,
}

/// Oracle Response - What the Oracle returns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResponse {
    pub intent: OracleIntent,
    pub message: String,
    pub requires_confirmation: bool,
    pub suggested_actions: Vec<String>,
    pub confidence: f32,
}

/// The main Oracle processor
pub struct Oracle {
    conversation_history: Vec<ConversationTurn>,
    current_context: OracleContext,
    user_preferences: HashMap<String, String>,
}

impl Oracle {
    pub fn new() -> Self {
        Self {
            conversation_history: Vec::new(),
            current_context: OracleContext::default(),
            user_preferences: HashMap::new(),
        }
    }
    
    /// Process a natural language input and return an Oracle response
    pub fn process(&mut self, input: &str, context: Option<OracleContext>) -> OracleResponse {
        // Update context if provided
        if let Some(ctx) = context {
            self.current_context = ctx;
        }
        
        // Add to conversation history
        self.conversation_history.push(ConversationTurn {
            role: "user".to_string(),
            content: input.to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        });
        
        // Parse intent with context awareness
        let (intent, confidence) = self.parse_intent(input);
        
        // Generate response based on intent
        let response = self.generate_response(&intent, confidence);
        
        // Add response to history
        self.conversation_history.push(ConversationTurn {
            role: "assistant".to_string(),
            content: response.message.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        });
        
        response
    }
    
    /// Parse the user's intent from natural language
    fn parse_intent(&self, input: &str) -> (OracleIntent, f32) {
        let text = input.to_lowercase();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        // ========================================
        // BLOCKCHAIN INTENTS (High Priority)
        // ========================================
        
        // Transfer/Send money
        if self.matches_any(&text, &["send", "transfer", "pay", "give"]) {
            if let Some((amount, recipient)) = self.extract_transfer_details(&text) {
                let memo = self.extract_memo(&text);
                return (OracleIntent::Transfer { amount, recipient, memo }, 0.95);
            }
        }
        
        // Check balance
        if self.matches_any(&text, &["balance", "how much", "my money", "funds", "kara"]) 
            && self.matches_any(&text, &["check", "show", "what", "how", "my", "have"]) {
            return (OracleIntent::CheckBalance, 0.92);
        }
        
        // Transaction history
        if self.matches_any(&text, &["transactions", "history", "payments", "activity"]) {
            return (OracleIntent::TransactionHistory, 0.90);
        }
        
        // Staking
        if self.matches_any(&text, &["stake", "staking", "delegate"]) {
            if let Some(amount) = self.extract_amount(&text) {
                return (OracleIntent::StakeTokens { amount }, 0.88);
            }
        }
        
        // ========================================
        // AR APP CONTROL
        // ========================================
        
        // Open apps
        if self.matches_any(&text, &["open", "launch", "start", "show"]) {
            if self.matches_any(&text, &["browser", "web", "internet", "chrome", "firefox"]) {
                let url = self.extract_url(&text);
                return (OracleIntent::OpenBrowser { url }, 0.93);
            }
            if self.matches_any(&text, &["video", "youtube", "movie", "watch", "player"]) {
                let query = self.extract_search_query(&text, &["video", "youtube", "movie", "watch", "player"]);
                return (OracleIntent::PlayVideo { query, url: None }, 0.91);
            }
            if self.matches_any(&text, &["notes", "notepad", "memo", "write"]) {
                return (OracleIntent::TakeNote { content: None }, 0.90);
            }
            if self.matches_any(&text, &["music", "spotify", "song", "audio"]) {
                let query = self.extract_search_query(&text, &["music", "spotify", "song", "play"]);
                return (OracleIntent::PlayMusic { query }, 0.89);
            }
            if self.matches_any(&text, &["terminal", "console", "shell", "command"]) {
                return (OracleIntent::OpenApp { app_type: "terminal".to_string() }, 0.92);
            }
            if self.matches_any(&text, &["calendar", "schedule", "events"]) {
                return (OracleIntent::OpenApp { app_type: "calendar".to_string() }, 0.90);
            }
            if self.matches_any(&text, &["mail", "email", "inbox"]) {
                return (OracleIntent::OpenApp { app_type: "mail".to_string() }, 0.90);
            }
        }
        
        // Watch/Play video
        if self.matches_any(&text, &["watch", "play"]) && self.matches_any(&text, &["video", "movie", "film", "show"]) {
            let query = self.extract_search_query(&text, &["watch", "play", "video", "movie", "film"]);
            return (OracleIntent::PlayVideo { query, url: None }, 0.90);
        }
        
        // Close apps
        if self.matches_any(&text, &["close", "exit", "quit", "stop"]) && 
           self.matches_any(&text, &["app", "window", "browser", "video", "player", "all"]) {
            return (OracleIntent::CloseApp { app_id: None }, 0.88);
        }
        
        // ========================================
        // REMINDERS & NOTES
        // ========================================
        
        if self.matches_any(&text, &["remind", "reminder", "alert", "timer", "alarm", "set a timer", "set timer"]) {
            let (message, duration) = self.extract_reminder(&text);
            return (OracleIntent::SetReminder { message, duration }, 0.87);
        }
        
        if self.matches_any(&text, &["note", "write down", "remember", "save"]) {
            let content = self.extract_note_content(&text);
            return (OracleIntent::TakeNote { content }, 0.85);
        }
        
        // ========================================
        // VISION & ANALYSIS
        // ========================================
        
        if self.matches_any(&text, &["what is", "what's", "analyze", "scan", "identify", "recognize", "look at"]) {
            // Check if we have vision context
            if self.current_context.vision_object.is_some() {
                return (OracleIntent::ExplainObject { 
                    context: self.current_context.vision_object.clone().unwrap() 
                }, 0.92);
            }
            return (OracleIntent::AnalyzeVision, 0.90);
        }
        
        // ========================================
        // NAVIGATION
        // ========================================
        
        if self.matches_any(&text, &["navigate", "directions", "go to", "take me", "how to get"]) {
            let destination = self.extract_destination(&text);
            return (OracleIntent::Navigate { destination }, 0.88);
        }
        
        // ========================================
        // OS CONTROLS
        // ========================================
        
        if self.matches_any(&text, &["system", "status", "diagnostics"]) {
            return (OracleIntent::SystemStatus, 0.85);
        }
        
        if self.matches_any(&text, &["brightness"]) {
            let level = self.extract_brightness_level(&text);
            return (OracleIntent::AdjustBrightness { level }, 0.82);
        }
        
        if self.matches_any(&text, &["night mode", "dark mode"]) {
            return (OracleIntent::ToggleNightMode, 0.85);
        }
        
        if self.matches_any(&text, &["privacy", "private mode", "incognito"]) {
            return (OracleIntent::EnablePrivacyMode, 0.84);
        }
        
        // ========================================
        // HELP
        // ========================================
        
        if self.matches_any(&text, &["help", "what can you do", "commands", "abilities"]) {
            return (OracleIntent::Help, 0.95);
        }
        
        // ========================================
        // GENERAL CONVERSATION (Fallback)
        // ========================================
        
        // If nothing matches, generate a conversational response
        let response = self.generate_conversational_response(input);
        (OracleIntent::Conversation { response }, 0.60)
    }
    
    /// Generate a response based on the parsed intent
    fn generate_response(&self, intent: &OracleIntent, confidence: f32) -> OracleResponse {
        match intent {
            OracleIntent::Transfer { amount, recipient, memo } => {
                let msg = if let Some(m) = memo {
                    format!("Ready to send {} KARA to {} with memo: '{}'", amount, recipient, m)
                } else {
                    format!("Ready to send {} KARA to {}", amount, recipient)
                };
                OracleResponse {
                    intent: intent.clone(),
                    message: msg,
                    requires_confirmation: true,
                    suggested_actions: vec!["Confirm".to_string(), "Cancel".to_string(), "Change amount".to_string()],
                    confidence,
                }
            }
            
            OracleIntent::CheckBalance => OracleResponse {
                intent: intent.clone(),
                message: "Checking your KARA balance...".to_string(),
                requires_confirmation: false,
                suggested_actions: vec!["Send funds".to_string(), "View history".to_string()],
                confidence,
            },
            
            OracleIntent::TransactionHistory => OracleResponse {
                intent: intent.clone(),
                message: "Loading your transaction history...".to_string(),
                requires_confirmation: false,
                suggested_actions: vec!["Filter by date".to_string(), "Export".to_string()],
                confidence,
            },
            
            OracleIntent::OpenBrowser { url } => {
                let msg = if let Some(u) = url {
                    format!("Opening browser to {}", u)
                } else {
                    "Opening browser".to_string()
                };
                OracleResponse {
                    intent: intent.clone(),
                    message: msg,
                    requires_confirmation: false,
                    suggested_actions: vec!["Search".to_string(), "Bookmarks".to_string()],
                    confidence,
                }
            }
            
            OracleIntent::PlayVideo { query, url } => {
                let msg = if let Some(q) = query {
                    format!("Searching for video: '{}'", q)
                } else if let Some(u) = url {
                    format!("Playing video from {}", u)
                } else {
                    "Opening video player".to_string()
                };
                OracleResponse {
                    intent: intent.clone(),
                    message: msg,
                    requires_confirmation: false,
                    suggested_actions: vec!["Fullscreen".to_string(), "Add to queue".to_string()],
                    confidence,
                }
            }
            
            OracleIntent::TakeNote { content } => {
                let msg = if let Some(c) = content {
                    format!("Creating note: '{}'", c)
                } else {
                    "Opening notes app".to_string()
                };
                OracleResponse {
                    intent: intent.clone(),
                    message: msg,
                    requires_confirmation: false,
                    suggested_actions: vec!["Voice input".to_string(), "Add reminder".to_string()],
                    confidence,
                }
            }
            
            OracleIntent::SetReminder { message, duration } => OracleResponse {
                intent: intent.clone(),
                message: format!("Setting reminder: '{}' in {}", message, duration),
                requires_confirmation: true,
                suggested_actions: vec!["Confirm".to_string(), "Change time".to_string()],
                confidence,
            },
            
            OracleIntent::Navigate { destination } => OracleResponse {
                intent: intent.clone(),
                message: format!("Getting directions to {}", destination),
                requires_confirmation: false,
                suggested_actions: vec!["Start navigation".to_string(), "Alternative routes".to_string()],
                confidence,
            },
            
            OracleIntent::AnalyzeVision => OracleResponse {
                intent: intent.clone(),
                message: "Activating vision analysis... Point at what you want to identify.".to_string(),
                requires_confirmation: false,
                suggested_actions: vec!["Capture".to_string(), "Cancel".to_string()],
                confidence,
            },
            
            OracleIntent::ExplainObject { context } => OracleResponse {
                intent: intent.clone(),
                message: format!("Analyzing: {}...", context),
                requires_confirmation: false,
                suggested_actions: vec!["More details".to_string(), "Find similar".to_string()],
                confidence,
            },
            
            OracleIntent::Help => OracleResponse {
                intent: intent.clone(),
                message: self.get_help_text(),
                requires_confirmation: false,
                suggested_actions: vec![
                    "Send payment".to_string(), 
                    "Open browser".to_string(),
                    "Watch video".to_string(),
                    "Take note".to_string(),
                ],
                confidence,
            },
            
            OracleIntent::SystemStatus => OracleResponse {
                intent: intent.clone(),
                message: "Fetching system status...".to_string(),
                requires_confirmation: false,
                suggested_actions: vec!["Diagnostics".to_string(), "Settings".to_string()],
                confidence,
            },
            
            OracleIntent::Conversation { response } => OracleResponse {
                intent: intent.clone(),
                message: response.clone(),
                requires_confirmation: false,
                suggested_actions: vec![],
                confidence,
            },
            
            _ => OracleResponse {
                intent: intent.clone(),
                message: "Processing your request...".to_string(),
                requires_confirmation: false,
                suggested_actions: vec![],
                confidence,
            },
        }
    }
    
    // ========================================
    // HELPER METHODS
    // ========================================
    
    fn matches_any(&self, text: &str, keywords: &[&str]) -> bool {
        keywords.iter().any(|k| text.contains(k))
    }
    
    fn extract_transfer_details(&self, text: &str) -> Option<(u64, String)> {
        // Extract amount
        let amount = self.extract_amount(text)?;
        
        // Extract recipient - look for "to <name>"
        let recipient = if let Some(idx) = text.find(" to ") {
            let after = &text[idx + 4..];
            after.split_whitespace()
                .next()
                .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            return None;
        };
        
        Some((amount, recipient))
    }
    
    fn extract_amount(&self, text: &str) -> Option<u64> {
        text.split_whitespace()
            .find_map(|word| word.parse::<u64>().ok())
    }
    
    fn extract_memo(&self, text: &str) -> Option<String> {
        // Look for "for <memo>" or "memo: <text>"
        if let Some(idx) = text.find(" for ") {
            let after = &text[idx + 5..];
            let memo: String = after.split_whitespace()
                .take_while(|w| !["to", "from", "send"].contains(w))
                .collect::<Vec<_>>()
                .join(" ");
            if !memo.is_empty() {
                return Some(memo);
            }
        }
        None
    }
    
    fn extract_url(&self, text: &str) -> Option<String> {
        // Look for URLs in text
        for word in text.split_whitespace() {
            if word.contains("http") || word.contains("www.") || word.contains(".com") || word.contains(".org") {
                return Some(word.to_string());
            }
        }
        None
    }
    
    fn extract_search_query(&self, text: &str, exclude_words: &[&str]) -> Option<String> {
        let words: Vec<&str> = text.split_whitespace()
            .filter(|w| !exclude_words.contains(&w.to_lowercase().as_str()))
            .filter(|w| !["open", "launch", "start", "show", "play", "watch", "the", "a", "an"].contains(w))
            .collect();
        
        if words.is_empty() {
            None
        } else {
            Some(words.join(" "))
        }
    }
    
    fn extract_reminder(&self, text: &str) -> (String, String) {
        // Default values
        let mut message = "Timer".to_string();
        let mut duration = "5 minutes".to_string();
        
        // Look for time patterns - "X minutes", "X hours", "X seconds"
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            // Check for number
            if let Ok(num) = word.parse::<u32>() {
                // Look at next word for unit
                if let Some(next) = words.get(i + 1) {
                    let next_lower = next.to_lowercase();
                    if next_lower.starts_with("min") {
                        duration = format!("{} minutes", num);
                    } else if next_lower.starts_with("hour") || next_lower == "hr" {
                        duration = format!("{} hours", num);
                    } else if next_lower.starts_with("sec") {
                        duration = format!("{} seconds", num);
                    }
                }
            }
            // Also check for patterns like "5min" without space
            let word_lower = word.to_lowercase();
            if word_lower.contains("min") || word_lower.contains("hour") || word_lower.contains("sec") {
                // Extract number from start
                let num_str: String = word_lower.chars().take_while(|c| c.is_digit(10)).collect();
                if let Ok(num) = num_str.parse::<u32>() {
                    if word_lower.contains("min") {
                        duration = format!("{} minutes", num);
                    } else if word_lower.contains("hour") {
                        duration = format!("{} hours", num);
                    } else if word_lower.contains("sec") {
                        duration = format!("{} seconds", num);
                    }
                }
            }
        }
        
        // Look for "for <time>" pattern (e.g., "timer for 5 minutes")
        if let Some(idx) = text.find(" for ") {
            let after = &text[idx + 5..];
            // Check if there's a number after "for"
            for (i, word) in after.split_whitespace().enumerate() {
                if let Ok(num) = word.parse::<u32>() {
                    let words_after: Vec<&str> = after.split_whitespace().collect();
                    if let Some(next) = words_after.get(i + 1) {
                        let next_lower = next.to_lowercase();
                        if next_lower.starts_with("min") {
                            duration = format!("{} minutes", num);
                        } else if next_lower.starts_with("hour") || next_lower == "hr" {
                            duration = format!("{} hours", num);
                        } else if next_lower.starts_with("sec") {
                            duration = format!("{} seconds", num);
                        }
                    }
                    break;
                }
            }
        }
        
        // Look for "to <message>" (for reminders)
        if let Some(idx) = text.find(" to ") {
            // Make sure it's not "timer for 5 minutes to..."
            if !text[..idx].contains("for") && !text[..idx].contains("timer") {
                let after = &text[idx + 4..];
                let msg: String = after.split_whitespace()
                    .take_while(|w| {
                        let w = w.to_lowercase();
                        !w.contains("min") && !w.contains("hour") && !w.contains("in") && !w.parse::<u32>().is_ok()
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                if !msg.is_empty() {
                    message = msg;
                }
            }
        }
        
        // Determine if it's a timer or reminder based on input
        if text.contains("timer") || text.contains("alarm") {
            message = format!("Timer ({})", duration);
        } else if text.contains("remind") {
            // Keep extracted message or use default
            if message == "Timer" {
                message = "Reminder".to_string();
            }
        }
        
        (message, duration)
    }
    
    fn extract_note_content(&self, text: &str) -> Option<String> {
        // Look for text after "note:", "write:", "save:"
        for prefix in &["note:", "write:", "save:", "note ", "write down ", "remember "] {
            if let Some(idx) = text.find(prefix) {
                let content = text[idx + prefix.len()..].trim();
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
        None
    }
    
    fn extract_destination(&self, text: &str) -> String {
        // Look for location after "to"
        if let Some(idx) = text.find(" to ") {
            let after = &text[idx + 4..];
            return after.trim().to_string();
        }
        if let Some(idx) = text.find("get to ") {
            let after = &text[idx + 7..];
            return after.trim().to_string();
        }
        "destination".to_string()
    }
    
    fn extract_brightness_level(&self, text: &str) -> u8 {
        if text.contains("max") || text.contains("full") || text.contains("100") {
            return 100;
        }
        if text.contains("min") || text.contains("low") || text.contains("dim") {
            return 20;
        }
        if text.contains("half") || text.contains("50") {
            return 50;
        }
        // Try to extract a number
        for word in text.split_whitespace() {
            if let Ok(num) = word.parse::<u8>() {
                return num.min(100);
            }
        }
        70 // Default
    }
    
    fn generate_conversational_response(&self, input: &str) -> String {
        // Simple conversational responses based on context
        let input_lower = input.to_lowercase();
        
        if input_lower.contains("hello") || input_lower.contains("hi") {
            return "Hello! I'm your Kāraṇa OS assistant. How can I help you today?".to_string();
        }
        if input_lower.contains("thank") {
            return "You're welcome! Let me know if you need anything else.".to_string();
        }
        if input_lower.contains("how are you") {
            return "I'm running smoothly! Ready to help you with payments, browsing, or anything else.".to_string();
        }
        if input_lower.contains("who are you") || input_lower.contains("what are you") {
            return "I'm the Kāraṇa OS Oracle - your AI assistant. I can help with blockchain payments, open AR apps, analyze what you're looking at, and much more. Just ask!".to_string();
        }
        
        // Check if there's recent context we can reference
        if let Some(vision_obj) = &self.current_context.vision_object {
            if input_lower.contains("this") || input_lower.contains("it") || input_lower.contains("that") {
                return format!("You're looking at {}. Would you like me to tell you more about it, find related items, or take an action?", vision_obj);
            }
        }
        
        // Default response
        format!("I understood: '{}'. You can ask me to send payments, open apps, analyze objects, set reminders, and more. Try 'help' for a full list!", input)
    }
    
    fn get_help_text(&self) -> String {
        r#"🔮 Kāraṇa OS Oracle - Here's what I can do:

💰 **Payments & Blockchain**
• "Send 50 KARA to alice"
• "Check my balance"  
• "Show transaction history"

🖥️ **AR Apps**
• "Open browser" / "Open youtube.com"
• "Play a video about space"
• "Take a note: meeting at 3pm"
• "Open terminal"

⏰ **Reminders**
• "Remind me to call mom in 30 minutes"

👁️ **Vision**
• "What is this?" (when looking at something)
• "Analyze what I'm seeing"

🧭 **Navigation**
• "Navigate to Times Square"

⚙️ **System**
• "System status"
• "Enable privacy mode"

Just speak naturally - I'll understand!"#.to_string()
    }
    
    /// Get conversation history for context
    pub fn get_history(&self) -> &[ConversationTurn] {
        &self.conversation_history
    }
    
    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.conversation_history.clear();
    }
    
    /// Update context
    pub fn set_context(&mut self, context: OracleContext) {
        self.current_context = context;
    }
}

impl Default for Oracle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transfer_intent() {
        let mut oracle = Oracle::new();
        let response = oracle.process("send 100 to alice", None);
        
        match response.intent {
            OracleIntent::Transfer { amount, recipient, .. } => {
                assert_eq!(amount, 100);
                assert_eq!(recipient, "alice");
            }
            _ => panic!("Expected Transfer intent"),
        }
    }
    
    #[test]
    fn test_balance_intent() {
        let mut oracle = Oracle::new();
        let response = oracle.process("check my balance", None);
        assert!(matches!(response.intent, OracleIntent::CheckBalance));
    }
    
    #[test]
    fn test_video_intent() {
        let mut oracle = Oracle::new();
        let response = oracle.process("play a video about cats", None);
        
        match response.intent {
            OracleIntent::PlayVideo { query, .. } => {
                assert!(query.is_some());
                assert!(query.unwrap().contains("cats"));
            }
            _ => panic!("Expected PlayVideo intent"),
        }
    }
}

// ============================================================================
// Legacy Compatibility Layer (KaranaOracle)
// ============================================================================

use std::sync::Arc;
use anyhow::Result;

/// Legacy KaranaOracle for backwards compatibility with existing code
/// This wraps the new Oracle engine while maintaining the old API
pub struct KaranaOracle {
    oracle: Oracle,
    user_did: Option<String>,
    wallet_balance: u64,
}

impl KaranaOracle {
    /// Create a new KaranaOracle (legacy constructor)
    /// All dependencies are ignored since we use the new Oracle engine internally
    pub fn new<T, U, V, W, X>(
        _ai: T,
        _chain: U,
        _storage: V,
        _ledger: W,
        _gov: X,
    ) -> Self {
        Self {
            oracle: Oracle::new(),
            user_did: None,
            wallet_balance: 0,
        }
    }
    
    /// Create with wallet
    pub fn with_wallet<T, U, V, W, X>(
        ai: T,
        chain: U,
        storage: V,
        ledger: W,
        gov: X,
        _wallet: crate::wallet::KaranaWallet,
    ) -> Self {
        Self::new(ai, chain, storage, ledger, gov)
    }
    
    /// Set wallet (for legacy compatibility)
    pub fn set_wallet(&mut self, _wallet: crate::wallet::KaranaWallet) {
        // The new oracle doesn't hold wallet directly
    }
    
    /// Process a query - legacy interface
    pub fn process_query(&self, query: &str, _user_did: &str) -> Result<String> {
        // Create a temporary mutable oracle for processing
        let mut temp_oracle = Oracle::new();
        
        // Set context with available info
        let mut ctx = OracleContext::default();
        ctx.wallet_balance = self.wallet_balance;
        
        let response = temp_oracle.process(query, Some(ctx));
        
        Ok(response.message)
    }
    
    /// Update wallet balance context
    pub fn set_balance(&mut self, balance: u64) {
        self.wallet_balance = balance;
    }
}
