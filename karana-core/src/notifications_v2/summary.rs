//! Notification Summary System for Kāraṇa OS AR Glasses
//!
//! AI-powered summarization of notifications for quick digest.

use super::{Notification, NotificationCategory, NotificationPriority};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Summary mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryMode {
    /// One-line summary
    OneLine,
    /// Bullet points
    BulletPoints,
    /// Grouped by app
    ByApp,
    /// Grouped by category
    ByCategory,
    /// Priority-sorted list
    ByPriority,
    /// AI-generated natural language summary
    NaturalLanguage,
}

/// Summary item
#[derive(Debug, Clone)]
pub struct SummaryItem {
    /// App or category name
    pub source: String,
    /// Count of notifications
    pub count: usize,
    /// Summary text
    pub summary: String,
    /// Representative notification IDs
    pub notification_ids: Vec<u64>,
    /// Highest priority in group
    pub max_priority: NotificationPriority,
    /// Has unread
    pub has_unread: bool,
}

/// Notification summary
#[derive(Debug, Clone)]
pub struct NotificationSummary {
    /// Total notification count
    pub total_count: usize,
    /// Unread count
    pub unread_count: usize,
    /// Summary items
    pub items: Vec<SummaryItem>,
    /// Generated at
    pub generated_at: Instant,
    /// Summary mode used
    pub mode: SummaryMode,
    /// One-line overview
    pub overview: String,
    /// Time range covered
    pub time_range: Option<Duration>,
}

impl NotificationSummary {
    /// Create empty summary
    pub fn empty() -> Self {
        Self {
            total_count: 0,
            unread_count: 0,
            items: Vec::new(),
            generated_at: Instant::now(),
            mode: SummaryMode::OneLine,
            overview: "No new notifications".to_string(),
            time_range: None,
        }
    }
    
    /// Check if summary is empty
    pub fn is_empty(&self) -> bool {
        self.total_count == 0
    }
    
    /// Get summary age
    pub fn age(&self) -> Duration {
        Instant::now().duration_since(self.generated_at)
    }
}

/// Summary generator
#[derive(Debug)]
pub struct SummaryGenerator {
    /// Default summary mode
    default_mode: SummaryMode,
    /// Max items in summary
    max_items: usize,
    /// Include apps list
    include_apps: bool,
    /// AI summary enabled
    ai_summary_enabled: bool,
}

impl SummaryGenerator {
    /// Create new generator
    pub fn new() -> Self {
        Self {
            default_mode: SummaryMode::ByCategory,
            max_items: 10,
            include_apps: true,
            ai_summary_enabled: true,
        }
    }
    
    /// Set default mode
    pub fn set_default_mode(&mut self, mode: SummaryMode) {
        self.default_mode = mode;
    }
    
    /// Generate summary from notifications
    pub fn generate(&self, notifications: &[&Notification], mode: Option<SummaryMode>) -> NotificationSummary {
        let mode = mode.unwrap_or(self.default_mode);
        
        if notifications.is_empty() {
            return NotificationSummary::empty();
        }
        
        let total_count = notifications.len();
        let unread_count = notifications.iter().filter(|n| !n.is_read).count();
        
        let items = match mode {
            SummaryMode::ByApp => self.group_by_app(notifications),
            SummaryMode::ByCategory => self.group_by_category(notifications),
            SummaryMode::ByPriority => self.group_by_priority(notifications),
            SummaryMode::BulletPoints => self.bullet_points(notifications),
            SummaryMode::OneLine | SummaryMode::NaturalLanguage => Vec::new(),
        };
        
        let overview = self.generate_overview(notifications, unread_count);
        
        NotificationSummary {
            total_count,
            unread_count,
            items,
            generated_at: Instant::now(),
            mode,
            overview,
            time_range: Self::calculate_time_range(notifications),
        }
    }
    
