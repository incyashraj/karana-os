//! Settings synchronization

use std::collections::HashMap;
use std::time::{Duration, Instant};
use super::schema::SettingValue;

/// Sync status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    /// Not connected
    Disconnected,
    /// Connected and idle
    Idle,
    /// Currently syncing
    Syncing,
    /// Sync completed successfully
    Synced,
    /// Sync failed
    Failed,
    /// Conflict detected
    Conflict,
}

/// Sync direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    /// Upload to cloud
    Upload,
    /// Download from cloud
    Download,
    /// Bidirectional sync
    Bidirectional,
}

/// Sync conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Local wins
    LocalWins,
    /// Remote wins
    RemoteWins,
    /// Newer timestamp wins
    NewerWins,
    /// Manual resolution required
    Manual,
}

/// Sync conflict
#[derive(Debug, Clone)]
pub struct SyncConflict {
    /// Setting key
    pub key: String,
    /// Local value
    pub local_value: SettingValue,
    /// Remote value
    pub remote_value: SettingValue,
    /// Local timestamp
    pub local_time: Instant,
    /// Remote timestamp (as duration since some epoch)
    pub remote_time: Duration,
}

/// Settings sync manager
#[derive(Debug)]
pub struct SettingsSync {
    /// Current sync status
    status: SyncStatus,
    /// Sync direction
    direction: SyncDirection,
    /// Conflict resolution strategy
    conflict_resolution: ConflictResolution,
    /// Last sync time
    last_sync: Option<Instant>,
    /// Auto sync enabled
    auto_sync: bool,
    /// Auto sync interval
    sync_interval: Duration,
    /// Pending conflicts
    conflicts: Vec<SyncConflict>,
    /// Settings to exclude from sync
    excluded_keys: Vec<String>,
    /// Cloud endpoint URL
    endpoint_url: Option<String>,
    /// Authentication token
    auth_token: Option<String>,
    /// Device ID
    device_id: String,
    /// Sync in progress
    syncing: bool,
}

impl SettingsSync {
    /// Create new settings sync
    pub fn new(device_id: &str) -> Self {
        Self {
            status: SyncStatus::Disconnected,
            direction: SyncDirection::Bidirectional,
            conflict_resolution: ConflictResolution::NewerWins,
            last_sync: None,
            auto_sync: false,
            sync_interval: Duration::from_secs(300), // 5 minutes
            conflicts: Vec::new(),
            excluded_keys: vec![
                "privacy.".to_string(), // Don't sync privacy settings
            ],
            endpoint_url: None,
            auth_token: None,
            device_id: device_id.to_string(),
            syncing: false,
        }
    }
    
    /// Configure cloud endpoint
    pub fn configure(&mut self, url: &str, token: &str) {
        self.endpoint_url = Some(url.to_string());
        self.auth_token = Some(token.to_string());
        self.status = SyncStatus::Idle;
    }
    
    /// Disconnect from cloud
    pub fn disconnect(&mut self) {
        self.endpoint_url = None;
        self.auth_token = None;
        self.status = SyncStatus::Disconnected;
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.endpoint_url.is_some() && self.auth_token.is_some()
    }
    
    /// Get current status
    pub fn status(&self) -> SyncStatus {
        self.status
    }
    
    /// Enable/disable auto sync
    pub fn set_auto_sync(&mut self, enabled: bool) {
        self.auto_sync = enabled;
    }
    
    /// Check if auto sync is enabled
    pub fn is_auto_sync(&self) -> bool {
        self.auto_sync
    }
    
    /// Set sync interval
    pub fn set_sync_interval(&mut self, interval: Duration) {
        self.sync_interval = interval;
    }
    
    /// Get sync interval
    pub fn sync_interval(&self) -> Duration {
        self.sync_interval
    }
    
    /// Set sync direction
    pub fn set_direction(&mut self, direction: SyncDirection) {
        self.direction = direction;
    }
    
    /// Get sync direction
    pub fn direction(&self) -> SyncDirection {
        self.direction
    }
    
    /// Set conflict resolution strategy
    pub fn set_conflict_resolution(&mut self, strategy: ConflictResolution) {
        self.conflict_resolution = strategy;
    }
    
    /// Get conflict resolution strategy
    pub fn conflict_resolution(&self) -> ConflictResolution {
        self.conflict_resolution
    }
    
    /// Add key to exclusion list
    pub fn exclude_key(&mut self, key: &str) {
        if !self.excluded_keys.contains(&key.to_string()) {
            self.excluded_keys.push(key.to_string());
        }
    }
    
