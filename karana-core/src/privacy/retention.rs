// Data Retention Policies - GDPR-compliant data lifecycle management
// Phase 50: User control over data retention and deletion

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Data category for retention policies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataCategory {
    /// Camera/sensor data
    SensorData,
    
    /// Voice recordings
    VoiceRecordings,
    
    /// Vision/scene captures
    VisionCaptures,
    
    /// Chat/conversation history
    Conversations,
    
    /// Location history
    LocationHistory,
    
    /// App usage data
    UsageData,
    
    /// User preferences
    Preferences,
    
    /// Knowledge graph entries
    KnowledgeEntries,
    
    /// Transaction history
    Transactions,
    
    /// Diagnostic/telemetry data
    Diagnostics,
    
    /// Temporary/cache data
    TemporaryData,
}

impl DataCategory {
    /// Get default retention period
    pub fn default_retention(&self) -> Duration {
        match self {
            DataCategory::SensorData => Duration::from_secs(7 * 24 * 3600), // 7 days
            DataCategory::VoiceRecordings => Duration::from_secs(30 * 24 * 3600), // 30 days
            DataCategory::VisionCaptures => Duration::from_secs(7 * 24 * 3600), // 7 days
            DataCategory::Conversations => Duration::from_secs(90 * 24 * 3600), // 90 days
            DataCategory::LocationHistory => Duration::from_secs(30 * 24 * 3600), // 30 days
            DataCategory::UsageData => Duration::from_secs(365 * 24 * 3600), // 1 year
            DataCategory::Preferences => Duration::from_secs(u64::MAX), // Forever (until deleted)
            DataCategory::KnowledgeEntries => Duration::from_secs(180 * 24 * 3600), // 6 months
            DataCategory::Transactions => Duration::from_secs(365 * 7 * 24 * 3600), // 7 years (legal)
            DataCategory::Diagnostics => Duration::from_secs(30 * 24 * 3600), // 30 days
            DataCategory::TemporaryData => Duration::from_secs(24 * 3600), // 24 hours
        }
    }
    
    /// Check if category is legally protected (cannot be auto-deleted)
    pub fn is_protected(&self) -> bool {
        matches!(self, DataCategory::Transactions)
    }
    
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            DataCategory::SensorData => "Sensor Data",
            DataCategory::VoiceRecordings => "Voice Recordings",
            DataCategory::VisionCaptures => "Vision Captures",
            DataCategory::Conversations => "Conversations",
            DataCategory::LocationHistory => "Location History",
            DataCategory::UsageData => "Usage Data",
            DataCategory::Preferences => "Preferences",
            DataCategory::KnowledgeEntries => "Knowledge Entries",
            DataCategory::Transactions => "Transactions",
            DataCategory::Diagnostics => "Diagnostics",
            DataCategory::TemporaryData => "Temporary Data",
        }
    }
}

/// Retention policy for a data category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub category: DataCategory,
    pub retention_period: Duration,
    pub auto_delete: bool,
    pub user_modified: bool,
    pub last_cleanup: Option<SystemTime>,
}

impl RetentionPolicy {
    /// Create default policy for a category
    pub fn default_for(category: DataCategory) -> Self {
        Self {
            retention_period: category.default_retention(),
            category,
            auto_delete: true,
            user_modified: false,
            last_cleanup: None,
        }
    }
    
    /// Check if data from timestamp should be deleted
    pub fn should_delete(&self, data_timestamp: SystemTime) -> bool {
        if !self.auto_delete || self.category.is_protected() {
            return false;
        }
        
        if let Ok(age) = SystemTime::now().duration_since(data_timestamp) {
            age > self.retention_period
        } else {
            false
        }
    }
}

/// Data item tracked for retention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataItem {
    pub id: String,
    pub category: DataCategory,
    pub created_at: SystemTime,
    pub size_bytes: u64,
    pub metadata: HashMap<String, String>,
}

/// Retention statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetentionStats {
    pub total_items_tracked: u64,
    pub total_size_bytes: u64,
    pub items_deleted: u64,
    pub bytes_freed: u64,
    pub last_cleanup_time: Option<SystemTime>,
    pub categories_managed: usize,
}

/// Data retention manager
pub struct DataRetentionManager {
    policies: Arc<RwLock<HashMap<DataCategory, RetentionPolicy>>>,
    tracked_items: Arc<RwLock<HashMap<String, DataItem>>>,
    stats: Arc<RwLock<RetentionStats>>,
    running: Arc<RwLock<bool>>,
}

