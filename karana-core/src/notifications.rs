//! # Kāraṇa Notifications System
//!
//! Push notifications, alerts, and message queue for the glasses HUD.
//!
//! ## Features
//! - Priority-based notification queue
//! - Persistent notification storage
//! - Notification categories and filtering
//! - HUD display integration

use anyhow::Result;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

/// Notification priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

/// Notification category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    System,
    Blockchain,
    Message,
    Call,
    Timer,
    Calendar,
    Social,
    Weather,
    Navigation,
    Custom(String),
}

/// Notification state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationState {
    Unread,
    Read,
    Dismissed,
    Expired,
}

/// A single notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique ID
    pub id: u64,
    /// Title text
    pub title: String,
    /// Body text
    pub body: String,
    /// Priority level
    pub priority: Priority,
    /// Category
    pub category: Category,
    /// Current state
    pub state: NotificationState,
    /// Timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Expiration time (optional)
    pub expires_at: Option<u64>,
    /// Source app/service
    pub source: String,
    /// Optional icon/emoji
    pub icon: Option<String>,
    /// Optional action to take on tap
    pub action: Option<String>,
}

impl Notification {
    pub fn new(title: &str, body: &str) -> Self {
        Self {
            id: 0, // Will be set by manager
            title: title.to_string(),
            body: body.to_string(),
            priority: Priority::Normal,
            category: Category::System,
            state: NotificationState::Unread,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            expires_at: None,
            source: "karana".to_string(),
            icon: None,
            action: None,
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_category(mut self, category: Category) -> Self {
        self.category = category;
        self
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.source = source.to_string();
        self
    }

    pub fn with_action(mut self, action: &str) -> Self {
        self.action = Some(action.to_string());
        self
    }

    pub fn with_expiry(mut self, duration: Duration) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.expires_at = Some(now + duration.as_secs());
        self
    }

    /// Check if notification is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            now > expires
        } else {
            false
        }
    }

    /// Format for HUD display
    pub fn format_hud(&self) -> String {
        let icon = self.icon.as_deref().unwrap_or(match self.category {
            Category::System => "⚙️",
            Category::Blockchain => "⛓️",
            Category::Message => "💬",
            Category::Call => "📞",
            Category::Timer => "⏰",
            Category::Calendar => "📅",
            Category::Social => "👥",
            Category::Weather => "🌤️",
            Category::Navigation => "🧭",
            Category::Custom(_) => "📌",
        });

        format!("{} {}", icon, self.title)
    }

    /// Get time ago string
    pub fn time_ago(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        let diff = now.saturating_sub(self.timestamp);
        
        if diff < 60 {
            "just now".to_string()
        } else if diff < 3600 {
            format!("{}m ago", diff / 60)
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else {
            format!("{}d ago", diff / 86400)
        }
    }
}

