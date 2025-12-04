//! Notifications System for Kāraṇa OS AR Glasses
//!
//! Manages notifications display, filtering, priority, and delivery
//! optimized for AR glasses viewing experience.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use nalgebra::Vector3;

pub mod channel;
pub mod filter;
pub mod display;
pub mod summary;

pub use channel::{NotificationChannel, ChannelConfig, ChannelImportance};
pub use filter::{NotificationFilter, FilterRule, FilterAction};
pub use display::{NotificationDisplay, DisplayPosition, DisplayStyle};
pub use summary::{NotificationSummary, SummaryMode, SummaryGenerator};

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NotificationPriority {
    /// Lowest priority, silent
    Min,
    /// Low priority
    Low,
    /// Default priority
    Default,
    /// High priority
    High,
    /// Urgent, interrupts user
    Urgent,
    /// Critical, cannot be dismissed
    Critical,
}

impl NotificationPriority {
    /// Should this interrupt the user?
    pub fn is_interruptive(&self) -> bool {
        matches!(self, NotificationPriority::Urgent | NotificationPriority::Critical)
    }
    
    /// Should this play a sound?
    pub fn plays_sound(&self) -> bool {
        *self >= NotificationPriority::Default
    }
    
    /// Should this trigger haptic feedback?
    pub fn has_haptic(&self) -> bool {
        *self >= NotificationPriority::High
    }
    
    /// Auto-dismiss delay (None = manual dismiss only)
    pub fn auto_dismiss_delay(&self) -> Option<Duration> {
        match self {
            NotificationPriority::Min => Some(Duration::from_secs(3)),
            NotificationPriority::Low => Some(Duration::from_secs(5)),
            NotificationPriority::Default => Some(Duration::from_secs(8)),
            NotificationPriority::High => Some(Duration::from_secs(15)),
            NotificationPriority::Urgent => None, // Manual dismiss
            NotificationPriority::Critical => None,
        }
    }
}

/// Notification category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationCategory {
    /// Social/messaging
    Social,
    /// Email
    Email,
    /// Calendar/reminders
    Calendar,
    /// Calls
    Call,
    /// System alerts
    System,
    /// App notifications
    App,
    /// Navigation/maps
    Navigation,
    /// Health/fitness
    Health,
    /// News/updates
    News,
    /// Promotions/marketing
    Promotion,
    /// Alarms
    Alarm,
    /// Media playback
    Media,
}

impl NotificationCategory {
    /// Default importance for category
    pub fn default_importance(&self) -> ChannelImportance {
        match self {
            NotificationCategory::Call => ChannelImportance::Urgent,
            NotificationCategory::Alarm => ChannelImportance::Urgent,
            NotificationCategory::Calendar => ChannelImportance::High,
            NotificationCategory::Social => ChannelImportance::Default,
            NotificationCategory::Email => ChannelImportance::Default,
            NotificationCategory::Navigation => ChannelImportance::High,
            NotificationCategory::Health => ChannelImportance::High,
            NotificationCategory::System => ChannelImportance::Default,
            NotificationCategory::App => ChannelImportance::Default,
            NotificationCategory::News => ChannelImportance::Low,
            NotificationCategory::Media => ChannelImportance::Low,
            NotificationCategory::Promotion => ChannelImportance::Min,
        }
    }
}

/// Notification action
#[derive(Debug, Clone)]
pub struct NotificationAction {
    /// Action ID
    pub id: String,
    /// Action label
    pub label: String,
    /// Icon name
    pub icon: Option<String>,
    /// Is destructive action
    pub destructive: bool,
    /// Requires unlock
    pub requires_unlock: bool,
}