impl DataRetentionManager {
    /// Create new retention manager
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            tracked_items: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(RetentionStats::default())),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Initialize with default policies
    pub async fn initialize_defaults(&self) {
        let categories = vec![
            DataCategory::SensorData,
            DataCategory::VoiceRecordings,
            DataCategory::VisionCaptures,
            DataCategory::Conversations,
            DataCategory::LocationHistory,
            DataCategory::UsageData,
            DataCategory::Preferences,
            DataCategory::KnowledgeEntries,
            DataCategory::Transactions,
            DataCategory::Diagnostics,
            DataCategory::TemporaryData,
        ];
        
        let mut policies = self.policies.write().await;
        for category in categories {
            policies.insert(category.clone(), RetentionPolicy::default_for(category));
        }
        
        let mut stats = self.stats.write().await;
        stats.categories_managed = policies.len();
    }
    
    /// Start automatic cleanup
    pub async fn start(&self, cleanup_interval: Duration) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);
        
        let manager = self.clone();
        tokio::spawn(async move {
            manager.cleanup_loop(cleanup_interval).await;
        });
        
        Ok(())
    }
    
    /// Stop automatic cleanup
    pub async fn stop(&self) {
        *self.running.write().await = false;
    }
    
    /// Main cleanup loop
    async fn cleanup_loop(&self, interval: Duration) {
        while *self.running.read().await {
            let _ = self.run_cleanup().await;
            tokio::time::sleep(interval).await;
        }
    }
    
    /// Run cleanup for all categories
    pub async fn run_cleanup(&self) -> Result<()> {
        let policies = self.policies.read().await.clone();
        let mut items = self.tracked_items.write().await;
        let mut stats = self.stats.write().await;
        
        let mut deleted_count = 0u64;
        let mut bytes_freed = 0u64;
        
        // Find items to delete
        let to_delete: Vec<String> = items
            .iter()
            .filter(|(_, item)| {
                if let Some(policy) = policies.get(&item.category) {
                    policy.should_delete(item.created_at)
                } else {
                    false
                }
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        // Delete items
        for id in to_delete {
            if let Some(item) = items.remove(&id) {
                deleted_count += 1;
                bytes_freed += item.size_bytes;
            }
        }
        
        // Update statistics
        stats.items_deleted += deleted_count;
        stats.bytes_freed += bytes_freed;
        stats.last_cleanup_time = Some(SystemTime::now());
        stats.total_items_tracked = items.len() as u64;
        stats.total_size_bytes = items.values().map(|i| i.size_bytes).sum();
        
        // Update policy cleanup times
        drop(items);
        drop(stats);
        
        let mut policies_write = self.policies.write().await;
        for policy in policies_write.values_mut() {
            policy.last_cleanup = Some(SystemTime::now());
        }
        
        Ok(())
    }
    
    /// Set retention policy for a category
    pub async fn set_policy(&self, category: DataCategory, retention_period: Duration, auto_delete: bool) -> Result<()> {
        if category.is_protected() && auto_delete {
            return Err(anyhow!("Cannot enable auto-delete for protected category: {:?}", category));
        }
        
        let mut policies = self.policies.write().await;
        let policy = policies.entry(category.clone()).or_insert_with(|| RetentionPolicy::default_for(category));
        
        policy.retention_period = retention_period;
        policy.auto_delete = auto_delete;
        policy.user_modified = true;
        
        Ok(())
    }
    
    /// Get retention policy for a category
    pub async fn get_policy(&self, category: &DataCategory) -> Option<RetentionPolicy> {
        self.policies.read().await.get(category).cloned()
    }
    
    /// Track a data item
    pub async fn track_item(&self, item: DataItem) {
        let mut items = self.tracked_items.write().await;
        items.insert(item.id.clone(), item);
        
        let mut stats = self.stats.write().await;
        stats.total_items_tracked = items.len() as u64;
        stats.total_size_bytes = items.values().map(|i| i.size_bytes).sum();
    }
    
    /// Untrack a data item (when manually deleted)
    pub async fn untrack_item(&self, id: &str) -> Option<DataItem> {
        let mut items = self.tracked_items.write().await;
        let item = items.remove(id);
        
        if item.is_some() {
            let mut stats = self.stats.write().await;
            stats.total_items_tracked = items.len() as u64;
            stats.total_size_bytes = items.values().map(|i| i.size_bytes).sum();
        }
        
        item
    }
    
    /// Get items for a category
    pub async fn get_items_for_category(&self, category: &DataCategory) -> Vec<DataItem> {
        self.tracked_items.read().await
            .values()
            .filter(|item| &item.category == category)
            .cloned()
            .collect()
    }
    
    /// Delete all items in a category
    pub async fn delete_category(&self, category: &DataCategory) -> Result<u64> {
        let mut items = self.tracked_items.write().await;
        
        let to_delete: Vec<String> = items
            .iter()
            .filter(|(_, item)| &item.category == category)
            .map(|(id, _)| id.clone())
            .collect();
        
        let count = to_delete.len() as u64;
        
        for id in to_delete {
            items.remove(&id);
        }
        
        let mut stats = self.stats.write().await;
        stats.items_deleted += count;
        stats.total_items_tracked = items.len() as u64;
        stats.total_size_bytes = items.values().map(|i| i.size_bytes).sum();
        
        Ok(count)
    }
    
    /// Get statistics
    pub async fn get_stats(&self) -> RetentionStats {
        self.stats.read().await.clone()
    }
    
    /// Export all retention policies
    pub async fn export_policies(&self) -> HashMap<DataCategory, RetentionPolicy> {
        self.policies.read().await.clone()
    }
    
    /// Import retention policies
    pub async fn import_policies(&self, policies: HashMap<DataCategory, RetentionPolicy>) -> Result<()> {
        // Validate policies
        for (category, policy) in &policies {
            if category.is_protected() && policy.auto_delete {
                return Err(anyhow!("Cannot enable auto-delete for protected category: {:?}", category));
            }
        }
        
        *self.policies.write().await = policies;
        Ok(())
    }
}

impl Clone for DataRetentionManager {
    fn clone(&self) -> Self {
        Self {
            policies: Arc::clone(&self.policies),
            tracked_items: Arc::clone(&self.tracked_items),
            stats: Arc::clone(&self.stats),
            running: Arc::clone(&self.running),
        }
    }
}

impl Default for DataRetentionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_retention_manager_creation() {
        let manager = DataRetentionManager::new();
        manager.initialize_defaults().await;
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.categories_managed, 11);
    }
    
    #[tokio::test]
    async fn test_default_policies() {
        let manager = DataRetentionManager::new();
        manager.initialize_defaults().await;
        
        let policy = manager.get_policy(&DataCategory::SensorData).await.unwrap();
        assert_eq!(policy.retention_period, Duration::from_secs(7 * 24 * 3600));
        assert!(policy.auto_delete);
    }
    
    #[tokio::test]
    async fn test_track_items() {
        let manager = DataRetentionManager::new();
        manager.initialize_defaults().await;
        
        let item = DataItem {
            id: "test-1".to_string(),
            category: DataCategory::SensorData,
            created_at: SystemTime::now(),
            size_bytes: 1024,
            metadata: HashMap::new(),
        };
        
        manager.track_item(item).await;
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_items_tracked, 1);
        assert_eq!(stats.total_size_bytes, 1024);
    }
    
    #[tokio::test]
    async fn test_cleanup_old_items() {
        let manager = DataRetentionManager::new();
        manager.initialize_defaults().await;
        
        // Add old item
        let old_item = DataItem {
            id: "old-1".to_string(),
            category: DataCategory::TemporaryData,
            created_at: SystemTime::now() - Duration::from_secs(48 * 3600), // 48 hours ago
            size_bytes: 2048,
            metadata: HashMap::new(),
        };
        
        manager.track_item(old_item).await;
        
        // Run cleanup
        manager.run_cleanup().await.unwrap();
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.items_deleted, 1);
        assert_eq!(stats.bytes_freed, 2048);
    }
    
    #[tokio::test]
    async fn test_protected_category() {
        let manager = DataRetentionManager::new();
        manager.initialize_defaults().await;
        
        // Try to enable auto-delete for transactions (protected)
        let result = manager.set_policy(
            DataCategory::Transactions,
            Duration::from_secs(30 * 24 * 3600),
            true
        ).await;
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_delete_category() {
        let manager = DataRetentionManager::new();
        manager.initialize_defaults().await;
        
        // Add items in different categories
        for i in 0..5 {
            let item = DataItem {
                id: format!("sensor-{}", i),
                category: DataCategory::SensorData,
                created_at: SystemTime::now(),
                size_bytes: 100,
                metadata: HashMap::new(),
            };
            manager.track_item(item).await;
        }
        
        for i in 0..3 {
            let item = DataItem {
                id: format!("voice-{}", i),
                category: DataCategory::VoiceRecordings,
                created_at: SystemTime::now(),
                size_bytes: 200,
                metadata: HashMap::new(),
            };
            manager.track_item(item).await;
        }
        
        // Delete sensor data
        let deleted = manager.delete_category(&DataCategory::SensorData).await.unwrap();
        assert_eq!(deleted, 5);
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_items_tracked, 3);
    }
    
    #[tokio::test]
    async fn test_policy_export_import() {
        let manager = DataRetentionManager::new();
        manager.initialize_defaults().await;
        
        // Modify a non-protected policy
        manager.set_policy(
            DataCategory::VoiceRecordings,
            Duration::from_secs(60 * 24 * 3600),
            false
        ).await.unwrap();
        
        // Export policies
        let mut policies = manager.export_policies().await;
        
        // Remove or fix protected categories for import
        if let Some(policy) = policies.get_mut(&DataCategory::Transactions) {
            policy.auto_delete = false; // Disable auto-delete for protected category
        }
        
        // Create new manager and import
        let manager2 = DataRetentionManager::new();
        manager2.import_policies(policies).await.unwrap();
        
        let policy = manager2.get_policy(&DataCategory::VoiceRecordings).await.unwrap();
        assert_eq!(policy.retention_period, Duration::from_secs(60 * 24 * 3600));
        assert!(!policy.auto_delete);
    }
}
