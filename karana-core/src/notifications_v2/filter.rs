//! Notification Filtering for Kāraṇa OS AR Glasses
//!
//! Rules-based filtering for notification management.

use super::{Notification, NotificationCategory, NotificationPriority};
use std::time::{Duration, Instant};

/// Filter action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterAction {
    /// Allow notification
    Allow,
    /// Block notification
    Block,
    /// Make notification silent
    Silent,
    /// Modify priority
    Modify(NotificationPriority),
}

/// Filter rule types
#[derive(Debug, Clone)]
pub enum FilterRule {
    /// Match by app ID
    AppId(String),
    /// Match by app ID pattern (glob)
    AppPattern(String),
    /// Match by category
    Category(NotificationCategory),
    /// Match by priority range (min, max)
    PriorityRange(NotificationPriority, NotificationPriority),
    /// Match by title contains
    TitleContains(String),
    /// Match by body contains
    BodyContains(String),
    /// Match by keyword in title or body
    Keyword(String),
    /// Match all
    All,
    /// Match by time of day (start_hour, end_hour)
    TimeOfDay(u8, u8),
    /// Match by group key
    GroupKey(String),
    /// Combine rules with AND
    And(Vec<FilterRule>),
    /// Combine rules with OR
    Or(Vec<FilterRule>),
    /// Negate a rule
    Not(Box<FilterRule>),
}

impl FilterRule {
    /// Check if rule matches notification
    pub fn matches(&self, notification: &Notification) -> bool {
        match self {
            FilterRule::AppId(id) => notification.app_id == *id,
            FilterRule::AppPattern(pattern) => {
                Self::glob_match(pattern, &notification.app_id)
            }
            FilterRule::Category(category) => notification.category == *category,
            FilterRule::PriorityRange(min, max) => {
                notification.priority >= *min && notification.priority <= *max
            }
            FilterRule::TitleContains(text) => {
                notification.title.to_lowercase().contains(&text.to_lowercase())
            }
            FilterRule::BodyContains(text) => {
                notification.body.to_lowercase().contains(&text.to_lowercase())
            }
            FilterRule::Keyword(keyword) => {
                let kw_lower = keyword.to_lowercase();
                notification.title.to_lowercase().contains(&kw_lower) ||
                notification.body.to_lowercase().contains(&kw_lower)
            }
            FilterRule::All => true,
            FilterRule::TimeOfDay(start, end) => {
                // Would check current time in real implementation
                true
            }
            FilterRule::GroupKey(key) => {
                notification.group_key.as_deref() == Some(key.as_str())
            }
            FilterRule::And(rules) => rules.iter().all(|r| r.matches(notification)),
            FilterRule::Or(rules) => rules.iter().any(|r| r.matches(notification)),
            FilterRule::Not(rule) => !rule.matches(notification),
        }
    }
    
    /// Simple glob matching (* for any chars)
    fn glob_match(pattern: &str, text: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        if pattern.starts_with('*') && pattern.ends_with('*') {
            let inner = &pattern[1..pattern.len()-1];
            return text.contains(inner);
        }
        
        if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            return text.ends_with(suffix);
        }
        
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len()-1];
            return text.starts_with(prefix);
        }
        
        pattern == text
    }
}

/// Notification filter
#[derive(Debug, Clone)]
pub struct NotificationFilter {
    /// Filter name
    pub name: String,
    /// Filter rule
    pub rule: FilterRule,
    /// Action to take when matched
    pub action: FilterAction,
    /// Is filter enabled
    pub enabled: bool,
    /// Priority (higher = checked first)
    pub priority: i32,
    /// Filter expiration
    pub expires_at: Option<Instant>,
}

impl NotificationFilter {
    /// Create new filter
    pub fn new(name: &str, rule: FilterRule, action: FilterAction) -> Self {
        Self {
            name: name.to_string(),
            rule,
            action,
            enabled: true,
            priority: 0,
            expires_at: None,
        }
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set expiration
    pub fn with_expiration(mut self, duration: Duration) -> Self {
        self.expires_at = Some(Instant::now() + duration);
        self
    }
    
    /// Check notification and return action if matched
    pub fn check(&self, notification: &Notification) -> Option<FilterAction> {
        if !self.enabled {
            return None;
        }
        
        // Check expiration
        if let Some(expires) = self.expires_at {
            if Instant::now() >= expires {
                return None;
            }
        }
        
        if self.rule.matches(notification) {
            Some(self.action)
        } else {
            None
        }
    }
    
    /// Is filter expired
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|e| Instant::now() >= e).unwrap_or(false)
    }
}

/// Pre-built filter templates
impl NotificationFilter {
    /// Block all promotions
    pub fn block_promotions() -> Self {
        Self::new(
            "Block Promotions",
            FilterRule::Category(NotificationCategory::Promotion),
            FilterAction::Block,
        )
    }
    
    /// Silent mode for app
    pub fn silent_app(app_id: &str) -> Self {
        Self::new(
            &format!("Silent {}", app_id),
            FilterRule::AppId(app_id.to_string()),
            FilterAction::Silent,
        )
    }
    
    /// Block keyword
    pub fn block_keyword(keyword: &str) -> Self {
        Self::new(
            &format!("Block '{}'", keyword),
            FilterRule::Keyword(keyword.to_string()),
            FilterAction::Block,
        )
    }
    
