//! Social & Communication for Kāraṇa OS AR Glasses
//!
//! Contact management, messaging, calls, and social features.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

pub mod contacts;
pub mod messaging;
pub mod calls;
pub mod sharing;

pub use contacts::{Contact, ContactManager, ContactGroup};
pub use messaging::{Message, Conversation, MessagingManager};
pub use calls::{Call, CallState, CallManager};
pub use sharing::{ShareTarget, ShareContent, ShareManager};

/// Communication channel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelType {
    /// SMS text message
    SMS,
    /// Phone call
    Call,
    /// Email
    Email,
    /// Instant message
    IM,
    /// Social media
    Social,
    /// Video call
    VideoCall,
}

impl ChannelType {
    /// Get display name
    pub fn name(&self) -> &str {
        match self {
            Self::SMS => "SMS",
            Self::Call => "Call",
            Self::Email => "Email",
            Self::IM => "Message",
            Self::Social => "Social",
            Self::VideoCall => "Video Call",
        }
    }
    
    /// Get icon
    pub fn icon(&self) -> &str {
        match self {
            Self::SMS => "💬",
            Self::Call => "📞",
            Self::Email => "📧",
            Self::IM => "💭",
            Self::Social => "👥",
            Self::VideoCall => "📹",
        }
    }
}

/// Presence status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresenceStatus {
    /// Available
    Available,
    /// Busy
    Busy,
    /// Away
    Away,
    /// Do not disturb
    DoNotDisturb,
    /// Offline
    Offline,
    /// Unknown
    Unknown,
}

impl PresenceStatus {
    /// Get display name
    pub fn name(&self) -> &str {
        match self {
            Self::Available => "Available",
            Self::Busy => "Busy",
            Self::Away => "Away",
            Self::DoNotDisturb => "Do Not Disturb",
            Self::Offline => "Offline",
            Self::Unknown => "Unknown",
        }
    }
    
    /// Get indicator color
    pub fn color(&self) -> &str {
        match self {
            Self::Available => "green",
            Self::Busy => "red",
            Self::Away => "yellow",
            Self::DoNotDisturb => "red",
            Self::Offline => "gray",
            Self::Unknown => "gray",
        }
    }
}

/// Communication event
#[derive(Debug, Clone)]
pub enum CommEvent {
    /// Incoming call
    IncomingCall { contact_id: String, channel: ChannelType },
    /// Call ended
    CallEnded { contact_id: String, duration: Duration },
    /// New message
    NewMessage { contact_id: String, preview: String },
    /// Message sent
    MessageSent { contact_id: String },
    /// Message read
    MessageRead { contact_id: String },
    /// Presence changed
    PresenceChanged { contact_id: String, status: PresenceStatus },
    /// Contact online
    ContactOnline { contact_id: String },
    /// Contact offline
    ContactOffline { contact_id: String },
}

/// Communication preferences
#[derive(Debug, Clone)]
pub struct CommPreferences {
    /// Default reply method
    pub default_reply: ChannelType,
    /// Auto-read receipts
    pub auto_read_receipts: bool,
    /// Show typing indicators
    pub show_typing: bool,
    /// Show presence status
    pub show_presence: bool,
    /// Voice to text enabled
    pub voice_to_text: bool,
    /// Text to voice enabled
    pub text_to_voice: bool,
    /// Contact photo size
    pub photo_size: PhotoSize,
    /// Quick replies enabled
    pub quick_replies: bool,
    /// Quick reply templates
    pub quick_reply_templates: Vec<String>,
}

impl Default for CommPreferences {
    fn default() -> Self {
        Self {
            default_reply: ChannelType::IM,
            auto_read_receipts: true,
            show_typing: true,
            show_presence: true,
            voice_to_text: true,
            text_to_voice: false,
            photo_size: PhotoSize::Medium,
            quick_replies: true,
            quick_reply_templates: vec![
                "On my way".to_string(),
                "Be there soon".to_string(),
                "Can't talk now".to_string(),
                "Call you back".to_string(),
                "👍".to_string(),
            ],
        }
    }
}

/// Photo size preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotoSize {
    /// Small (32px)
    Small,
    /// Medium (48px)
    Medium,
    /// Large (64px)
    Large,
}

/// Communication statistics
#[derive(Debug, Clone)]
pub struct CommStats {
    /// Total messages sent today
    pub messages_sent_today: u32,
    /// Total messages received today
    pub messages_received_today: u32,
    /// Total call time today
    pub call_time_today: Duration,
    /// Calls made today
    pub calls_made_today: u32,
    /// Calls received today
    pub calls_received_today: u32,
    /// Most contacted person
    pub most_contacted: Option<String>,
}

/// Communication manager
#[derive(Debug)]
pub struct CommunicationManager {
    /// Preferences
    preferences: CommPreferences,
    /// Contact manager
    contacts: ContactManager,
    /// Messaging manager
    messaging: MessagingManager,
    /// Call manager
    calls: CallManager,
    /// Share manager
    sharing: ShareManager,
    /// User's presence status
    my_status: PresenceStatus,
    /// Event history
    event_history: VecDeque<CommEvent>,
    /// Max history size
    max_history: usize,
    /// Unread message count
    unread_count: u32,
    /// Missed call count
    missed_calls: u32,
}

impl CommunicationManager {
    /// Create new communication manager
    pub fn new() -> Self {
        Self {
            preferences: CommPreferences::default(),
            contacts: ContactManager::new(),
            messaging: MessagingManager::new(),
            calls: CallManager::new(),
            sharing: ShareManager::new(),
            my_status: PresenceStatus::Available,
            event_history: VecDeque::with_capacity(100),
            max_history: 100,
            unread_count: 0,
            missed_calls: 0,
        }
    }
    
