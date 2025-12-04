//! Data synchronization manager

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Sync direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    /// Upload to cloud
    Upload,
    /// Download from cloud
    Download,
    /// Bidirectional
    Bidirectional,
}

/// Sync priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SyncPriority {
    /// Low priority (background)
    Low,
    /// Normal priority
    Normal,
    /// High priority (user-initiated)
    High,
    /// Critical (settings, auth)
    Critical,
}

/// Sync operation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncOperationStatus {
    /// Queued
    Queued,
    /// In progress
    InProgress,
    /// Completed
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

/// Sync operation
#[derive(Debug, Clone)]
pub struct SyncOperation {
    /// Unique operation ID
    pub id: String,
    /// Resource path/identifier
    pub resource: String,
    /// Direction
    pub direction: SyncDirection,
    /// Priority
    pub priority: SyncPriority,
    /// Status
    pub status: SyncOperationStatus,
    /// Data size in bytes
    pub size_bytes: u64,
    /// Bytes transferred
    pub bytes_transferred: u64,
    /// Retry count
    pub retries: u32,
    /// Max retries
    pub max_retries: u32,
    /// Created time
    pub created: Instant,
    /// Error message if failed
    pub error: Option<String>,
}

impl SyncOperation {
    /// Create new sync operation
    pub fn new(id: &str, resource: &str, direction: SyncDirection, size: u64) -> Self {
        Self {
            id: id.to_string(),
            resource: resource.to_string(),
            direction,
            priority: SyncPriority::Normal,
            status: SyncOperationStatus::Queued,
            size_bytes: size,
            bytes_transferred: 0,
            retries: 0,
            max_retries: 3,
            created: Instant::now(),
            error: None,
        }
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: SyncPriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Get progress percentage
    pub fn progress(&self) -> f32 {
        if self.size_bytes == 0 {
            return 100.0;
        }
        (self.bytes_transferred as f32 / self.size_bytes as f32) * 100.0
    }
    
    /// Check if complete
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            SyncOperationStatus::Completed | SyncOperationStatus::Cancelled
        )
    }
    
    /// Check if can retry
    pub fn can_retry(&self) -> bool {
        self.status == SyncOperationStatus::Failed && self.retries < self.max_retries
    }
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    /// Local wins
    LocalWins,
    /// Remote wins
    RemoteWins,
    /// Most recent wins
    MostRecent,
    /// Keep both versions
    KeepBoth,
    /// Ask user
    AskUser,
}

/// Data conflict
#[derive(Debug, Clone)]
pub struct SyncConflict {
    /// Resource ID
    pub resource: String,
    /// Local version timestamp
    pub local_timestamp: Instant,
    /// Remote version timestamp
    pub remote_timestamp: Instant,
    /// Conflict detected time
    pub detected: Instant,
    /// Resolution (None if unresolved)
    pub resolution: Option<ConflictStrategy>,
}

/// Sync manager
#[derive(Debug)]
pub struct SyncManager {
    /// Sync enabled
    enabled: bool,
    /// Operation queue
    queue: VecDeque<SyncOperation>,
    /// Completed operations
    completed: Vec<SyncOperation>,
    /// Active operation
    active: Option<String>,
    /// Conflicts
    conflicts: Vec<SyncConflict>,
    /// Default conflict strategy
    default_strategy: ConflictStrategy,
    /// Max concurrent operations
    max_concurrent: usize,
    /// Bandwidth limit (bytes/sec, 0 = unlimited)
    bandwidth_limit: u64,
    /// Total bytes uploaded
    total_uploaded: u64,
    /// Total bytes downloaded
    total_downloaded: u64,
    /// Operations by ID
    operations: HashMap<String, SyncOperation>,
    /// Operation counter
    op_counter: u64,
}

impl SyncManager {
    /// Create new sync manager
    pub fn new() -> Self {
        Self {
            enabled: true,
            queue: VecDeque::new(),
            completed: Vec::new(),
            active: None,
            conflicts: Vec::new(),
            default_strategy: ConflictStrategy::MostRecent,
            max_concurrent: 3,
            bandwidth_limit: 0,
            total_uploaded: 0,
            total_downloaded: 0,
            operations: HashMap::new(),
            op_counter: 0,
        }
    }
    