    /// Low priority for non-social apps
    pub fn deprioritize_except_social() -> Self {
        Self::new(
            "Deprioritize Non-Social",
            FilterRule::Not(Box::new(FilterRule::Or(vec![
                FilterRule::Category(NotificationCategory::Social),
                FilterRule::Category(NotificationCategory::Call),
            ]))),
            FilterAction::Modify(NotificationPriority::Low),
        )
    }
    
    /// Temporary mute
    pub fn temporary_mute(duration: Duration) -> Self {
        Self::new(
            "Temporary Mute",
            FilterRule::All,
            FilterAction::Silent,
        ).with_expiration(duration)
    }
}

/// Filter manager for rule-based processing
#[derive(Debug)]
pub struct FilterManager {
    /// Active filters
    filters: Vec<NotificationFilter>,
}

impl FilterManager {
    /// Create new filter manager
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }
    
    /// Add filter
    pub fn add(&mut self, filter: NotificationFilter) {
        self.filters.push(filter);
        // Sort by priority (descending)
        self.filters.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    /// Remove filter by name
    pub fn remove(&mut self, name: &str) {
        self.filters.retain(|f| f.name != name);
    }
    
    /// Process notification through filters
    pub fn process(&self, notification: &Notification) -> FilterAction {
        for filter in &self.filters {
            if let Some(action) = filter.check(notification) {
                return action;
            }
        }
        FilterAction::Allow
    }
    
    /// Remove expired filters
    pub fn clean_expired(&mut self) {
        self.filters.retain(|f| !f.is_expired());
    }
    
    /// Get filter count
    pub fn count(&self) -> usize {
        self.filters.len()
    }
    
    /// Get filters
    pub fn filters(&self) -> &[NotificationFilter] {
        &self.filters
    }
}

impl Default for FilterManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_notification(app_id: &str, title: &str) -> Notification {
        Notification::new(app_id, title, "Test body")
    }
    
    #[test]
    fn test_app_id_rule() {
        let rule = FilterRule::AppId("test.app".to_string());
        
        assert!(rule.matches(&test_notification("test.app", "Title")));
        assert!(!rule.matches(&test_notification("other.app", "Title")));
    }
    
    #[test]
    fn test_keyword_rule() {
        let rule = FilterRule::Keyword("urgent".to_string());
        
        let mut notification = test_notification("app", "URGENT: Check this");
        assert!(rule.matches(&notification));
        
        notification.title = "Normal title".to_string();
        notification.body = "This is urgent!".to_string();
        assert!(rule.matches(&notification));
    }
    
    #[test]
    fn test_category_rule() {
        let rule = FilterRule::Category(NotificationCategory::Social);
        
        let notification = Notification::new("app", "Title", "Body")
            .with_category(NotificationCategory::Social);
        assert!(rule.matches(&notification));
    }
    
    #[test]
    fn test_and_rule() {
        let rule = FilterRule::And(vec![
            FilterRule::AppId("social.app".to_string()),
            FilterRule::Keyword("important".to_string()),
        ]);
        
        let mut notification = Notification::new("social.app", "Important update", "Body");
        assert!(rule.matches(&notification));
        
        notification.title = "Regular update".to_string();
        assert!(!rule.matches(&notification));
    }
    
    #[test]
    fn test_or_rule() {
        let rule = FilterRule::Or(vec![
            FilterRule::AppId("app1".to_string()),
            FilterRule::AppId("app2".to_string()),
        ]);
        
        assert!(rule.matches(&test_notification("app1", "Title")));
        assert!(rule.matches(&test_notification("app2", "Title")));
        assert!(!rule.matches(&test_notification("app3", "Title")));
    }
    
    #[test]
    fn test_not_rule() {
        let rule = FilterRule::Not(Box::new(FilterRule::Category(NotificationCategory::Promotion)));
        
        let social = Notification::new("app", "Title", "Body")
            .with_category(NotificationCategory::Social);
        let promo = Notification::new("app", "Title", "Body")
            .with_category(NotificationCategory::Promotion);
        
        assert!(rule.matches(&social));
        assert!(!rule.matches(&promo));
    }
    
    #[test]
    fn test_filter() {
        let filter = NotificationFilter::new(
            "Block test",
            FilterRule::AppId("blocked.app".to_string()),
            FilterAction::Block,
        );
        
        let notification = test_notification("blocked.app", "Title");
        assert_eq!(filter.check(&notification), Some(FilterAction::Block));
    }
    
    #[test]
    fn test_filter_manager() {
        let mut manager = FilterManager::new();
        
        manager.add(NotificationFilter::block_promotions());
        
        let promo = Notification::new("app", "Sale!", "50% off")
            .with_category(NotificationCategory::Promotion);
        
        assert_eq!(manager.process(&promo), FilterAction::Block);
    }
    
    #[test]
    fn test_glob_match() {
        assert!(FilterRule::glob_match("*.social", "com.app.social"));
        assert!(FilterRule::glob_match("com.*", "com.anything"));
        assert!(FilterRule::glob_match("*app*", "my.app.test"));
        assert!(FilterRule::glob_match("*", "anything"));
    }
    
    #[test]
    fn test_filter_priority() {
        let mut manager = FilterManager::new();
        
        manager.add(NotificationFilter::new(
            "Low priority",
            FilterRule::All,
            FilterAction::Allow,
        ).with_priority(0));
        
        manager.add(NotificationFilter::new(
            "High priority",
            FilterRule::All,
            FilterAction::Block,
        ).with_priority(10));
        
        // High priority filter should win
        let notification = test_notification("app", "Title");
        assert_eq!(manager.process(&notification), FilterAction::Block);
    }
}
