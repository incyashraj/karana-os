//! Haptic notification system

use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Notification priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NotificationPriority {
    /// Low priority (subtle feedback)
    Low = 1,
    /// Normal priority
    Normal = 2,
    /// High priority (more noticeable)
    High = 3,
    /// Urgent (immediate attention needed)
    Urgent = 4,
    /// Critical (emergency/safety)
    Critical = 5,
}

impl NotificationPriority {
    /// Get intensity multiplier
    pub fn intensity_multiplier(&self) -> f32 {
        match self {
            NotificationPriority::Low => 0.4,
            NotificationPriority::Normal => 0.6,
            NotificationPriority::High => 0.8,
            NotificationPriority::Urgent => 0.95,
            NotificationPriority::Critical => 1.0,
        }
    }
    
    /// Get repeat count for pattern
    pub fn repeat_count(&self) -> u32 {
        match self {
            NotificationPriority::Low => 1,
            NotificationPriority::Normal => 1,
            NotificationPriority::High => 2,
            NotificationPriority::Urgent => 3,
            NotificationPriority::Critical => 4,
        }
    }
    
    /// Get pattern name
    pub fn pattern_name(&self) -> &str {
        match self {
            NotificationPriority::Low => "notification",
            NotificationPriority::Normal => "notification",
            NotificationPriority::High => "double_tap",
            NotificationPriority::Urgent => "warning",
            NotificationPriority::Critical => "error",
        }
    }
}

/// Notification category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationCategory {
    /// Message/communication
    Message,
    /// Calendar/reminder
    Reminder,
    /// System alert
    System,
    /// Navigation instruction
    Navigation,
    /// Health/safety alert
    Health,
    /// App notification
    App,
    /// Social interaction
    Social,
    /// Timer/alarm
    Timer,
}

impl NotificationCategory {
    /// Get default pattern for category
    pub fn default_pattern(&self) -> &str {
        match self {
            NotificationCategory::Message => "double_tap",
            NotificationCategory::Reminder => "notification",
            NotificationCategory::System => "tap",
            NotificationCategory::Navigation => "scroll_tick",
            NotificationCategory::Health => "warning",
            NotificationCategory::App => "notification",
            NotificationCategory::Social => "heartbeat",
            NotificationCategory::Timer => "nav_arrival",
        }
    }
}

/// Haptic notification
#[derive(Debug, Clone)]
pub struct HapticNotification {
    /// Notification ID
    pub id: u64,
    /// Category
    pub category: NotificationCategory,
    /// Priority
    pub priority: NotificationPriority,
    /// Title (for logging/debugging)
    pub title: String,
    /// Custom pattern override
    pub pattern: Option<String>,
    /// Custom intensity override
    pub intensity: Option<f32>,
    /// When notification was created
    pub created_at: Instant,
    /// Time-to-live (after which notification is discarded)
    pub ttl: Duration,
    /// Whether notification has been played
    pub played: bool,
}

impl HapticNotification {
    /// Create new notification
    pub fn new(category: NotificationCategory, priority: NotificationPriority, title: &str) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        
        Self {
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            category,
            priority,
            title: title.to_string(),
            pattern: None,
            intensity: None,
            created_at: Instant::now(),
            ttl: Duration::from_secs(30),
            played: false,
        }
    }
    
    /// Set custom pattern
    pub fn with_pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(pattern.to_string());
        self
    }
    
    /// Set custom intensity
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = Some(intensity.clamp(0.0, 1.0));
        self
    }
    
    /// Set TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }
    
    /// Get effective pattern name
    pub fn effective_pattern(&self) -> &str {
        self.pattern.as_deref()
            .unwrap_or_else(|| self.category.default_pattern())
    }
    
    /// Get effective intensity
    pub fn effective_intensity(&self) -> f32 {
        self.intensity.unwrap_or_else(|| self.priority.intensity_multiplier())
    }
    
    /// Check if expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
    
    /// Mark as played
    pub fn mark_played(&mut self) {
        self.played = true;
    }
}

