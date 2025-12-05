// OTA (Over-The-Air) Update System for Kāraṇa OS
// Manages system updates, version control, and safe rollback

pub mod version;
pub mod downloader;
pub mod installer;
pub mod rollback;
pub mod manifest;

pub use version::*;
pub use downloader::*;
pub use installer::*;
pub use rollback::*;
pub use manifest::*;

use std::collections::HashMap;

/// Configuration for OTA update system
#[derive(Debug, Clone)]
pub struct OTAConfig {
    /// Update server URL
    pub server_url: String,
    /// Check interval in seconds
    pub check_interval_secs: u64,
    /// Whether to auto-download updates
    pub auto_download: bool,
    /// Whether to auto-install updates
    pub auto_install: bool,
    /// Maximum download retry attempts
    pub max_retries: u32,
    /// Download timeout in seconds
    pub download_timeout_secs: u64,
    /// Require WiFi for downloads
    pub require_wifi: bool,
    /// Minimum battery level for update (0.0 - 1.0)
    pub min_battery_level: f32,
    /// Enable delta updates
    pub enable_delta_updates: bool,
    /// Keep number of previous versions for rollback
    pub rollback_versions: u32,
    /// Update channels available
    pub channels: Vec<UpdateChannel>,
    /// Active channel
    pub active_channel: UpdateChannel,
}

impl Default for OTAConfig {
    fn default() -> Self {
        Self {
            server_url: "https://updates.karana-os.io".to_string(),
            check_interval_secs: 86400, // Daily
            auto_download: true,
            auto_install: false, // Require user confirmation
            max_retries: 3,
            download_timeout_secs: 300,
            require_wifi: true,
            min_battery_level: 0.3,
            enable_delta_updates: true,
            rollback_versions: 2,
            channels: vec![
                UpdateChannel::Stable,
                UpdateChannel::Beta,
                UpdateChannel::Dev,
            ],
            active_channel: UpdateChannel::Stable,
        }
    }
}

/// Update release channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpdateChannel {
    /// Stable releases
    Stable,
    /// Beta releases for testing
    Beta,
    /// Development builds
    Dev,
    /// Enterprise channel
    Enterprise,
    /// Custom channel
    Custom,
}

impl UpdateChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Dev => "dev",
            Self::Enterprise => "enterprise",
            Self::Custom => "custom",
        }
    }
}

/// Current update status
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    /// No update available
    UpToDate,
    /// Update available but not downloaded
    Available(UpdateInfo),
    /// Currently downloading
    Downloading { progress: f32, info: UpdateInfo },
    /// Download complete, ready to install
    ReadyToInstall(UpdateInfo),
    /// Currently installing
    Installing { progress: f32, info: UpdateInfo },
    /// Update complete, restart required
    PendingRestart(UpdateInfo),
    /// Update failed
    Failed { error: String, info: Option<UpdateInfo> },
    /// Checking for updates
    Checking,
}

/// Information about an available update
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateInfo {
    /// Version string
    pub version: SemanticVersion,
    /// Release channel
    pub channel: UpdateChannel,
    /// Release notes
    pub release_notes: String,
    /// Download size in bytes
    pub download_size: u64,
    /// Installed size in bytes
    pub installed_size: u64,
    /// Whether this is a delta update
    pub is_delta: bool,
    /// SHA256 checksum
    pub checksum: String,
    /// Release timestamp
    pub released_at: u64,
    /// Whether update is mandatory
    pub mandatory: bool,
    /// Minimum required version for delta
    pub min_version_for_delta: Option<SemanticVersion>,
    /// Security update flag
    pub is_security_update: bool,
    /// Features in this update
    pub features: Vec<String>,
    /// Fixed issues
    pub fixes: Vec<String>,
}

/// Main OTA update manager
pub struct OTAManager {
    config: OTAConfig,
    current_version: SemanticVersion,
    status: UpdateStatus,
    version_manager: VersionManager,
    downloader: UpdateDownloader,
    installer: UpdateInstaller,
    rollback_manager: RollbackManager,
    manifest_parser: ManifestParser,
    update_history: Vec<UpdateHistoryEntry>,
    listeners: Vec<Box<dyn OTAListener + Send + Sync>>,
    device_id: String,
    last_check: Option<u64>,
    pending_update: Option<UpdateInfo>,
}

/// History entry for completed updates
#[derive(Debug, Clone)]
pub struct UpdateHistoryEntry {
    pub version: SemanticVersion,
    pub channel: UpdateChannel,
    pub installed_at: u64,
    pub duration_secs: u64,
    pub was_rollback: bool,
    pub success: bool,
    pub error: Option<String>,
}

