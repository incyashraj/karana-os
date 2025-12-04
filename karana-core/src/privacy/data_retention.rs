//! Data Retention Policy System for Kāraṇa OS AR Glasses
//!
//! Manages automatic data cleanup based on policies to protect privacy
//! and comply with regulations.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Data category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataCategory {
    /// System logs
    SystemLogs,
    /// App logs
    AppLogs,
    /// Location history
    LocationHistory,
    /// Gaze/eye tracking data
    GazeHistory,
    /// Voice recordings
    VoiceRecordings,
    /// Photos
    Photos,
    /// Videos
    Videos,
    /// Screenshots
    Screenshots,
    /// Search history
    SearchHistory,
    /// App usage data
    AppUsage,
    /// Biometric templates
    BiometricData,
    /// Crash reports
    CrashReports,
    /// Analytics data
    Analytics,
    /// Cache data
    Cache,
    /// Temporary files
    Temporary,
    /// Health data
    HealthData,
    /// Spatial maps
    SpatialMaps,
    /// Message history
    Messages,
    /// Contact interaction
    ContactInteraction,
}

impl DataCategory {
    /// Get description
    pub fn description(&self) -> &str {
        match self {
            DataCategory::SystemLogs => "System operation logs",
            DataCategory::AppLogs => "Application logs",
            DataCategory::LocationHistory => "Location and movement history",
            DataCategory::GazeHistory => "Eye tracking and gaze data",
            DataCategory::VoiceRecordings => "Voice command recordings",
            DataCategory::Photos => "Captured photos",
            DataCategory::Videos => "Recorded videos",
            DataCategory::Screenshots => "Captured screenshots",
            DataCategory::SearchHistory => "Search queries",
            DataCategory::AppUsage => "App usage statistics",
            DataCategory::BiometricData => "Biometric templates",
            DataCategory::CrashReports => "Crash and error reports",
            DataCategory::Analytics => "Analytics and telemetry",
            DataCategory::Cache => "Cached data",
            DataCategory::Temporary => "Temporary files",
            DataCategory::HealthData => "Health and wellness data",
            DataCategory::SpatialMaps => "Environment spatial maps",
            DataCategory::Messages => "Message history",
            DataCategory::ContactInteraction => "Contact interaction data",
        }
    }
    
    /// Default retention period
    pub fn default_retention(&self) -> Duration {
        match self {
            DataCategory::Temporary => Duration::from_secs(3600), // 1 hour
            DataCategory::Cache => Duration::from_secs(24 * 3600), // 1 day
            DataCategory::SystemLogs => Duration::from_secs(7 * 24 * 3600), // 7 days
            DataCategory::AppLogs => Duration::from_secs(7 * 24 * 3600),
            DataCategory::GazeHistory => Duration::from_secs(24 * 3600), // 1 day
            DataCategory::SearchHistory => Duration::from_secs(30 * 24 * 3600), // 30 days
            DataCategory::LocationHistory => Duration::from_secs(30 * 24 * 3600),
            DataCategory::AppUsage => Duration::from_secs(90 * 24 * 3600), // 90 days
            DataCategory::VoiceRecordings => Duration::from_secs(7 * 24 * 3600),
            DataCategory::Analytics => Duration::from_secs(90 * 24 * 3600),
            DataCategory::CrashReports => Duration::from_secs(365 * 24 * 3600), // 1 year
            DataCategory::Photos => Duration::from_secs(0), // No auto-delete
            DataCategory::Videos => Duration::from_secs(0),
            DataCategory::Screenshots => Duration::from_secs(90 * 24 * 3600),
            DataCategory::BiometricData => Duration::from_secs(0), // No auto-delete
            DataCategory::HealthData => Duration::from_secs(365 * 24 * 3600),
            DataCategory::SpatialMaps => Duration::from_secs(30 * 24 * 3600),
            DataCategory::Messages => Duration::from_secs(0), // No auto-delete
            DataCategory::ContactInteraction => Duration::from_secs(90 * 24 * 3600),
        }
    }
    
    /// Is this sensitive data?
    pub fn is_sensitive(&self) -> bool {
        matches!(
            self,
            DataCategory::LocationHistory |
            DataCategory::GazeHistory |
            DataCategory::VoiceRecordings |
            DataCategory::BiometricData |
            DataCategory::HealthData |
            DataCategory::Messages
        )
    }
    
    /// Requires user consent to collect
    pub fn requires_consent(&self) -> bool {
        matches!(
            self,
            DataCategory::LocationHistory |
            DataCategory::GazeHistory |
            DataCategory::VoiceRecordings |
            DataCategory::Analytics |
            DataCategory::AppUsage |
            DataCategory::ContactInteraction
        )
    }
}

