//! Cloud service integration

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Cloud provider
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudProvider {
    /// Kāraṇa Cloud (native)
    KaranaCloud,
    /// Google Drive
    GoogleDrive,
    /// iCloud
    ICloud,
    /// Dropbox
    Dropbox,
    /// OneDrive
    OneDrive,
    /// Custom/self-hosted
    Custom,
}

/// Cloud state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudState {
    /// Not configured
    NotConfigured,
    /// Configured but disconnected
    Disconnected,
    /// Connecting
    Connecting,
    /// Connected
    Connected,
    /// Syncing
    Syncing,
    /// Error
    Error,
}

/// Cloud sync item type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncItemType {
    /// Settings
    Settings,
    /// Profiles
    Profiles,
    /// Spatial anchors
    Anchors,
    /// AR content
    Content,
    /// Photos/videos
    Media,
    /// User data
    UserData,
}

/// Sync status for an item
#[derive(Debug, Clone)]
pub struct SyncItem {
    /// Item type
    pub item_type: SyncItemType,
    /// Item identifier
    pub id: String,
    /// Local modified time
    pub local_modified: Instant,
    /// Cloud modified time (None if never synced)
    pub cloud_modified: Option<Instant>,
    /// Sync status
    pub status: ItemSyncStatus,
    /// Size in bytes
    pub size_bytes: u64,
}

/// Item sync status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSyncStatus {
    /// In sync
    Synced,
    /// Local changes pending upload
    PendingUpload,
    /// Remote changes pending download
    PendingDownload,
    /// Conflict between local and remote
    Conflict,
    /// Currently syncing
    Syncing,
    /// Error
    Error,
}

/// Cloud service
#[derive(Debug)]
pub struct CloudService {
    /// Current provider
    provider: Option<CloudProvider>,
    /// Connection state
    state: CloudState,
    /// Authentication token
    auth_token: Option<String>,
    /// Endpoint URL
    endpoint_url: Option<String>,
    /// User ID
    user_id: Option<String>,
    /// Sync items
    sync_items: HashMap<String, SyncItem>,
    /// Auto-sync enabled
    auto_sync: bool,
    /// Sync interval
    sync_interval: Duration,
    /// Last sync time
    last_sync: Option<Instant>,
    /// Items to sync (types enabled)
    enabled_types: Vec<SyncItemType>,
    /// WiFi only sync
    wifi_only: bool,
    /// Storage quota (bytes)
    storage_quota: u64,
    /// Storage used (bytes)
    storage_used: u64,
}

impl CloudService {
    /// Create new cloud service
    pub fn new() -> Self {
        Self {
            provider: None,
            state: CloudState::NotConfigured,
            auth_token: None,
            endpoint_url: None,
            user_id: None,
            sync_items: HashMap::new(),
            auto_sync: true,
            sync_interval: Duration::from_secs(300),
            last_sync: None,
            enabled_types: vec![
                SyncItemType::Settings,
                SyncItemType::Profiles,
                SyncItemType::Anchors,
            ],
            wifi_only: false,
            storage_quota: 5 * 1024 * 1024 * 1024, // 5 GB default
            storage_used: 0,
        }
    }
    
    /// Configure with provider
    pub fn configure(&mut self, provider: CloudProvider, token: &str, endpoint: Option<&str>) {
        self.provider = Some(provider);
        self.auth_token = Some(token.to_string());
        self.endpoint_url = endpoint.map(|s| s.to_string());
        self.state = CloudState::Disconnected;
    }
    
    /// Connect to cloud
    pub fn connect(&mut self) -> Result<(), String> {
        if self.provider.is_none() {
            return Err("Cloud not configured".to_string());
        }
        
        if self.auth_token.is_none() {
            return Err("Not authenticated".to_string());
        }
        
        self.state = CloudState::Connecting;
        
        // Simulate successful connection
        self.state = CloudState::Connected;
        
        Ok(())
    }
    
    /// Disconnect
    pub fn disconnect(&mut self) {
        self.state = CloudState::Disconnected;
    }
    
    /// Sign out (disconnect and clear credentials)
    pub fn sign_out(&mut self) {
        self.disconnect();
        self.auth_token = None;
        self.user_id = None;
        self.provider = None;
        self.state = CloudState::NotConfigured;
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.state == CloudState::Connected || self.state == CloudState::Syncing
    }
    
    /// Get current state
    pub fn state(&self) -> CloudState {
        self.state
    }
    
    /// Get provider
    pub fn provider(&self) -> Option<CloudProvider> {
        self.provider
    }
    
    /// Start sync
    pub fn sync(&mut self) -> Result<(), String> {
        if !self.is_connected() {
            return Err("Not connected to cloud".to_string());
        }
        
        self.state = CloudState::Syncing;
        
        // In real implementation, would sync each enabled type
        
        self.state = CloudState::Connected;
        self.last_sync = Some(Instant::now());
        
        Ok(())
    }
    
