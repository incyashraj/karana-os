// Rollback system for OTA updates

use super::{OTAError, SemanticVersion};
use std::collections::HashMap;

/// Manages version rollback
pub struct RollbackManager {
    max_versions: u32,
    backup_versions: Vec<BackupEntry>,
    rollback_history: Vec<RollbackEvent>,
    current_version: Option<SemanticVersion>,
}

/// Backup entry for a version
#[derive(Debug, Clone)]
pub struct BackupEntry {
    pub version: SemanticVersion,
    pub created_at: u64,
    pub size_bytes: u64,
    pub slot: Option<String>,
    pub bootable: bool,
    pub verified: bool,
}

/// Rollback event record
#[derive(Debug, Clone)]
pub struct RollbackEvent {
    pub from_version: SemanticVersion,
    pub to_version: SemanticVersion,
    pub timestamp: u64,
    pub reason: RollbackReason,
    pub success: bool,
    pub duration_ms: u64,
}

/// Reason for rollback
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RollbackReason {
    UserRequested,
    InstallationFailed,
    BootFailure,
    SystemCrash,
    PerformanceIssue,
    CompatibilityIssue,
    SecurityVulnerability,
    AutomaticRecovery,
}

impl RollbackReason {
    pub fn description(&self) -> &'static str {
        match self {
            Self::UserRequested => "User requested rollback",
            Self::InstallationFailed => "Update installation failed",
            Self::BootFailure => "System failed to boot",
            Self::SystemCrash => "System crashed after update",
            Self::PerformanceIssue => "Performance degradation detected",
            Self::CompatibilityIssue => "Compatibility issue detected",
            Self::SecurityVulnerability => "Security vulnerability in update",
            Self::AutomaticRecovery => "Automatic recovery triggered",
        }
    }
}

impl RollbackManager {
    pub fn new(max_versions: u32) -> Self {
        Self {
            max_versions,
            backup_versions: Vec::new(),
            rollback_history: Vec::new(),
            current_version: None,
        }
    }
    
    /// Create a backup of current version
    pub fn create_backup(&mut self, version: &SemanticVersion) -> Result<(), OTAError> {
        // Check if already backed up
        if self.backup_versions.iter().any(|b| &b.version == version) {
            return Ok(());
        }
        
        // Remove old backups if at limit
        while self.backup_versions.len() >= self.max_versions as usize {
            // Remove oldest (not current)
            if let Some(oldest_idx) = self.find_oldest_removable() {
                self.backup_versions.remove(oldest_idx);
            } else {
                break;
            }
        }
        
        // Create new backup entry
        let backup = BackupEntry {
            version: version.clone(),
            created_at: self.current_timestamp(),
            size_bytes: 0, // Would be actual size
            slot: None,
            bootable: true,
            verified: true,
        };
        
        self.backup_versions.push(backup);
        self.current_version = Some(version.clone());
        
        Ok(())
    }
    
    /// Restore a previous version
    pub fn restore(&mut self, version: &SemanticVersion) -> Result<(), OTAError> {
        let backup = self.backup_versions
            .iter()
            .find(|b| &b.version == version)
            .ok_or_else(|| OTAError::RollbackFailed("Version not found in backups".to_string()))?;
        
        if !backup.bootable {
            return Err(OTAError::RollbackFailed("Backup version is not bootable".to_string()));
        }
        
        let from_version = self.current_version.clone()
            .unwrap_or_else(|| SemanticVersion::new(0, 0, 0));
        
        // Perform restore (simulated)
        self.perform_restore(backup)?;
        
        // Record rollback event
        self.rollback_history.push(RollbackEvent {
            from_version,
            to_version: version.clone(),
            timestamp: self.current_timestamp(),
            reason: RollbackReason::UserRequested,
            success: true,
            duration_ms: 0,
        });
        
        self.current_version = Some(version.clone());
        
        Ok(())
    }
    
    /// Get previous version for rollback
    pub fn get_previous_version(&self) -> Option<SemanticVersion> {
        let current = self.current_version.as_ref()?;
        
        // Find latest backup that isn't current
        self.backup_versions
            .iter()
            .filter(|b| &b.version != current && b.bootable)
            .max_by_key(|b| b.created_at)
            .map(|b| b.version.clone())
    }
    
    /// Get all available backup versions
    pub fn available_versions(&self) -> Vec<SemanticVersion> {
        self.backup_versions
            .iter()
            .filter(|b| b.bootable)
            .map(|b| b.version.clone())
            .collect()
    }
    