/// Data retention policy
#[derive(Debug, Clone)]
pub struct DataRetentionPolicy {
    /// Data category
    pub category: DataCategory,
    /// Retention duration (0 = keep forever)
    pub retention: Duration,
    /// Auto-delete enabled
    pub auto_delete: bool,
    /// Secure delete (overwrite)
    pub secure_delete: bool,
    /// Max storage size (bytes, 0 = unlimited)
    pub max_storage: u64,
    /// Exclude from backups
    pub exclude_backup: bool,
    /// Require confirmation before delete
    pub confirm_delete: bool,
}

impl DataRetentionPolicy {
    /// Create default policy for category
    pub fn default_for(category: DataCategory) -> Self {
        let retention = category.default_retention();
        Self {
            category,
            retention,
            auto_delete: retention.as_secs() > 0,
            secure_delete: category.is_sensitive(),
            max_storage: 0,
            exclude_backup: matches!(category, DataCategory::Cache | DataCategory::Temporary),
            confirm_delete: matches!(category, DataCategory::Photos | DataCategory::Videos | DataCategory::Messages),
        }
    }
}

/// Data item record
#[derive(Debug, Clone)]
pub struct DataRecord {
    /// Record ID
    pub id: String,
    /// Category
    pub category: DataCategory,
    /// Size in bytes
    pub size: u64,
    /// Creation time
    pub created_at: Instant,
    /// Last accessed
    pub last_accessed: Option<Instant>,
    /// Is marked for deletion
    pub marked_for_deletion: bool,
    /// Deletion scheduled at
    pub delete_at: Option<Instant>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    /// Check if record is expired based on policy
    pub fn is_expired(&self, policy: &DataRetentionPolicy) -> bool {
        if policy.retention.as_secs() == 0 {
            return false; // Keep forever
        }
        
        let age = Instant::now().duration_since(self.created_at);
        age >= policy.retention
    }
}

/// Deletion result
#[derive(Debug, Clone)]
pub struct DeletionResult {
    /// Records deleted
    pub records_deleted: usize,
    /// Bytes freed
    pub bytes_freed: u64,
    /// Records failed to delete
    pub failed: usize,
    /// Categories processed
    pub categories_processed: Vec<DataCategory>,
    /// Duration taken
    pub duration: Duration,
}

/// Retention manager
#[derive(Debug)]
pub struct RetentionManager {
    /// Policies by category
    policies: HashMap<DataCategory, DataRetentionPolicy>,
    /// Data records (simulated storage index)
    records: HashMap<String, DataRecord>,
    /// Last cleanup time
    last_cleanup: Option<Instant>,
    /// Cleanup interval
    cleanup_interval: Duration,
    /// Total deletions performed
    total_deletions: u64,
    /// Total bytes freed
    total_bytes_freed: u64,
    /// Callbacks pending (simulated)
    pending_confirmations: Vec<String>,
}

impl RetentionManager {
    /// Create new retention manager
    pub fn new() -> Self {
        let mut manager = Self {
            policies: HashMap::new(),
            records: HashMap::new(),
            last_cleanup: None,
            cleanup_interval: Duration::from_secs(3600), // Hourly
            total_deletions: 0,
            total_bytes_freed: 0,
            pending_confirmations: Vec::new(),
        };
        
        // Initialize default policies
        manager.init_default_policies();
        manager
    }
    
    /// Initialize default policies
    fn init_default_policies(&mut self) {
        let categories = [
            DataCategory::SystemLogs,
            DataCategory::AppLogs,
            DataCategory::LocationHistory,
            DataCategory::GazeHistory,
            DataCategory::VoiceRecordings,
            DataCategory::Photos,
            DataCategory::Videos,
            DataCategory::Screenshots,
            DataCategory::SearchHistory,
            DataCategory::AppUsage,
            DataCategory::BiometricData,
            DataCategory::CrashReports,
            DataCategory::Analytics,
            DataCategory::Cache,
            DataCategory::Temporary,
            DataCategory::HealthData,
            DataCategory::SpatialMaps,
            DataCategory::Messages,
            DataCategory::ContactInteraction,
        ];
        
        for category in categories {
            self.policies.insert(category, DataRetentionPolicy::default_for(category));
        }
    }
    
    /// Set policy for category
    pub fn set_policy(&mut self, policy: DataRetentionPolicy) {
        self.policies.insert(policy.category, policy);
    }
    
    /// Get policy for category
    pub fn get_policy(&self, category: DataCategory) -> Option<&DataRetentionPolicy> {
        self.policies.get(&category)
    }
    