/// Notification queue manager
#[derive(Debug)]
pub struct NotificationQueue {
    /// Pending notifications
    queue: VecDeque<HapticNotification>,
    /// Maximum queue size
    max_size: usize,
    /// Minimum interval between notifications
    min_interval: Duration,
    /// Last notification time
    last_played: Option<Instant>,
    /// Do Not Disturb mode
    dnd_enabled: bool,
    /// DND exceptions (priorities that bypass DND)
    dnd_exceptions: Vec<NotificationPriority>,
    /// Statistics
    stats: NotificationStats,
}

/// Notification statistics
#[derive(Debug, Default)]
pub struct NotificationStats {
    /// Total notifications received
    pub total_received: u64,
    /// Notifications played
    pub total_played: u64,
    /// Notifications dropped (expired/overflow)
    pub total_dropped: u64,
    /// Notifications blocked by DND
    pub dnd_blocked: u64,
}

impl NotificationQueue {
    /// Create new queue
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(max_size),
            max_size,
            min_interval: Duration::from_millis(500),
            last_played: None,
            dnd_enabled: false,
            dnd_exceptions: vec![NotificationPriority::Critical, NotificationPriority::Urgent],
            stats: NotificationStats::default(),
        }
    }
    
    /// Add notification to queue
    pub fn push(&mut self, notification: HapticNotification) {
        self.stats.total_received += 1;
        
        // Check DND
        if self.dnd_enabled && !self.dnd_exceptions.contains(&notification.priority) {
            self.stats.dnd_blocked += 1;
            return;
        }
        
        // Remove expired notifications
        self.cleanup_expired();
        
        // Check queue size
        if self.queue.len() >= self.max_size {
            // Remove lowest priority item
            if let Some(idx) = self.find_lowest_priority() {
                if self.queue[idx].priority < notification.priority {
                    self.queue.remove(idx);
                    self.stats.total_dropped += 1;
                } else {
                    self.stats.total_dropped += 1;
                    return;
                }
            }
        }
        
        // Insert by priority (higher priority first)
        let insert_idx = self.queue
            .iter()
            .position(|n| n.priority < notification.priority)
            .unwrap_or(self.queue.len());
        
        self.queue.insert(insert_idx, notification);
    }
    
    /// Get next notification to play
    pub fn pop(&mut self) -> Option<HapticNotification> {
        // Check minimum interval
        if let Some(last) = self.last_played {
            if last.elapsed() < self.min_interval {
                return None;
            }
        }
        
        self.cleanup_expired();
        
        if let Some(mut notification) = self.queue.pop_front() {
            notification.mark_played();
            self.last_played = Some(Instant::now());
            self.stats.total_played += 1;
            Some(notification)
        } else {
            None
        }
    }
    
    /// Peek at next notification without removing
    pub fn peek(&self) -> Option<&HapticNotification> {
        self.queue.front()
    }
    
    /// Clean up expired notifications
    fn cleanup_expired(&mut self) {
        let before = self.queue.len();
        self.queue.retain(|n| !n.is_expired());
        self.stats.total_dropped += (before - self.queue.len()) as u64;
    }
    
    /// Find index of lowest priority notification
    fn find_lowest_priority(&self) -> Option<usize> {
        self.queue
            .iter()
            .enumerate()
            .min_by_key(|(_, n)| n.priority)
            .map(|(i, _)| i)
    }
    
    /// Enable/disable DND mode
    pub fn set_dnd(&mut self, enabled: bool) {
        self.dnd_enabled = enabled;
    }
    
    /// Check if DND is enabled
    pub fn is_dnd(&self) -> bool {
        self.dnd_enabled
    }
    
    /// Set minimum interval between notifications
    pub fn set_min_interval(&mut self, interval: Duration) {
        self.min_interval = interval;
    }
    
    /// Get queue length
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    
    /// Clear all notifications
    pub fn clear(&mut self) {
        self.stats.total_dropped += self.queue.len() as u64;
        self.queue.clear();
    }
    
    /// Get statistics
    pub fn stats(&self) -> &NotificationStats {
        &self.stats
    }
}