    /// Check if rollback is possible
    pub fn can_rollback(&self) -> bool {
        self.get_previous_version().is_some()
    }
    
    /// Get rollback history
    pub fn history(&self) -> &[RollbackEvent] {
        &self.rollback_history
    }
    
    /// Mark a backup as bootable/not bootable
    pub fn set_bootable(&mut self, version: &SemanticVersion, bootable: bool) {
        if let Some(backup) = self.backup_versions.iter_mut()
            .find(|b| &b.version == version) 
        {
            backup.bootable = bootable;
        }
    }
    
    /// Verify a backup
    pub fn verify_backup(&mut self, version: &SemanticVersion) -> Result<bool, OTAError> {
        // First check if version exists
        if !self.backup_versions.iter().any(|b| &b.version == version) {
            return Err(OTAError::RollbackFailed("Version not found".to_string()));
        }
        
        // Simulated verification
        let verified = self.perform_verification(version);
        
        // Now update the backup
        if let Some(backup) = self.backup_versions.iter_mut().find(|b| &b.version == version) {
            backup.verified = verified;
        }
        
        Ok(verified)
    }
    
    /// Get backup info for a version
    pub fn get_backup(&self, version: &SemanticVersion) -> Option<&BackupEntry> {
        self.backup_versions.iter().find(|b| &b.version == version)
    }
    
    /// Delete a specific backup
    pub fn delete_backup(&mut self, version: &SemanticVersion) -> Result<(), OTAError> {
        // Don't delete current version
        if self.current_version.as_ref() == Some(version) {
            return Err(OTAError::RollbackFailed("Cannot delete current version backup".to_string()));
        }
        
        let initial_len = self.backup_versions.len();
        self.backup_versions.retain(|b| &b.version != version);
        
        if self.backup_versions.len() == initial_len {
            return Err(OTAError::RollbackFailed("Version not found".to_string()));
        }
        
        Ok(())
    }
    
    /// Cleanup old backups, keeping only specified number
    pub fn cleanup(&mut self, keep: usize) {
        if self.backup_versions.len() <= keep {
            return;
        }
        
        // Sort by creation time (newest first)
        self.backup_versions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        // Keep current version + newest backups
        let mut kept = Vec::new();
        let mut count = 0;
        
        for backup in &self.backup_versions {
            // Always keep current version
            if self.current_version.as_ref() == Some(&backup.version) {
                kept.push(backup.clone());
            } else if count < keep {
                kept.push(backup.clone());
                count += 1;
            }
        }
        
        self.backup_versions = kept;
    }
    
    /// Record a rollback with specific reason
    pub fn record_rollback(
        &mut self, 
        from: &SemanticVersion, 
        to: &SemanticVersion, 
        reason: RollbackReason,
        success: bool,
    ) {
        self.rollback_history.push(RollbackEvent {
            from_version: from.clone(),
            to_version: to.clone(),
            timestamp: self.current_timestamp(),
            reason,
            success,
            duration_ms: 0,
        });
    }
    
    /// Get statistics about rollbacks
    pub fn statistics(&self) -> RollbackStatistics {
        let total = self.rollback_history.len();
        let successful = self.rollback_history.iter().filter(|r| r.success).count();
        
        let mut reasons: HashMap<RollbackReason, u32> = HashMap::new();
        for event in &self.rollback_history {
            *reasons.entry(event.reason.clone()).or_insert(0) += 1;
        }
        
        let most_common_reason = reasons
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(reason, _)| reason.clone());
        
        RollbackStatistics {
            total_rollbacks: total,
            successful_rollbacks: successful,
            failed_rollbacks: total - successful,
            most_common_reason,
            reasons_breakdown: reasons,
        }
    }
    
    // Private helper methods
    
    fn find_oldest_removable(&self) -> Option<usize> {
        // Find oldest backup that isn't current
        self.backup_versions
            .iter()
            .enumerate()
            .filter(|(_, b)| self.current_version.as_ref() != Some(&b.version))
            .min_by_key(|(_, b)| b.created_at)
            .map(|(idx, _)| idx)
    }
    
    fn perform_restore(&self, _backup: &BackupEntry) -> Result<(), OTAError> {
        // Simulated restore - would actually restore system
        Ok(())
    }
    
    fn perform_verification(&self, _version: &SemanticVersion) -> bool {
        // Simulated verification
        true
    }
    
    fn current_timestamp(&self) -> u64 {
        0 // Simulated
    }
}

