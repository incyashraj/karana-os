//! Messaging for Kāraṇa OS AR Glasses
//!
//! Text and multimedia messaging with AR display.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Message type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Plain text
    Text,
    /// Image
    Image,
    /// Voice note
    Voice,
    /// Video
    Video,
    /// File attachment
    File,
    /// Location
    Location,
    /// Contact card
    Contact,
    /// Sticker/emoji
    Sticker,
}

/// Message status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageStatus {
    /// Sending
    Sending,
    /// Sent to server
    Sent,
    /// Delivered to recipient
    Delivered,
    /// Read by recipient
    Read,
    /// Failed to send
    Failed,
}

/// Message direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDirection {
    /// Incoming message
    Incoming,
    /// Outgoing message
    Outgoing,
}

/// Message
#[derive(Debug, Clone)]
pub struct Message {
    /// Message ID
    pub id: String,
    /// Sender ID
    pub sender_id: String,
    /// Recipient ID (for direct messages)
    pub recipient_id: String,
    /// Message type
    pub message_type: MessageType,
    /// Text content
    pub text: Option<String>,
    /// Media URL (for images, voice, video)
    pub media_url: Option<String>,
    /// Timestamp
    pub timestamp: Instant,
    /// Status
    pub status: MessageStatus,
    /// Direction
    pub direction: MessageDirection,
    /// Is read
    pub is_read: bool,
    /// Reply to message ID
    pub reply_to: Option<String>,
    /// Reactions
    pub reactions: Vec<MessageReaction>,
}

/// Message reaction
#[derive(Debug, Clone)]
pub struct MessageReaction {
    /// User ID
    pub user_id: String,
    /// Reaction emoji
    pub emoji: String,
}

impl Message {
    /// Create new text message
    pub fn text(
        id: String,
        sender_id: String,
        recipient_id: String,
        text: String,
        direction: MessageDirection,
    ) -> Self {
        Self {
            id,
            sender_id,
            recipient_id,
            message_type: MessageType::Text,
            text: Some(text),
            media_url: None,
            timestamp: Instant::now(),
            status: if direction == MessageDirection::Outgoing {
                MessageStatus::Sending
            } else {
                MessageStatus::Delivered
            },
            direction,
            is_read: direction == MessageDirection::Outgoing,
            reply_to: None,
            reactions: Vec::new(),
        }
    }
    
    /// Get preview text
    pub fn preview(&self) -> String {
        match self.message_type {
            MessageType::Text => self.text.clone().unwrap_or_default(),
            MessageType::Image => "📷 Image".to_string(),
            MessageType::Voice => "🎤 Voice message".to_string(),
            MessageType::Video => "🎬 Video".to_string(),
            MessageType::File => "📎 Attachment".to_string(),
            MessageType::Location => "📍 Location".to_string(),
            MessageType::Contact => "👤 Contact".to_string(),
            MessageType::Sticker => "😀 Sticker".to_string(),
        }
    }
    
    /// Mark as read
    pub fn mark_read(&mut self) {
        self.is_read = true;
    }
    
    /// Add reaction
    pub fn add_reaction(&mut self, user_id: String, emoji: String) {
        self.reactions.retain(|r| r.user_id != user_id);
        self.reactions.push(MessageReaction { user_id, emoji });
    }
    
    /// Remove reaction
    pub fn remove_reaction(&mut self, user_id: &str) {
        self.reactions.retain(|r| r.user_id != user_id);
    }
}

/// Conversation (chat thread)
#[derive(Debug)]
pub struct Conversation {
    /// Conversation ID
    pub id: String,
    /// Participant IDs
    pub participants: Vec<String>,
    /// Messages
    messages: VecDeque<Message>,
    /// Max messages to keep in memory
    max_messages: usize,
    /// Unread count
    pub unread_count: u32,
    /// Last message timestamp
    pub last_activity: Option<Instant>,
    /// Is muted
    pub muted: bool,
    /// Is pinned
    pub pinned: bool,
    /// Is group chat
    pub is_group: bool,
    /// Group name (if group)
    pub group_name: Option<String>,
    /// Draft message
    pub draft: Option<String>,
    /// Is typing
    pub is_typing: bool,
}