    /// Enable/disable sync
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Queue a sync operation
    pub fn queue_operation(&mut self, resource: &str, direction: SyncDirection, size: u64) -> String {
        self.op_counter += 1;
        let id = format!("sync_{}", self.op_counter);
        
        let op = SyncOperation::new(&id, resource, direction, size);
        self.queue.push_back(op.clone());
        self.operations.insert(id.clone(), op);
        
        id
    }
    
    /// Queue high priority operation
    pub fn queue_priority(&mut self, resource: &str, direction: SyncDirection, size: u64, priority: SyncPriority) -> String {
        self.op_counter += 1;
        let id = format!("sync_{}", self.op_counter);
        
        let op = SyncOperation::new(&id, resource, direction, size).with_priority(priority);
        
        // Insert based on priority
        let insert_pos = self.queue
            .iter()
            .position(|o| o.priority < priority)
            .unwrap_or(self.queue.len());
        
        self.queue.insert(insert_pos, op.clone());
        self.operations.insert(id.clone(), op);
        
        id
    }
    
    /// Get operation status
    pub fn get_operation(&self, id: &str) -> Option<&SyncOperation> {
        self.operations.get(id)
    }
    
    /// Cancel operation
    pub fn cancel_operation(&mut self, id: &str) -> bool {
        if let Some(op) = self.operations.get_mut(id) {
            if !op.is_complete() {
                op.status = SyncOperationStatus::Cancelled;
                self.queue.retain(|o| o.id != id);
                return true;
            }
        }
        false
    }
    
    /// Process next operation
    pub fn process_next(&mut self) -> Option<String> {
        if !self.enabled {
            return None;
        }
        
        if let Some(mut op) = self.queue.pop_front() {
            op.status = SyncOperationStatus::InProgress;
            let id = op.id.clone();
            self.active = Some(id.clone());
            self.operations.insert(id.clone(), op);
            Some(id)
        } else {
            None
        }
    }
    
    /// Complete current operation
    pub fn complete_operation(&mut self, id: &str, success: bool, error: Option<&str>) {
        if let Some(op) = self.operations.get_mut(id) {
            if success {
                op.status = SyncOperationStatus::Completed;
                op.bytes_transferred = op.size_bytes;
                
                match op.direction {
                    SyncDirection::Upload => self.total_uploaded += op.size_bytes,
                    SyncDirection::Download => self.total_downloaded += op.size_bytes,
                    SyncDirection::Bidirectional => {
                        self.total_uploaded += op.size_bytes / 2;
                        self.total_downloaded += op.size_bytes / 2;
                    }
                }
            } else {
                op.status = SyncOperationStatus::Failed;
                op.error = error.map(|s| s.to_string());
            }
            
            self.completed.push(op.clone());
            
            if self.active.as_deref() == Some(id) {
                self.active = None;
            }
        }
    }
    
    /// Retry failed operation
    pub fn retry_operation(&mut self, id: &str) -> bool {
        if let Some(op) = self.operations.get_mut(id) {
            if op.can_retry() {
                op.retries += 1;
                op.status = SyncOperationStatus::Queued;
                op.error = None;
                self.queue.push_back(op.clone());
                return true;
            }
        }
        false
    }
    
    /// Report conflict
    pub fn report_conflict(&mut self, resource: &str, local_time: Instant, remote_time: Instant) {
        self.conflicts.push(SyncConflict {
            resource: resource.to_string(),
            local_timestamp: local_time,
            remote_timestamp: remote_time,
            detected: Instant::now(),
            resolution: None,
        });
    }
    
    /// Resolve conflict
    pub fn resolve_conflict(&mut self, resource: &str, strategy: ConflictStrategy) -> bool {
        for conflict in &mut self.conflicts {
            if conflict.resource == resource && conflict.resolution.is_none() {
                conflict.resolution = Some(strategy);
                return true;
            }
        }
        false
    }
    
    /// Get unresolved conflicts
    pub fn unresolved_conflicts(&self) -> Vec<&SyncConflict> {
        self.conflicts
            .iter()
            .filter(|c| c.resolution.is_none())
            .collect()
    }
    
    /// Set default conflict strategy
    pub fn set_conflict_strategy(&mut self, strategy: ConflictStrategy) {
        self.default_strategy = strategy;
    }
    
    /// Get default conflict strategy
    pub fn conflict_strategy(&self) -> ConflictStrategy {
        self.default_strategy
    }
    