/// Listener for OTA events
pub trait OTAListener {
    fn on_update_available(&mut self, info: &UpdateInfo);
    fn on_download_progress(&mut self, progress: f32);
    fn on_download_complete(&mut self, info: &UpdateInfo);
    fn on_install_progress(&mut self, progress: f32);
    fn on_install_complete(&mut self, info: &UpdateInfo);
    fn on_error(&mut self, error: &str);
    fn on_rollback(&mut self, from: &SemanticVersion, to: &SemanticVersion);
}

impl OTAManager {
    pub fn new(config: OTAConfig, device_id: String) -> Self {
        let current_version = SemanticVersion::new(0, 1, 0);
        
        Self {
            config: config.clone(),
            current_version: current_version.clone(),
            status: UpdateStatus::UpToDate,
            version_manager: VersionManager::new(current_version.clone()),
            downloader: UpdateDownloader::new(config.clone()),
            installer: UpdateInstaller::new(config.clone()),
            rollback_manager: RollbackManager::new(config.rollback_versions),
            manifest_parser: ManifestParser::new(),
            update_history: Vec::new(),
            listeners: Vec::new(),
            device_id,
            last_check: None,
            pending_update: None,
        }
    }
    
    /// Check for available updates
    pub fn check_for_updates(&mut self) -> Result<Option<UpdateInfo>, OTAError> {
        self.status = UpdateStatus::Checking;
        
        // Simulate update check
        let update_available = self.simulate_update_check()?;
        
        self.last_check = Some(self.current_timestamp());
        
        if let Some(ref info) = update_available {
            self.status = UpdateStatus::Available(info.clone());
            self.pending_update = Some(info.clone());
            
            for listener in &mut self.listeners {
                listener.on_update_available(info);
            }
        } else {
            self.status = UpdateStatus::UpToDate;
        }
        
        Ok(update_available)
    }
    
    /// Download available update
    pub fn download_update(&mut self) -> Result<(), OTAError> {
        let info = self.pending_update.clone()
            .ok_or(OTAError::NoUpdateAvailable)?;
        
        // Check preconditions
        self.check_download_preconditions()?;
        
        self.status = UpdateStatus::Downloading { 
            progress: 0.0, 
            info: info.clone() 
        };
        
        // Simulate download
        let result = self.downloader.download(&info);
        
        match result {
            Ok(_) => {
                self.status = UpdateStatus::ReadyToInstall(info.clone());
                for listener in &mut self.listeners {
                    listener.on_download_complete(&info);
                }
                Ok(())
            }
            Err(e) => {
                self.status = UpdateStatus::Failed {
                    error: e.to_string(),
                    info: Some(info),
                };
                Err(e)
            }
        }
    }
    
    /// Install downloaded update
    pub fn install_update(&mut self) -> Result<(), OTAError> {
        let info = match &self.status {
            UpdateStatus::ReadyToInstall(info) => info.clone(),
            _ => return Err(OTAError::UpdateNotReady),
        };
        
        // Check preconditions
        self.check_install_preconditions()?;
        
        // Create backup for rollback
        self.rollback_manager.create_backup(&self.current_version)?;
        
        self.status = UpdateStatus::Installing {
            progress: 0.0,
            info: info.clone(),
        };
        
        // Simulate installation
        let result = self.installer.install(&info);
        
        match result {
            Ok(_) => {
                // Record history
                self.update_history.push(UpdateHistoryEntry {
                    version: info.version.clone(),
                    channel: info.channel,
                    installed_at: self.current_timestamp(),
                    duration_secs: 0, // Would be actual duration
                    was_rollback: false,
                    success: true,
                    error: None,
                });
                
                self.current_version = info.version.clone();
                self.status = UpdateStatus::PendingRestart(info.clone());
                
                for listener in &mut self.listeners {
                    listener.on_install_complete(&info);
                }
                
                Ok(())
            }
            Err(e) => {
                // Attempt rollback
                let _ = self.rollback();
                
                self.status = UpdateStatus::Failed {
                    error: e.to_string(),
                    info: Some(info),
                };
                Err(e)
            }
        }
    }
    
    /// Rollback to previous version
    pub fn rollback(&mut self) -> Result<SemanticVersion, OTAError> {
        let previous = self.rollback_manager.get_previous_version()
            .ok_or(OTAError::NoPreviousVersion)?;
        
        let from = self.current_version.clone();
        
        self.rollback_manager.restore(&previous)?;
        
        self.current_version = previous.clone();
        
        // Record history
        self.update_history.push(UpdateHistoryEntry {
            version: previous.clone(),
            channel: self.config.active_channel,
            installed_at: self.current_timestamp(),
            duration_secs: 0,
            was_rollback: true,
            success: true,
            error: None,
        });
        
        for listener in &mut self.listeners {
            listener.on_rollback(&from, &previous);
        }
        
        self.status = UpdateStatus::UpToDate;
        
        Ok(previous)
    }
    