/// Notification manager
pub struct NotificationManager {
    notifications: Arc<Mutex<VecDeque<Notification>>>,
    next_id: Arc<Mutex<u64>>,
    max_notifications: usize,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(Mutex::new(VecDeque::new())),
            next_id: Arc::new(Mutex::new(1)),
            max_notifications: 100,
        }
    }

    /// Add a notification
    pub fn push(&self, mut notification: Notification) -> u64 {
        let mut id = self.next_id.lock().unwrap();
        notification.id = *id;
        *id += 1;

        let mut notifications = self.notifications.lock().unwrap();
        
        // Remove oldest if at capacity
        while notifications.len() >= self.max_notifications {
            notifications.pop_back();
        }

        // Insert by priority (higher priority at front)
        let insert_pos = notifications.iter()
            .position(|n| n.priority < notification.priority)
            .unwrap_or(notifications.len());
        
        notifications.insert(insert_pos, notification.clone());

        log::info!("[NOTIFY] 📬 New notification: {} (id={}, priority={:?})", 
            notification.title, notification.id, notification.priority);

        notification.id
    }

    /// Quick notification helper
    pub fn notify(&self, title: &str, body: &str) -> u64 {
        self.push(Notification::new(title, body))
    }

    /// System notification
    pub fn system(&self, title: &str, body: &str) -> u64 {
        self.push(Notification::new(title, body)
            .with_category(Category::System)
            .with_icon("⚙️"))
    }

    /// Blockchain notification
    pub fn blockchain(&self, title: &str, body: &str) -> u64 {
        self.push(Notification::new(title, body)
            .with_category(Category::Blockchain)
            .with_icon("⛓️")
            .with_priority(Priority::High))
    }

    /// Message notification
    pub fn message(&self, from: &str, text: &str) -> u64 {
        self.push(Notification::new(&format!("From: {}", from), text)
            .with_category(Category::Message)
            .with_icon("💬"))
    }

    /// Call notification
    pub fn call(&self, caller: &str) -> u64 {
        self.push(Notification::new("Incoming Call", caller)
            .with_category(Category::Call)
            .with_icon("📞")
            .with_priority(Priority::Urgent))
    }

    /// Timer notification
    pub fn timer(&self, name: &str) -> u64 {
        self.push(Notification::new("Timer Complete", name)
            .with_category(Category::Timer)
            .with_icon("⏰")
            .with_priority(Priority::High))
    }

    /// Get all unread notifications
    pub fn unread(&self) -> Vec<Notification> {
        let notifications = self.notifications.lock().unwrap();
        notifications.iter()
            .filter(|n| n.state == NotificationState::Unread && !n.is_expired())
            .cloned()
            .collect()
    }

    /// Get unread count
    pub fn unread_count(&self) -> usize {
        self.unread().len()
    }

    /// Get all notifications
    pub fn all(&self) -> Vec<Notification> {
        let notifications = self.notifications.lock().unwrap();
        notifications.iter()
            .filter(|n| !n.is_expired())
            .cloned()
            .collect()
    }

    /// Get notifications by category
    pub fn by_category(&self, category: &Category) -> Vec<Notification> {
        let notifications = self.notifications.lock().unwrap();
        notifications.iter()
            .filter(|n| &n.category == category && !n.is_expired())
            .cloned()
            .collect()
    }

    /// Mark notification as read
    pub fn mark_read(&self, id: u64) -> bool {
        let mut notifications = self.notifications.lock().unwrap();
        if let Some(notification) = notifications.iter_mut().find(|n| n.id == id) {
            notification.state = NotificationState::Read;
            true
        } else {
            false
        }
    }

    /// Mark all as read
    pub fn mark_all_read(&self) {
        let mut notifications = self.notifications.lock().unwrap();
        for notification in notifications.iter_mut() {
            if notification.state == NotificationState::Unread {
                notification.state = NotificationState::Read;
            }
        }
    }

    /// Dismiss notification
    pub fn dismiss(&self, id: u64) -> bool {
        let mut notifications = self.notifications.lock().unwrap();
        if let Some(notification) = notifications.iter_mut().find(|n| n.id == id) {
            notification.state = NotificationState::Dismissed;
            true
        } else {
            false
        }
    }

    /// Clear all notifications
    pub fn clear(&self) {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.clear();
        log::info!("[NOTIFY] Cleared all notifications");
    }

    /// Remove expired notifications
    pub fn cleanup_expired(&self) -> usize {
        let mut notifications = self.notifications.lock().unwrap();
        let before = notifications.len();
        notifications.retain(|n| !n.is_expired());
        before - notifications.len()
    }

    /// Get HUD summary (for glasses display)
    pub fn hud_summary(&self) -> Option<String> {
        let unread = self.unread();
        if unread.is_empty() {
            return None;
        }

        // Show most recent unread notification
        if let Some(latest) = unread.first() {
            let count = unread.len();
            if count > 1 {
                Some(format!("{} (+{} more)", latest.format_hud(), count - 1))
            } else {
                Some(latest.format_hud())
            }
        } else {
            None
        }
    }

    /// Get notification badge count for HUD
    pub fn badge(&self) -> String {
        let count = self.unread_count();
        if count == 0 {
            String::new()
        } else if count > 9 {
            "9+".to_string()
        } else {
            count.to_string()
        }
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Create common notification templates
pub mod templates {
    use super::*;

    pub fn transfer_success(amount: u64, to: &str) -> Notification {
        Notification::new(
            "Transfer Complete",
            &format!("Sent {} KARA to {}", amount, to)
        )
        .with_category(Category::Blockchain)
        .with_icon("✅")
        .with_priority(Priority::High)
    }

    pub fn transfer_received(amount: u64, from: &str) -> Notification {
        Notification::new(
            "Transfer Received",
            &format!("Received {} KARA from {}", amount, from)
        )
        .with_category(Category::Blockchain)
        .with_icon("💰")
        .with_priority(Priority::High)
    }

    pub fn proposal_created(id: u64, title: &str) -> Notification {
        Notification::new(
            "New Proposal",
            &format!("#{}: {}", id, title)
        )
        .with_category(Category::Blockchain)
        .with_icon("📜")
    }

    pub fn vote_recorded(proposal_id: u64, vote: bool) -> Notification {
        Notification::new(
            "Vote Recorded",
            &format!("Voted {} on proposal #{}", if vote { "YES" } else { "NO" }, proposal_id)
        )
        .with_category(Category::Blockchain)
        .with_icon("🗳️")
    }

    pub fn low_battery(percent: u8) -> Notification {
        Notification::new(
            "Low Battery",
            &format!("{}% remaining - consider charging", percent)
        )
        .with_category(Category::System)
        .with_icon("🔋")
        .with_priority(Priority::High)
    }

    pub fn photo_captured(path: &str) -> Notification {
        Notification::new(
            "Photo Captured",
            &format!("Saved to {}", path)
        )
        .with_category(Category::System)
        .with_icon("📸")
        .with_expiry(Duration::from_secs(10))
    }

    pub fn navigation_arrived() -> Notification {
        Notification::new(
            "You've Arrived",
            "Destination reached"
        )
        .with_category(Category::Navigation)
        .with_icon("🎯")
        .with_priority(Priority::High)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notif = Notification::new("Test", "Body")
            .with_priority(Priority::High)
            .with_category(Category::Blockchain);
        
        assert_eq!(notif.title, "Test");
        assert_eq!(notif.priority, Priority::High);
        assert_eq!(notif.category, Category::Blockchain);
    }

    #[test]
    fn test_notification_manager() {
        let manager = NotificationManager::new();
        
        let id1 = manager.notify("First", "Body");
        let id2 = manager.notify("Second", "Body");
        
        assert_eq!(manager.unread_count(), 2);
        
        manager.mark_read(id1);
        assert_eq!(manager.unread_count(), 1);
    }

    #[test]
    fn test_priority_ordering() {
        let manager = NotificationManager::new();
        
        manager.push(Notification::new("Low", "").with_priority(Priority::Low));
        manager.push(Notification::new("Urgent", "").with_priority(Priority::Urgent));
        manager.push(Notification::new("Normal", "").with_priority(Priority::Normal));
        
        let unread = manager.unread();
        assert_eq!(unread[0].title, "Urgent");
        assert_eq!(unread[1].title, "Normal");
        assert_eq!(unread[2].title, "Low");
    }

    #[test]
    fn test_expiry() {
        // Create a notification that expired 1 second ago (in the past)
        let mut notif = Notification::new("Test", "");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        notif.expires_at = Some(now.saturating_sub(1)); // 1 second in the past
        
        assert!(notif.is_expired());
    }

    #[test]
    fn test_templates() {
        let notif = templates::transfer_success(100, "alice");
        assert_eq!(notif.category, Category::Blockchain);
        assert!(notif.body.contains("100"));
        assert!(notif.body.contains("alice"));
    }
}