/// Notification content
#[derive(Debug, Clone)]
pub struct Notification {
    /// Unique notification ID
    pub id: u64,
    /// Source app ID
    pub app_id: String,
    /// Source app name
    pub app_name: String,
    /// Channel ID
    pub channel_id: String,
    /// Category
    pub category: NotificationCategory,
    /// Priority
    pub priority: NotificationPriority,
    /// Title
    pub title: String,
    /// Body text
    pub body: String,
    /// Small icon
    pub small_icon: Option<String>,
    /// Large icon (profile pic, etc.)
    pub large_icon: Option<String>,
    /// Image attachment
    pub image: Option<String>,
    /// Actions
    pub actions: Vec<NotificationAction>,
    /// Timestamp
    pub timestamp: Instant,
    /// Group key for bundling
    pub group_key: Option<String>,
    /// Is summary notification
    pub is_summary: bool,
    /// Sort key for ordering within group
    pub sort_key: Option<String>,
    /// Ongoing (cannot be dismissed)
    pub ongoing: bool,
    /// Silent (no sound/vibration)
    pub silent: bool,
    /// Local only (don't sync)
    pub local_only: bool,
    /// Auto-cancel when tapped
    pub auto_cancel: bool,
    /// Progress (min, max, current) for progress notifications
    pub progress: Option<(i32, i32, i32)>,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Read status
    pub is_read: bool,
    /// Dismissed status
    pub is_dismissed: bool,
}

impl Notification {
    /// Create new notification
    pub fn new(app_id: &str, title: &str, body: &str) -> Self {
        Self {
            id: 0, // Set by manager
            app_id: app_id.to_string(),
            app_name: app_id.to_string(), // Default to app_id
            channel_id: "default".to_string(),
            category: NotificationCategory::App,
            priority: NotificationPriority::Default,
            title: title.to_string(),
            body: body.to_string(),
            small_icon: None,
            large_icon: None,
            image: None,
            actions: Vec::new(),
            timestamp: Instant::now(),
            group_key: None,
            is_summary: false,
            sort_key: None,
            ongoing: false,
            silent: false,
            local_only: false,
            auto_cancel: true,
            progress: None,
            metadata: HashMap::new(),
            is_read: false,
            is_dismissed: false,
        }
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: NotificationPriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set category
    pub fn with_category(mut self, category: NotificationCategory) -> Self {
        self.category = category;
        self
    }
    
    /// Set channel
    pub fn with_channel(mut self, channel_id: &str) -> Self {
        self.channel_id = channel_id.to_string();
        self
    }
    
    /// Add action
    pub fn with_action(mut self, id: &str, label: &str) -> Self {
        self.actions.push(NotificationAction {
            id: id.to_string(),
            label: label.to_string(),
            icon: None,
            destructive: false,
            requires_unlock: false,
        });
        self
    }
    
    /// Set as ongoing
    pub fn ongoing(mut self) -> Self {
        self.ongoing = true;
        self
    }
    
    /// Set as silent
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }
    
    /// Set group
    pub fn with_group(mut self, key: &str) -> Self {
        self.group_key = Some(key.to_string());
        self
    }
    
    /// Age of notification
    pub fn age(&self) -> Duration {
        Instant::now().duration_since(self.timestamp)
    }
}

/// Focus mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusMode {
    /// All notifications allowed
    None,
    /// Only priority notifications
    Priority,
    /// Only alarms
    AlarmsOnly,
    /// Complete silence
    TotalSilence,
    /// Custom focus mode
    Custom,
}

/// Notification manager
pub struct NotificationManager {
    /// Active notifications
    notifications: HashMap<u64, Notification>,
    /// Notification history
    history: VecDeque<Notification>,
    /// Max history size
    max_history: usize,
    /// Next notification ID
    next_id: u64,
    /// Channels
    channels: HashMap<String, NotificationChannel>,
    /// Filters
    filters: Vec<NotificationFilter>,
    /// Current focus mode
    focus_mode: FocusMode,
    /// Priority apps (bypass focus mode)
    priority_apps: Vec<String>,
    /// Priority contacts (bypass focus mode)
    priority_contacts: Vec<String>,
    /// Do not disturb enabled
    dnd_enabled: bool,
    /// DND schedule (start_hour, end_hour)
    dnd_schedule: Option<(u8, u8)>,
    /// Display manager
    display: NotificationDisplay,
    /// Dismissed count
    dismissed_count: u64,
    /// Delivered count
    delivered_count: u64,
}