    /// Register data record
    pub fn register_record(
        &mut self,
        id: &str,
        category: DataCategory,
        size: u64,
        metadata: HashMap<String, String>,
    ) {
        let now = Instant::now();
        let delete_at = self.policies.get(&category)
            .filter(|p| p.retention.as_secs() > 0)
            .map(|p| now + p.retention);
        
        let record = DataRecord {
            id: id.to_string(),
            category,
            size,
            created_at: now,
            last_accessed: None,
            marked_for_deletion: false,
            delete_at,
            metadata,
        };
        
        self.records.insert(id.to_string(), record);
    }
    
    /// Mark record as accessed
    pub fn record_access(&mut self, id: &str) {
        if let Some(record) = self.records.get_mut(id) {
            record.last_accessed = Some(Instant::now());
        }
    }
    
    /// Mark record for deletion
    pub fn mark_for_deletion(&mut self, id: &str) {
        if let Some(record) = self.records.get_mut(id) {
            record.marked_for_deletion = true;
        }
    }
    
    /// Run cleanup process
    pub fn run_cleanup(&mut self) -> DeletionResult {
        let start = Instant::now();
        let mut deleted = Vec::new();
        let mut bytes_freed = 0u64;
        let mut categories = Vec::new();
        
        for (id, record) in &self.records {
            if let Some(policy) = self.policies.get(&record.category) {
                // Skip if auto-delete disabled
                if !policy.auto_delete {
                    continue;
                }
                
                // Check if expired or marked
                if record.is_expired(policy) || record.marked_for_deletion {
                    // Check if confirmation needed
                    if policy.confirm_delete && !record.marked_for_deletion {
                        self.pending_confirmations.push(id.clone());
                        continue;
                    }
                    
                    bytes_freed += record.size;
                    deleted.push(id.clone());
                    
                    if !categories.contains(&record.category) {
                        categories.push(record.category);
                    }
                }
            }
        }
        
        // Remove deleted records
        for id in &deleted {
            self.records.remove(id);
        }
        
        self.total_deletions += deleted.len() as u64;
        self.total_bytes_freed += bytes_freed;
        self.last_cleanup = Some(Instant::now());
        
        DeletionResult {
            records_deleted: deleted.len(),
            bytes_freed,
            failed: 0,
            categories_processed: categories,
            duration: start.elapsed(),
        }
    }
    
    /// Delete all data for category
    pub fn delete_category(&mut self, category: DataCategory) -> DeletionResult {
        let start = Instant::now();
        let mut deleted = Vec::new();
        let mut bytes_freed = 0u64;
        
        for (id, record) in &self.records {
            if record.category == category {
                bytes_freed += record.size;
                deleted.push(id.clone());
            }
        }
        
        for id in &deleted {
            self.records.remove(id);
        }
        
        self.total_deletions += deleted.len() as u64;
        self.total_bytes_freed += bytes_freed;
        
        DeletionResult {
            records_deleted: deleted.len(),
            bytes_freed,
            failed: 0,
            categories_processed: vec![category],
            duration: start.elapsed(),
        }
    }
    
    /// Delete all user data (factory reset)
    pub fn delete_all_user_data(&mut self) -> DeletionResult {
        let start = Instant::now();
        let count = self.records.len();
        let bytes: u64 = self.records.values().map(|r| r.size).sum();
        
        let categories: Vec<DataCategory> = self.records.values()
            .map(|r| r.category)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        self.records.clear();
        self.total_deletions += count as u64;
        self.total_bytes_freed += bytes;
        
        DeletionResult {
            records_deleted: count,
            bytes_freed: bytes,
            failed: 0,
            categories_processed: categories,
            duration: start.elapsed(),
        }
    }
    
    /// Get storage usage by category
    pub fn storage_by_category(&self) -> HashMap<DataCategory, u64> {
        let mut usage = HashMap::new();
        
        for record in self.records.values() {
            *usage.entry(record.category).or_insert(0) += record.size;
        }
        
        usage
    }
    
    /// Get total storage used
    pub fn total_storage(&self) -> u64 {
        self.records.values().map(|r| r.size).sum()
    }
    
    /// Get record count
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
    
    /// Get records for category
    pub fn records_for_category(&self, category: DataCategory) -> Vec<&DataRecord> {
        self.records.values()
            .filter(|r| r.category == category)
            .collect()
    }
    
    /// Check if cleanup is due
    pub fn is_cleanup_due(&self) -> bool {
        match self.last_cleanup {
            Some(last) => Instant::now().duration_since(last) >= self.cleanup_interval,
            None => true,
        }
    }
    
    /// Get pending confirmations
    pub fn pending_confirmations(&self) -> &[String] {
        &self.pending_confirmations
    }
    
    /// Confirm deletion
    pub fn confirm_deletion(&mut self, id: &str) {
        self.pending_confirmations.retain(|i| i != id);
        self.mark_for_deletion(id);
    }
    