    /// Get preferences
    pub fn preferences(&self) -> &CommPreferences {
        &self.preferences
    }
    
    /// Update preferences
    pub fn update_preferences(&mut self, prefs: CommPreferences) {
        self.preferences = prefs;
    }
    
    /// Get contact manager
    pub fn contacts(&self) -> &ContactManager {
        &self.contacts
    }
    
    /// Get mutable contact manager
    pub fn contacts_mut(&mut self) -> &mut ContactManager {
        &mut self.contacts
    }
    
    /// Get messaging manager
    pub fn messaging(&self) -> &MessagingManager {
        &self.messaging
    }
    
    /// Get mutable messaging manager
    pub fn messaging_mut(&mut self) -> &mut MessagingManager {
        &mut self.messaging
    }
    
    /// Get call manager
    pub fn calls(&self) -> &CallManager {
        &self.calls
    }
    
    /// Get mutable call manager
    pub fn calls_mut(&mut self) -> &mut CallManager {
        &mut self.calls
    }
    
    /// Get share manager
    pub fn sharing(&self) -> &ShareManager {
        &self.sharing
    }
    
    /// Get mutable share manager
    pub fn sharing_mut(&mut self) -> &mut ShareManager {
        &mut self.sharing
    }
    
    /// Set my presence status
    pub fn set_status(&mut self, status: PresenceStatus) {
        self.my_status = status;
    }
    
    /// Get my presence status
    pub fn my_status(&self) -> PresenceStatus {
        self.my_status
    }
    
    /// Get unread message count
    pub fn unread_count(&self) -> u32 {
        self.unread_count
    }
    
    /// Get missed call count
    pub fn missed_calls(&self) -> u32 {
        self.missed_calls
    }
    
    /// Clear missed calls
    pub fn clear_missed_calls(&mut self) {
        self.missed_calls = 0;
    }
    
    /// Send quick reply
    pub fn send_quick_reply(&mut self, contact_id: &str, template_index: usize) -> bool {
        if template_index >= self.preferences.quick_reply_templates.len() {
            return false;
        }
        
        let message = self.preferences.quick_reply_templates[template_index].clone();
        self.messaging.send_message(contact_id, &message);
        
        self.add_event(CommEvent::MessageSent {
            contact_id: contact_id.to_string(),
        });
        
        true
    }
    
    /// Add event to history
    fn add_event(&mut self, event: CommEvent) {
        if self.event_history.len() >= self.max_history {
            self.event_history.pop_front();
        }
        self.event_history.push_back(event);
    }
    
    /// Get recent events
    pub fn recent_events(&self, count: usize) -> Vec<&CommEvent> {
        self.event_history.iter().rev().take(count).collect()
    }
    
    /// Handle incoming call
    pub fn handle_incoming_call(&mut self, contact_id: &str, channel: ChannelType) {
        self.add_event(CommEvent::IncomingCall {
            contact_id: contact_id.to_string(),
            channel,
        });
    }
    
    /// Handle new message
    pub fn handle_new_message(&mut self, contact_id: &str, preview: &str) {
        self.unread_count += 1;
        self.add_event(CommEvent::NewMessage {
            contact_id: contact_id.to_string(),
            preview: preview.to_string(),
        });
    }
    
    /// Mark messages as read
    pub fn mark_read(&mut self, contact_id: &str) {
        self.unread_count = self.unread_count.saturating_sub(1);
        self.add_event(CommEvent::MessageRead {
            contact_id: contact_id.to_string(),
        });
    }
    
    /// Get communication statistics
    pub fn stats(&self) -> CommStats {
        CommStats {
            messages_sent_today: self.messaging.sent_today(),
            messages_received_today: self.messaging.received_today(),
            call_time_today: self.calls.call_time_today(),
            calls_made_today: self.calls.calls_made_today(),
            calls_received_today: self.calls.calls_received_today(),
            most_contacted: self.contacts.most_contacted(),
        }
    }
}

impl Default for CommunicationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_communication_manager_creation() {
        let manager = CommunicationManager::new();
        assert_eq!(manager.my_status(), PresenceStatus::Available);
    }
    
    #[test]
    fn test_channel_type() {
        assert_eq!(ChannelType::SMS.name(), "SMS");
        assert_eq!(ChannelType::Call.icon(), "📞");
    }
    
    #[test]
    fn test_presence_status() {
        assert_eq!(PresenceStatus::Available.name(), "Available");
        assert_eq!(PresenceStatus::Available.color(), "green");
    }
    
    #[test]
    fn test_set_status() {
        let mut manager = CommunicationManager::new();
        manager.set_status(PresenceStatus::Busy);
        assert_eq!(manager.my_status(), PresenceStatus::Busy);
    }
    
    #[test]
    fn test_quick_reply() {
        let mut manager = CommunicationManager::new();
        assert!(manager.send_quick_reply("contact-1", 0));
        assert!(!manager.send_quick_reply("contact-1", 100));
    }
    
    #[test]
    fn test_message_handling() {
        let mut manager = CommunicationManager::new();
        
        assert_eq!(manager.unread_count(), 0);
        
        manager.handle_new_message("contact-1", "Hello");
        assert_eq!(manager.unread_count(), 1);
        
        manager.mark_read("contact-1");
        assert_eq!(manager.unread_count(), 0);
    }
    
    #[test]
    fn test_stats() {
        let manager = CommunicationManager::new();
        let stats = manager.stats();
        
        assert_eq!(stats.messages_sent_today, 0);
    }
}