impl NotificationManager {
    /// Create new notification manager
    pub fn new() -> Self {
        Self {
            notifications: HashMap::new(),
            history: VecDeque::new(),
            max_history: 1000,
            next_id: 1,
            channels: Self::default_channels(),
            filters: Vec::new(),
            focus_mode: FocusMode::None,
            priority_apps: Vec::new(),
            priority_contacts: Vec::new(),
            dnd_enabled: false,
            dnd_schedule: None,
            display: NotificationDisplay::new(),
            dismissed_count: 0,
            delivered_count: 0,
        }
    }
    
    /// Default notification channels
    fn default_channels() -> HashMap<String, NotificationChannel> {
        let mut channels = HashMap::new();
        
        channels.insert("default".to_string(), NotificationChannel::new(
            "default", "Default", ChannelImportance::Default
        ));
        channels.insert("messages".to_string(), NotificationChannel::new(
            "messages", "Messages", ChannelImportance::High
        ));
        channels.insert("calls".to_string(), NotificationChannel::new(
            "calls", "Calls", ChannelImportance::Urgent
        ));
        channels.insert("system".to_string(), NotificationChannel::new(
            "system", "System", ChannelImportance::Default
        ));
        channels.insert("alarms".to_string(), NotificationChannel::new(
            "alarms", "Alarms", ChannelImportance::Urgent
        ));
        
        channels
    }
    
    /// Post a notification
    pub fn notify(&mut self, mut notification: Notification) -> u64 {
        notification.id = self.next_id;
        self.next_id += 1;
        
        // Apply filters
        if let Some(action) = self.apply_filters(&notification) {
            match action {
                FilterAction::Block => return 0,
                FilterAction::Silent => notification.silent = true,
                FilterAction::Modify(priority) => notification.priority = priority,
                FilterAction::Allow => {}
            }
        }
        
        // Check focus mode
        if !self.should_deliver(&notification) {
            notification.silent = true;
        }
        
        let id = notification.id;
        self.delivered_count += 1;
        
        // Fire callbacks (would be async in production)
        // self.on_notification callbacks...
        
        self.notifications.insert(id, notification);
        
        id
    }
    
    /// Apply filters to notification
    fn apply_filters(&self, notification: &Notification) -> Option<FilterAction> {
        for filter in &self.filters {
            if let Some(action) = filter.check(notification) {
                return Some(action);
            }
        }
        None
    }
    
    /// Check if notification should be delivered based on focus mode
    fn should_deliver(&self, notification: &Notification) -> bool {
        // DND check
        if self.dnd_enabled {
            // In real implementation, would check schedule
            if notification.priority < NotificationPriority::Urgent {
                return false;
            }
        }
        
        // Focus mode check
        match self.focus_mode {
            FocusMode::None => true,
            FocusMode::TotalSilence => false,
            FocusMode::AlarmsOnly => notification.category == NotificationCategory::Alarm,
            FocusMode::Priority => {
                notification.priority >= NotificationPriority::High ||
                self.priority_apps.contains(&notification.app_id)
            }
            FocusMode::Custom => {
                // Would check custom rules
                true
            }
        }
    }
    
    /// Dismiss notification
    pub fn dismiss(&mut self, notification_id: u64) {
        if let Some(mut notification) = self.notifications.remove(&notification_id) {
            if !notification.ongoing {
                notification.is_dismissed = true;
                self.dismissed_count += 1;
                
                // Move to history
                if self.history.len() >= self.max_history {
                    self.history.pop_front();
                }
                self.history.push_back(notification);
            } else {
                // Can't dismiss ongoing notifications
                self.notifications.insert(notification_id, notification);
            }
        }
    }
    
    /// Dismiss all notifications
    pub fn dismiss_all(&mut self) {
        let ids: Vec<u64> = self.notifications.keys().copied().collect();
        for id in ids {
            self.dismiss(id);
        }
    }
    