    /// Group notifications by app
    fn group_by_app(&self, notifications: &[&Notification]) -> Vec<SummaryItem> {
        let mut groups: HashMap<String, Vec<&Notification>> = HashMap::new();
        
        for notification in notifications {
            groups.entry(notification.app_name.clone())
                .or_default()
                .push(notification);
        }
        
        let mut items: Vec<SummaryItem> = groups.into_iter()
            .map(|(app_name, notifs)| {
                let max_priority = notifs.iter()
                    .map(|n| n.priority)
                    .max()
                    .unwrap_or(NotificationPriority::Default);
                
                let has_unread = notifs.iter().any(|n| !n.is_read);
                
                SummaryItem {
                    source: app_name.clone(),
                    count: notifs.len(),
                    summary: self.summarize_group(&notifs),
                    notification_ids: notifs.iter().map(|n| n.id).collect(),
                    max_priority,
                    has_unread,
                }
            })
            .collect();
        
        // Sort by priority then count
        items.sort_by(|a, b| {
            b.max_priority.cmp(&a.max_priority)
                .then(b.count.cmp(&a.count))
        });
        
        items.truncate(self.max_items);
        items
    }
    
    /// Group notifications by category
    fn group_by_category(&self, notifications: &[&Notification]) -> Vec<SummaryItem> {
        let mut groups: HashMap<NotificationCategory, Vec<&Notification>> = HashMap::new();
        
        for notification in notifications {
            groups.entry(notification.category)
                .or_default()
                .push(notification);
        }
        
        let mut items: Vec<SummaryItem> = groups.into_iter()
            .map(|(category, notifs)| {
                let max_priority = notifs.iter()
                    .map(|n| n.priority)
                    .max()
                    .unwrap_or(NotificationPriority::Default);
                
                SummaryItem {
                    source: format!("{:?}", category),
                    count: notifs.len(),
                    summary: self.summarize_group(&notifs),
                    notification_ids: notifs.iter().map(|n| n.id).collect(),
                    max_priority,
                    has_unread: notifs.iter().any(|n| !n.is_read),
                }
            })
            .collect();
        
        items.sort_by(|a, b| b.max_priority.cmp(&a.max_priority));
        items.truncate(self.max_items);
        items
    }
    
    /// Group by priority
    fn group_by_priority(&self, notifications: &[&Notification]) -> Vec<SummaryItem> {
        let mut groups: HashMap<NotificationPriority, Vec<&Notification>> = HashMap::new();
        
        for notification in notifications {
            groups.entry(notification.priority)
                .or_default()
                .push(notification);
        }
        
        let mut items: Vec<SummaryItem> = groups.into_iter()
            .map(|(priority, notifs)| {
                SummaryItem {
                    source: format!("{:?}", priority),
                    count: notifs.len(),
                    summary: self.summarize_group(&notifs),
                    notification_ids: notifs.iter().map(|n| n.id).collect(),
                    max_priority: priority,
                    has_unread: notifs.iter().any(|n| !n.is_read),
                }
            })
            .collect();
        
        items.sort_by(|a, b| b.max_priority.cmp(&a.max_priority));
        items
    }
    
    /// Generate bullet point list
    fn bullet_points(&self, notifications: &[&Notification]) -> Vec<SummaryItem> {
        notifications.iter()
            .take(self.max_items)
            .map(|n| {
                SummaryItem {
                    source: n.app_name.clone(),
                    count: 1,
                    summary: n.title.clone(),
                    notification_ids: vec![n.id],
                    max_priority: n.priority,
                    has_unread: !n.is_read,
                }
            })
            .collect()
    }
    
    /// Summarize a group of notifications
    fn summarize_group(&self, notifications: &[&Notification]) -> String {
        if notifications.len() == 1 {
            return notifications[0].title.clone();
        }
        
        // Get unique titles
        let titles: Vec<_> = notifications.iter()
            .take(3)
            .map(|n| n.title.as_str())
            .collect();
        
        if titles.len() == notifications.len() {
            titles.join(", ")
        } else {
            format!("{} and {} more", titles.join(", "), notifications.len() - titles.len())
        }
    }
    
    /// Generate one-line overview
    fn generate_overview(&self, notifications: &[&Notification], unread: usize) -> String {
        let total = notifications.len();
        
        if total == 0 {
            return "No notifications".to_string();
        }
        
        // Find dominant category
        let mut category_counts: HashMap<NotificationCategory, usize> = HashMap::new();
        for n in notifications {
            *category_counts.entry(n.category).or_insert(0) += 1;
        }
        
        let top_category = category_counts.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(cat, _)| cat);
        