/// Rollback statistics
#[derive(Debug, Clone)]
pub struct RollbackStatistics {
    pub total_rollbacks: usize,
    pub successful_rollbacks: usize,
    pub failed_rollbacks: usize,
    pub most_common_reason: Option<RollbackReason>,
    pub reasons_breakdown: HashMap<RollbackReason, u32>,
}

/// Automatic rollback trigger configuration
#[derive(Debug, Clone)]
pub struct AutoRollbackConfig {
    /// Enable automatic rollback on boot failure
    pub on_boot_failure: bool,
    /// Number of failed boots before rollback
    pub boot_failure_threshold: u32,
    /// Enable rollback on crash loop
    pub on_crash_loop: bool,
    /// Number of crashes before rollback
    pub crash_threshold: u32,
    /// Time window for crash detection (seconds)
    pub crash_window_secs: u64,
    /// Enable health check rollback
    pub on_health_failure: bool,
    /// Health check timeout (seconds)
    pub health_timeout_secs: u64,
}

impl Default for AutoRollbackConfig {
    fn default() -> Self {
        Self {
            on_boot_failure: true,
            boot_failure_threshold: 3,
            on_crash_loop: true,
            crash_threshold: 5,
            crash_window_secs: 300, // 5 minutes
            on_health_failure: true,
            health_timeout_secs: 60,
        }
    }
}

/// Automatic rollback system
pub struct AutoRollback {
    config: AutoRollbackConfig,
    boot_attempts: u32,
    crash_timestamps: Vec<u64>,
    health_failures: u32,
    rollback_manager: RollbackManager,
}

impl AutoRollback {
    pub fn new(config: AutoRollbackConfig, rollback_manager: RollbackManager) -> Self {
        Self {
            config,
            boot_attempts: 0,
            crash_timestamps: Vec::new(),
            health_failures: 0,
            rollback_manager,
        }
    }
    
    /// Record a boot attempt
    pub fn record_boot_attempt(&mut self) {
        self.boot_attempts += 1;
    }
    
    /// Mark boot as successful (reset counter)
    pub fn boot_succeeded(&mut self) {
        self.boot_attempts = 0;
    }
    
    /// Record a crash
    pub fn record_crash(&mut self, timestamp: u64) {
        self.crash_timestamps.push(timestamp);
        
        // Remove old crashes outside window
        let window_start = timestamp.saturating_sub(self.config.crash_window_secs);
        self.crash_timestamps.retain(|t| *t >= window_start);
    }
    
    /// Record a health check failure
    pub fn record_health_failure(&mut self) {
        self.health_failures += 1;
    }
    
    /// Reset health failures
    pub fn health_succeeded(&mut self) {
        self.health_failures = 0;
    }
    
    /// Check if automatic rollback should be triggered
    pub fn should_rollback(&self) -> Option<RollbackReason> {
        // Check boot failures
        if self.config.on_boot_failure && 
           self.boot_attempts >= self.config.boot_failure_threshold 
        {
            return Some(RollbackReason::BootFailure);
        }
        
        // Check crash loop
        if self.config.on_crash_loop && 
           self.crash_timestamps.len() >= self.config.crash_threshold as usize 
        {
            return Some(RollbackReason::SystemCrash);
        }
        
        // Check health failures
        if self.config.on_health_failure && self.health_failures > 0 {
            return Some(RollbackReason::PerformanceIssue);
        }
        
        None
    }
    
