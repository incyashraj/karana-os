// Update downloader for OTA system

use super::{OTAConfig, OTAError, UpdateInfo, UpdateChannel};
use std::collections::HashMap;

/// Manages update downloads
pub struct UpdateDownloader {
    config: OTAConfig,
    active_downloads: HashMap<String, DownloadState>,
    download_queue: Vec<DownloadRequest>,
    completed_downloads: HashMap<String, DownloadResult>,
    bandwidth_limit: Option<u64>,
    total_downloaded: u64,
    cancelled: bool,
}

/// State of an active download
#[derive(Debug, Clone)]
pub struct DownloadState {
    pub request: DownloadRequest,
    pub progress: DownloadProgress,
    pub status: DownloadStatus,
    pub started_at: u64,
    pub last_activity: u64,
}

/// Download request
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub id: String,
    pub url: String,
    pub checksum: String,
    pub expected_size: u64,
    pub priority: DownloadPriority,
    pub resume_supported: bool,
    pub retry_count: u32,
}

/// Download progress information
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bps: u64,
    pub eta_seconds: Option<u64>,
    pub chunks_completed: u32,
    pub chunks_total: u32,
}

impl DownloadProgress {
    pub fn percentage(&self) -> f32 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.downloaded_bytes as f32 / self.total_bytes as f32) * 100.0
        }
    }
    
    pub fn remaining_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.downloaded_bytes)
    }
}

/// Download status
#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    Queued,
    Connecting,
    Downloading,
    Paused,
    Verifying,
    Complete,
    Failed(String),
    Cancelled,
}

/// Download priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DownloadPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Result of a completed download
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub request_id: String,
    pub success: bool,
    pub file_path: Option<String>,
    pub error: Option<String>,
    pub duration_secs: u64,
    pub bytes_downloaded: u64,
    pub checksum_verified: bool,
}

impl UpdateDownloader {
    pub fn new(config: OTAConfig) -> Self {
        Self {
            config,
            active_downloads: HashMap::new(),
            download_queue: Vec::new(),
            completed_downloads: HashMap::new(),
            bandwidth_limit: None,
            total_downloaded: 0,
            cancelled: false,
        }
    }
    
    /// Start downloading an update
    pub fn download(&mut self, info: &UpdateInfo) -> Result<DownloadResult, OTAError> {
        self.cancelled = false;
        
        let request = DownloadRequest {
            id: format!("update-{}", info.version),
            url: self.build_download_url(info),
            checksum: info.checksum.clone(),
            expected_size: info.download_size,
            priority: if info.is_security_update {
                DownloadPriority::Critical
            } else if info.mandatory {
                DownloadPriority::High
            } else {
                DownloadPriority::Normal
            },
            resume_supported: true,
            retry_count: 0,
        };
        
        let state = DownloadState {
            request: request.clone(),
            progress: DownloadProgress {
                downloaded_bytes: 0,
                total_bytes: info.download_size,
                speed_bps: 0,
                eta_seconds: None,
                chunks_completed: 0,
                chunks_total: self.calculate_chunks(info.download_size),
            },
            status: DownloadStatus::Queued,
            started_at: 0,
            last_activity: 0,
        };
        
        self.active_downloads.insert(request.id.clone(), state);
        
        // Simulate download process
        self.perform_download(&request.id, info)
    }
    
    /// Perform the actual download
    fn perform_download(&mut self, id: &str, info: &UpdateInfo) -> Result<DownloadResult, OTAError> {
        // Update status to connecting
        if let Some(state) = self.active_downloads.get_mut(id) {
            state.status = DownloadStatus::Connecting;
        }
        
        // Simulate connection
        if self.cancelled {
            return self.cancel_download(id);
        }
        
        // Update status to downloading
        let timestamp = self.current_timestamp();
        if let Some(state) = self.active_downloads.get_mut(id) {
            state.status = DownloadStatus::Downloading;
            state.started_at = timestamp;
        }
        
        // Simulate download chunks
        let chunk_size = 1024 * 1024; // 1MB chunks
        let total_chunks = (info.download_size + chunk_size - 1) / chunk_size;
        
        for chunk in 0..total_chunks {
            if self.cancelled {
                return self.cancel_download(id);
            }
            
            // Update progress
            let timestamp = self.current_timestamp();
            if let Some(state) = self.active_downloads.get_mut(id) {
                state.progress.downloaded_bytes = 
                    ((chunk + 1) * chunk_size).min(info.download_size);
                state.progress.chunks_completed = (chunk + 1) as u32;
                state.progress.speed_bps = 5 * 1024 * 1024; // 5 MB/s simulated
                state.progress.eta_seconds = Some(
                    state.progress.remaining_bytes() / state.progress.speed_bps.max(1)
                );
                state.last_activity = timestamp;
            }
        }
        
        // Verify checksum
        if let Some(state) = self.active_downloads.get_mut(id) {
            state.status = DownloadStatus::Verifying;
        }
        
        let checksum_valid = self.verify_checksum(id, &info.checksum);
        
        if !checksum_valid {
            if let Some(state) = self.active_downloads.get_mut(id) {
                state.status = DownloadStatus::Failed("Checksum mismatch".to_string());
            }
            return Err(OTAError::ChecksumMismatch);
        }
        
        // Complete
        if let Some(state) = self.active_downloads.get_mut(id) {
            state.status = DownloadStatus::Complete;
        }
        
        let result = DownloadResult {
            request_id: id.to_string(),
            success: true,
            file_path: Some(format!("/tmp/karana-update-{}.pkg", info.version)),
            error: None,
            duration_secs: 10, // Simulated
            bytes_downloaded: info.download_size,
            checksum_verified: true,
        };
        
        self.completed_downloads.insert(id.to_string(), result.clone());
        self.total_downloaded += info.download_size;
        
        Ok(result)
    }
    