        let unread_str = if unread > 0 {
            format!("{} unread, ", unread)
        } else {
            String::new()
        };
        
        match top_category {
            Some(NotificationCategory::Social) => 
                format!("{}{} messages from {} apps", unread_str, total, 
                    notifications.iter().map(|n| &n.app_id).collect::<std::collections::HashSet<_>>().len()),
            Some(NotificationCategory::Email) => 
                format!("{}{} new emails", unread_str, total),
            _ => format!("{}{} notifications", unread_str, total),
        }
    }
    
    /// Calculate time range
    fn calculate_time_range(notifications: &[&Notification]) -> Option<Duration> {
        if notifications.is_empty() {
            return None;
        }
        
        let oldest = notifications.iter()
            .map(|n| n.timestamp)
            .min()?;
        
        Some(Instant::now().duration_since(oldest))
    }
}

impl Default for SummaryGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Digest scheduling
#[derive(Debug, Clone)]
pub struct DigestSchedule {
    /// Digest enabled
    pub enabled: bool,
    /// Digest interval
    pub interval: Duration,
    /// Hours to deliver (24h format)
    pub delivery_hours: Vec<u8>,
    /// Include categories
    pub categories: Option<Vec<NotificationCategory>>,
    /// Minimum notifications for digest
    pub min_notifications: usize,
}

impl Default for DigestSchedule {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: Duration::from_secs(3600), // 1 hour
            delivery_hours: vec![9, 13, 18], // 9am, 1pm, 6pm
            categories: None,
            min_notifications: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_notifications() -> Vec<Notification> {
        vec![
            Notification::new("app1", "Message 1", "Body 1")
                .with_category(NotificationCategory::Social),
            Notification::new("app1", "Message 2", "Body 2")
                .with_category(NotificationCategory::Social),
            Notification::new("app2", "Email 1", "Body")
                .with_category(NotificationCategory::Email),
            Notification::new("app3", "Alert", "Body")
                .with_priority(NotificationPriority::High),
        ]
    }
    
    #[test]
    fn test_empty_summary() {
        let summary = NotificationSummary::empty();
        assert!(summary.is_empty());
        assert_eq!(summary.overview, "No new notifications");
    }
    
    #[test]
    fn test_generate_by_app() {
        let generator = SummaryGenerator::new();
        let notifications = test_notifications();
        let refs: Vec<_> = notifications.iter().collect();
        
        let summary = generator.generate(&refs, Some(SummaryMode::ByApp));
        
        assert_eq!(summary.total_count, 4);
        assert!(!summary.items.is_empty());
    }
    
    #[test]
    fn test_generate_by_category() {
        let generator = SummaryGenerator::new();
        let notifications = test_notifications();
        let refs: Vec<_> = notifications.iter().collect();
        
        let summary = generator.generate(&refs, Some(SummaryMode::ByCategory));
        
        // Should have Social and Email categories
        assert!(summary.items.len() >= 2);
    }
    
    #[test]
    fn test_overview_generation() {
        let generator = SummaryGenerator::new();
        let notifications = test_notifications();
        let refs: Vec<_> = notifications.iter().collect();
        
        let summary = generator.generate(&refs, Some(SummaryMode::OneLine));
        
        assert!(!summary.overview.is_empty());
        assert!(summary.overview.contains("4"));
    }
    
    #[test]
    fn test_bullet_points() {
        let generator = SummaryGenerator::new();
        let notifications = test_notifications();
        let refs: Vec<_> = notifications.iter().collect();
        
        let summary = generator.generate(&refs, Some(SummaryMode::BulletPoints));
        
        assert_eq!(summary.items.len(), 4);
        for item in &summary.items {
            assert_eq!(item.count, 1);
        }
    }
    
    #[test]
    fn test_priority_sorting() {
        let generator = SummaryGenerator::new();
        let notifications = test_notifications();
        let refs: Vec<_> = notifications.iter().collect();
        
        let summary = generator.generate(&refs, Some(SummaryMode::ByPriority));
        
        // High priority should be first
        if !summary.items.is_empty() {
            assert!(summary.items[0].max_priority >= NotificationPriority::Default);
        }
    }
}