    /// Perform automatic rollback if conditions met
    pub fn check_and_rollback(&mut self) -> Result<Option<SemanticVersion>, OTAError> {
        if let Some(reason) = self.should_rollback() {
            if let Some(previous) = self.rollback_manager.get_previous_version() {
                let current = self.rollback_manager.current_version
                    .clone()
                    .unwrap_or_else(|| SemanticVersion::new(0, 0, 0));
                
                self.rollback_manager.restore(&previous)?;
                self.rollback_manager.record_rollback(
                    &current,
                    &previous,
                    reason,
                    true,
                );
                
                // Reset counters
                self.boot_attempts = 0;
                self.crash_timestamps.clear();
                self.health_failures = 0;
                
                return Ok(Some(previous));
            }
        }
        
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rollback_manager_creation() {
        let manager = RollbackManager::new(3);
        assert!(!manager.can_rollback());
        assert!(manager.available_versions().is_empty());
    }
    
    #[test]
    fn test_create_backup() {
        let mut manager = RollbackManager::new(3);
        
        let v1 = SemanticVersion::new(1, 0, 0);
        manager.create_backup(&v1).unwrap();
        
        assert_eq!(manager.available_versions().len(), 1);
    }
    
    #[test]
    fn test_get_previous_version() {
        let mut manager = RollbackManager::new(3);
        
        let v1 = SemanticVersion::new(1, 0, 0);
        let v2 = SemanticVersion::new(1, 0, 1);
        
        manager.create_backup(&v1).unwrap();
        manager.create_backup(&v2).unwrap();
        
        let previous = manager.get_previous_version();
        assert_eq!(previous, Some(v1));
    }
    
    #[test]
    fn test_restore() {
        let mut manager = RollbackManager::new(3);
        
        let v1 = SemanticVersion::new(1, 0, 0);
        let v2 = SemanticVersion::new(1, 0, 1);
        
        manager.create_backup(&v1).unwrap();
        manager.create_backup(&v2).unwrap();
        
        let result = manager.restore(&v1);
        assert!(result.is_ok());
        assert_eq!(manager.current_version, Some(v1));
    }
    
    #[test]
    fn test_max_backups() {
        let mut manager = RollbackManager::new(2);
        
        for i in 0..5 {
            let v = SemanticVersion::new(1, 0, i);
            manager.create_backup(&v).unwrap();
        }
        
        // Should only keep max_versions
        assert!(manager.backup_versions.len() <= 2);
    }
    
    #[test]
    fn test_rollback_reason() {
        let reason = RollbackReason::SystemCrash;
        assert!(!reason.description().is_empty());
    }
    
    #[test]
    fn test_set_bootable() {
        let mut manager = RollbackManager::new(3);
        
        let v1 = SemanticVersion::new(1, 0, 0);
        manager.create_backup(&v1).unwrap();
        
        manager.set_bootable(&v1, false);
        
        let backup = manager.get_backup(&v1).unwrap();
        assert!(!backup.bootable);
    }
    
    #[test]
    fn test_delete_backup() {
        let mut manager = RollbackManager::new(3);
        
        let v1 = SemanticVersion::new(1, 0, 0);
        let v2 = SemanticVersion::new(1, 0, 1);
        
        manager.create_backup(&v1).unwrap();
        manager.create_backup(&v2).unwrap();
        
        // Can delete v1 (not current)
        assert!(manager.delete_backup(&v1).is_ok());
        
        // Cannot delete current (v2)
        assert!(manager.delete_backup(&v2).is_err());
    }
    
    #[test]
    fn test_statistics() {
        let mut manager = RollbackManager::new(3);
        
        let v1 = SemanticVersion::new(1, 0, 0);
        let v2 = SemanticVersion::new(1, 0, 1);
        
        manager.record_rollback(&v2, &v1, RollbackReason::SystemCrash, true);
        manager.record_rollback(&v2, &v1, RollbackReason::SystemCrash, false);
        
        let stats = manager.statistics();
        assert_eq!(stats.total_rollbacks, 2);
        assert_eq!(stats.successful_rollbacks, 1);
        assert_eq!(stats.failed_rollbacks, 1);
    }
    
    #[test]
    fn test_auto_rollback_config() {
        let config = AutoRollbackConfig::default();
        assert!(config.on_boot_failure);
        assert_eq!(config.boot_failure_threshold, 3);
    }
    
    #[test]
    fn test_auto_rollback_boot() {
        let config = AutoRollbackConfig::default();
        let manager = RollbackManager::new(3);
        let mut auto = AutoRollback::new(config, manager);
        
        // No rollback initially
        assert!(auto.should_rollback().is_none());
        
        // Record boot failures
        for _ in 0..3 {
            auto.record_boot_attempt();
        }
        
        assert_eq!(auto.should_rollback(), Some(RollbackReason::BootFailure));
        
        // Successful boot resets
        auto.boot_succeeded();
        assert!(auto.should_rollback().is_none());
    }
    
    #[test]
    fn test_auto_rollback_crash() {
        let config = AutoRollbackConfig::default();
        let manager = RollbackManager::new(3);
        let mut auto = AutoRollback::new(config, manager);
        
        // Record crashes in window
        for i in 0..5 {
            auto.record_crash(i);
        }
        
        assert_eq!(auto.should_rollback(), Some(RollbackReason::SystemCrash));
    }
    
    #[test]
    fn test_cleanup() {
        let mut manager = RollbackManager::new(10);
        
        for i in 0..5 {
            let v = SemanticVersion::new(1, 0, i);
            let _ = manager.create_backup(&v);
        }
        
        manager.cleanup(2);
        
        // Should keep current + 2
        assert!(manager.backup_versions.len() <= 3);
    }
}