impl Conversation {
    /// Create new conversation
    pub fn new(id: String, participants: Vec<String>) -> Self {
        let is_group = participants.len() > 1;
        Self {
            id,
            participants,
            messages: VecDeque::new(),
            max_messages: 100,
            unread_count: 0,
            last_activity: None,
            muted: false,
            pinned: false,
            is_group,
            group_name: None,
            draft: None,
            is_typing: false,
        }
    }
    
    /// Create group conversation
    pub fn group(id: String, name: String, participants: Vec<String>) -> Self {
        let mut conv = Self::new(id, participants);
        conv.is_group = true;
        conv.group_name = Some(name);
        conv
    }
    
    /// Add message
    pub fn add_message(&mut self, message: Message) {
        if message.direction == MessageDirection::Incoming && !message.is_read {
            self.unread_count += 1;
        }
        
        self.last_activity = Some(message.timestamp);
        
        if self.messages.len() >= self.max_messages {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }
    
    /// Get messages
    pub fn messages(&self) -> &VecDeque<Message> {
        &self.messages
    }
    
    /// Get last message
    pub fn last_message(&self) -> Option<&Message> {
        self.messages.back()
    }
    
    /// Get message by ID
    pub fn get_message(&self, id: &str) -> Option<&Message> {
        self.messages.iter().find(|m| m.id == id)
    }
    
    /// Mark all as read
    pub fn mark_all_read(&mut self) {
        for msg in &mut self.messages {
            msg.is_read = true;
        }
        self.unread_count = 0;
    }
    
    /// Toggle mute
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }
    
    /// Toggle pin
    pub fn toggle_pin(&mut self) {
        self.pinned = !self.pinned;
    }
    
    /// Set typing indicator
    pub fn set_typing(&mut self, typing: bool) {
        self.is_typing = typing;
    }
    
    /// Get preview text
    pub fn preview(&self) -> String {
        self.last_message()
            .map(|m| m.preview())
            .unwrap_or_default()
    }
}

/// Messaging manager
#[derive(Debug)]
pub struct MessagingManager {
    /// Conversations
    conversations: HashMap<String, Conversation>,
    /// Messages sent today
    sent_today: u32,
    /// Messages received today
    received_today: u32,
    /// Last message ID
    last_message_id: u64,
    /// My user ID
    my_id: String,
}

impl MessagingManager {
    /// Create new messaging manager
    pub fn new() -> Self {
        Self {
            conversations: HashMap::new(),
            sent_today: 0,
            received_today: 0,
            last_message_id: 0,
            my_id: "me".to_string(),
        }
    }
    
    /// Set my user ID
    pub fn set_my_id(&mut self, id: String) {
        self.my_id = id;
    }
    
    /// Get or create conversation
    pub fn get_or_create_conversation(&mut self, contact_id: &str) -> &mut Conversation {
        if !self.conversations.contains_key(contact_id) {
            let conv = Conversation::new(
                contact_id.to_string(),
                vec![contact_id.to_string()],
            );
            self.conversations.insert(contact_id.to_string(), conv);
        }
        self.conversations.get_mut(contact_id).unwrap()
    }
    
    /// Get conversation
    pub fn get_conversation(&self, id: &str) -> Option<&Conversation> {
        self.conversations.get(id)
    }
    
    /// Get mutable conversation
    pub fn get_conversation_mut(&mut self, id: &str) -> Option<&mut Conversation> {
        self.conversations.get_mut(id)
    }
    
    /// Send message
    pub fn send_message(&mut self, contact_id: &str, text: &str) -> String {
        self.last_message_id += 1;
        let message_id = format!("msg-{}", self.last_message_id);
        
        let message = Message::text(
            message_id.clone(),
            self.my_id.clone(),
            contact_id.to_string(),
            text.to_string(),
            MessageDirection::Outgoing,
        );
        
        let conv = self.get_or_create_conversation(contact_id);
        conv.add_message(message);
        conv.draft = None;
        
        self.sent_today += 1;
        
        message_id
    }
    