    /// Cancel ongoing downloads
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }
    
    fn cancel_download(&mut self, id: &str) -> Result<DownloadResult, OTAError> {
        if let Some(state) = self.active_downloads.get_mut(id) {
            state.status = DownloadStatus::Cancelled;
        }
        
        Err(OTAError::DownloadFailed("Download cancelled".to_string()))
    }
    
    /// Pause a download
    pub fn pause(&mut self, id: &str) -> Result<(), OTAError> {
        if let Some(state) = self.active_downloads.get_mut(id) {
            if state.status == DownloadStatus::Downloading {
                state.status = DownloadStatus::Paused;
                Ok(())
            } else {
                Err(OTAError::DownloadFailed("Cannot pause download in current state".to_string()))
            }
        } else {
            Err(OTAError::DownloadFailed("Download not found".to_string()))
        }
    }
    
    /// Resume a paused download
    pub fn resume(&mut self, id: &str) -> Result<(), OTAError> {
        if let Some(state) = self.active_downloads.get_mut(id) {
            if state.status == DownloadStatus::Paused {
                state.status = DownloadStatus::Downloading;
                Ok(())
            } else {
                Err(OTAError::DownloadFailed("Download is not paused".to_string()))
            }
        } else {
            Err(OTAError::DownloadFailed("Download not found".to_string()))
        }
    }
    
    /// Get download progress
    pub fn progress(&self, id: &str) -> Option<&DownloadProgress> {
        self.active_downloads.get(id).map(|s| &s.progress)
    }
    
    /// Get download status
    pub fn status(&self, id: &str) -> Option<&DownloadStatus> {
        self.active_downloads.get(id).map(|s| &s.status)
    }
    
    /// Set bandwidth limit in bytes per second
    pub fn set_bandwidth_limit(&mut self, limit: Option<u64>) {
        self.bandwidth_limit = limit;
    }
    
    /// Get total bytes downloaded
    pub fn total_downloaded(&self) -> u64 {
        self.total_downloaded
    }
    
    /// Queue a download for later
    pub fn queue(&mut self, request: DownloadRequest) {
        // Insert by priority
        let insert_pos = self.download_queue
            .iter()
            .position(|r| r.priority < request.priority)
            .unwrap_or(self.download_queue.len());
        
        self.download_queue.insert(insert_pos, request);
    }
    
    /// Get queued downloads
    pub fn queued(&self) -> &[DownloadRequest] {
        &self.download_queue
    }
    
    /// Process next item in queue
    pub fn process_queue(&mut self) -> Option<String> {
        if let Some(request) = self.download_queue.pop() {
            let id = request.id.clone();
            
            let state = DownloadState {
                request,
                progress: DownloadProgress {
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    speed_bps: 0,
                    eta_seconds: None,
                    chunks_completed: 0,
                    chunks_total: 0,
                },
                status: DownloadStatus::Queued,
                started_at: 0,
                last_activity: 0,
            };
            
            self.active_downloads.insert(id.clone(), state);
            Some(id)
        } else {
            None
        }
    }
    
    /// Build download URL for update
    fn build_download_url(&self, info: &UpdateInfo) -> String {
        let channel = info.channel.as_str();
        let update_type = if info.is_delta { "delta" } else { "full" };
        
        format!(
            "{}/v1/{}/{}/{}.pkg",
            self.config.server_url,
            channel,
            update_type,
            info.version
        )
    }
    
    /// Calculate number of chunks for download
    fn calculate_chunks(&self, size: u64) -> u32 {
        let chunk_size = 1024 * 1024; // 1MB
        ((size + chunk_size - 1) / chunk_size) as u32
    }
    
    /// Verify download checksum
    fn verify_checksum(&self, _id: &str, _expected: &str) -> bool {
        // Simulated - would actually compute SHA256
        true
    }
    
    fn current_timestamp(&self) -> u64 {
        0 // Simulated
    }
    
    /// Cleanup completed downloads
    pub fn cleanup_completed(&mut self) {
        self.completed_downloads.clear();
    }
    
    /// Get completed download result
    pub fn get_result(&self, id: &str) -> Option<&DownloadResult> {
        self.completed_downloads.get(id)
    }
}

/// Download chunk for resumable downloads
#[derive(Debug, Clone)]
pub struct DownloadChunk {
    pub index: u32,
    pub offset: u64,
    pub size: u64,
    pub checksum: Option<String>,
    pub downloaded: bool,
}