    /// Get last sync time
    pub fn last_sync(&self) -> Option<Instant> {
        self.last_sync
    }
    
    /// Enable/disable auto-sync
    pub fn set_auto_sync(&mut self, enabled: bool) {
        self.auto_sync = enabled;
    }
    
    /// Check if auto-sync enabled
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
    
    /// Enable sync type
    pub fn enable_sync_type(&mut self, item_type: SyncItemType) {
        if !self.enabled_types.contains(&item_type) {
            self.enabled_types.push(item_type);
        }
    }
    
    /// Disable sync type
    pub fn disable_sync_type(&mut self, item_type: SyncItemType) {
        self.enabled_types.retain(|t| *t != item_type);
    }
    
    /// Check if sync type enabled
    pub fn is_type_enabled(&self, item_type: SyncItemType) -> bool {
        self.enabled_types.contains(&item_type)
    }
    
    /// Set WiFi only mode
    pub fn set_wifi_only(&mut self, wifi_only: bool) {
        self.wifi_only = wifi_only;
    }
    
    /// Check if WiFi only
    pub fn is_wifi_only(&self) -> bool {
        self.wifi_only
    }
    
    /// Get storage quota
    pub fn storage_quota(&self) -> u64 {
        self.storage_quota
    }
    
    /// Get storage used
    pub fn storage_used(&self) -> u64 {
        self.storage_used
    }
    
    /// Get storage available
    pub fn storage_available(&self) -> u64 {
        self.storage_quota.saturating_sub(self.storage_used)
    }
    
    /// Get storage usage percentage
    pub fn storage_percentage(&self) -> f32 {
        if self.storage_quota == 0 {
            return 0.0;
        }
        (self.storage_used as f32 / self.storage_quota as f32) * 100.0
    }
    
    /// Check if needs sync
    pub fn needs_sync(&self) -> bool {
        if !self.auto_sync {
            return false;
        }
        
        match self.last_sync {
            Some(last) => last.elapsed() >= self.sync_interval,
            None => true,
        }
    }
    
    /// Update cloud service
    pub fn update(&mut self) {
        if self.is_connected() && self.needs_sync() {
            let _ = self.sync();
        }
    }
    
    /// Get pending items count
    pub fn pending_items(&self) -> usize {
        self.sync_items
            .values()
            .filter(|i| matches!(i.status, ItemSyncStatus::PendingUpload | ItemSyncStatus::PendingDownload))
            .count()
    }
    
    /// Get conflict items count
    pub fn conflict_items(&self) -> usize {
        self.sync_items
            .values()
            .filter(|i| i.status == ItemSyncStatus::Conflict)
            .count()
    }
}

impl Default for CloudService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cloud_service_creation() {
        let service = CloudService::new();
        assert_eq!(service.state(), CloudState::NotConfigured);
        assert!(service.provider().is_none());
    }
    
    #[test]
    fn test_configure_connect() {
        let mut service = CloudService::new();
        
        service.configure(CloudProvider::KaranaCloud, "token123", None);
        assert_eq!(service.state(), CloudState::Disconnected);
        
        assert!(service.connect().is_ok());
        assert!(service.is_connected());
    }
    
    #[test]
    fn test_sign_out() {
        let mut service = CloudService::new();
        
        service.configure(CloudProvider::KaranaCloud, "token123", None);
        service.connect().unwrap();
        
        service.sign_out();
        
        assert_eq!(service.state(), CloudState::NotConfigured);
        assert!(service.provider().is_none());
    }
    
    #[test]
    fn test_sync_types() {
        let mut service = CloudService::new();
        
        assert!(service.is_type_enabled(SyncItemType::Settings));
        
        service.disable_sync_type(SyncItemType::Settings);
        assert!(!service.is_type_enabled(SyncItemType::Settings));
        
        service.enable_sync_type(SyncItemType::Media);
        assert!(service.is_type_enabled(SyncItemType::Media));
    }
    
    #[test]
    fn test_storage_calculations() {
        let mut service = CloudService::new();
        
        // 5 GB quota
        assert_eq!(service.storage_quota(), 5 * 1024 * 1024 * 1024);
        
        // Simulate 1 GB used
        service.storage_used = 1 * 1024 * 1024 * 1024;
        
        assert_eq!(service.storage_available(), 4 * 1024 * 1024 * 1024);
        assert!((service.storage_percentage() - 20.0).abs() < 0.1);
    }
    
    #[test]
    fn test_wifi_only() {
        let mut service = CloudService::new();
        
        assert!(!service.is_wifi_only());
        
        service.set_wifi_only(true);
        assert!(service.is_wifi_only());
    }
}