impl Default for NotificationQueue {
    fn default() -> Self {
        Self::new(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_notification_creation() {
        let notif = HapticNotification::new(
            NotificationCategory::Message,
            NotificationPriority::Normal,
            "New message"
        );
        
        assert_eq!(notif.category, NotificationCategory::Message);
        assert!(!notif.played);
        assert!(!notif.is_expired());
    }
    
    #[test]
    fn test_notification_priority_ordering() {
        assert!(NotificationPriority::Critical > NotificationPriority::High);
        assert!(NotificationPriority::High > NotificationPriority::Normal);
        assert!(NotificationPriority::Normal > NotificationPriority::Low);
    }
    
    #[test]
    fn test_effective_pattern() {
        let notif = HapticNotification::new(
            NotificationCategory::Message,
            NotificationPriority::Normal,
            "Test"
        );
        
        assert_eq!(notif.effective_pattern(), "double_tap");
        
        let custom = notif.with_pattern("custom_pattern");
        assert_eq!(custom.effective_pattern(), "custom_pattern");
    }
    
    #[test]
    fn test_queue_push_pop() {
        let mut queue = NotificationQueue::new(10);
        queue.set_min_interval(Duration::ZERO); // Disable for testing
        
        queue.push(HapticNotification::new(
            NotificationCategory::System,
            NotificationPriority::Normal,
            "Test"
        ));
        
        assert_eq!(queue.len(), 1);
        
        let notif = queue.pop();
        assert!(notif.is_some());
        assert!(queue.is_empty());
    }
    
    #[test]
    fn test_queue_priority_ordering() {
        let mut queue = NotificationQueue::new(10);
        queue.set_min_interval(Duration::ZERO);
        
        queue.push(HapticNotification::new(
            NotificationCategory::System,
            NotificationPriority::Low,
            "Low"
        ));
        queue.push(HapticNotification::new(
            NotificationCategory::System,
            NotificationPriority::High,
            "High"
        ));
        queue.push(HapticNotification::new(
            NotificationCategory::System,
            NotificationPriority::Normal,
            "Normal"
        ));
        
        // Should come out in priority order
        assert_eq!(queue.pop().unwrap().priority, NotificationPriority::High);
        assert_eq!(queue.pop().unwrap().priority, NotificationPriority::Normal);
        assert_eq!(queue.pop().unwrap().priority, NotificationPriority::Low);
    }
    
    #[test]
    fn test_dnd_mode() {
        let mut queue = NotificationQueue::new(10);
        queue.set_dnd(true);
        
        queue.push(HapticNotification::new(
            NotificationCategory::Message,
            NotificationPriority::Normal,
            "Blocked"
        ));
        
        assert!(queue.is_empty());
        assert_eq!(queue.stats().dnd_blocked, 1);
        
        // Critical should bypass DND
        queue.push(HapticNotification::new(
            NotificationCategory::Health,
            NotificationPriority::Critical,
            "Emergency"
        ));
        
        assert_eq!(queue.len(), 1);
    }
    
    #[test]
    fn test_queue_overflow() {
        let mut queue = NotificationQueue::new(3);
        
        for i in 0..5 {
            queue.push(HapticNotification::new(
                NotificationCategory::System,
                NotificationPriority::Normal,
                &format!("Notification {}", i)
            ));
        }
        
        assert_eq!(queue.len(), 3);
        assert!(queue.stats().total_dropped > 0);
    }
    
    #[test]
    fn test_intensity_multiplier() {
        let low = NotificationPriority::Low.intensity_multiplier();
        let critical = NotificationPriority::Critical.intensity_multiplier();
        
        assert!(critical > low);
    }
}