    /// Set bandwidth limit
    pub fn set_bandwidth_limit(&mut self, bytes_per_sec: u64) {
        self.bandwidth_limit = bytes_per_sec;
    }
    
    /// Get bandwidth limit
    pub fn bandwidth_limit(&self) -> u64 {
        self.bandwidth_limit
    }
    
    /// Get queue length
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }
    
    /// Get pending count (queued + in progress)
    pub fn pending_count(&self) -> usize {
        self.queue.len() + if self.active.is_some() { 1 } else { 0 }
    }
    
    /// Get total uploaded bytes
    pub fn total_uploaded(&self) -> u64 {
        self.total_uploaded
    }
    
    /// Get total downloaded bytes
    pub fn total_downloaded(&self) -> u64 {
        self.total_downloaded
    }
    
    /// Clear completed operations
    pub fn clear_completed(&mut self) {
        let completed_ids: Vec<_> = self.completed
            .iter()
            .map(|op| op.id.clone())
            .collect();
        
        for id in completed_ids {
            self.operations.remove(&id);
        }
        
        self.completed.clear();
    }
    
    /// Get sync progress (overall)
    pub fn overall_progress(&self) -> f32 {
        let total_size: u64 = self.operations.values().map(|op| op.size_bytes).sum();
        let transferred: u64 = self.operations.values().map(|op| op.bytes_transferred).sum();
        
        if total_size == 0 {
            return 100.0;
        }
        
        (transferred as f32 / total_size as f32) * 100.0
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sync_manager_creation() {
        let manager = SyncManager::new();
        assert!(manager.is_enabled());
        assert_eq!(manager.queue_len(), 0);
    }
    
    #[test]
    fn test_queue_operation() {
        let mut manager = SyncManager::new();
        
        let id = manager.queue_operation("/settings/prefs.json", SyncDirection::Upload, 1024);
        
        assert_eq!(manager.queue_len(), 1);
        
        let op = manager.get_operation(&id).unwrap();
        assert_eq!(op.status, SyncOperationStatus::Queued);
        assert_eq!(op.size_bytes, 1024);
    }
    
    #[test]
    fn test_priority_queue() {
        let mut manager = SyncManager::new();
        
        let low = manager.queue_operation("/file1", SyncDirection::Upload, 100);
        let high = manager.queue_priority("/file2", SyncDirection::Upload, 200, SyncPriority::High);
        
        // High priority should be first
        let next = manager.process_next().unwrap();
        assert_eq!(next, high);
    }
    
    #[test]
    fn test_process_and_complete() {
        let mut manager = SyncManager::new();
        
        let id = manager.queue_operation("/data", SyncDirection::Download, 5000);
        
        let active_id = manager.process_next().unwrap();
        assert_eq!(active_id, id);
        
        manager.complete_operation(&id, true, None);
        
        let op = manager.get_operation(&id).unwrap();
        assert_eq!(op.status, SyncOperationStatus::Completed);
        assert_eq!(manager.total_downloaded(), 5000);
    }
    
    #[test]
    fn test_conflict_handling() {
        let mut manager = SyncManager::new();
        
        let now = Instant::now();
        manager.report_conflict("/settings.json", now, now);
        
        assert_eq!(manager.unresolved_conflicts().len(), 1);
        
        manager.resolve_conflict("/settings.json", ConflictStrategy::LocalWins);
        
        assert_eq!(manager.unresolved_conflicts().len(), 0);
    }
    
    #[test]
    fn test_cancel_operation() {
        let mut manager = SyncManager::new();
        
        let id = manager.queue_operation("/large_file", SyncDirection::Upload, 1_000_000);
        
        assert!(manager.cancel_operation(&id));
        
        let op = manager.get_operation(&id).unwrap();
        assert_eq!(op.status, SyncOperationStatus::Cancelled);
        assert_eq!(manager.queue_len(), 0);
    }
    
    #[test]
    fn test_retry_operation() {
        let mut manager = SyncManager::new();
        
        let id = manager.queue_operation("/file", SyncDirection::Upload, 100);
        manager.process_next();
        manager.complete_operation(&id, false, Some("Network error"));
        
        assert!(manager.retry_operation(&id));
        
        let op = manager.get_operation(&id).unwrap();
        assert_eq!(op.retries, 1);
        assert_eq!(op.status, SyncOperationStatus::Queued);
    }
}