    /// Get current update status
    pub fn status(&self) -> &UpdateStatus {
        &self.status
    }
    
    /// Get current version
    pub fn current_version(&self) -> &SemanticVersion {
        &self.current_version
    }
    
    /// Get update history
    pub fn history(&self) -> &[UpdateHistoryEntry] {
        &self.update_history
    }
    
    /// Set update channel
    pub fn set_channel(&mut self, channel: UpdateChannel) {
        self.config.active_channel = channel;
    }
    
    /// Get available channels
    pub fn available_channels(&self) -> &[UpdateChannel] {
        &self.config.channels
    }
    
    /// Register update listener
    pub fn add_listener(&mut self, listener: Box<dyn OTAListener + Send + Sync>) {
        self.listeners.push(listener);
    }
    
    /// Check if update is available
    pub fn has_pending_update(&self) -> bool {
        self.pending_update.is_some()
    }
    
    /// Get pending update info
    pub fn pending_update(&self) -> Option<&UpdateInfo> {
        self.pending_update.as_ref()
    }
    
    /// Cancel ongoing download
    pub fn cancel_download(&mut self) -> Result<(), OTAError> {
        match &self.status {
            UpdateStatus::Downloading { .. } => {
                self.downloader.cancel();
                if let Some(info) = &self.pending_update {
                    self.status = UpdateStatus::Available(info.clone());
                } else {
                    self.status = UpdateStatus::UpToDate;
                }
                Ok(())
            }
            _ => Err(OTAError::NoActiveDownload),
        }
    }
    
    /// Check download preconditions
    fn check_download_preconditions(&self) -> Result<(), OTAError> {
        // Check battery level
        let battery = self.get_battery_level();
        if battery < self.config.min_battery_level {
            return Err(OTAError::LowBattery(battery));
        }
        
        // Check network
        if self.config.require_wifi && !self.is_wifi_connected() {
            return Err(OTAError::NoWifiConnection);
        }
        
        // Check storage
        if let Some(info) = &self.pending_update {
            let available = self.get_available_storage();
            if available < info.download_size {
                return Err(OTAError::InsufficientStorage {
                    required: info.download_size,
                    available,
                });
            }
        }
        
        Ok(())
    }
    
    /// Check install preconditions
    fn check_install_preconditions(&self) -> Result<(), OTAError> {
        // Check battery level (higher threshold for install)
        let battery = self.get_battery_level();
        if battery < self.config.min_battery_level + 0.2 {
            return Err(OTAError::LowBattery(battery));
        }
        
        // Check storage for installation
        if let Some(info) = &self.pending_update {
            let available = self.get_available_storage();
            if available < info.installed_size {
                return Err(OTAError::InsufficientStorage {
                    required: info.installed_size,
                    available,
                });
            }
        }
        
        Ok(())
    }
    
    // Simulated hardware checks
    fn get_battery_level(&self) -> f32 {
        0.8 // Simulated
    }
    
    fn is_wifi_connected(&self) -> bool {
        true // Simulated
    }
    
    fn get_available_storage(&self) -> u64 {
        1024 * 1024 * 1024 // 1GB simulated
    }
    
    fn current_timestamp(&self) -> u64 {
        0 // Simulated
    }
    
    fn simulate_update_check(&self) -> Result<Option<UpdateInfo>, OTAError> {
        // Simulate checking for update
        // In real implementation, would query server
        
        let newer_version = SemanticVersion::new(
            self.current_version.major,
            self.current_version.minor,
            self.current_version.patch + 1,
        );
        
        Ok(Some(UpdateInfo {
            version: newer_version,
            channel: self.config.active_channel,
            release_notes: "Bug fixes and improvements".to_string(),
            download_size: 50 * 1024 * 1024, // 50MB
            installed_size: 100 * 1024 * 1024, // 100MB
            is_delta: self.config.enable_delta_updates,
            checksum: "abc123".to_string(),
            released_at: self.current_timestamp(),
            mandatory: false,
            min_version_for_delta: Some(self.current_version.clone()),
            is_security_update: false,
            features: vec!["New gestures".to_string()],
            fixes: vec!["Performance improvements".to_string()],
        }))
    }
}

/// OTA-specific errors
#[derive(Debug, Clone, PartialEq)]
pub enum OTAError {
    NoUpdateAvailable,
    UpdateNotReady,
    NoPreviousVersion,
    NoActiveDownload,
    LowBattery(f32),
    NoWifiConnection,
    InsufficientStorage { required: u64, available: u64 },
    DownloadFailed(String),
    InstallFailed(String),
    ChecksumMismatch,
    NetworkError(String),
    ServerError(String),
    ManifestParseError(String),
    RollbackFailed(String),
    VersionConflict,
    SignatureInvalid,
}