    /// Mark notification as read
    pub fn mark_read(&mut self, notification_id: u64) {
        if let Some(notification) = self.notifications.get_mut(&notification_id) {
            notification.is_read = true;
        }
    }
    
    /// Cancel notification by app
    pub fn cancel(&mut self, notification_id: u64) {
        self.notifications.remove(&notification_id);
    }
    
    /// Cancel all notifications from app
    pub fn cancel_all_from_app(&mut self, app_id: &str) {
        self.notifications.retain(|_, n| n.app_id != app_id);
    }
    
    /// Get active notifications
    pub fn active_notifications(&self) -> Vec<&Notification> {
        let mut notifications: Vec<_> = self.notifications.values().collect();
        notifications.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        notifications
    }
    
    /// Get notification count
    pub fn notification_count(&self) -> usize {
        self.notifications.len()
    }
    
    /// Get unread count
    pub fn unread_count(&self) -> usize {
        self.notifications.values().filter(|n| !n.is_read).count()
    }
    
    /// Get notifications by category
    pub fn by_category(&self, category: NotificationCategory) -> Vec<&Notification> {
        self.notifications.values()
            .filter(|n| n.category == category)
            .collect()
    }
    
    /// Get notifications by priority
    pub fn by_priority(&self, priority: NotificationPriority) -> Vec<&Notification> {
        self.notifications.values()
            .filter(|n| n.priority == priority)
            .collect()
    }
    
    /// Set focus mode
    pub fn set_focus_mode(&mut self, mode: FocusMode) {
        self.focus_mode = mode;
    }
    
    /// Get focus mode
    pub fn focus_mode(&self) -> FocusMode {
        self.focus_mode
    }
    
    /// Enable/disable DND
    pub fn set_dnd(&mut self, enabled: bool) {
        self.dnd_enabled = enabled;
    }
    
    /// Is DND enabled
    pub fn is_dnd_enabled(&self) -> bool {
        self.dnd_enabled
    }
    
    /// Add priority app
    pub fn add_priority_app(&mut self, app_id: &str) {
        if !self.priority_apps.contains(&app_id.to_string()) {
            self.priority_apps.push(app_id.to_string());
        }
    }
    
    /// Remove priority app
    pub fn remove_priority_app(&mut self, app_id: &str) {
        self.priority_apps.retain(|a| a != app_id);
    }
    
    /// Create/update channel
    pub fn create_channel(&mut self, channel: NotificationChannel) {
        self.channels.insert(channel.id.clone(), channel);
    }
    
    /// Get channel
    pub fn get_channel(&self, channel_id: &str) -> Option<&NotificationChannel> {
        self.channels.get(channel_id)
    }
    
    /// Add filter
    pub fn add_filter(&mut self, filter: NotificationFilter) {
        self.filters.push(filter);
    }
    
    /// Remove filter
    pub fn remove_filter(&mut self, index: usize) {
        if index < self.filters.len() {
            self.filters.remove(index);
        }
    }
    
    /// Get display settings
    pub fn display(&self) -> &NotificationDisplay {
        &self.display
    }
    
    /// Get mutable display settings
    pub fn display_mut(&mut self) -> &mut NotificationDisplay {
        &mut self.display
    }
    