    /// Remove key from exclusion list
    pub fn include_key(&mut self, key: &str) {
        self.excluded_keys.retain(|k| k != key);
    }
    
    /// Check if key is excluded from sync
    pub fn is_excluded(&self, key: &str) -> bool {
        self.excluded_keys.iter().any(|k| key.starts_with(k))
    }
    
    /// Start sync operation
    pub fn start_sync(&mut self, _local_settings: &HashMap<String, SettingValue>) -> Result<(), String> {
        if !self.is_connected() {
            return Err("Not connected to cloud".to_string());
        }
        
        if self.syncing {
            return Err("Sync already in progress".to_string());
        }
        
        self.syncing = true;
        self.status = SyncStatus::Syncing;
        
        // In real implementation, would:
        // 1. Filter excluded keys
        // 2. Send local settings to cloud
        // 3. Receive remote settings
        // 4. Detect and resolve conflicts
        // 5. Merge changes
        
        // For now, simulate successful sync
        self.syncing = false;
        self.status = SyncStatus::Synced;
        self.last_sync = Some(Instant::now());
        
        Ok(())
    }
    
    /// Get pending conflicts
    pub fn pending_conflicts(&self) -> &[SyncConflict] {
        &self.conflicts
    }
    
    /// Resolve a conflict
    pub fn resolve_conflict(&mut self, key: &str, use_local: bool) -> Option<SettingValue> {
        if let Some(pos) = self.conflicts.iter().position(|c| c.key == key) {
            let conflict = self.conflicts.remove(pos);
            
            if self.conflicts.is_empty() {
                self.status = SyncStatus::Synced;
            }
            
            if use_local {
                Some(conflict.local_value)
            } else {
                Some(conflict.remote_value)
            }
        } else {
            None
        }
    }
    
    /// Check if sync is needed
    pub fn needs_sync(&self) -> bool {
        if !self.auto_sync || !self.is_connected() {
            return false;
        }
        
        match self.last_sync {
            Some(last) => last.elapsed() >= self.sync_interval,
            None => true,
        }
    }
    
    /// Get last sync time
    pub fn last_sync(&self) -> Option<Instant> {
        self.last_sync
    }
    
    /// Get time since last sync
    pub fn time_since_sync(&self) -> Option<Duration> {
        self.last_sync.map(|t| t.elapsed())
    }
}

impl Default for SettingsSync {
    fn default() -> Self {
        Self::new("default-device")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sync_creation() {
        let sync = SettingsSync::new("device-123");
        
        assert_eq!(sync.status(), SyncStatus::Disconnected);
        assert!(!sync.is_connected());
    }
    
    #[test]
    fn test_configure() {
        let mut sync = SettingsSync::new("device-123");
        
        sync.configure("https://sync.example.com", "token123");
        
        assert!(sync.is_connected());
        assert_eq!(sync.status(), SyncStatus::Idle);
    }
    
    #[test]
    fn test_disconnect() {
        let mut sync = SettingsSync::new("device-123");
        sync.configure("https://sync.example.com", "token123");
        
        sync.disconnect();
        
        assert!(!sync.is_connected());
        assert_eq!(sync.status(), SyncStatus::Disconnected);
    }
    
    #[test]
    fn test_exclusion() {
        let mut sync = SettingsSync::new("device-123");
        
        // Privacy keys are excluded by default
        assert!(sync.is_excluded("privacy.location"));
        
        // Other keys are not
        assert!(!sync.is_excluded("display.brightness"));
        
        // Can add new exclusions
        sync.exclude_key("custom.");
        assert!(sync.is_excluded("custom.setting"));
    }
    
    #[test]
    fn test_auto_sync() {
        let mut sync = SettingsSync::new("device-123");
        
        assert!(!sync.is_auto_sync());
        
        sync.set_auto_sync(true);
        assert!(sync.is_auto_sync());
    }
    
    #[test]
    fn test_sync_interval() {
        let mut sync = SettingsSync::new("device-123");
        
        sync.set_sync_interval(Duration::from_secs(60));
        assert_eq!(sync.sync_interval(), Duration::from_secs(60));
    }
    
    #[test]
    fn test_needs_sync() {
        let mut sync = SettingsSync::new("device-123");
        
        // Not connected, doesn't need sync
        assert!(!sync.needs_sync());
        
        // Connected but auto sync disabled
        sync.configure("https://sync.example.com", "token123");
        assert!(!sync.needs_sync());
        
        // Auto sync enabled, needs sync (never synced)
        sync.set_auto_sync(true);
        assert!(sync.needs_sync());
    }
}