impl std::fmt::Display for OTAError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoUpdateAvailable => write!(f, "No update available"),
            Self::UpdateNotReady => write!(f, "Update not ready for installation"),
            Self::NoPreviousVersion => write!(f, "No previous version for rollback"),
            Self::NoActiveDownload => write!(f, "No active download to cancel"),
            Self::LowBattery(level) => write!(f, "Battery too low: {:.0}%", level * 100.0),
            Self::NoWifiConnection => write!(f, "WiFi connection required"),
            Self::InsufficientStorage { required, available } => {
                write!(f, "Insufficient storage: need {} bytes, have {}", required, available)
            }
            Self::DownloadFailed(msg) => write!(f, "Download failed: {}", msg),
            Self::InstallFailed(msg) => write!(f, "Installation failed: {}", msg),
            Self::ChecksumMismatch => write!(f, "Checksum verification failed"),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ServerError(msg) => write!(f, "Server error: {}", msg),
            Self::ManifestParseError(msg) => write!(f, "Manifest parse error: {}", msg),
            Self::RollbackFailed(msg) => write!(f, "Rollback failed: {}", msg),
            Self::VersionConflict => write!(f, "Version conflict detected"),
            Self::SignatureInvalid => write!(f, "Update signature is invalid"),
        }
    }
}

impl std::error::Error for OTAError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ota_config_default() {
        let config = OTAConfig::default();
        assert!(config.auto_download);
        assert!(!config.auto_install);
        assert_eq!(config.active_channel, UpdateChannel::Stable);
        assert_eq!(config.rollback_versions, 2);
    }
    
    #[test]
    fn test_update_channel() {
        assert_eq!(UpdateChannel::Stable.as_str(), "stable");
        assert_eq!(UpdateChannel::Beta.as_str(), "beta");
        assert_eq!(UpdateChannel::Dev.as_str(), "dev");
    }
    
    #[test]
    fn test_ota_manager_creation() {
        let config = OTAConfig::default();
        let manager = OTAManager::new(config, "device123".to_string());
        
        assert_eq!(*manager.status(), UpdateStatus::UpToDate);
        assert_eq!(manager.current_version().major, 0);
    }
    
    #[test]
    fn test_check_for_updates() {
        let config = OTAConfig::default();
        let mut manager = OTAManager::new(config, "device123".to_string());
        
        let result = manager.check_for_updates();
        assert!(result.is_ok());
        
        let update = result.unwrap();
        assert!(update.is_some());
    }
    
    #[test]
    fn test_update_status_variants() {
        let info = UpdateInfo {
            version: SemanticVersion::new(1, 0, 0),
            channel: UpdateChannel::Stable,
            release_notes: "Test".to_string(),
            download_size: 1000,
            installed_size: 2000,
            is_delta: false,
            checksum: "test".to_string(),
            released_at: 0,
            mandatory: false,
            min_version_for_delta: None,
            is_security_update: false,
            features: vec![],
            fixes: vec![],
        };
        
        let status = UpdateStatus::Available(info.clone());
        assert!(matches!(status, UpdateStatus::Available(_)));
        
        let status = UpdateStatus::Downloading { progress: 0.5, info: info.clone() };
        if let UpdateStatus::Downloading { progress, .. } = status {
            assert_eq!(progress, 0.5);
        }
    }
    
    #[test]
    fn test_set_channel() {
        let config = OTAConfig::default();
        let mut manager = OTAManager::new(config, "device123".to_string());
        
        manager.set_channel(UpdateChannel::Beta);
        // Channel is internal, but history would reflect it
    }
    
    #[test]
    fn test_ota_error_display() {
        let error = OTAError::LowBattery(0.15);
        assert!(error.to_string().contains("15%"));
        
        let error = OTAError::InsufficientStorage {
            required: 1000,
            available: 500,
        };
        assert!(error.to_string().contains("1000"));
    }
    
    #[test]
    fn test_has_pending_update() {
        let config = OTAConfig::default();
        let mut manager = OTAManager::new(config, "device123".to_string());
        
        assert!(!manager.has_pending_update());
        
        let _ = manager.check_for_updates();
        assert!(manager.has_pending_update());
    }
    
    #[test]
    fn test_cancel_download_no_active() {
        let config = OTAConfig::default();
        let mut manager = OTAManager::new(config, "device123".to_string());
        
        let result = manager.cancel_download();
        assert!(matches!(result, Err(OTAError::NoActiveDownload)));
    }
}