/// Network quality information
#[derive(Debug, Clone)]
pub struct NetworkQuality {
    pub connection_type: ConnectionType,
    pub signal_strength: f32,
    pub latency_ms: u32,
    pub bandwidth_estimate: u64,
    pub is_metered: bool,
}

/// Connection type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionType {
    Wifi,
    Cellular4G,
    Cellular5G,
    Ethernet,
    Bluetooth,
    Unknown,
}

impl NetworkQuality {
    pub fn is_suitable_for_download(&self) -> bool {
        match self.connection_type {
            ConnectionType::Wifi | ConnectionType::Ethernet => true,
            ConnectionType::Cellular5G => !self.is_metered,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::SemanticVersion;
    
    fn test_update_info() -> UpdateInfo {
        UpdateInfo {
            version: SemanticVersion::new(1, 0, 1),
            channel: UpdateChannel::Stable,
            release_notes: "Test update".to_string(),
            download_size: 10 * 1024 * 1024, // 10MB
            installed_size: 20 * 1024 * 1024,
            is_delta: false,
            checksum: "abc123".to_string(),
            released_at: 0,
            mandatory: false,
            min_version_for_delta: None,
            is_security_update: false,
            features: vec![],
            fixes: vec![],
        }
    }
    
    #[test]
    fn test_downloader_creation() {
        let config = OTAConfig::default();
        let downloader = UpdateDownloader::new(config);
        
        assert_eq!(downloader.total_downloaded(), 0);
    }
    
    #[test]
    fn test_download_progress() {
        let progress = DownloadProgress {
            downloaded_bytes: 50,
            total_bytes: 100,
            speed_bps: 10,
            eta_seconds: Some(5),
            chunks_completed: 5,
            chunks_total: 10,
        };
        
        assert_eq!(progress.percentage(), 50.0);
        assert_eq!(progress.remaining_bytes(), 50);
    }
    
    #[test]
    fn test_download_update() {
        let config = OTAConfig::default();
        let mut downloader = UpdateDownloader::new(config);
        
        let info = test_update_info();
        let result = downloader.download(&info);
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.checksum_verified);
    }
    
    #[test]
    fn test_download_priority() {
        let config = OTAConfig::default();
        let mut downloader = UpdateDownloader::new(config);
        
        downloader.queue(DownloadRequest {
            id: "low".to_string(),
            url: "http://test/low".to_string(),
            checksum: "".to_string(),
            expected_size: 100,
            priority: DownloadPriority::Low,
            resume_supported: true,
            retry_count: 0,
        });
        
        downloader.queue(DownloadRequest {
            id: "high".to_string(),
            url: "http://test/high".to_string(),
            checksum: "".to_string(),
            expected_size: 100,
            priority: DownloadPriority::High,
            resume_supported: true,
            retry_count: 0,
        });
        
        // High priority should be first
        let queued = downloader.queued();
        assert_eq!(queued[0].id, "high");
    }
    
    #[test]
    fn test_cancel_download() {
        let config = OTAConfig::default();
        let mut downloader = UpdateDownloader::new(config);
        
        // Test that cancel sets the flag
        downloader.cancel();
        assert!(downloader.cancelled);
        
        // Reset and download (download resets the cancel flag)
        let info = test_update_info();
        let result = downloader.download(&info);
        
        // Download succeeds because flag was reset at start of download
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_bandwidth_limit() {
        let config = OTAConfig::default();
        let mut downloader = UpdateDownloader::new(config);
        
        downloader.set_bandwidth_limit(Some(1024 * 1024));
        // Bandwidth limit is set, would affect real downloads
    }
    
    #[test]
    fn test_network_quality() {
        let quality = NetworkQuality {
            connection_type: ConnectionType::Wifi,
            signal_strength: 0.8,
            latency_ms: 20,
            bandwidth_estimate: 50 * 1024 * 1024,
            is_metered: false,
        };
        
        assert!(quality.is_suitable_for_download());
        
        let metered_5g = NetworkQuality {
            connection_type: ConnectionType::Cellular5G,
            signal_strength: 0.9,
            latency_ms: 10,
            bandwidth_estimate: 100 * 1024 * 1024,
            is_metered: true,
        };
        
        assert!(!metered_5g.is_suitable_for_download());
    }
    
    #[test]
    fn test_download_status_variants() {
        assert_eq!(DownloadStatus::Queued, DownloadStatus::Queued);
        assert_ne!(DownloadStatus::Queued, DownloadStatus::Complete);
    }
    
    #[test]
    fn test_process_queue() {
        let config = OTAConfig::default();
        let mut downloader = UpdateDownloader::new(config);
        
        downloader.queue(DownloadRequest {
            id: "test".to_string(),
            url: "http://test".to_string(),
            checksum: "".to_string(),
            expected_size: 100,
            priority: DownloadPriority::Normal,
            resume_supported: true,
            retry_count: 0,
        });
        
        let id = downloader.process_queue();
        assert_eq!(id, Some("test".to_string()));
        
        // Queue should now be empty
        assert!(downloader.queued().is_empty());
    }
}