    /// Receive message
    pub fn receive_message(&mut self, from_id: &str, text: &str) -> String {
        self.last_message_id += 1;
        let message_id = format!("msg-{}", self.last_message_id);
        
        let message = Message::text(
            message_id.clone(),
            from_id.to_string(),
            self.my_id.clone(),
            text.to_string(),
            MessageDirection::Incoming,
        );
        
        let conv = self.get_or_create_conversation(from_id);
        conv.add_message(message);
        
        self.received_today += 1;
        
        message_id
    }
    
    /// Get all conversations sorted by activity
    pub fn conversations_sorted(&self) -> Vec<&Conversation> {
        let mut convs: Vec<_> = self.conversations.values().collect();
        
        // Pinned first, then by last activity
        convs.sort_by(|a, b| {
            match (a.pinned, b.pinned) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => {
                    let a_time = a.last_activity.map(|t| t.elapsed());
                    let b_time = b.last_activity.map(|t| t.elapsed());
                    a_time.cmp(&b_time)
                }
            }
        });
        
        convs
    }
    
    /// Get unread conversations
    pub fn unread_conversations(&self) -> Vec<&Conversation> {
        self.conversations.values()
            .filter(|c| c.unread_count > 0)
            .collect()
    }
    
    /// Get total unread count
    pub fn total_unread(&self) -> u32 {
        self.conversations.values()
            .map(|c| c.unread_count)
            .sum()
    }
    
    /// Get messages sent today
    pub fn sent_today(&self) -> u32 {
        self.sent_today
    }
    
    /// Get messages received today
    pub fn received_today(&self) -> u32 {
        self.received_today
    }
    
    /// Delete conversation
    pub fn delete_conversation(&mut self, id: &str) {
        self.conversations.remove(id);
    }
    
    /// Search messages
    pub fn search(&self, query: &str) -> Vec<(&Conversation, &Message)> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        for conv in self.conversations.values() {
            for msg in conv.messages() {
                if let Some(ref text) = msg.text {
                    if text.to_lowercase().contains(&query_lower) {
                        results.push((conv, msg));
                    }
                }
            }
        }
        
        results
    }
}

impl Default for MessagingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let msg = Message::text(
            "1".to_string(),
            "alice".to_string(),
            "bob".to_string(),
            "Hello".to_string(),
            MessageDirection::Outgoing,
        );
        
        assert_eq!(msg.text, Some("Hello".to_string()));
        assert_eq!(msg.direction, MessageDirection::Outgoing);
    }
    
    #[test]
    fn test_message_preview() {
        let text_msg = Message::text(
            "1".to_string(),
            "alice".to_string(),
            "bob".to_string(),
            "Hello".to_string(),
            MessageDirection::Incoming,
        );
        
        assert_eq!(text_msg.preview(), "Hello");
    }
    
    #[test]
    fn test_conversation() {
        let mut conv = Conversation::new(
            "conv-1".to_string(),
            vec!["alice".to_string()],
        );
        
        assert_eq!(conv.unread_count, 0);
        
        let msg = Message::text(
            "1".to_string(),
            "alice".to_string(),
            "me".to_string(),
            "Hello".to_string(),
            MessageDirection::Incoming,
        );
        
        conv.add_message(msg);
        assert_eq!(conv.unread_count, 1);
        
        conv.mark_all_read();
        assert_eq!(conv.unread_count, 0);
    }
    
    #[test]
    fn test_messaging_manager() {
        let mut manager = MessagingManager::new();
        
        manager.send_message("alice", "Hello");
        
        assert_eq!(manager.sent_today(), 1);
        
        let conv = manager.get_conversation("alice");
        assert!(conv.is_some());
        assert_eq!(conv.unwrap().messages().len(), 1);
    }
    
    #[test]
    fn test_receive_message() {
        let mut manager = MessagingManager::new();
        
        manager.receive_message("alice", "Hi there");
        
        assert_eq!(manager.received_today(), 1);
        assert_eq!(manager.total_unread(), 1);
    }
    
    #[test]
    fn test_search_messages() {
        let mut manager = MessagingManager::new();
        
        manager.send_message("alice", "Hello world");
        manager.send_message("bob", "Goodbye");
        
        let results = manager.search("hello");
        assert_eq!(results.len(), 1);
    }
}