    /// Get statistics
    pub fn stats(&self) -> RetentionStats {
        RetentionStats {
            total_records: self.records.len(),
            total_storage: self.total_storage(),
            total_deletions: self.total_deletions,
            total_bytes_freed: self.total_bytes_freed,
            policies_defined: self.policies.len(),
            pending_confirmations: self.pending_confirmations.len(),
            last_cleanup: self.last_cleanup,
        }
    }
}

impl Default for RetentionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Retention statistics
#[derive(Debug, Clone)]
pub struct RetentionStats {
    /// Total records tracked
    pub total_records: usize,
    /// Total storage used
    pub total_storage: u64,
    /// Total deletions performed
    pub total_deletions: u64,
    /// Total bytes freed
    pub total_bytes_freed: u64,
    /// Policies defined
    pub policies_defined: usize,
    /// Pending deletion confirmations
    pub pending_confirmations: usize,
    /// Last cleanup time
    pub last_cleanup: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_retention_manager_creation() {
        let rm = RetentionManager::new();
        assert!(rm.policies.len() > 0);
    }
    
    #[test]
    fn test_register_record() {
        let mut rm = RetentionManager::new();
        
        rm.register_record(
            "rec1",
            DataCategory::Cache,
            1024,
            HashMap::new(),
        );
        
        assert_eq!(rm.record_count(), 1);
        assert_eq!(rm.total_storage(), 1024);
    }
    
    #[test]
    fn test_category_defaults() {
        assert!(DataCategory::Cache.default_retention().as_secs() > 0);
        assert_eq!(DataCategory::Photos.default_retention().as_secs(), 0); // Keep forever
    }
    
    #[test]
    fn test_sensitive_categories() {
        assert!(DataCategory::BiometricData.is_sensitive());
        assert!(DataCategory::LocationHistory.is_sensitive());
        assert!(!DataCategory::Cache.is_sensitive());
    }
    
    #[test]
    fn test_storage_by_category() {
        let mut rm = RetentionManager::new();
        
        rm.register_record("rec1", DataCategory::Cache, 1024, HashMap::new());
        rm.register_record("rec2", DataCategory::Cache, 2048, HashMap::new());
        rm.register_record("rec3", DataCategory::Photos, 5000, HashMap::new());
        
        let usage = rm.storage_by_category();
        assert_eq!(*usage.get(&DataCategory::Cache).unwrap(), 3072);
        assert_eq!(*usage.get(&DataCategory::Photos).unwrap(), 5000);
    }
    
    #[test]
    fn test_delete_category() {
        let mut rm = RetentionManager::new();
        
        rm.register_record("rec1", DataCategory::Cache, 1024, HashMap::new());
        rm.register_record("rec2", DataCategory::Cache, 2048, HashMap::new());
        rm.register_record("rec3", DataCategory::Photos, 5000, HashMap::new());
        
        let result = rm.delete_category(DataCategory::Cache);
        
        assert_eq!(result.records_deleted, 2);
        assert_eq!(result.bytes_freed, 3072);
        assert_eq!(rm.record_count(), 1);
    }
    
    #[test]
    fn test_mark_for_deletion() {
        let mut rm = RetentionManager::new();
        
        rm.register_record("rec1", DataCategory::SystemLogs, 1024, HashMap::new());
        rm.mark_for_deletion("rec1");
        
        let records = rm.records_for_category(DataCategory::SystemLogs);
        assert!(records[0].marked_for_deletion);
    }
    
    #[test]
    fn test_delete_all_user_data() {
        let mut rm = RetentionManager::new();
        
        rm.register_record("rec1", DataCategory::Cache, 1024, HashMap::new());
        rm.register_record("rec2", DataCategory::Photos, 5000, HashMap::new());
        
        let result = rm.delete_all_user_data();
        
        assert_eq!(result.records_deleted, 2);
        assert_eq!(rm.record_count(), 0);
    }
    
    #[test]
    fn test_policy_customization() {
        let mut rm = RetentionManager::new();
        
        let custom_policy = DataRetentionPolicy {
            category: DataCategory::Cache,
            retention: Duration::from_secs(3600),
            auto_delete: true,
            secure_delete: true,
            max_storage: 100_000_000,
            exclude_backup: true,
            confirm_delete: false,
        };
        
        rm.set_policy(custom_policy.clone());
        
        let policy = rm.get_policy(DataCategory::Cache).unwrap();
        assert_eq!(policy.max_storage, 100_000_000);
    }
    
    #[test]
    fn test_retention_stats() {
        let mut rm = RetentionManager::new();
        
        rm.register_record("rec1", DataCategory::Cache, 1024, HashMap::new());
        
        let stats = rm.stats();
        assert_eq!(stats.total_records, 1);
        assert_eq!(stats.total_storage, 1024);
    }
}