    /// Get statistics
    pub fn stats(&self) -> NotificationStats {
        let by_category: HashMap<NotificationCategory, usize> = self.notifications.values()
            .fold(HashMap::new(), |mut map, n| {
                *map.entry(n.category).or_insert(0) += 1;
                map
            });
        
        NotificationStats {
            active_count: self.notifications.len(),
            unread_count: self.unread_count(),
            history_count: self.history.len(),
            delivered_total: self.delivered_count,
            dismissed_total: self.dismissed_count,
            channels_count: self.channels.len(),
            filters_count: self.filters.len(),
            by_category,
            focus_mode: self.focus_mode,
            dnd_enabled: self.dnd_enabled,
        }
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Notification statistics
#[derive(Debug, Clone)]
pub struct NotificationStats {
    /// Active notification count
    pub active_count: usize,
    /// Unread count
    pub unread_count: usize,
    /// History count
    pub history_count: usize,
    /// Total delivered
    pub delivered_total: u64,
    /// Total dismissed
    pub dismissed_total: u64,
    /// Channel count
    pub channels_count: usize,
    /// Filter count
    pub filters_count: usize,
    /// By category
    pub by_category: HashMap<NotificationCategory, usize>,
    /// Current focus mode
    pub focus_mode: FocusMode,
    /// DND enabled
    pub dnd_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_notification_manager_creation() {
        let nm = NotificationManager::new();
        assert_eq!(nm.notification_count(), 0);
    }
    
    #[test]
    fn test_notify() {
        let mut nm = NotificationManager::new();
        
        let id = nm.notify(Notification::new("app1", "Title", "Body"));
        
        assert!(id > 0);
        assert_eq!(nm.notification_count(), 1);
    }
    
    #[test]
    fn test_dismiss() {
        let mut nm = NotificationManager::new();
        
        let id = nm.notify(Notification::new("app1", "Title", "Body"));
        nm.dismiss(id);
        
        assert_eq!(nm.notification_count(), 0);
    }
    
    #[test]
    fn test_ongoing_notification() {
        let mut nm = NotificationManager::new();
        
        let id = nm.notify(Notification::new("app1", "Music", "Playing").ongoing());
        nm.dismiss(id);
        
        // Ongoing notifications can't be dismissed
        assert_eq!(nm.notification_count(), 1);
    }
    
    #[test]
    fn test_mark_read() {
        let mut nm = NotificationManager::new();
        
        let id = nm.notify(Notification::new("app1", "Title", "Body"));
        assert_eq!(nm.unread_count(), 1);
        
        nm.mark_read(id);
        assert_eq!(nm.unread_count(), 0);
    }
    
    #[test]
    fn test_priority_levels() {
        assert!(NotificationPriority::Critical > NotificationPriority::Low);
        assert!(NotificationPriority::Urgent.is_interruptive());
        assert!(!NotificationPriority::Low.is_interruptive());
    }
    
    #[test]
    fn test_focus_mode() {
        let mut nm = NotificationManager::new();
        
        nm.set_focus_mode(FocusMode::AlarmsOnly);
        
        // Regular notification should be silent
        nm.notify(Notification::new("app1", "Title", "Body"));
        
        // Alarm should deliver
        nm.notify(Notification::new("alarm", "Wake up", "Time to get up")
            .with_category(NotificationCategory::Alarm));
        
        assert_eq!(nm.notification_count(), 2);
    }
    
    #[test]
    fn test_cancel_from_app() {
        let mut nm = NotificationManager::new();
        
        nm.notify(Notification::new("app1", "Title1", "Body1"));
        nm.notify(Notification::new("app1", "Title2", "Body2"));
        nm.notify(Notification::new("app2", "Title3", "Body3"));
        
        nm.cancel_all_from_app("app1");
        
        assert_eq!(nm.notification_count(), 1);
    }
    
    #[test]
    fn test_notification_categories() {
        let mut nm = NotificationManager::new();
        
        nm.notify(Notification::new("app1", "Message", "Hi!")
            .with_category(NotificationCategory::Social));
        nm.notify(Notification::new("app2", "Email", "New email")
            .with_category(NotificationCategory::Email));
        
        let social = nm.by_category(NotificationCategory::Social);
        assert_eq!(social.len(), 1);
    }
    
    #[test]
    fn test_notification_actions() {
        let notification = Notification::new("app1", "Title", "Body")
            .with_action("reply", "Reply")
            .with_action("dismiss", "Dismiss");
        
        assert_eq!(notification.actions.len(), 2);
    }
    
    #[test]
    fn test_notification_stats() {
        let mut nm = NotificationManager::new();
        
        nm.notify(Notification::new("app1", "Title", "Body"));
        
        let stats = nm.stats();
        assert_eq!(stats.active_count, 1);
        assert_eq!(stats.delivered_total, 1);
    }
}
